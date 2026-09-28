use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use codex_network_proxy::NetworkProxy;
use codex_protocol::ThreadId;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::error::CodexErr;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::exec_output::StreamOutput;
use codex_protocol::parse_command::ParsedCommand;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecCommandSource;
use codex_protocol::protocol::ExecCommandStatus;
use codex_protocol::protocol::SandboxPolicy;
use codex_sandboxing::SandboxType;
use codex_utils_pty::ProcessHandle;
use codex_utils_pty::SpawnedProcess;
use codex_utils_pty::TerminalSize;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::exec::ExecCapturePolicy;
use crate::exec::ExecExpiration;
use crate::exec::StdoutStream;
use crate::exec::execute_exec_request;
use crate::sandboxing::ExecRequest;
use crate::shell::Shell;
use crate::tools::runtimes::maybe_wrap_shell_lc_with_snapshot;
use codex_shell_command::parse_command::parse_command;

pub const USER_SHELL_TIMEOUT_MS: u64 = 60 * 60 * 1000; // 1 hour

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostExecSandboxPolicy {
    NoSandbox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostExecApprovalPolicy {
    SkipApproval,
}

#[derive(Clone)]
pub struct HostExecRegistration {
    pub registry: Arc<HostExecRegistry>,
    pub thread_id: ThreadId,
}

#[derive(Clone)]
pub struct HostExecContext {
    pub turn_id: String,
    pub call_id: String,
    pub process_id: Option<String>,
    pub raw_command: String,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub shell: Arc<Shell>,
    pub use_login_shell: bool,
    pub shell_environment_overrides: HashMap<String, String>,
    pub source: ExecCommandSource,
    pub sandbox_policy: HostExecSandboxPolicy,
    pub approval_policy: HostExecApprovalPolicy,
    pub notification_sink: Arc<dyn HostExecNotificationSink>,
    pub cancellation_token: CancellationToken,
    pub registration: Option<HostExecRegistration>,
    pub network: Option<NetworkProxy>,
    pub windows_sandbox_level: WindowsSandboxLevel,
    pub windows_sandbox_private_desktop: bool,
}

#[derive(Clone, Debug)]
pub struct HostExecBegin {
    pub call_id: String,
    pub process_id: Option<String>,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub parsed_cmd: Vec<ParsedCommand>,
    pub source: ExecCommandSource,
}

#[derive(Clone, Debug)]
pub struct HostExecEnd {
    pub call_id: String,
    pub process_id: Option<String>,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub parsed_cmd: Vec<ParsedCommand>,
    pub source: ExecCommandSource,
    pub output: ExecToolCallOutput,
    pub status: ExecCommandStatus,
}

#[derive(Clone, Debug)]
pub struct HostExecOutcome {
    pub call_id: String,
    pub process_id: Option<String>,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub parsed_cmd: Vec<ParsedCommand>,
    pub source: ExecCommandSource,
    pub output: ExecToolCallOutput,
    pub status: ExecCommandStatus,
}

#[derive(Clone, Debug)]
pub struct HostExecTerminalInteraction {
    pub call_id: String,
    pub process_id: Option<String>,
    pub stdin: String,
}

/// Maps shared host-exec lifecycle events into the caller's notification system.
#[async_trait]
pub trait HostExecNotificationSink: Send + Sync {
    async fn emit_begin(&self, event: HostExecBegin) -> Result<()>;

    async fn emit_delta(&self, event: ExecCommandOutputDeltaEvent) -> Result<()>;

    async fn emit_end(&self, event: HostExecEnd) -> Result<()>;

    async fn emit_terminal_interaction(&self, event: HostExecTerminalInteraction) -> Result<()>;
}

#[derive(Debug)]
pub struct InteractiveProcessState {
    session: Arc<ProcessHandle>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
    output_notify: Arc<Notify>,
    exited: Arc<AtomicBool>,
}

impl InteractiveProcessState {
    pub async fn write_stdin(&self, data: &[u8]) -> Result<()> {
        self.session
            .writer_sender()
            .send(data.to_vec())
            .await
            .map_err(|_| anyhow!("stdin is already closed"))
    }

    pub fn resize(&self, size: TerminalSize) -> Result<()> {
        self.session
            .resize(size)
            .map_err(|err| anyhow!("failed to resize PTY: {err}"))
    }

    pub async fn read_buffered_output(&self) -> Vec<u8> {
        let mut buffer = self.output_buffer.lock().await;
        let output = buffer.clone();
        buffer.clear();
        output
    }

    pub fn has_exited(&self) -> bool {
        self.exited.load(Ordering::Acquire)
    }

    fn request_terminate(&self) {
        self.session.request_terminate();
    }
}

#[derive(Clone, Debug)]
pub struct HostExecProcessEntry {
    pub call_id: String,
    pub process_id: Option<String>,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub source: ExecCommandSource,
    pub started_at: i64,
    cancellation_token: CancellationToken,
    interactive: Option<Arc<InteractiveProcessState>>,
}

impl HostExecProcessEntry {
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive.is_some()
    }

    pub fn interactive_state(&self) -> Option<&Arc<InteractiveProcessState>> {
        self.interactive.as_ref()
    }
}

#[derive(Default)]
pub struct HostExecRegistry {
    entries: Mutex<HashMap<ThreadId, HashMap<String, HostExecProcessEntry>>>,
}

impl HostExecRegistry {
    pub async fn register(&self, thread_id: ThreadId, entry: HostExecProcessEntry) {
        self.entries
            .lock()
            .await
            .entry(thread_id)
            .or_default()
            .insert(entry.call_id.clone(), entry);
    }

    pub async fn unregister(&self, thread_id: ThreadId, call_id: &str) {
        let mut entries = self.entries.lock().await;
        if let Some(thread_entries) = entries.get_mut(&thread_id) {
            thread_entries.remove(call_id);
            if thread_entries.is_empty() {
                entries.remove(&thread_id);
            }
        }
    }

    pub async fn cancel_thread(&self, thread_id: ThreadId) {
        let to_cancel = self
            .entries
            .lock()
            .await
            .get(&thread_id)
            .map(|thread_entries| {
                thread_entries
                    .values()
                    .map(|e| (e.cancellation_token.clone(), e.interactive.is_some()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (token, _has_interactive) in &to_cancel {
            token.cancel();
        }
        let interactive_sessions: Vec<Arc<InteractiveProcessState>> = self
            .entries
            .lock()
            .await
            .get(&thread_id)
            .map(|thread_entries| {
                thread_entries
                    .values()
                    .filter_map(|e| e.interactive.clone())
                    .collect()
            })
            .unwrap_or_default();
        for state in interactive_sessions {
            state.request_terminate();
        }
    }

    pub async fn has_thread_commands(&self, thread_id: ThreadId) -> bool {
        self.entries
            .lock()
            .await
            .get(&thread_id)
            .is_some_and(|thread_entries| !thread_entries.is_empty())
    }

    pub async fn thread_processes(&self, thread_id: ThreadId) -> Vec<HostExecProcessEntry> {
        self.entries
            .lock()
            .await
            .get(&thread_id)
            .map(|thread_entries| thread_entries.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn write_stdin(
        &self,
        thread_id: ThreadId,
        call_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        let entry = self.get_entry(thread_id, call_id).await?;
        let state = entry
            .interactive
            .as_ref()
            .ok_or_else(|| anyhow!("process is not interactive"))?;
        state.write_stdin(&data).await
    }

    pub async fn resize_process(
        &self,
        thread_id: ThreadId,
        call_id: &str,
        rows: u16,
        cols: u16,
    ) -> Result<()> {
        let entry = self.get_entry(thread_id, call_id).await?;
        let state = entry
            .interactive
            .as_ref()
            .ok_or_else(|| anyhow!("process is not interactive"))?;
        state.resize(TerminalSize { rows, cols })
    }

    pub async fn read_buffered_output(
        &self,
        thread_id: ThreadId,
        call_id: &str,
    ) -> Result<Vec<u8>> {
        let entry = self.get_entry(thread_id, call_id).await?;
        let state = entry
            .interactive
            .as_ref()
            .ok_or_else(|| anyhow!("process is not interactive"))?;
        Ok(state.read_buffered_output().await)
    }

    async fn get_entry(&self, thread_id: ThreadId, call_id: &str) -> Result<HostExecProcessEntry> {
        self.entries
            .lock()
            .await
            .get(&thread_id)
            .and_then(|thread_entries| thread_entries.get(call_id).cloned())
            .ok_or_else(|| anyhow!("process not found: {call_id}"))
    }
}

pub async fn run_one_shot_shell_command(context: HostExecContext) -> Result<HostExecOutcome> {
    let display_command = context
        .shell
        .derive_exec_args(&context.raw_command, context.use_login_shell);
    let exec_command = maybe_wrap_shell_lc_with_snapshot(
        &display_command,
        context.shell.as_ref(),
        context.cwd.as_path(),
        &context.shell_environment_overrides,
    );
    let parsed_cmd = parse_command(&display_command);
    let begin = HostExecBegin {
        call_id: context.call_id.clone(),
        process_id: context.process_id.clone(),
        turn_id: context.turn_id.clone(),
        command: display_command.clone(),
        cwd: context.cwd.clone(),
        parsed_cmd: parsed_cmd.clone(),
        source: context.source,
    };
    context.notification_sink.emit_begin(begin).await?;

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if let Some(registration) = &context.registration {
        registration
            .registry
            .register(
                registration.thread_id,
                HostExecProcessEntry {
                    call_id: context.call_id.clone(),
                    process_id: context.process_id.clone(),
                    command: display_command.clone(),
                    cwd: context.cwd.clone(),
                    source: context.source,
                    started_at: now_ts,
                    cancellation_token: context.cancellation_token.clone(),
                    interactive: None,
                },
            )
            .await;
    }

    let (tx_event, rx_event) = async_channel::unbounded::<Event>();
    let sink = Arc::clone(&context.notification_sink);
    let delta_forwarder = tokio::spawn(async move {
        while let Ok(event) = rx_event.recv().await {
            if let EventMsg::ExecCommandOutputDelta(delta) = event.msg {
                let _ = sink.emit_delta(delta).await;
            }
        }
    });

    let stdout_stream = Some(StdoutStream {
        sub_id: context.turn_id.clone(),
        call_id: context.call_id.clone(),
        tx_event,
    });
    let exec_request = ExecRequest {
        command: exec_command,
        cwd: context.cwd.clone(),
        env: context.env.clone(),
        network: context.network.clone(),
        expiration: ExecExpiration::Cancellation(context.cancellation_token.clone()),
        capture_policy: ExecCapturePolicy::ShellTool,
        sandbox: SandboxType::None,
        windows_sandbox_level: context.windows_sandbox_level,
        windows_sandbox_private_desktop: context.windows_sandbox_private_desktop,
        sandbox_policy: SandboxPolicy::DangerFullAccess,
        file_system_sandbox_policy: FileSystemSandboxPolicy::from(&SandboxPolicy::DangerFullAccess),
        network_sandbox_policy: NetworkSandboxPolicy::from(&SandboxPolicy::DangerFullAccess),
        windows_restricted_token_filesystem_overlay: None,
        arg0: None,
    };
    let exec_result = execute_exec_request(exec_request, stdout_stream, None).await;
    let _ = delta_forwarder.await;

    let (output, status) = match exec_result {
        Ok(output) => {
            let status = if output.exit_code == 0 {
                ExecCommandStatus::Completed
            } else {
                ExecCommandStatus::Failed
            };
            (output, status)
        }
        Err(CodexErr::Sandbox(SandboxErr::Timeout { .. }))
            if context.cancellation_token.is_cancelled() =>
        {
            let output = ExecToolCallOutput {
                exit_code: -1,
                stdout: StreamOutput::new(String::new()),
                stderr: StreamOutput::new("command aborted by user".to_string()),
                aggregated_output: StreamOutput::new("command aborted by user".to_string()),
                duration: Duration::ZERO,
                timed_out: false,
            };
            (output, ExecCommandStatus::Failed)
        }
        Err(err) => {
            let message = format!("execution error: {err:?}");
            let output = ExecToolCallOutput {
                exit_code: -1,
                stdout: StreamOutput::new(String::new()),
                stderr: StreamOutput::new(message.clone()),
                aggregated_output: StreamOutput::new(message),
                duration: Duration::ZERO,
                timed_out: false,
            };
            (output, ExecCommandStatus::Failed)
        }
    };

    let end = HostExecEnd {
        call_id: context.call_id.clone(),
        process_id: context.process_id.clone(),
        turn_id: context.turn_id.clone(),
        command: display_command.clone(),
        cwd: context.cwd.clone(),
        parsed_cmd: parsed_cmd.clone(),
        source: context.source,
        output: output.clone(),
        status: status.clone(),
    };
    context.notification_sink.emit_end(end).await?;

    if let Some(registration) = &context.registration {
        registration
            .registry
            .unregister(registration.thread_id, &context.call_id)
            .await;
    }

    Ok(HostExecOutcome {
        call_id: context.call_id,
        process_id: context.process_id,
        turn_id: context.turn_id,
        command: display_command,
        cwd: context.cwd,
        parsed_cmd,
        source: context.source,
        output,
        status,
    })
}

#[derive(Clone)]
pub struct InteractiveHostExecContext {
    pub turn_id: String,
    pub call_id: String,
    pub process_id: Option<String>,
    pub raw_command: String,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub shell: Arc<Shell>,
    pub use_login_shell: bool,
    pub shell_environment_overrides: HashMap<String, String>,
    pub source: ExecCommandSource,
    pub sandbox_policy: HostExecSandboxPolicy,
    pub approval_policy: HostExecApprovalPolicy,
    pub notification_sink: Arc<dyn HostExecNotificationSink>,
    pub cancellation_token: CancellationToken,
    pub registration: Option<HostExecRegistration>,
    pub windows_sandbox_level: WindowsSandboxLevel,
    pub windows_sandbox_private_desktop: bool,
}

pub async fn spawn_interactive_process(context: InteractiveHostExecContext) -> Result<()> {
    let display_command = context
        .shell
        .derive_exec_args(&context.raw_command, context.use_login_shell);
    let exec_command = maybe_wrap_shell_lc_with_snapshot(
        &display_command,
        context.shell.as_ref(),
        context.cwd.as_path(),
        &context.shell_environment_overrides,
    );
    let parsed_cmd = parse_command(&display_command);

    let begin = HostExecBegin {
        call_id: context.call_id.clone(),
        process_id: context.process_id.clone(),
        turn_id: context.turn_id.clone(),
        command: display_command.clone(),
        cwd: context.cwd.clone(),
        parsed_cmd: parsed_cmd.clone(),
        source: context.source,
    };
    context.notification_sink.emit_begin(begin).await?;

    let (program, args) = exec_command
        .split_first()
        .ok_or_else(|| anyhow!("command must not be empty"))?;

    let spawned = codex_utils_pty::spawn_pty_process(
        program,
        args,
        context.cwd.as_path(),
        &context.env,
        &None,
        TerminalSize::default(),
    )
    .await
    .map_err(|err| anyhow!("failed to spawn interactive process: {err}"))?;

    let SpawnedProcess {
        session,
        stdout_rx,
        stderr_rx: _stderr_rx,
        exit_rx,
    } = spawned;

    let session = Arc::new(session);

    let interactive_state = Arc::new(InteractiveProcessState {
        session: Arc::clone(&session),
        output_buffer: Arc::new(Mutex::new(Vec::new())),
        output_notify: Arc::new(Notify::new()),
        exited: Arc::new(AtomicBool::new(false)),
    });

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if let Some(registration) = &context.registration {
        registration
            .registry
            .register(
                registration.thread_id,
                HostExecProcessEntry {
                    call_id: context.call_id.clone(),
                    process_id: context.process_id.clone(),
                    command: display_command.clone(),
                    cwd: context.cwd.clone(),
                    source: context.source,
                    started_at: now_ts,
                    cancellation_token: context.cancellation_token.clone(),
                    interactive: Some(Arc::clone(&interactive_state)),
                },
            )
            .await;
    }

    let sink = Arc::clone(&context.notification_sink);
    let call_id = context.call_id.clone();
    let process_id = context.process_id.clone();
    let turn_id = context.turn_id.clone();
    let output_buffer = Arc::clone(&interactive_state.output_buffer);
    let output_notify = Arc::clone(&interactive_state.output_notify);
    let exited = Arc::clone(&interactive_state.exited);
    let cancellation_token = context.cancellation_token.clone();
    let registration = context.registration.clone();
    let command = display_command.clone();
    let cwd = context.cwd.clone();

    tokio::spawn(async move {
        let mut stdout_rx = stdout_rx;
        let mut exit_rx = std::pin::pin!(exit_rx);

        loop {
            tokio::select! {
                chunk = stdout_rx.recv() => {
                    match chunk {
                        Some(chunk) => {
                            {
                                let mut buffer = output_buffer.lock().await;
                                buffer.extend_from_slice(&chunk);
                            }
                            output_notify.notify_waiters();

                            let delta = ExecCommandOutputDeltaEvent {
                                call_id: call_id.clone(),
                                stream: codex_protocol::protocol::ExecOutputStream::Stdout,
                                chunk,
                            };
                            let _ = sink.emit_delta(delta).await;
                        }
                        None => break,
                    }
                }
                _ = cancellation_token.cancelled() => {
                    session.request_terminate();
                }
                exit_code = &mut exit_rx => {
                    let exit_code = exit_code.unwrap_or(-1);
                    exited.store(true, Ordering::Release);

                    let status = if exit_code == 0 {
                        ExecCommandStatus::Completed
                    } else {
                        ExecCommandStatus::Failed
                    };

                    let aggregated = {
                        let buffer = output_buffer.lock().await;
                        String::from_utf8_lossy(&buffer).to_string()
                    };
                    let output = ExecToolCallOutput {
                        exit_code,
                        stdout: StreamOutput::new(aggregated.clone()),
                        stderr: StreamOutput::new(String::new()),
                        aggregated_output: StreamOutput::new(aggregated),
                        duration: Duration::ZERO,
                        timed_out: false,
                    };

                    let end = HostExecEnd {
                        call_id: call_id.clone(),
                        process_id: process_id.clone(),
                        turn_id: turn_id.clone(),
                        command: command.clone(),
                        cwd: cwd.clone(),
                        parsed_cmd: parsed_cmd.clone(),
                        source: ExecCommandSource::UnifiedExecStartup,
                        output,
                        status,
                    };
                    let _ = sink.emit_end(end).await;

                    if let Some(registration) = &registration {
                        registration
                            .registry
                            .unregister(registration.thread_id, &call_id)
                            .await;
                    }
                    break;
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
#[path = "host_exec_tests.rs"]
mod tests;
