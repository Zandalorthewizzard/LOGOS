# Item 11 Background Terminals Implementation Report

## Closed

Item 11 now closes shared LOGOS-owned process cleanup ownership for the touched OpenRouter background-terminal surface.

The shared `HostExecRegistry` in `codex_core::host_exec` is now:
1. the single owner of thread-scoped process registration, metadata, and cancellation
2. instantiated at the `CodexMessageProcessor` level (app-server layer), not inside the OpenRouter provider lane
3. passed to `OpenRouterBetaThreadManager` as a constructor dependency

The OpenRouter lane no longer creates or owns the registry. It receives the shared registry as a dependency and targets it for cleanup.

The registry now tracks `HostExecProcessEntry` metadata (call_id, process_id, command, cwd, source) alongside cancellation tokens, supports `thread_processes()` queries, and auto-prunes on command completion via `unregister()`.

The `thread/backgroundTerminals/clean` contract is preserved: the method still returns `ThreadBackgroundTerminalsCleanResponse {}` after cancelling all processes for the target thread.

## Still Explicitly Unsupported

1. Full PTY lifecycle (resize, scrollback, reconnect, attach/detach)
2. Persistent process recovery across app-server restart
3. Stdin/write interaction for non-canonical lanes through the shared registry
4. Bounded output buffering for background processes
5. Interrupt semantics beyond cancellation-token cancellation
6. Steering and rollback for running commands
7. Approval/sandbox re-architecture for the touched surface

## Blocked

None. The slice completed without blockers.

## Borrowed / Reused

### Local Codex / Codex App Server Reuse

1. `codex_core::host_exec::HostExecRegistry` (Item 05 seam)
   Extended with process metadata entries instead of creating a second independent process manager.
2. `codex_core::host_exec::run_one_shot_shell_command`
   Continues to be the shared one-shot shell runner. Now registers richer `HostExecProcessEntry` entries.
3. `codex_core::exec::execute_exec_request`
   The underlying execution substrate remains unchanged.
4. `OpenRouterBetaThreadManager::background_terminals_clean`
   Already delegated to `host_exec_registry.cancel_thread()` from Item 05. Now targets the promoted shared registry.

## Adapted In LOGOS

1. Extended `HostExecRegistry::register` to accept `HostExecProcessEntry` instead of bare `(call_id, CancellationToken)`.
2. Added `HostExecProcessEntry` struct with call_id, process_id, command, cwd, source, and cancellation_token fields.
3. Added `HostExecRegistry::thread_processes(thread_id)` query method for process metadata visibility.
4. Added `HostExecProcessEntry::is_cancelled()` convenience method.
5. Promoted `HostExecRegistry` instantiation from `OpenRouterBetaThreadManager::new()` to `CodexMessageProcessor::new()`.
6. `OpenRouterBetaThreadManager::new()` now accepts `Arc<HostExecRegistry>` as a constructor parameter instead of creating one internally.
7. Added `host_exec_registry: Arc<HostExecRegistry>` field on `CodexMessageProcessor` for future provider-lane reuse.
8. Added `openrouter_beta_threads: OpenRouterBetaThreadManager` and `host_exec_registry` fields on `CodexMessageProcessor` and `ListenerTaskContext`.

## Stale Code Removed

1. `OpenRouterBetaThreadManager::new(codex_home)` creating its own `HostExecRegistry::default()` internally.
   Replaced by `new(codex_home, host_exec_registry)` accepting the shared instance.
2. The `host_exec_registry` field is no longer created inside `OpenRouterBetaThreadManager::new()`. The field still exists but is now populated from the shared app-server-level instance.

## Stale Code Still Remaining

1. `OpenRouterBetaThreadManager::background_terminals_clean`
   Still present as the entry point for the OpenRouter cleanup route. It validates thread existence and delegates to `host_exec_registry.cancel_thread()`. This is correct runtime-validation glue, not owner logic.
2. `OpenRouterBetaThreadManager::shell_command`
   Still present as lane-specific validation, turn attachment orchestration, and shared runner spawning. This is correct lane orchestration glue.
3. Canonical `close_unified_exec_processes()` path
   Untouched. The canonical path still owns its own `UnifiedExecProcessManager` for interactive exec processes. The shared `HostExecRegistry` covers the one-shot shell-command and cleanup surface only.

## Changed Crates / Files

1. `codex-rs/core/src/host_exec.rs`
   - Added `HostExecProcessEntry` struct
   - Extended `HostExecRegistry::register` to accept `HostExecProcessEntry`
   - Added `HostExecRegistry::thread_processes` method
   - Updated `run_one_shot_shell_command` to register process metadata

2. `codex-rs/core/src/host_exec_tests.rs`
   - Updated existing `host_exec_registry_cancels_all_commands_for_thread` test
   - Added `host_exec_registry_tracks_process_metadata` test
   - Added `host_exec_registry_auto_prunes_on_unregister` test
   - Added `host_exec_registry_thread_processes_isolation` test

3. `codex-rs/app-server/src/openrouter_beta_threads.rs`
   - Changed `OpenRouterBetaThreadManager::new` to accept `Arc<HostExecRegistry>` parameter
   - Updated `Default` impl to match new signature

4. `codex-rs/app-server/src/codex_message_processor.rs`
   - Added `host_exec_registry: Arc<HostExecRegistry>` field to `CodexMessageProcessor`
   - Added `openrouter_beta_threads: OpenRouterBetaThreadManager` field to `CodexMessageProcessor`
   - Added `openrouter_beta_threads` field to `ListenerTaskContext`
   - Added `openrouter_beta_thread_id` helper method
   - Promoted `HostExecRegistry` creation to `CodexMessageProcessor::new()`
   - Passed shared registry to `OpenRouterBetaThreadManager::new()`

5. `docs-new/07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md`
   - Updated SEAM-2026-04-18-04 with Item 11 extensions (process metadata, thread_processes query)

## Tests / Checks Run

1. `cargo test -p codex-core host_exec` — 8 passed (4 new Item 11 tests + 4 existing)
2. `cargo test -p codex-app-server-protocol` — 143 passed (protocol contract unchanged)
3. `cargo test -p codex-app-server openrouter_background_terminal` — 1 passed (cleanup still works through promoted shared registry)
4. `cargo fmt -p codex-core -p codex-app-server` — clean
5. `cargo clippy --fix --tests --allow-dirty -p codex-core --all-targets -- -D warnings` — clean
6. `cargo clippy --fix --tests --allow-dirty -p codex-app-server --all-targets -- -D warnings` — only pre-existing `expect_used` / `too_many_arguments` warnings in untouched code

`just` recipes were unavailable in this Windows session. Used `cargo fmt` and `cargo clippy --fix` as honest equivalents.

## Verdict

**slice closed**

## Explicit Answers

1. **What is now the steady-state owner of background-terminal/process cleanup for the touched OpenRouter surface?**
   `HostExecRegistry` in `codex_core::host_exec`, instantiated at the `CodexMessageProcessor` level and passed to the OpenRouter lane as a dependency.

2. **Does OpenRouter still own any final cleanup loop or process registry for this surface?**
   No. `OpenRouterBetaThreadManager` receives the shared registry via constructor. Its `background_terminals_clean` method validates thread existence and delegates to the shared registry's `cancel_thread()`. It does not own any process bookkeeping or cleanup loop.

3. **Did this slice require a new Codex-base seam beyond the existing `host_exec` seam?**
   No. The existing SEAM-2026-04-18-04 was extended with `HostExecProcessEntry` and `thread_processes()`. No new seam was needed.

4. **Did any touched path require TUI changes?**
   No. The TUI contract (`thread/backgroundTerminals/clean` request/response, `TerminalInteractionNotification`) remains unchanged. No TUI files were touched.

5. **What exact background-terminal capabilities still remain out of scope after this slice?**
   Full PTY lifecycle (resize, scrollback, reconnect), persistent process recovery, stdin/write interaction for non-canonical lanes, bounded output buffering, interrupt semantics, steering, rollback, and approval/sandbox re-architecture.

## Residual Risks

### Adjacent-Open Surfaces

1. Full PTY lifecycle (resize, scrollback, reconnect, attach/detach) — still open, out of scope.
2. Stdin/write interaction for non-canonical lanes — still open, out of scope.
3. Persistent process recovery across restart — still open, out of scope.
4. Interrupt semantics — still open (Item 10).
5. Steering — still open (Item 12).
6. Approval/sandbox posture for the touched surface — still weaker than canonical (unsandboxed, skip approval).

### Transitional Residue

1. `OpenRouterBetaThreadManager::background_terminals_clean` is runtime-validation glue, not owner logic. It validates thread existence and delegates. This is correct and not stale.
2. `CodexMessageProcessor::host_exec_registry` is stored but currently only consumed by the OR manager. Future provider lanes will use it. The `#[allow(dead_code)]` annotation marks this explicitly.
3. The canonical `UnifiedExecProcessManager` remains the richer process owner for interactive exec. No merge is planned.

### Seam-Expansion Risk

1. SEAM-2026-04-18-04 was extended with process metadata and query. The extension is narrow and additive. Future work should resist piling unrelated provider glue into `host_exec`.
2. If upstream later exposes equivalent public process-lifecycle substrate, this seam should be retired in favor of the upstream surface.

### Weaker Semantics That Remain Explicit

1. The touched OpenRouter shell-command path still runs unsandboxed with skip-approval. This is not upgraded by Item 11.
2. Process metadata tracking is limited to what `HostExecProcessEntry` exposes (call_id, process_id, command, cwd, source). It does not include PID, exit status history, or output buffering.
3. The shared registry covers only the touched one-shot shell-command surface. It does not track canonical `UnifiedExecProcessManager` processes.
