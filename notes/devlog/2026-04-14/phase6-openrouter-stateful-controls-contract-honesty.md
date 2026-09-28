# Phase 6 - OpenRouter Stateful Controls Contract Honesty Pass

**Branch:** `feat/phase6-openrouter-agent-beta`
**Date:** 2026-04-14
**Status:** Closed

## Scope

Tighten the truthfulness of `thread/resume`, `thread/fork`, and `review/start` for OpenRouter beta threads. This pass did not widen feature scope. It made unsupported modes explicit, completed the missing proof coverage, and restored one relevant OpenAI app-server no-regression proof.

## What was fixed

### A. `thread/resume` contract honesty

- `history` mode is rejected explicitly with an OpenRouter beta error.
- `path` mode is rejected explicitly with an OpenRouter beta error.
- Config override fields are rejected explicitly when non-default values are present.
- `thread_id` mode for an existing live OpenRouter beta thread still reconnects successfully.
- Unknown-thread requests that explicitly target `model_provider=openrouter-*` now validate unsupported modes before returning, so `history` and `path` no longer disappear behind a generic unknown-thread error.

### B. `thread/fork` contract honesty

- `path` mode is rejected explicitly.
- Config override fields are rejected explicitly when non-default values are present.
- `thread_id` mode for an existing live OpenRouter beta thread still forks correctly onto the OpenRouter beta lane with coherent `forked_from_id`.
- Unknown-thread requests that explicitly target `model_provider=openrouter-*` now surface unsupported `path` honestly instead of silently drifting into a generic unknown-thread error.

### C. `review/start` detached review delivery

- Detached review now subscribes the requesting connection to the new review thread before thread-scoped notifications are sent.
- Detached review emits reachable `thread/started`, turn lifecycle, and completion notifications instead of dropping them on an unsubscribed thread.
- Inline review behavior remains unchanged and still works.

### D. App-server OpenAI no-regression proof restoration

- The app-server test harness now exports `LOGOS_HOME` alongside `CODEX_HOME`, so tests remain hermetic after the LOGOS/Codex home split.
- The originator-header OpenAI regression proof is green again in this shell.
- The test config used by that proof now forces `supports_websockets = false` on the mock provider so it stays on the intended HTTP path.

## Still explicitly unsupported

- `thread/resume` via `history`
- `thread/resume` via `path`
- `thread/resume` config overrides
- `thread/fork` via `path`
- `thread/fork` config overrides
- `thread/shellCommand`
- `thread/backgroundTerminals/clean`
- `thread/rollback`
- `thread/compact/start`
- `turn/interrupt`
- `turn/steer`
- realtime operations

## Changed crates/files

- `codex-rs/app-server/src/openrouter_beta_threads.rs`
  - added `validate_openrouter_resume_params`
  - added `validate_openrouter_fork_params`
  - updated detached review subscription and notification delivery
- `codex-rs/app-server/src/codex_message_processor.rs`
  - validates OpenRouter resume/fork params before the unknown-thread branch
  - passes `request_id.connection_id` into detached review startup
- `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs`
  - added proof tests for unsupported modes, override rejection, unknown-thread mode truth, and detached review delivery
- `codex-rs/app-server/tests/common/mcp_process.rs`
  - test harness now sets `LOGOS_HOME`
- `codex-rs/app-server/tests/suite/v2/turn_start.rs`
  - OpenAI mock provider config disables websockets for the originator-header proof

## Tests/checks run

- `cargo test -p codex-app-server app_server_openrouter_resume_with_history_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_resume_with_path_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_fork_with_path_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_resume_with_config_override_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_fork_with_config_override_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_resume_with_history_on_unknown_thread_via_model_provider_is_explicitly_rejected`
- `cargo test -p codex-app-server app_server_openrouter_review_start_detached_creates_thread_and_emits_notifications`
- `cargo test -p codex-app-server turn_start_sends_originator_header`

## Verdict

**Slice closed.** The remaining gap was proof integrity, not missing core behavior. OpenRouter stateful controls are now honest at the app-server contract level for this slice, and the required touched-path OpenAI no-regression proof is green again.

## Explicit answers

### How OpenRouter `thread/resume` now behaves

- `thread_id` on a loaded live OpenRouter beta thread reconnects and subscribes the caller.
- `history` is rejected explicitly.
- `path` is rejected explicitly.
- Config override fields are rejected explicitly.
- Unknown-thread requests that explicitly target `model_provider=openrouter-*` still reject unsupported modes explicitly before any generic unknown-thread error path.

### How OpenRouter `thread/fork` now behaves

- `thread_id` on a loaded live OpenRouter beta thread forks history into a new OpenRouter beta thread.
- `path` is rejected explicitly.
- Config override fields are rejected explicitly.

### How detached OpenRouter review now reaches the caller

1. A new review thread is created from the source thread history.
2. The requesting connection is subscribed to that new thread.
3. `thread/started` is sent to a real subscriber.
4. The detached review turn emits normal turn lifecycle notifications.
5. The caller receives `review_thread_id` for the new thread.

### Review-specific config behavior

- `ReviewStartParams` has no `review_model` field in the current protocol.
- No per-review config override surface was added in this pass.
- Both inline and detached review delivery modes are supported on the OpenRouter beta lane.
