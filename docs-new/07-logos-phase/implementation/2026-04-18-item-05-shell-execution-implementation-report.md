# Item 05 Shell Execution Implementation Report

## Closed

Item 05 now implements `thread/shellCommand` parity for the OpenRouter beta lane through a shared LOGOS-owned host-exec substrate in `codex-rs/core/src/host_exec.rs`.

The touched product surface now shares one one-shot shell runner across:

1. the preserved canonical Codex `thread/shellCommand` path via `core/src/tasks/user_shell.rs`
2. the OpenRouter beta `thread/shellCommand` path via `app-server/src/openrouter_beta_threads.rs`

The request/response/notification contract stays unchanged:

1. `thread/shellCommand`
2. `ThreadShellCommandParams`
3. `ThreadShellCommandResponse`
4. `CommandExecutionOutputDeltaNotification`

OpenRouter now returns `{}` immediately, streams progress through notifications, and persists command output into thread history in both standalone-turn and active-turn modes.

## Borrowed / Reused

### Local Codex / Codex App Server Reuse

1. `codex-rs/core/src/exec.rs`
   The shared runner delegates actual process execution to `execute_exec_request()` so the mature Codex stdout/stderr capture, output delta streaming, output capping, and cancellation-backed process termination remain the execution core.
2. `codex-rs/core/src/tasks/user_shell.rs`
   Canonical `thread/shellCommand` now bridges into the shared runner instead of owning a separate one-shot flow.
3. `codex-rs/core/src/tools/runtimes/mod.rs`
   Shared shell snapshot wrapping continues through `maybe_wrap_shell_lc_with_snapshot()`.
4. `codex-rs/app-server/src/bespoke_event_handling.rs`
   Canonical app-server notification mapping remains unchanged because the Codex sink still emits the same `ExecCommand*` events.
5. `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs`
   Existing canonical contract tests remained the no-regression proof.

### External Guardrails Used

1. OpenCode shell guardrails remained reference-only; no external shell stack was imported.
2. Goose approval/sandbox vocabulary remained reference-only; no Goose subprocess implementation was imported.

## Adapted In LOGOS

1. Added `codex_core::host_exec` as a narrow shared host-exec seam.
2. Added explicit `HostExecContext`, `HostExecNotificationSink`, and `HostExecRegistry` boundaries.
3. Rewired canonical `execute_user_shell_command()` to construct a provider-agnostic host-exec context and delegate to `run_one_shot_shell_command()`.
4. Rewired OpenRouter `shell_command()` to keep turn attachment/orchestration local while delegating steady-state execution, output delta streaming, and item lifecycle notifications to the shared runner.
5. Added OpenRouter persistence assertions so standalone-turn and active-turn shell commands now prove thread-history persistence, not just live notifications.

## Stale Code Removed

1. `OpenRouterBetaThread::running_commands`
   Removed. Thread-scoped shell cancellation for this surface now lives in `HostExecRegistry`.
2. `drive_openrouter_shell_command`
   Removed.
3. provider-lane direct use of `spawn_pipe_process_no_stdin()` for `thread/shellCommand`
   Removed.
4. provider-lane manual stdout/stderr streaming loop for this surface
   Removed with `drive_openrouter_shell_command()`.
5. provider-lane direct `item/started`, `item/commandExecution/outputDelta`, and `item/completed` choreography for shell-command execution
   Removed from `OpenRouterBetaThreadManager::shell_command()` and replaced by `OpenRouterHostExecSink` over the shared runner.

## Stale Code Still Remaining

1. `OpenRouterBetaThreadManager::shell_command`
   Still present, but now reduced to lane-specific validation, active-turn vs standalone-turn attachment, and spawning the shared host-exec run.
2. `thread/backgroundTerminals/clean` on the OpenRouter lane still exists as transitional Item 11 residue.
   It now cancels thread-scoped shared host-exec tokens for this touched one-shot shell surface only.
   It is not a full shared background-terminal/process-lifecycle closure.
3. Standalone-turn `turn/started` and `turn/completed` emission still happens in the OpenRouter lane.
   That residue is turn-lifecycle orchestration, not shell execution ownership.

## Tests / Checks Run

1. `cargo test -p codex-app-server-protocol`
2. `cargo test -p codex-app-server thread_shell_command -- --nocapture`
3. `cargo test -p codex-app-server app_server_openrouter_shell_command -- --nocapture`
4. `cargo test -p codex-app-server app_server_openrouter_background_terminal_cleanup_stops_running_shell -- --nocapture`
5. `cargo test -p codex-core host_exec -- --nocapture`

6. `just fix -p codex-core`
   Could not run in this Windows session because the `just` recipe shell was unavailable here. Used the honest equivalent `cargo clippy --fix --tests --allow-dirty -p codex-core --all-targets -- -D warnings` instead.
7. `just fix -p codex-app-server`
   Could not run in this Windows session because the `just` recipe shell was unavailable here. Used the honest equivalent `cargo clippy --fix --tests --allow-dirty -p codex-app-server --all-targets -- -D warnings` instead. That pass still reported pre-existing untouched `clippy::too_many_arguments` / `clippy::expect_used` warnings elsewhere in `openrouter_beta_threads.rs`.
8. `just fmt`
   Could not run in this Windows session because the `just` recipe shell was unavailable here. Used `cargo fmt` instead.

## Residual Risks

1. The new shared runner is intentionally scoped to one-shot `thread/shellCommand`; it does not close Item 11 general background-terminal ownership.
2. OpenRouter standalone turn completion still lives in lane orchestration, so future turn-lifecycle extraction work could further shrink the lane.
3. `HostExecRegistry` currently tracks only the touched shell-command surface, not all future host-exec surfaces.

## Explicit Answers

1. What is now the steady-state owner of `thread/shellCommand` execution for OpenRouter?
   `codex_core::host_exec::run_one_shot_shell_command()` plus `HostExecRegistry`.
2. Was `drive_openrouter_shell_command()` removed?
   Yes.
3. Did this slice require a Codex-base seam?
   Yes. The new shared public `codex_core::host_exec` seam was added and registered in the seam index.
4. What Item 11 residue, if any, still remains?
   `thread/backgroundTerminals/clean` now cancels shared host-exec tokens for this one-shot shell surface, but general background-terminal/process-lifecycle extraction is still not implemented.
5. What are the actual sandbox/approval semantics of the touched OpenRouter shell-command path after this slice?
   The touched OpenRouter `thread/shellCommand` path runs unsandboxed with full access and skips approvals, matching the existing product contract for this surface.
6. Is Item 05 honestly closed after this slice, or still not?
   Closed for the bounded `thread/shellCommand` parity goal in this spec. Item 11 broader background-terminal ownership remains open.
