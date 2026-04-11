---
id: logos-phase-3-tool-loop-boundary-acceptance-report
title: LOGOS Phase 3 Tool Loop Boundary Acceptance Report
type: report
status: active
tags: [implementation, report, phase-3, tool-loop, mcp, logos]
related:
  [
    [README.md],
    [2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
    [2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md],
    [../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [2026-04-03-codex-core-seam-index.md],
  ]
depends_on:
  [
    [2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
    [2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 3 Tool Loop Boundary Acceptance Report

## Scope

This report records the completed phase-3 tool-loop boundary implementation lane after the phase-1 baseline and phase-2 OpenAI preservation work were already accepted.

## Delivered In Phase 3

1. MCP-backed runtime tools now have an explicit runtime-owned correlation ingress hook instead of ad hoc attachment.
2. Normalized tool approval now carries optional MCP correlation metadata without forcing fake-neutral provider semantics.
3. Terminal tool-call state is now explicit and normalized across completed, denied, failed, and interrupted branches.
4. `logos-session-engine` now records one terminal `ToolCallRecord` per `call_id` and ignores duplicate terminal writes.
5. Interrupted provider streams after tool-call start now leave an explicit interrupted terminal record instead of silently dropping lifecycle state.
6. No new `codex-core` seam was required to deliver the phase-3 scope.

## Earlier Delivered Scope Preserved

The following phase-1 and phase-2 outcomes remained intact and were not reopened by phase-3:

1. normalized tool-call boundary
2. stable `call_id` baseline
3. runtime-owned tool execution
4. `ToolResult` reinjection
5. approval requested/resolved events
6. canonical OpenAI turn-loop preservation around tool follow-up turns

## Acceptance Mapping

### A1. MCP correlation bundle is explicit

- Met.
- `McpToolCallCorrelation` is now part of the normalized runtime contract.
- `ToolRuntime::prepare_call()` is the explicit runtime-owned ingress handoff that attaches `request_id` and `submission_id` before approval/execution for `McpBackedRuntime` calls.
- `mcp_call_requires_explicit_ingress`, `mcp_ingress_attaches_request_and_submission_ids`, and `mcp_backed_runtime_tool_attaches_correlation_before_approval` prove the ingress contract.

### A2. Terminality is exact

- Met for the phase-3 scope.
- `ToolCallTerminalState` and `ToolCallRecord` make terminal classification explicit.
- `runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn` proves `CompletedWithResult`.
- `runtime_tool_error_is_reinjected_as_tool_result` proves `FailedBeforeResult`.
- `denied_runtime_tool_is_reinjected_without_execution` proves `DeniedByApproval`.
- `interrupted_tool_call_records_terminal_state_without_result` proves `Interrupted`.
- `duplicate_terminal_records_are_ignored_by_call_id` proves one `call_id` cannot record multiple terminal outcomes.

### A3. Approval correlation is lossless

- Met for the phase-3 scope.
- `ToolApprovalRequest` now carries optional MCP correlation metadata.
- `mcp_backed_runtime_tool_attaches_correlation_before_approval` proves MCP-backed approval requests preserve the correlation bundle before execution starts.

### A4. Ownership boundaries stay honest

- Met.
- MCP correlation ingress, pending-call preservation, and terminal recording all landed in `logos-runtime-types`, `logos-tool-runtime`, and `logos-session-engine`.
- `logos-provider-openai` still only owns provider event normalization.
- No new `codex-core` seam entry was required.

### A5. Future persistence/resume does not need another correlation redesign

- Met for the phase-3 scope.
- `ToolCallRecord` now preserves terminal state, result disposition, and optional MCP correlation bundle in one normalized shape that later persistence/resume layers can consume.

### A6. Seam governance remains conditional

- Met.
- Phase-3 completed without adding a new seam to `2026-04-03-codex-core-seam-index.md`.

## Phase-3 Completion State

The phase-3 tool-loop boundary scope is now logically complete for the current strict specs.

Completed in this phase:

1. exact MCP ingress ownership is explicit and implemented;
2. MCP-backed approval/execution correlation is preserved through normalized runtime state;
3. completed, denied, failed, and interrupted tool-call branches are classified explicitly;
4. duplicate terminal tool-call writes are prevented by `call_id`;
5. later persistence/resume work now has a stable normalized terminal record surface.

Still out of scope for this phase:

1. durable journal/resume implementation;
2. non-OpenAI provider implementation work;
3. broad `codex-core` refactor or fake-neutral provider semantics.

## Codex-Base Seams Used

Phase-3 uses the earlier registered seams only.

1. Prompt tool injection seam
2. explicit continuation carryover seam
3. turn-state transfer hook seam

No new seam was required for the phase-3 tool-loop boundary scope.

## Tests Run

1. `cargo test -p logos-runtime-types`
2. `cargo test -p logos-tool-runtime`
3. `cargo test -p logos-session-engine --test phase1_boundary`
4. `just fix -p logos-runtime-types`
5. `just fix -p logos-tool-runtime`
6. `just fix -p logos-session-engine`
7. `just fmt`

## Deferred Follow-Ups

1. phase-4 durable journal and resume implementation
2. non-OpenAI provider lanes
3. later acceptance/reporting for persistence-driven replay behavior

## Evidence

- path: `../../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `McpToolCallCorrelation`
  lines: `29-67`
- path: `../../../codex-rs/logos-runtime-types/src/events.rs`
  symbol: `InternalEvent`
  lines: `1-70`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `McpCorrelationIngress`
  lines: `97-124`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolRuntime::prepare_call`
  lines: `166-198`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolRuntime::execute_call`
  lines: `200-240`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `run_turn_with_tools_inner`
  lines: `167-313`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `record_interrupted_calls`
  lines: `372-408`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `duplicate_terminal_records_are_ignored_by_call_id`
  lines: `414-442`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn`
  lines: `447-531`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `runtime_tool_error_is_reinjected_as_tool_result`
  lines: `513-574`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `denied_runtime_tool_is_reinjected_without_execution`
  lines: `566-620`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `mcp_backed_runtime_tool_attaches_correlation_before_approval`
  lines: `610-681`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `interrupted_tool_call_records_terminal_state_without_result`
  lines: `683-733`
- path: `2026-04-08-phase-3-tool-loop-boundary-strict-spec.md`
  symbol: `Acceptance And Gates`
  lines: `138-180`
- path: `2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md`
  symbol: `Acceptance And Gates`
  lines: `189-224`

## Links

- [[README.md]]
- [[2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
- [[2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md]]
- [[../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
- [[../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
- [[2026-04-03-codex-core-seam-index.md]]
