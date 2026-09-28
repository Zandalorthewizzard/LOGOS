---
id: 2026-04-17-task-02-openrouter-shell-runtime-report
title: "Task 02 OpenRouter Shell Runtime Blocker Report"
type: implementation-report
status: completed
tags: [logos, phase-6, openrouter, shell-execution, task-02, implementation-report]
related:
  - "[[../../docs-new/07-logos-phase/architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md]]"
  - "[[../../docs-new/07-logos-phase/architecture/2026-04-14-openrouter-thread-shell-command-support-discussion.md]]"
  - "[[../../docs-new/02-guides/operations/2026-04-14-orchestrated-agent-loop.md]]"
  - "[[../../docs-new/02-guides/operations/2026-04-14-glm-5.1-agent-profile.md]]"
generated: false
source_of_truth: non-canonical
---

# Task 02: OpenRouter Shell Runtime Blocker Report

## Scope

This report covers the bounded blocker-fix slice from the current OpenRouter beta pass:

1. Fast-exit `thread/shellCommand` commands on the OpenRouter beta app-server path were hanging after `item/started`.
2. The closure target was the OpenRouter shell runtime path only.
3. Shared `Item 05` shell-execution extraction was explicitly out of scope.
4. This report does not claim the final shared shell-exec architecture is complete.

## 1. Closed

- Added a dedicated output-forwarding helper in `codex-rs/app-server/src/process_output.rs`.
  - evidence: `codex-rs/app-server/src/process_output.rs:8-74`
- Replaced the hand-rolled OpenRouter shell `stdout/stderr/exit` loop with split output forwarders plus a post-exit drain window in `drive_openrouter_shell_command()`.
  - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:2600-2673`
- Kept OpenRouter-owned turn/item semantics, persistence, and `running_commands` cleanup in the OpenRouter lane while changing only output/exit orchestration.
  - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:2679-2771`
- Registered the new helper module from app-server crate root.
  - evidence: `codex-rs/app-server/src/lib.rs:78-81`
- Tightened the canonical thread-shell test so it assembles `item/commandExecution/outputDelta` chunks instead of assuming one newline-inclusive delta.
  - evidence: `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:41-115`
  - evidence: `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:401-417`
- Tightened the OpenRouter inline review test so it waits for the async turn to finish before treating the slice as complete.
  - evidence: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs:1112-1177`
- Resolved the bounded blocker on the validated path: fast-exit OpenRouter shell commands now emit output and reach `item/completed`, and standalone turns reach `turn/completed`.
  - evidence: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs:640-729`
  - evidence: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs:733-823`
  - evidence: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs:1112-1177`

## 2. Borrowed From References

1. Split per-stream output readers plus post-exit drain timeout.
   - source project: `LOGOS local codex-app-server canonical command_exec lane (Codex-derived)`
   - local snapshot file path: `codex-rs/app-server/src/command_exec.rs`
   - line range: `442-552`
   - pattern or code fragment borrowed: wait for process exit, then start `IO_DRAIN_TIMEOUT_MS`, await the stream readers, and only then finalize completion output.
2. Chunk coalescing before forwarding output downstream.
   - source project: `LOGOS local codex-app-server canonical command_exec lane (Codex-derived)`
   - local snapshot file path: `codex-rs/app-server/src/command_exec.rs`
   - line range: `566-620`
   - pattern or code fragment borrowed: each reader task coalesces adjacent chunks with `try_recv()` while under `OUTPUT_CHUNK_SIZE_HINT` before forwarding or buffering.

No external research snapshot or third-party runtime was borrowed in this slice; the borrow source for this fix was the local canonical `command_exec` implementation above.

## 3. Adapted / Renamed In LOGOS

1. Extracted split-output forwarding into a new helper module.
   - destination LOGOS file path: `codex-rs/app-server/src/process_output.rs`
   - new symbol or owner name: `spawn_split_output_forwarders`, `ForwardedProcessOutput`, `ProcessOutputStream`
   - what changed from the original: instead of returning final `stdout` / `stderr` strings or emitting canonical `command/exec` notifications directly, the helper returns a forwarded chunk stream that the OpenRouter lane consumes.
   - why the change was required: OpenRouter owns `item/commandExecution/outputDelta`, aggregated item output, and thread/turn completion semantics; the helper had to stay below those lane-specific behaviors.
2. Reused the exit-first plus post-exit-drain orchestration inside the OpenRouter shell driver without adopting canonical command-exec ownership.
   - destination LOGOS file path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
   - new symbol or owner name: `drive_openrouter_shell_command`
   - what changed from the original: the OpenRouter path now uses split output forwarders and a drain timeout, but keeps `running_commands` cleanup, persistence, `item/completed`, and `turn/completed` in the OpenRouter-owned path.
   - why the change was required: the bounded slice had to fix the hang without widening into shared `Item 05` host-exec extraction, `Session` reuse, or `codex-core` intervention.

## 4. Rejected Or Not Taken

1. Treating Windows shell flags as the real fix was rejected.
   - `use_login_shell = !cfg!(windows)` and PowerShell `-NonInteractive` remain in the command builder, but the hang reproduced until the runtime lifecycle changed, so this report does not credit those flags as the blocker fix.
2. Relying only on a small post-exit drain around the old OpenRouter loop was rejected.
   - The earlier proving attempt did not remove the hang; the final fix required split output forwarders plus coordinated drain behavior.
3. Keeping OpenRouter on the old combined-receiver hand-rolled path was rejected.
   - The final patch removed the OpenRouter dependency on the old `stdout/stderr/exit` loop in favor of the extracted forwarding helper.
4. Direct reuse of canonical `run_command()` / `execute_user_shell_command()` was not taken in this slice.
   - That path would widen the change into shared `Item 05` host-exec extraction and change ownership boundaries beyond the bounded blocker fix.

## 5. Stale Code Removed

1. Removed the OpenRouter-specific hand-rolled combined `stdout/stderr/exit` orchestration from `drive_openrouter_shell_command()`.
   - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:2600-2673`
2. Removed the stale test assumption that one `item/commandExecution/outputDelta` always carries the whole newline-inclusive output.
   - evidence: `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:97-115`
   - evidence: `codex-rs/app-server/tests/suite/v2/thread_shell_command.rs:401-417`
3. Removed the stale test assumption that inline review completion is synchronous with the `review/start` RPC response.
   - evidence: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs:1147-1177`

## 6. Stale Code Still Remaining

1. Windows-specific shell builder tweaks still remain in the OpenRouter path.
   - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:1082-1087`
   - why still remaining: this slice did not prove whether `use_login_shell = !cfg!(windows)` or PowerShell `-NonInteractive` are still required after the runtime fix.
2. OpenRouter shell execution still duplicates ownership relative to the canonical command-exec substrate.
   - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:2600-2771`
   - evidence: `codex-rs/app-server/src/command_exec.rs:442-620`
   - why still remaining: shared `Item 05` host-exec extraction was explicitly out of scope for this bounded blocker pass.
3. OpenRouter shell commands still do not reuse canonical approval, sandbox, `Session`, or `TurnContext` shell-exec plumbing.
   - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:1082-1163`
   - why still remaining: adopting those owners would widen the slice beyond the narrow runtime hang fix.
4. Notification delivery still differs by lane.
   - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs:2732-2769`
   - evidence: `codex-rs/app-server/src/command_exec.rs:606-618`
   - why still remaining: canonical `command_exec` waits on notification delivery, while the OpenRouter lane still emits its own app-server notifications directly.

## 7. Tests / Checks Run

1. `cargo fmt --all`
   - result: passed
2. `cargo test -p codex-app-server --test all --no-run`
   - result: passed
   - note: produced `target/debug/deps/all-4d09b532b1bd0be9.exe` for focused test execution
3. `target/debug/deps/all-4d09b532b1bd0be9.exe --exact suite::v2::thread_shell_command::thread_shell_command_runs_as_standalone_turn_and_persists_history --nocapture`
   - result: passed
4. `target/debug/deps/all-4d09b532b1bd0be9.exe --exact suite::v2::openrouter_beta::app_server_openrouter_shell_command_runs_as_standalone_turn --nocapture`
   - result: passed
5. `target/debug/deps/all-4d09b532b1bd0be9.exe --exact suite::v2::openrouter_beta::app_server_openrouter_shell_command_uses_existing_active_turn --nocapture`
   - result: passed
6. `target/debug/deps/all-4d09b532b1bd0be9.exe --exact suite::v2::openrouter_beta::app_server_openrouter_review_start_inline_succeeds --nocapture`
   - result: passed
7. `target/debug/deps/all-4d09b532b1bd0be9.exe suite::v2::openrouter_beta::`
   - result: `21 passed, 0 failed`

## 8. Residual Risks

1. The bounded blocker is fixed, but the final shared shell-exec architecture is still not proven; `Item 05` remains the real owner-consolidation task.
2. The OpenRouter lane still owns direct process bookkeeping through `running_commands`, so future background-terminal or host-exec extraction work can still invalidate parts of this path.
3. The helper intentionally preserves chunked delta delivery. Tests now tolerate that, but any future UI or protocol assumption that output arrives as one newline-inclusive delta would still be brittle.
4. The focused proofs are strong for this slice, but a full post-patch `cargo test -p codex-app-server` rerun was not done after the last adjustments because repo rules require explicit user go-ahead for the complete suite.

## Resume Point

1. If work resumes on this exact slice, rerun:
   - `suite::v2::openrouter_beta::app_server_openrouter_shell_command_runs_as_standalone_turn`
   - `suite::v2::openrouter_beta::app_server_openrouter_shell_command_uses_existing_active_turn`
   - `suite::v2::openrouter_beta::app_server_openrouter_review_start_inline_succeeds`
2. If work widens into the real `Item 05`, start from `docs-new/07-logos-phase/architecture/2026-04-18-item-05-shell-execution-draft-implementation-discussion.md`.
3. Do not reuse this report as evidence that shared shell-exec extraction is complete. It only proves the OpenRouter beta blocker is fixed and covered by focused app-server proofs.
