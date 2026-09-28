use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use codex_protocol::user_input::UserInput;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::codex::TurnContext;
use crate::exec_env::create_env;
use crate::host_exec::HostExecApprovalPolicy;
use crate::host_exec::HostExecBegin;
use crate::host_exec::HostExecContext;
use crate::host_exec::HostExecEnd;
use crate::host_exec::HostExecNotificationSink;
use crate::host_exec::HostExecSandboxPolicy;
use crate::host_exec::run_one_shot_shell_command;
use crate::state::TaskKind;
use crate::tools::format_exec_output_str;
use crate::user_shell_command::user_shell_command_record_item;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::exec_output::StreamOutput;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandBeginEvent;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecCommandSource;
use codex_protocol::protocol::TurnStartedEvent;

use super::SessionTask;
use super::SessionTaskContext;
use crate::codex::Session;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UserShellCommandMode {
    /// Executes as an independent turn lifecycle (emits TurnStarted/TurnComplete
    /// via task lifecycle plumbing).
    StandaloneTurn,
    /// Executes while another turn is already active. This mode must not emit a
    /// second TurnStarted/TurnComplete pair for the same active turn.
    ActiveTurnAuxiliary,
}

#[derive(Clone)]
pub(crate) struct UserShellCommandTask {
    command: String,
}

impl UserShellCommandTask {
    pub(crate) fn new(command: String) -> Self {
        Self { command }
    }
}

impl SessionTask for UserShellCommandTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.user_shell"
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        turn_context: Arc<TurnContext>,
        _input: Vec<UserInput>,
        cancellation_token: CancellationToken,
    ) -> Option<String> {
        execute_user_shell_command(
            session.clone_session(),
            turn_context,
            self.command.clone(),
            cancellation_token,
            UserShellCommandMode::StandaloneTurn,
        )
        .await;
        None
    }
}

pub(crate) async fn execute_user_shell_command(
    session: Arc<Session>,
    turn_context: Arc<TurnContext>,
    command: String,
    cancellation_token: CancellationToken,
    mode: UserShellCommandMode,
) {
    session
        .services
        .session_telemetry
        .counter("codex.task.user_shell", /*inc*/ 1, &[]);

    if mode == UserShellCommandMode::StandaloneTurn {
        // Auxiliary mode runs within an existing active turn. That turn already
        // emitted TurnStarted, so emitting another TurnStarted here would create
        // duplicate turn lifecycle events and confuse clients.
        // TODO(ccunningham): After TurnStarted, emit model-visible turn context diffs for
        // standalone lifecycle tasks (for example /shell, and review once it emits TurnStarted).
        // `/compact` is an intentional exception because compaction requests should not include
        // freshly reinjected context before the summary/replacement history is applied.
        let event = EventMsg::TurnStarted(TurnStartedEvent {
            turn_id: turn_context.sub_id.clone(),
            model_context_window: turn_context.model_context_window(),
            collaboration_mode_kind: turn_context.collaboration_mode.mode,
        });
        session.send_event(turn_context.as_ref(), event).await;
    }

    let call_id = Uuid::new_v4().to_string();
    let raw_command = command;
    let sink = Arc::new(CodexHostExecSink {
        session: session.clone(),
        turn_context: turn_context.clone(),
    });
    let outcome = run_one_shot_shell_command(HostExecContext {
        turn_id: turn_context.sub_id.clone(),
        call_id,
        process_id: None,
        raw_command: raw_command.clone(),
        cwd: turn_context.cwd.to_path_buf(),
        env: create_env(
            &turn_context.shell_environment_policy,
            Some(session.conversation_id),
        ),
        shell: session.user_shell(),
        use_login_shell: true,
        shell_environment_overrides: turn_context.shell_environment_policy.r#set.clone(),
        source: ExecCommandSource::UserShell,
        sandbox_policy: HostExecSandboxPolicy::NoSandbox,
        approval_policy: HostExecApprovalPolicy::SkipApproval,
        notification_sink: sink,
        cancellation_token,
        registration: None,
        network: turn_context.network.clone(),
        windows_sandbox_level: turn_context.windows_sandbox_level,
        windows_sandbox_private_desktop: turn_context
            .config
            .permissions
            .windows_sandbox_private_desktop,
    })
    .await;

    let output = match outcome {
        Ok(outcome) => outcome.output,
        Err(err) => {
            let message = format!("execution error: {err:#}");
            ExecToolCallOutput {
                exit_code: -1,
                stdout: StreamOutput::new(String::new()),
                stderr: StreamOutput::new(message.clone()),
                aggregated_output: StreamOutput::new(message),
                duration: Duration::ZERO,
                timed_out: false,
            }
        }
    };

    persist_user_shell_output(&session, turn_context.as_ref(), &raw_command, &output, mode).await;
}

struct CodexHostExecSink {
    session: Arc<Session>,
    turn_context: Arc<TurnContext>,
}

#[async_trait]
impl HostExecNotificationSink for CodexHostExecSink {
    async fn emit_begin(&self, event: HostExecBegin) -> Result<()> {
        self.session
            .send_event(
                self.turn_context.as_ref(),
                EventMsg::ExecCommandBegin(ExecCommandBeginEvent {
                    call_id: event.call_id,
                    process_id: event.process_id,
                    turn_id: event.turn_id,
                    command: event.command,
                    cwd: event.cwd,
                    parsed_cmd: event.parsed_cmd,
                    source: event.source,
                    interaction_input: None,
                }),
            )
            .await;
        Ok(())
    }

    async fn emit_delta(&self, event: ExecCommandOutputDeltaEvent) -> Result<()> {
        self.session
            .send_event(
                self.turn_context.as_ref(),
                EventMsg::ExecCommandOutputDelta(event),
            )
            .await;
        Ok(())
    }

    async fn emit_end(&self, event: HostExecEnd) -> Result<()> {
        self.session
            .send_event(
                self.turn_context.as_ref(),
                EventMsg::ExecCommandEnd(ExecCommandEndEvent {
                    call_id: event.call_id,
                    process_id: event.process_id,
                    turn_id: event.turn_id,
                    command: event.command,
                    cwd: event.cwd,
                    parsed_cmd: event.parsed_cmd,
                    source: event.source,
                    interaction_input: None,
                    stdout: event.output.stdout.text.clone(),
                    stderr: event.output.stderr.text.clone(),
                    aggregated_output: event.output.aggregated_output.text.clone(),
                    exit_code: event.output.exit_code,
                    duration: event.output.duration,
                    formatted_output: format_exec_output_str(
                        &event.output,
                        self.turn_context.truncation_policy,
                    ),
                    status: event.status,
                }),
            )
            .await;
        Ok(())
    }

    async fn emit_terminal_interaction(
        &self,
        event: crate::host_exec::HostExecTerminalInteraction,
    ) -> Result<()> {
        use codex_protocol::protocol::EventMsg;
        use codex_protocol::protocol::TerminalInteractionEvent;
        self.session
            .send_event(
                self.turn_context.as_ref(),
                EventMsg::TerminalInteraction(TerminalInteractionEvent {
                    call_id: event.call_id,
                    process_id: event.process_id.unwrap_or_default(),
                    stdin: event.stdin,
                }),
            )
            .await;
        Ok(())
    }
}

async fn persist_user_shell_output(
    session: &Session,
    turn_context: &TurnContext,
    raw_command: &str,
    exec_output: &ExecToolCallOutput,
    mode: UserShellCommandMode,
) {
    let output_item = user_shell_command_record_item(raw_command, exec_output, turn_context);

    if mode == UserShellCommandMode::StandaloneTurn {
        session
            .record_conversation_items(turn_context, std::slice::from_ref(&output_item))
            .await;
        // Standalone shell turns can run before any regular user turn, so
        // explicitly materialize rollout persistence after recording output.
        session.ensure_rollout_materialized().await;
        return;
    }

    let response_input_item = match output_item {
        ResponseItem::Message { role, content, .. } => ResponseInputItem::Message { role, content },
        _ => unreachable!("user shell command output record should always be a message"),
    };

    if let Err(items) = session
        .inject_response_items(vec![response_input_item])
        .await
    {
        let response_items = items
            .into_iter()
            .map(ResponseItem::from)
            .collect::<Vec<_>>();
        session
            .record_conversation_items(turn_context, &response_items)
            .await;
    }
}
