---
id: logos-item-05-shell-execution-draft-implementation-discussion
title: Item 05 Shell Command Execution Draft Implementation Discussion
type: discussion
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    shell-execution,
    host-ops,
    research-transfer,
  ]
related:
  [
    [README.md],
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../research/session layer/RA-1.5-shell-execution.md],
  ]
depends_on:
  [
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-shell-command-support-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 05: Shell Command Execution Draft Implementation Discussion

## Purpose

This document turns the validated `Item 05: Shell Command Execution` research verdict into a practical LOGOS implementation draft.

## Current Transfer Status

1. `Item 05` verdict is `ADAPT` with `DUAL_CONSENSUS_VALID`.
2. The ADAPT arises because the imported direction — shell execution as shared host/runtime ownership — is correct, but the existing Codex shell/exec substrate cannot be reused as-is. It is canonically coupled to `Session` + `TurnContext` + `SessionServices`, and must be extracted or wrapped before other adapter lanes can reuse it honestly.
3. The reason for ADAPT is **not** provider-sensitive semantics. Shell execution has no provider-specific behavior. The reason is coupling to the Codex session model.
4. This draft exists so future spec work can start from code-aware implementation shape instead of only from theory.

## Verdict Decomposition

### ACCEPT (fundamental mechanics)

1. Shell execution is a shared host/runtime concern, not a provider-lane-specific feature.
2. A shared shell handler should serve all adapter lanes through a uniform interface.
3. Platform-aware shell selection.
4. Unified stdout/stderr streaming + output capture.
5. Abort/cancellation as first-class.

### ADAPT (integration shape)

1. Implement through existing Codex/LOGOS exec substrate, not by copying OpenCode or Goose shell stacks.
2. Preserve sandbox + approval as non-optional capabilities.
3. Extract from `Session`/`TurnContext`/`SessionServices` coupling into a LOGOS-owned host-exec context.
4. Align with future background-terminal surface (Item 10).

## What This Draft Covers

1. Coupling diagnosis between current exec substrate and Codex session model.
2. Shared host-exec context boundary.
3. Shell handler extraction target.
4. Process ownership keyed by LOGOS thread/runtime identity.
5. Notification and lifecycle integration.
6. Sandbox and approval plumbing for non-canonical lanes.

## What This Draft Does Not Cover

These remain separate items and must not be silently folded into `Item 05`:

1. Thread registry and ephemeral runtime (Item 01).
2. Session persistence schema (Item 02).
3. Fork/resume orchestration (Item 03).
4. Turn lifecycle state machine (Item 04).
5. Review orchestration (Item 06).
6. Background terminal management (Item 10).
7. Doom loop detection (Item 04).
8. Turn interrupt semantics (Item 08).

## Current Code Diagnosis

### Canonical Codex Shell Path

The canonical path has a mature local exec stack:

**Dispatch:** `codex_message_processor.rs:3597-3617` — `Op::RunUserShellCommand { command }` submitted to canonical Session.

**Handler:** `codex.rs:4851-4876` — `run_user_shell_command()` dispatches into either active-turn-auxiliary or standalone-turn mode.

**Execution:** `tasks/user_shell.rs:92-312` — `execute_user_shell_command()`:

- Takes `Arc<Session>` + `Arc<TurnContext>` (coupling point 1)
- Uses `session.user_shell()` for shell detection (coupling point 2)
- Creates `ExecRequest` with `ExecCapturePolicy::ShellTool`
- Calls `execute_exec_request()` from `exec.rs`
- Sends events through `session.send_event(turn_context, EventMsg::...)` (coupling point 3)
- Records output via `session.record_conversation_items()` (coupling point 4)
- Supports cancellation through `CancellationToken`

**Unified exec (tool-triggered):** `unified_exec/mod.rs` + `process_manager.rs`:

- `UnifiedExecContext` holds `Arc<Session>` + `Arc<TurnContext>` + `call_id` (coupling point 5)
- `ProcessEntry` holds `Weak<Session>` (coupling point 6)
- Uses `ToolOrchestrator` for approval + sandbox + retry (coupling point 7)
- Process IDs keyed by `i32` session-local integers
- `SessionServices` contains `UnifiedExecProcessManager` among 30+ other services (`state/service.rs:30-61`)

**Approval:** `tools/orchestrator.rs:101-199` — `ToolOrchestrator::run()`:

- Takes `ToolCtx` (holds `Arc<Session>`, `Arc<TurnContext>`) (coupling point 8)
- Evaluates `ExecApprovalRequirement` per call
- Routes through `AskForApproval` policy
- Manages sandbox selection via `SandboxManager`
- All context derived from `TurnContext` fields

### OpenRouter Shell Path (Temporary Copy)

The OR proving lane has a completely separate implementation:

**State:** `openrouter_beta_threads.rs:81` — `running_commands: Mutex<HashMap<String, Arc<ProcessHandle>>>`

**Dispatch:** `openrouter_beta_threads.rs:646-769` — `shell_command()`:

- Manually creates turn if none active
- Uses `default_user_shell()` directly (not through Session)
- Spawns via `spawn_pipe_process_no_stdin()` (not through exec.rs)
- Manually constructs all notifications (`ItemStarted`, `ItemCompleted`, `TurnStarted`, `TurnCompleted`)
- No sandbox, no approval flow
- Process ownership keyed by `item_id` string

**Driver:** `openrouter_beta_threads.rs:2108-2283` — `drive_openrouter_shell_command()`:

- Manual stdout/stderr/exit collection loop
- Manual notification sending for `CommandExecutionOutputDelta`
- Manual persistence via `save_active_thread()`
- Manual state update (`turn.items.push`, `active_turn_id = None`)

### The Four Coupling Points

The canonical shell stack cannot be reused by non-canonical lanes because of four hard coupling points:

| #   | Coupling                                              | Location                                        | What It Needs                                                         |
| --- | ----------------------------------------------------- | ----------------------------------------------- | --------------------------------------------------------------------- |
| 1   | `Arc<Session>` in execution context                   | `user_shell.rs:92-93`, `unified_exec/mod.rs:73` | Full Codex session with event bus, services, conversation ID, rollout |
| 2   | `Arc<TurnContext>` in execution context               | `user_shell.rs:93`, `unified_exec/mod.rs:74`    | Submission ID, CWD, config, truncation policy, environment policy     |
| 3   | `session.send_event()` for notifications              | `user_shell.rs:117,151,209,233,277`             | Codex-internal event channel keyed by `TurnContext`                   |
| 4   | `session.record_conversation_items()` for persistence | `user_shell.rs:325,347`                         | Codex conversation journal keyed by session                           |

None of these are provider-specific. All are Codex-session-model-specific. A stateless-provider lane that uses `SessionEngine` instead of `Session` cannot satisfy any of them.

### What The OR Path Does Differently (And Why)

The OR path does not lack features by design. It duplicates the same mechanics with different types:

| Mechanism        | Canonical                                        | OR                                           |
| ---------------- | ------------------------------------------------ | -------------------------------------------- |
| Shell detection  | `session.user_shell()`                           | `default_user_shell()`                       |
| Process spawn    | `execute_exec_request()` via `exec.rs`           | `spawn_pipe_process_no_stdin()`              |
| Streaming        | `StdoutStream` + `session.get_tx_event()`        | Manual `tokio::select!` loop                 |
| Notification     | `session.send_event()` → app-server mapper       | Direct `send_notification()`                 |
| Persistence      | `session.record_conversation_items()`            | `save_active_thread()` JSON                  |
| Cancellation     | `CancellationToken`                              | `handle.request_terminate()`                 |
| Process registry | `UnifiedExecProcessManager` in `SessionServices` | `running_commands` in `OpenRouterBetaThread` |
| Approval         | `ToolOrchestrator`                               | None                                         |
| Sandbox          | `SandboxManager` via `ExecRequest`               | None                                         |

The OR path is not "simplified" — it is a **parallel implementation with different ownership**. The target is to collapse both into one shared host-exec layer.

## Draft Target Model

### 1. HostExecContext

Replace `Arc<Session>` + `Arc<TurnContext>` with a provider-agnostic execution context:

```rust
struct HostExecContext {
    thread_id: ThreadId,
    turn_id: String,
    cwd: PathBuf,
    environment: HashMap<String, String>,
    shell: Shell,
    sandbox_policy: HostExecSandboxPolicy,
    approval_policy: HostExecApprovalPolicy,
    notification_sink: Arc<dyn HostExecNotificationSink>,
    cancellation_token: CancellationToken,
}
```

Properties:

- Contains everything needed to spawn, stream, notify, and cancel a command.
- Does not reference `Session`, `TurnContext`, or `SessionServices`.
- Canonical lane constructs `HostExecContext` from `Session` + `TurnContext`.
- OR lane constructs `HostExecContext` from `OpenRouterBetaThread` state.
- Future adapter lanes construct it from their own thread state.

### 2. HostExecNotificationSink

Replace `session.send_event()` with a trait:

```rust
trait HostExecNotificationSink: Send + Sync {
    async fn emit_command_begin(&self, event: HostExecBeginEvent);
    async fn emit_command_delta(&self, event: HostExecDeltaEvent);
    async fn emit_command_end(&self, event: HostExecEndEvent);
}
```

Implementations:

- **Codex mapper:** Wraps `session.send_event(turn_context, EventMsg::ExecCommand*)`.
- **OR mapper:** Wraps `send_notification()` with `ServerNotification::ItemStarted/Completed/OutputDelta`.
- **Future lanes:** Map to their own notification system.

This is the boundary where the current `openrouter_beta_threads.rs:1973-1988` `send_notification` helper and the Codex `session.send_event` converge into one abstraction.

### 3. Shared Shell Handler

A single `HostExecService` owns the actual execution logic:

```rust
struct HostExecService {
    process_registry: Mutex<HashMap<String, Arc<HostExecProcess>>>,
    max_concurrent_processes: usize,
}

impl HostExecService {
    async fn execute_command(
        &self,
        ctx: HostExecContext,
        command: Vec<String>,
    ) -> Result<HostExecResult, HostExecError>;

    async fn terminate_command(
        &self,
        process_key: &str,
    ) -> Result<(), HostExecError>;

    async fn clean_all_for_thread(
        &self,
        thread_id: ThreadId,
    ) -> Vec<String>;
}
```

This replaces both `UnifiedExecProcessManager` (canonical) and `running_commands` (OR).

The `process_key` is a string combining thread_id and item_id, making the registry keyed by LOGOS identity rather than by session-local integers.

### 4. Sandbox And Approval As Configurable Policies

The `HostExecSandboxPolicy` and `HostExecApprovalPolicy` are enums, not boolean flags:

```rust
enum HostExecSandboxPolicy {
    FullSandbox { sandbox_type: SandboxType },
    NoSandbox,
}

enum HostExecApprovalPolicy {
    PerCommand { policy: AskForApproval },
    SkipApproval,
    Forbidden { reason: String },
}
```

Canonical lane uses `FullSandbox` + `PerCommand`.
OR beta uses `NoSandbox` + `SkipApproval` (current state).
Future production OR may use `FullSandbox` + `PerCommand`.

### 5. Bridging Canonical Path

The canonical Codex path does not change its outward behavior. Instead:

- `execute_user_shell_command()` constructs a `HostExecContext` from its `Session` + `TurnContext` and delegates to `HostExecService`.
- `UnifiedExecHandler::handle()` does the same for tool-triggered execution.
- `ToolOrchestrator` approval flow maps to `HostExecApprovalPolicy::PerCommand`.
- Codex `EventMsg::ExecCommandBegin/End/Delta` are produced by the Codex `HostExecNotificationSink` implementation.

This is an extraction, not a rewrite. The canonical path keeps its semantics.

### 6. Bridging OR Path

The OR path collapses:

- `shell_command()` constructs a `HostExecContext` from `OpenRouterThreadState` and delegates to `HostExecService`.
- `drive_openrouter_shell_command()` is eliminated — the shared handler drives the process lifecycle.
- `running_commands` map is eliminated — `HostExecService` owns process registry.
- `background_terminals_clean()` delegates to `HostExecService::clean_all_for_thread()`.

## Practical Draft Shape

### Stage A: HostExecContext And HostExecNotificationSink

1. Define `HostExecContext` struct with all fields needed for execution.
2. Define `HostExecNotificationSink` trait.
3. Implement `OrNotificationSink` (wraps current `send_notification` helper).
4. Implement `CodexNotificationSink` (wraps `session.send_event`).
5. No behavioral change yet — types only.

This stage establishes the abstraction boundary without changing any runtime behavior.

### Stage B: HostExecService With OR Path Integration

1. Define `HostExecService` with process registry keyed by `(thread_id, item_id)`.
2. Implement `execute_command()` using existing `spawn_pipe_process_no_stdin()` as the initial spawn mechanism.
3. Implement streaming loop (from `drive_openrouter_shell_command` logic).
4. Implement `terminate_command()` and `clean_all_for_thread()`.
5. Refactor OR `shell_command()` to construct `HostExecContext` and delegate to `HostExecService`.
6. Eliminate `drive_openrouter_shell_command()` and `running_commands` map.

This stage gives the OR path a shared execution layer without touching the canonical path.

### Stage C: Canonical Path Delegation

1. Implement `CodexNotificationSink` mapping to `EventMsg::ExecCommand*`.
2. Refactor `execute_user_shell_command()` to construct `HostExecContext` from `Session` + `TurnContext`.
3. Map sandbox/approval configuration to `HostExecSandboxPolicy`/`HostExecApprovalPolicy`.
4. Preserve all existing canonical behavior.

This stage brings the canonical path onto the shared substrate. The canonical path must not change observable behavior.

### Stage D: Approval Integration For Non-Canonical Lanes

1. Implement `PerCommand` approval flow in `HostExecService` for non-canonical lanes.
2. Define how approval prompts are delivered to clients (through `HostExecNotificationSink` or through a separate approval channel).
3. Enable configurable sandbox selection for non-canonical lanes.

This stage gives OR and future lanes the approval and sandbox capabilities that canonical already has. It can ship after Stages A and B if needed.

## Where The Extraction Boundary Lands

```
codex-rs/
  logos-host-exec/           ← new crate
    src/
      context.rs             ← HostExecContext, policies
      notification.rs        ← HostExecNotificationSink trait
      service.rs             ← HostExecService
      process.rs             ← HostExecProcess, process registry
      errors.rs              ← HostExecError
      shell_detection.rs     ← platform-aware shell selection (or re-export from codex-core)
```

`logos-host-exec` depends on:

- `codex-protocol` (types: `ExecCommandSource`, `SandboxType`, etc.)
- `codex-sandboxing` (sandbox execution, if sandbox is enabled)
- Standard library + tokio

`logos-host-exec` does NOT depend on:

- `codex-core` (no Session, TurnContext, SessionServices)
- `logos-session-engine` (no SessionEngine)
- `codex-app-server` (no OutgoingMessageSender, ThreadStateManager)

Canonical path and OR path both depend on `logos-host-exec`.

This is an additive seam. It does not require modifying `codex-core` internals — it extracts shared logic into a new crate that both paths consume.

## Why This Still Supports The ADAPT Verdict

1. The fundamental direction (shared host/runtime ownership) is accepted.
2. The implementation path is ADAPT because existing code requires extraction, not copy-paste.
3. No provider-sensitive semantics in shell execution — the ADAPT is purely about session-model coupling, not about provider differences.
4. OpenCode and Goose shell stacks are not imported — LOGOS builds on its own existing exec primitives.
5. Sandbox and approval are preserved as non-optional capabilities, not dropped to match competitors.
6. The extraction does not require broad `codex-core` rewrite — it is an additive seam through a new crate.

## Revisit Triggers

The ADAPT verdict should be re-opened only if:

1. The extraction proves impossible without breaking canonical-path semantics or performance.
2. `Session`/`TurnContext` coupling is so deep that a clean `HostExecContext` cannot represent the required execution state.
3. Codex upstream introduces its own shared exec abstraction that LOGOS should reuse directly.
4. Performance profiling shows the abstraction layer adds unacceptable latency to shell execution.

## Closeout Expectation

The later strict spec derived from this draft should be considered successful only if:

1. OR `shell_command()` no longer owns its own process registry or spawn logic.
2. `drive_openrouter_shell_command()` is eliminated.
3. Canonical path behavior is unchanged.
4. Both paths share a single `HostExecService`.
5. Sandbox and approval are configurable through `HostExecSandboxPolicy` and `HostExecApprovalPolicy`.
6. Process registry is keyed by LOGOS identity (thread_id, item_id), not by session-local integers.
7. The new crate does not depend on `codex-core`.
8. `background_terminals_clean()` delegates to shared service.

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 05`
  lines: `267-307`
- path: `../research/session layer/RA-1.5-shell-execution.md`
  symbol: `LOGOS Current Implementation`
  lines: `1-31`
- path: `../research/session layer/RA-1.5-shell-execution.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `174-184`
- path: `../research/session layer/RA-1.5-shell-execution.md`
  symbol: `Recommendations for LOGOS`
  lines: `186-207`
- path: `2026-04-14-openrouter-thread-shell-command-support-discussion.md`
  symbol: `Ownership Baseline`
  lines: `41-46`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Ownership Layers`
  lines: `79-93`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Over-owned By The OpenRouter Lane Today`
  lines: `55-64`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `running_commands`
  lines: `81`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `shell_command`
  lines: `646-769`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `drive_openrouter_shell_command`
  lines: `2108-2283`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `background_terminals_clean`
  lines: `626-644`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_shell_command`
  lines: `3551-3617`
- path: `../../../codex-rs/core/src/tasks/user_shell.rs`
  symbol: `execute_user_shell_command`
  lines: `92-312`
- path: `../../../codex-rs/core/src/tasks/user_shell.rs`
  symbol: `UserShellCommandTask::run`
  lines: `73-89`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `UnifiedExecContext`
  lines: `72-86`
- path: `../../../codex-rs/core/src/unified_exec/mod.rs`
  symbol: `ProcessEntry`
  lines: `146-155`
- path: `../../../codex-rs/core/src/unified_exec/process_manager.rs`
  symbol: `imports`
  lines: `1-53`
- path: `../../../codex-rs/core/src/tools/orchestrator.rs`
  symbol: `ToolOrchestrator::run`
  lines: `101-199`
- path: `../../../codex-rs/core/src/state/service.rs`
  symbol: `SessionServices`
  lines: `30-61`
- path: `../../../codex-rs/core/src/exec.rs`
  symbol: `ExecParams`
  lines: `83-95`
- path: `../../../codex-rs/core/src/codex.rs`
  symbol: `run_user_shell_command`
  lines: `4851-4876`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-14-openrouter-thread-shell-command-support-discussion.md]]
- [[2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../research/session layer/RA-1.5-shell-execution.md]]
