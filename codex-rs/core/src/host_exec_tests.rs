use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use codex_protocol::ThreadId;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecCommandSource;
use codex_protocol::protocol::ExecCommandStatus;
use pretty_assertions::assert_eq;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::HostExecApprovalPolicy;
use super::HostExecContext;
use super::HostExecNotificationSink;
use super::HostExecProcessEntry;
use super::HostExecRegistry;
use super::HostExecSandboxPolicy;
use super::HostExecTerminalInteraction;
use super::run_one_shot_shell_command;
use crate::shell::default_user_shell;

#[derive(Default)]
struct RecordingSink {
    begins: Mutex<Vec<String>>,
    deltas: Mutex<Vec<String>>,
    ends: Mutex<Vec<ExecCommandStatus>>,
    interactions: Mutex<Vec<String>>,
}

#[async_trait]
impl HostExecNotificationSink for RecordingSink {
    async fn emit_begin(&self, event: super::HostExecBegin) -> Result<()> {
        self.begins.lock().await.push(event.call_id);
        Ok(())
    }

    async fn emit_delta(&self, event: ExecCommandOutputDeltaEvent) -> Result<()> {
        self.deltas
            .lock()
            .await
            .push(String::from_utf8_lossy(&event.chunk).to_string());
        Ok(())
    }

    async fn emit_end(&self, event: super::HostExecEnd) -> Result<()> {
        self.ends.lock().await.push(event.status);
        Ok(())
    }

    async fn emit_terminal_interaction(&self, event: HostExecTerminalInteraction) -> Result<()> {
        self.interactions
            .lock()
            .await
            .push(format!("{}:{}", event.call_id, event.stdin));
        Ok(())
    }
}

#[tokio::test]
async fn host_exec_runs_shell_command_and_emits_lifecycle() -> Result<()> {
    let sink = Arc::new(RecordingSink::default());
    let (raw_command, expected_output) = current_shell_output_command("hello from host exec")?;

    let outcome = run_one_shot_shell_command(HostExecContext {
        turn_id: "turn-1".to_string(),
        call_id: "call-1".to_string(),
        process_id: Some("proc-1".to_string()),
        raw_command,
        cwd: std::env::current_dir()?,
        env: std::env::vars().collect::<HashMap<_, _>>(),
        shell: Arc::new(default_user_shell()),
        use_login_shell: !cfg!(windows),
        shell_environment_overrides: HashMap::new(),
        source: ExecCommandSource::UserShell,
        sandbox_policy: HostExecSandboxPolicy::NoSandbox,
        approval_policy: HostExecApprovalPolicy::SkipApproval,
        notification_sink: sink.clone(),
        cancellation_token: CancellationToken::new(),
        registration: None,
        network: None,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        windows_sandbox_private_desktop: false,
    })
    .await?;

    assert_eq!(outcome.status, ExecCommandStatus::Completed);
    assert_eq!(outcome.output.aggregated_output.text, expected_output);
    assert_eq!(sink.begins.lock().await.len(), 1);
    assert_eq!(sink.ends.lock().await.len(), 1);
    assert!(!sink.deltas.lock().await.is_empty());

    Ok(())
}

#[tokio::test]
async fn host_exec_registry_cancels_all_commands_for_thread() {
    let registry = HostExecRegistry::default();
    let thread_id = ThreadId::new();
    let first = CancellationToken::new();
    let second = CancellationToken::new();
    registry
        .register(
            thread_id,
            HostExecProcessEntry {
                call_id: "call-1".to_string(),
                process_id: Some("proc-1".to_string()),
                command: vec!["echo".to_string(), "hello".to_string()],
                cwd: PathBuf::from("/tmp"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: first.clone(),
                interactive: None,
            },
        )
        .await;
    registry
        .register(
            thread_id,
            HostExecProcessEntry {
                call_id: "call-2".to_string(),
                process_id: Some("proc-2".to_string()),
                command: vec!["echo".to_string(), "world".to_string()],
                cwd: PathBuf::from("/tmp"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: second.clone(),
                interactive: None,
            },
        )
        .await;

    registry.cancel_thread(thread_id).await;

    assert!(first.is_cancelled());
    assert!(second.is_cancelled());
}

#[tokio::test]
async fn host_exec_registry_tracks_process_metadata() {
    let registry = HostExecRegistry::default();
    let thread_id = ThreadId::new();
    registry
        .register(
            thread_id,
            HostExecProcessEntry {
                call_id: "call-meta".to_string(),
                process_id: Some("proc-meta".to_string()),
                command: vec!["echo".to_string(), "test".to_string()],
                cwd: PathBuf::from("/workspace"),
                source: ExecCommandSource::UserShell,
                started_at: 100,
                cancellation_token: CancellationToken::new(),
                interactive: None,
            },
        )
        .await;

    let processes = registry.thread_processes(thread_id).await;
    assert_eq!(processes.len(), 1);
    assert_eq!(processes[0].call_id, "call-meta");
    assert_eq!(processes[0].process_id.as_deref(), Some("proc-meta"));
    assert_eq!(processes[0].command, vec!["echo", "test"]);
    assert_eq!(processes[0].cwd, PathBuf::from("/workspace"));
    assert_eq!(processes[0].started_at, 100);
    assert!(!processes[0].is_cancelled());
    assert!(!processes[0].is_interactive());
}

#[tokio::test]
async fn host_exec_registry_auto_prunes_on_unregister() {
    let registry = HostExecRegistry::default();
    let thread_id = ThreadId::new();
    registry
        .register(
            thread_id,
            HostExecProcessEntry {
                call_id: "call-auto".to_string(),
                process_id: None,
                command: vec!["true".to_string()],
                cwd: PathBuf::from("/tmp"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: CancellationToken::new(),
                interactive: None,
            },
        )
        .await;

    assert!(registry.has_thread_commands(thread_id).await);

    registry.unregister(thread_id, "call-auto").await;

    assert!(!registry.has_thread_commands(thread_id).await);
    assert!(registry.thread_processes(thread_id).await.is_empty());
}

#[tokio::test]
async fn host_exec_registry_thread_processes_isolation() {
    let registry = HostExecRegistry::default();
    let thread_a = ThreadId::new();
    let thread_b = ThreadId::new();
    registry
        .register(
            thread_a,
            HostExecProcessEntry {
                call_id: "call-a".to_string(),
                process_id: Some("proc-a".to_string()),
                command: vec!["echo".to_string(), "a".to_string()],
                cwd: PathBuf::from("/a"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: CancellationToken::new(),
                interactive: None,
            },
        )
        .await;
    registry
        .register(
            thread_b,
            HostExecProcessEntry {
                call_id: "call-b".to_string(),
                process_id: Some("proc-b".to_string()),
                command: vec!["echo".to_string(), "b".to_string()],
                cwd: PathBuf::from("/b"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: CancellationToken::new(),
                interactive: None,
            },
        )
        .await;

    registry.cancel_thread(thread_a).await;

    let processes_a = registry.thread_processes(thread_a).await;
    let processes_b = registry.thread_processes(thread_b).await;
    assert_eq!(processes_a.len(), 1);
    assert!(processes_a[0].is_cancelled());
    assert_eq!(processes_b.len(), 1);
    assert!(!processes_b[0].is_cancelled());
}

#[tokio::test]
async fn host_exec_registry_write_stdin_resize_require_interactive() {
    let registry = HostExecRegistry::default();
    let thread_id = ThreadId::new();
    registry
        .register(
            thread_id,
            HostExecProcessEntry {
                call_id: "call-one-shot".to_string(),
                process_id: Some("proc-one-shot".to_string()),
                command: vec!["echo".to_string(), "test".to_string()],
                cwd: PathBuf::from("/tmp"),
                source: ExecCommandSource::UserShell,
                started_at: 0,
                cancellation_token: CancellationToken::new(),
                interactive: None,
            },
        )
        .await;

    let write_result = registry
        .write_stdin(thread_id, "call-one-shot", b"hello".to_vec())
        .await;
    assert!(write_result.is_err());
    assert!(
        write_result
            .unwrap_err()
            .to_string()
            .contains("not interactive")
    );

    let resize_result = registry
        .resize_process(thread_id, "call-one-shot", 24, 80)
        .await;
    assert!(resize_result.is_err());
    assert!(
        resize_result
            .unwrap_err()
            .to_string()
            .contains("not interactive")
    );

    let output_result = registry
        .read_buffered_output(thread_id, "call-one-shot")
        .await;
    assert!(output_result.is_err());
    assert!(
        output_result
            .unwrap_err()
            .to_string()
            .contains("not interactive")
    );
}

#[tokio::test]
async fn host_exec_registry_write_stdin_resize_unknown_process() {
    let registry = HostExecRegistry::default();
    let thread_id = ThreadId::new();

    let write_result = registry
        .write_stdin(thread_id, "nonexistent", b"hello".to_vec())
        .await;
    assert!(write_result.is_err());

    let resize_result = registry
        .resize_process(thread_id, "nonexistent", 24, 80)
        .await;
    assert!(resize_result.is_err());
}

fn current_shell_output_command(text: &str) -> Result<(String, String)> {
    let command_and_output = match default_user_shell().name() {
        "powershell" => {
            let escaped_text = text.replace('\'', "''");
            (
                format!("Write-Output '{escaped_text}'"),
                format!("{text}\r\n"),
            )
        }
        "cmd" => (format!("echo {text}"), format!("{text}\r\n")),
        _ => {
            let quoted_text = shlex::try_quote(text)?;
            (format!("printf '%s\\n' {quoted_text}"), format!("{text}\n"))
        }
    };
    Ok(command_and_output)
}
