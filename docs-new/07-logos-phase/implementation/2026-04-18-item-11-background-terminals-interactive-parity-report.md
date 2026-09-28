# Item 11 Background Terminals Interactive Parity Implementation Report

## Closed

Item 11 now closes the first interactive background-terminal parity wave for the touched OpenRouter surface.

The shared `HostExecRegistry` in `codex_core::host_exec` now supports:

1. **Interactive process spawning** via `spawn_interactive_process()` which reuses `codex_utils_pty::spawn_pty_process` — the same PTY spawning primitive used by the canonical `UnifiedExecProcessManager` and `command_exec.rs`.
2. **`write_stdin`** through `HostExecRegistry::write_stdin(thread_id, call_id, data)`.
3. **`resize`** through `HostExecRegistry::resize_process(thread_id, call_id, rows, cols)`.
4. **Output buffering** via `InteractiveProcessState::output_buffer` with async notify and `HostExecRegistry::read_buffered_output(thread_id, call_id)`.
5. **Output streaming** via `HostExecNotificationSink::emit_delta()` during the background output loop.
6. **`TerminalInteractionNotification` routing** via `HostExecNotificationSink::emit_terminal_interaction()` → `HostExecTerminalInteraction`.
7. **Process visibility** via existing `HostExecRegistry::thread_processes()` now returning entries with `started_at`, `is_interactive()`, and `interactive_state()` metadata.
8. **`/stop` cleanup** via existing `HostExecRegistry::cancel_thread()` now also calling `request_terminate()` on interactive process handles.
9. **Auto-cleanup on exit** via the spawned background task which unregisters from the registry on process exit.

The `InteractiveProcessState` wraps a `codex_utils_pty::ProcessHandle` (the same type used by the canonical `UnifiedExecProcessManager` for local PTY sessions) in an `Arc` for shared ownership between the registry and the background output/exit watcher task.

## Still Explicitly Unsupported

1. Persistent process recovery across app-server restart
2. Full scrollback UX redesign
3. Attach/detach redesign beyond what the shared owner exposes
4. Interrupt semantics beyond cancellation-token cancellation and process termination
5. Steering and rollback for running commands
6. Approval/sandbox re-architecture for the touched surface
7. Full PTY feature parity (alternate screen, signal forwarding beyond terminate, rich terminal emulation)
8. TUI-side `/ps` consumption of the new app-server list surface
9. `closeStdin` handling on `thread/backgroundTerminal/writeStdin`

## Blocked

None. The slice completed without blockers.

## Borrowed / Reused

### Local Codex PTY Primitives (Reuse)

1. `codex_utils_pty::spawn_pty_process` — the same PTY spawning function used by `command_exec.rs` and indirectly by `UnifiedExecProcessManager::open_session_with_exec_env`.
2. `codex_utils_pty::ProcessHandle` — the same process handle type used by `command_exec.rs` and `UnifiedExecProcess`. Provides `writer_sender()`, `resize()`, `request_terminate()`, `close_stdin()`.
3. `codex_utils_pty::TerminalSize` — the same terminal size type used throughout.
4. `maybe_wrap_shell_lc_with_snapshot` — shared shell command wrapping from `core/src/tools/runtimes/`.

### Local Codex Output Pattern (Reuse)

5. The output buffering pattern (Arc<Mutex<Vec<u8>>> + Arc<Notify>) mirrors the canonical `UnifiedExecProcess::output_buffer` / `output_notify` pattern from `core/src/unified_exec/process.rs`.

### Local Codex Notification Pattern (Reuse)

6. `HostExecNotificationSink::emit_begin/delta/end` — unchanged, reused from Item 05.
7. `HostExecNotificationSink::emit_terminal_interaction` — follows the canonical `TerminalInteractionEvent` emission pattern from `core/src/tools/handlers/unified_exec.rs:345-352`.

## Adapted In LOGOS

1. Added `InteractiveProcessState` struct in `host_exec.rs` wrapping `Arc<ProcessHandle>`, output buffer, output notify, and exited flag.
2. Added `InteractiveHostExecContext` struct for interactive process spawn parameters.
3. Added `spawn_interactive_process()` function that spawns PTY, registers in the shared registry, and runs a background output/exit watcher task.
4. Extended `HostExecProcessEntry` with `started_at: i64`, `interactive: Option<Arc<InteractiveProcessState>>`, `is_interactive()`, and `interactive_state()` accessors.
5. Extended `HostExecRegistry::cancel_thread()` to also call `request_terminate()` on interactive process handles.
6. Added `HostExecRegistry::write_stdin()`, `resize_process()`, `read_buffered_output()`, `get_entry()`.
7. Added `HostExecTerminalInteraction` struct and `HostExecNotificationSink::emit_terminal_interaction()` trait method.
8. Updated `CodexHostExecSink` in `core/src/tasks/user_shell.rs` to implement `emit_terminal_interaction()` by emitting canonical `TerminalInteractionEvent`.
9. Added app-server protocol types and methods for `thread/backgroundTerminal/start`, `thread/backgroundTerminal/writeStdin`, and `thread/backgroundTerminals/list`.
10. Added app-server routing in `codex_message_processor.rs` for the new interactive background-terminal surface.
11. Added `OpenRouterBetaThreadManager::background_terminal_start()` as the first non-canonical consumer of the shared interactive owner.
12. Updated `OpenRouterHostExecSink` in `app-server/src/openrouter_beta_threads.rs` to implement `emit_terminal_interaction()` by emitting `ServerNotification::TerminalInteraction`.
13. Narrowed `thread/backgroundTerminals/list` to interactive entries and made `BackgroundTerminalInfo.process_id` required on that product surface.
14. Extended `RecordingSink` in tests with `emit_terminal_interaction` support and `interactions` tracking.

## Stale Code Removed

None in this slice. The first wave (cleanup-only) already removed `running_commands` and `drive_openrouter_shell_command`. This slice is additive.

## Stale Code Still Remaining

1. `OpenRouterBetaThreadManager::background_terminals_clean`
   Still present as the entry point for the OpenRouter cleanup route. It validates thread existence and delegates to `host_exec_registry.cancel_thread()`. This is correct runtime-validation glue, not owner logic.
2. `OpenRouterBetaThreadManager::shell_command`
   Still present as lane-specific validation, turn attachment orchestration, and shared runner spawning. This is correct lane orchestration glue.
3. Canonical `close_unified_exec_processes()` path
   Untouched. The canonical path still owns its own `UnifiedExecProcessManager` for interactive exec processes. The shared `HostExecRegistry` covers the OpenRouter surface.

## Changed Crates / Files

1. `codex-rs/core/src/host_exec.rs`
   - Added `InteractiveProcessState` with `write_stdin`, `resize`, `read_buffered_output`, `has_exited`, `request_terminate`
   - Added `InteractiveHostExecContext` struct
   - Added `spawn_interactive_process()` function
   - Extended `HostExecProcessEntry` with `started_at`, `interactive`, `is_interactive()`, `interactive_state()`
   - Extended `HostExecRegistry` with `write_stdin`, `resize_process`, `read_buffered_output`, `get_entry`
   - Extended `cancel_thread` to terminate interactive processes
   - Added `HostExecTerminalInteraction` struct
   - Extended `HostExecNotificationSink` with `emit_terminal_interaction`
   - Added imports: `AtomicBool`, `Ordering`, `anyhow`, `Notify`, `ProcessHandle`, `SpawnedProcess`, `TerminalSize`

2. `codex-rs/core/src/host_exec_tests.rs`
   - Updated all existing test entries to include new `started_at` and `interactive` fields
   - Added `host_exec_registry_write_stdin_resize_require_interactive` test
   - Added `host_exec_registry_write_stdin_resize_unknown_process` test
   - Updated `RecordingSink` with `emit_terminal_interaction` implementation and `interactions` field
   - Added `HostExecTerminalInteraction` import

3. `codex-rs/core/src/tasks/user_shell.rs`
   - Added `emit_terminal_interaction` implementation to `CodexHostExecSink`

4. `codex-rs/app-server-protocol/src/protocol/common.rs`
   - Added additive protocol methods for background terminal start, writeStdin, and list

5. `codex-rs/app-server-protocol/src/protocol/v2.rs`
   - Added `ThreadBackgroundTerminalStartParams/Response`
   - Added `ThreadBackgroundTerminalWriteStdinParams/Response`
   - Added `ThreadBackgroundTerminalsListParams/Response`
   - Added `BackgroundTerminalInfo`
   - Regenerated vendored schema fixtures after the protocol shape change

6. `codex-rs/app-server/src/codex_message_processor.rs`
   - Added request routing for background terminal start, writeStdin, and list
   - Added server-side `/ps` bridge using `host_exec_registry.thread_processes(thread_id)`
   - Emits `TerminalInteractionNotification` for the writeStdin path

7. `codex-rs/app-server/src/openrouter_beta_threads.rs`
   - Added `TerminalInteractionNotification` import
   - Added `emit_terminal_interaction` implementation to `OpenRouterHostExecSink`
   - Added `background_terminal_start()` that invokes `spawn_interactive_process()` through the shared owner

8. `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs`
   - Added end-to-end consumer proof for start, writeStdin/poll, and list

9. `docs-new/07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md`
    - Extended SEAM-2026-04-18-04 with interactive wave public surface additions (items 6-14)

## Tests / Checks Run

1. `cargo test -p codex-core host_exec` — 10 passed (4 new interactive tests + 6 existing)
2. `cargo test -p codex-app-server-protocol` — 143 passed after regenerating vendored schema fixtures for the additive protocol change
3. `cargo test -p codex-app-server openrouter_background_terminal` — 4 passed (`start`, `writeStdin`/poll, `list`, `clean`)
4. `cargo test -p codex-app-server openrouter_shell` — 3 passed (shell command paths no regression)
5. `cargo fmt` in `codex-rs` — completed; `rustfmt` emitted existing nightly-only `imports_granularity = Item` warnings from repo config on stable toolchain
6. `cargo clippy -p codex-app-server-protocol --tests --all-targets -- -D warnings` — blocked by pre-existing `clippy::large_enum_variant` on `ServerNotification` in `app-server-protocol/src/protocol/common.rs`
7. `cargo clippy -p codex-core --tests --all-targets -- -D warnings` — blocked transitively by the same pre-existing `codex-app-server-protocol` lint when building the workspace graph
8. `cargo clippy -p codex-app-server --tests --all-targets -- -D warnings` — blocked by pre-existing `clippy::expect_used` in untouched `logos-session-state/src/registry.rs` and the same pre-existing `codex-app-server-protocol` lint

`just fmt` was unavailable in this Windows session because `just` could not locate its configured shell, so `cargo fmt` was used as the honest fallback.

## Verdict

**slice closed**

## Explicit Answers

1. **Does the touched OpenRouter surface now support `write_stdin` through the shared owner?**
   Yes. `HostExecRegistry::write_stdin(thread_id, call_id, data)` sends stdin bytes through the shared `InteractiveProcessState` which delegates to `ProcessHandle::writer_sender()`.

2. **Is `/ps` now backed by shared owner truth?**
   At the app-server/runtime layer, yes. `thread/backgroundTerminals/list` now returns interactive background terminals from `HostExecRegistry::thread_processes()` via a real product RPC. TUI consumption of that RPC remains the explicit remainder.

3. **Is output buffering/streaming now available for the touched interactive surface?**
   Yes. `InteractiveProcessState` buffers output in an `Arc<Mutex<Vec<u8>>>` and streams it via `emit_delta()` notifications. `read_buffered_output()` drains the buffer.

4. **Is `TerminalInteractionNotification` now emitted for the touched interactive OpenRouter terminal use?**
   Yes. The `HostExecNotificationSink::emit_terminal_interaction()` method was added and the `OpenRouterHostExecSink` routes it to `ServerNotification::TerminalInteraction`.

5. **Does `/stop` now route through shared owner truth?**
   Yes. `HostExecRegistry::cancel_thread()` cancels cancellation tokens AND calls `request_terminate()` on interactive process handles. The OpenRouter `background_terminals_clean` delegates to this shared method.

6. **Does the touched surface support `resize`?**
   Yes. `HostExecRegistry::resize_process(thread_id, call_id, rows, cols)` delegates to `InteractiveProcessState::resize()` which calls `ProcessHandle::resize()` — the same resize primitive used by the canonical `command_exec.rs`.

7. **Did this slice require a new Codex-base seam?**
   No. The existing SEAM-2026-04-18-04 was extended with interactive process support. No new seam was needed.

8. **Did any touched path require TUI changes?**
   No. The TUI contract (`thread/backgroundTerminals/clean`, `TerminalInteractionNotification`) remains unchanged. No TUI files were touched.

9. **What exact parity remainder still remains after this slice?**
   - TUI-side adoption of `thread/backgroundTerminals/list` for `/ps`
   - Persistent process recovery across app-server restart
   - Full scrollback UX redesign
   - Attach/detach redesign
   - Interrupt semantics beyond terminate
   - `closeStdin` handling on `thread/backgroundTerminal/writeStdin`
   - Signal forwarding beyond terminate
   - Full PTY feature parity (alternate screen, rich terminal emulation)

## Residual Risks

### Adjacent-Open Surfaces

1. TUI `/ps` still needs to consume `thread/backgroundTerminals/list`; the app-server/runtime bridge exists, but the user-facing client path is not yet switched.
2. `thread/backgroundTerminal/writeStdin` currently supports data writes and empty-input polling but ignores `closeStdin`; treating it as full canonical parity would overclaim the touched surface.
3. Persistent process recovery across restart — still open.
4. Interrupt semantics — still open (Item 10).
5. Steering — still open (Item 12).
6. Approval/sandbox posture for the touched surface — still weaker than canonical (unsandboxed, skip approval).

### Transitional Residue

1. `OpenRouterBetaThreadManager::background_terminals_clean` is runtime-validation glue, not owner logic. Correct and not stale.
2. `CodexMessageProcessor::host_exec_registry` is stored and consumed by the OR manager. Future provider lanes will use it.
3. The canonical `UnifiedExecProcessManager` remains the richer process owner for canonical interactive exec. No merge is planned.

### Seam-Expansion Risk

1. SEAM-2026-04-18-04 was extended with interactive process lifecycle support. The extension is additive and reuses `codex_utils_pty` primitives already used by the canonical path. Future work should resist piling unrelated provider glue into `host_exec`.

### Weaker Semantics That Remain Explicit

1. The touched OpenRouter shell-command path still runs unsandboxed with skip-approval. This is not upgraded by Item 11.
2. Interactive process state is limited to the `InteractiveProcessState` surface: write_stdin, resize, buffered output, terminate. It does not include signal forwarding, PTY slave handle, or full `UnifiedExecProcess` feature set.
3. The shared registry covers the touched one-shot and interactive surface. It does not track canonical `UnifiedExecProcessManager` processes.
