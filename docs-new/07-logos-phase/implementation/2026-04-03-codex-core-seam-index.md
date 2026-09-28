---
id: logos-codex-core-seam-index
title: LOGOS Codex-Core Seam Index
type: index
status: active
tags: [implementation, codex-core, seams, upstream, logos]
related:
  [
    [README.md],
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-upstream-sync-policy.md],
    [2026-04-03-phase-1-boundary-extraction-acceptance-report.md],
  ]
depends_on:
  [
    [2026-04-02-codex-base-repository-strategy.md],
    [2026-04-02-upstream-sync-policy.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Codex-Core Seam Index

## Purpose

This index records every intentional LOGOS intervention in `codex-rs/core/` or equivalent Codex-base seams that exist only to open a narrow additive integration surface for LOGOS.

## Rules

1. One seam intervention maps to one dedicated commit.
2. The seam commit should stay separate from larger consumer changes when reasonably possible.
3. Every seam entry must explain why a pure wrapper/additive seam outside `codex-core` was insufficient.
4. Prefer helper, builder, or re-export seams over moving orchestration into `codex-core`.
5. If upstream later grows an equivalent public seam, mark the LOGOS seam `replaced` and migrate consumers.

## Entries

### SEAM-2026-04-03-01 Prompt Tool Injection

- status: `active`
- introduced_by_commit: `983795862`
- area: `codex-rs/core/src/client_common.rs`
- why_needed: external LOGOS adapters could already reuse `ModelClientSession`, but `Prompt.tools` and `Prompt.parallel_tool_calls` were crate-private, so `request.tools` could not flow into the canonical OpenAI/Codex request path without duplicating internal request wiring or exposing deeper core internals.
- minimal_scope:
  1. add `Prompt::set_configured_tools`
  2. add `Prompt::configured_tools`
  3. add `Prompt::parallel_tool_calls_enabled`
- affected_files:
  1. `codex-rs/core/src/client_common.rs`
  2. `codex-rs/core/src/client_common_tests.rs`
- public_surface_added:
  1. `Prompt::set_configured_tools`
  2. `Prompt::configured_tools`
  3. `Prompt::parallel_tool_calls_enabled`
- consuming_logos_crates:
  1. `codex-rs/logos-provider-openai`
- non_goals:
  1. no tool execution orchestration moved into `codex-core`
  2. no raw Codex/OpenAI wire types exported through the LOGOS shared runtime
  3. no changes to canonical Codex handler ownership or transport flow
- upstream_conflict_risk: `low-to-medium`; `Prompt` is central, but the seam is localized and additive.
- rollback_or_replace_path: if upstream later exposes an equivalent public prompt-tool injection API, migrate LOGOS consumers to that upstream API and mark this seam `replaced`.

### SEAM-2026-04-03-02 Explicit Continuation Carryover

- status: `active`
- introduced_by_commit: `338d695eb`
- area:
  1. `codex-rs/core/src/client_common.rs`
  2. `codex-rs/core/src/client.rs`
  3. `codex-rs/codex-api/src/common.rs`
- why_needed: LOGOS needed an explicit boundary-owned continuation path for OpenAI turns, but the public Codex request objects did not carry `previous_response_id`. Without a seam, continuation only worked by incorrectly reusing one turn-scoped `ModelClientSession` across multiple turns.
- minimal_scope:
  1. add `Prompt.previous_response_id`
  2. add `ResponsesApiRequest.previous_response_id`
  3. propagate that field into websocket request conversion
  4. make explicit `previous_response_id` take priority over implicit websocket carryover
- affected_files:
  1. `codex-rs/core/src/client_common.rs`
  2. `codex-rs/core/src/client.rs`
  3. `codex-rs/core/src/client_common_tests.rs`
  4. `codex-rs/core/tests/suite/client_websockets.rs`
  5. `codex-rs/codex-api/src/common.rs`
- public_surface_added:
  1. `Prompt.previous_response_id`
  2. `ResponsesApiRequest.previous_response_id`
- consuming_logos_crates:
  1. `codex-rs/logos-provider-openai`
  2. `codex-rs/logos-session-engine`
- non_goals:
  1. no broad session redesign in `codex-core`
  2. no provider-specific continuation logic leaked into shared runtime types beyond the existing normalized handle
  3. no transport rewrite beyond explicit-field propagation
- upstream_conflict_risk: `medium`; this seam touches central request structs and websocket request preparation, but remains additive and tightly scoped to explicit continuation carryover.
- rollback_or_replace_path: if upstream later exposes a public explicit continuation field on prompt/request builders, migrate LOGOS to that surface and retire this seam.

### SEAM-2026-04-07-03 Turn-State Transfer Hook

- status: `active`
- introduced_by_commit: `21a3887d3`
- area: `codex-rs/core/src/client.rs`
- why_needed: phase-2 needs fresh `ModelClientSession` instances per request while still preserving canonical within-turn sticky-routing behavior. Without a narrow hook, `x-codex-turn-state` would be trapped inside one private turn-scoped client session and could not be carried across fresh request sessions within the same logical LOGOS turn.
- minimal_scope:
  1. add `ModelClientSession::set_turn_state`
  2. add `ModelClientSession::current_turn_state`
- affected_files:
  1. `codex-rs/core/src/client.rs`
  2. `codex-rs/logos-provider-openai/src/adapter.rs`
  3. `codex-rs/logos-session-engine/tests/phase1_boundary.rs`
- public_surface_added:
  1. `ModelClientSession::set_turn_state`
  2. `ModelClientSession::current_turn_state`
- consuming_logos_crates:
  1. `codex-rs/logos-provider-openai`
- non_goals:
  1. no broad redesign of websocket session reuse in `codex-core`
  2. no promotion of turn-state into shared LOGOS runtime contracts
  3. no attempt to make OpenAI sticky-routing behavior provider-neutral
- upstream_conflict_risk: `medium`; this seam touches the central turn-scoped client session type, but remains narrow and focused on explicit turn-state transfer.
- rollback_or_replace_path: if upstream later exposes an equivalent public turn-state transfer hook, migrate LOGOS to it and retire this seam.

### SEAM-2026-04-18-04 Shared Host Exec Shell Runner

- status: `active`
- introduced_by_commit: `pending-current-branch`
- area:
  1. `codex-rs/core/src/host_exec.rs`
  2. `codex-rs/core/src/lib.rs`
- why_needed: Item 05 needed OpenRouter `thread/shellCommand` to reuse the mature local one-shot shell-command mechanics without leaving `drive_openrouter_shell_command()` as a second permanent execution owner. A pure wrapper outside `codex-core` was insufficient because the reusable shell-command substrate already lives behind `execute_exec_request()`, `Shell`, and shell snapshot wrapping inside `codex-core`.
- minimal_scope:
  1. add a narrow public `host_exec` module for one-shot shell execution
  2. add `HostExecNotificationSink` and `HostExecContext` as explicit caller boundaries
  3. add `HostExecRegistry` for thread-scoped cancellation used by the touched app-server shell-command surface
  4. add `HostExecProcessEntry` for thread-scoped process metadata tracking (Item 11 extension)
  5. add `HostExecRegistry::thread_processes` query for shared process lifecycle visibility (Item 11 extension)
  6. add `InteractiveProcessState` and `spawn_interactive_process` for interactive PTY background-terminal lifecycle (Item 11 interactive wave)
  7. add `HostExecRegistry::write_stdin`, `resize_process`, `read_buffered_output` for interactive process control (Item 11 interactive wave)
  8. add `HostExecNotificationSink::emit_terminal_interaction` for terminal interaction routing (Item 11 interactive wave)
- affected_files:
  1. `codex-rs/core/src/host_exec.rs`
  2. `codex-rs/core/src/host_exec_tests.rs`
  3. `codex-rs/core/src/tasks/user_shell.rs`
  4. `codex-rs/core/src/lib.rs`
- public_surface_added:
  1. `codex_core::host_exec::HostExecContext`
  2. `codex_core::host_exec::HostExecNotificationSink`
  3. `codex_core::host_exec::HostExecRegistry`
  4. `codex_core::host_exec::run_one_shot_shell_command`
  5. `codex_core::host_exec::HostExecProcessEntry` (Item 11)
  6. `codex_core::host_exec::HostExecRegistry::thread_processes` (Item 11)
  7. `codex_core::host_exec::InteractiveProcessState` (Item 11 interactive wave)
  8. `codex_core::host_exec::InteractiveHostExecContext` (Item 11 interactive wave)
  9. `codex_core::host_exec::spawn_interactive_process` (Item 11 interactive wave)
  10. `codex_core::host_exec::HostExecRegistry::write_stdin` (Item 11 interactive wave)
  11. `codex_core::host_exec::HostExecRegistry::resize_process` (Item 11 interactive wave)
  12. `codex_core::host_exec::HostExecRegistry::read_buffered_output` (Item 11 interactive wave)
  13. `codex_core::host_exec::HostExecNotificationSink::emit_terminal_interaction` (Item 11 interactive wave)
  14. `codex_core::host_exec::HostExecTerminalInteraction` (Item 11 interactive wave)
- consuming_logos_crates:
  1. `codex-rs/app-server`
- non_goals:
  1. no broad unified-exec or background-terminal redesign in `codex-core`
  2. no provider-specific semantics embedded in the shared runner
  3. no protocol change for `thread/shellCommand`
- upstream_conflict_risk: `medium`; the seam adds a new public host-exec module in `codex-core`, but keeps ownership narrow to one-shot shell-command execution, thread-scoped cancellation, and thread-scoped process metadata for the touched surface.
- rollback_or_replace_path: if upstream later exposes an equivalent public one-shot host-exec runner and cancellation registry, migrate LOGOS callers to that upstream surface and retire this seam.

## Evidence

- equivalent: `git show --stat --summary 983795862`
- path: `../../../codex-rs/core/src/client_common.rs`
  symbol: `Prompt`
  lines: `26-85`
- path: `../../../codex-rs/core/src/client_common_tests.rs`
  symbol: `prompt_accepts_external_configured_tools`
  lines: `203-228`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::build_responses_request`
  lines: `689-755`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::prepare_websocket_request`
  lines: `831-857`
- path: `../../../codex-rs/codex-api/src/common.rs`
  symbol: `ResponsesApiRequest`
  lines: `153-194`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::set_turn_state`
  lines: `681-699`
- path: `../../../codex-rs/core/src/client.rs`
  symbol: `ModelClientSession::current_turn_state`
  lines: `690-692`

## Links

- [[README.md]]
- [[2026-04-02-codex-base-repository-strategy.md]]
- [[2026-04-02-upstream-sync-policy.md]]
- [[2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
