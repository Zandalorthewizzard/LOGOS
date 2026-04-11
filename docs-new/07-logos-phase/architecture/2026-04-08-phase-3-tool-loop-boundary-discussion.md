---
id: logos-phase-3-tool-loop-boundary-discussion
title: LOGOS Phase 3 Tool Loop Boundary Discussion
type: discussion
status: active
tags: [architecture, discussion, phase-3, tools, mcp, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [
      ../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md,
    ],
    [
      ../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md,
    ],
    [../implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 3 Tool Loop Boundary Discussion

## Purpose

- lock what `Phase 3: Tool Loop Boundary` now means after the completed phase-1 and phase-2 slices
- prevent phase-3 from reopening already accepted work under a vague "do tools again" banner
- identify the remaining tool-loop boundary problems that still need canonical treatment before more code lands

## Starting Point

- phase-1 already delivered a normalized tool-call boundary, stable `call_id` baseline, runtime-owned tool execution, `ToolResult` reinjection, and explicit approval request/resolution events on the OpenAI path
- phase-2 already preserved canonical OpenAI follow-up turn behavior around runtime tool turns, including explicit continuation carryover and within-turn turn-state preservation
- the remaining risk is no longer whether LOGOS can execute one tool roundtrip; the remaining risk is whether LOGOS can state and preserve the full tool-loop ownership contract without hiding MCP/Codex-native semantics

## What Is Already Delivered Earlier

The following outcomes are considered closed earlier and must not be re-litigated as phase-3 goals:

1. normalized tool-call boundary exists
2. stable `call_id` baseline exists across normalized events, approval events, execution, and reinjection for the current slice
3. runtime-owned tool execution exists in `logos-tool-runtime`
4. `ToolResult` reinjection exists in normalized history and follow-up turns
5. approval requested and approval resolved events exist as explicit normalized control-flow events
6. canonical OpenAI turn-loop preservation around tool follow-up turns is already accepted for the current phase-2 scope

## What Phase 3 Still Has To Solve

Phase 3 is not another baseline tool roundtrip pass. It is the remaining tool-loop boundary lock after the baseline exists.

### 1. Canonical Tool-Loop Preservation Beyond The Current Slice

- phase-1 proved one runtime-owned function-style roundtrip
- phase-2 proved canonical OpenAI preservation around that roundtrip
- phase-3 must state which additional tool-loop semantics are still required so later work does not drift, especially when multiple tool events, approval branches, and host/daemon replay responsibilities become richer than the current baseline

### 2. MCP Correlation Ownership And Guarantees

- earlier architecture docs require MCP `tools/call` correlation to survive normalization
- current runtime types and collectors only preserve `call_id`, tool name, and argument payload for the pending-call path
- phase-3 must therefore define which MCP correlation artifacts are canonical runtime data, where they live, and which layer owns them
- until that is narrowed, claims about full MCP preservation remain incomplete

### 3. Tool Lifecycle Ordering And Terminal Invariants

- current code proves no duplicate tool start on the OpenAI path and preserves a simple `started -> args -> finished` baseline
- phase-3 still needs the canonical invariant set for ordering, terminality, and denial/error branches beyond the current tests
- examples include whether every started call must reach exactly one terminal outcome, whether approval denial is terminal for the call lifecycle versus only for execution, and how later persistence/resume will recognize a complete versus incomplete call

### 4. Exact Ownership Split

- `codex-core` remains the home of canonical OpenAI/Codex transport and provider-native mechanics
- `logos-provider-openai` owns request/event/history mapping and OpenAI-specific preservation logic
- `logos-session-engine` owns normalized turn orchestration, approval flow, and follow-up turn control
- `logos-tool-runtime` owns runtime registry, approval delegation inputs, pending-call collection, and execution dispatch
- phase-3 must lock the exact boundary between these layers for richer tool-loop behavior so later work does not pull orchestration into `codex-core` or erase OpenAI/MCP-native semantics behind fake-neutral contracts

### 5. OpenAI/Codex-Native Versus LOGOS-Owned Tool Surfaces

- some tool-loop surfaces remain OpenAI/Codex-native and should stay there, including provider event ontology, request transport details, and any provider-native identifiers that only the adapter can interpret honestly
- some surfaces are already LOGOS-owned, including normalized events/messages, approval control flow, runtime execution policy, and `ToolResult` reinjection
- phase-3 must explicitly classify any remaining mixed surfaces instead of leaving implicit ownership to implementation accidents

### 6. Future Hooks For Persistence/Resume Without Opening Phase 4

- phase-4 remains the durable journal/resume phase
- phase-3 may define which tool-loop artifacts later persistence/resume will need, but it must stop at handoff points and record shape requirements
- no phase-3 work should silently implement durable journaling, replay, or resume restoration

## Narrowing Outcome

- the companion narrowing slice `2026-04-08-phase-3-mcp-tool-correlation-discussion.md` now closes the last pre-implementation blind spot for MCP correlation ownership and lifecycle terminality
- this phase-3 boundary doc remains the outer scope lock, while the MCP/tool-correlation doc fixes the exact payload and branch semantics needed for implementation

## Non-goals

1. no persistence implementation
2. no non-OpenAI adapter implementation work
3. no broad refactor of `codex-rs/core`
4. no fake-neutral rewrite of OpenAI or MCP semantics
5. no reopening or weakening accepted phase-1 or phase-2 outcomes

## Conditional Seam Position

- phase-3 does not assume a new Codex-base seam
- if later proof shows MCP or tool-lifecycle preservation cannot stay outside `codex-core`, the seam must be justified as conditional, narrow, additive, separately committed, and immediately registered in the seam index
- until that proof exists, phase-3 planning must treat new seams as optional rather than presumed

## Implementation Readiness Verdict

- `Phase 3: Tool Loop Boundary` is the roadmap phase that follows the completed phase-2 lane
- the required MCP/tool-correlation narrowing pass now exists
- phase-3 is implementation-ready on paper for the current scope
- remaining work should follow the locked boundary docs and the MCP/tool-correlation companion spec without reopening persistence, non-OpenAI adapters, or broad `codex-core` surgery

## Recommended Next Output

1. keep this phase-3 discussion/spec pair as the canonical lock for what phase-3 is and is not
2. treat `2026-04-08-phase-3-mcp-tool-correlation-discussion.md` as the required companion narrowing doc for implementation
3. open code work only against the combined phase-3 boundary and MCP/tool-correlation specs

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 3: Tool Loop Boundary`
  lines: `134-152`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-70`
- path: `2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Approval Flow Rules`
  lines: `78-102`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Adapter Architecture Boundary`
  lines: `70-106`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Daemon Owns`
  lines: `51-59`
- path: `../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md`
  symbol: `What Works`
  lines: `40-50`
- path: `../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md`
  symbol: `Completion State`
  lines: `103-120`
- path: `../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Delivered Preservation Work`
  lines: `36-44`
- path: `../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Phase-2 Completion State`
  lines: `78-96`
- path: `../../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `ToolExecutionKind`
  lines: `9-41`
- path: `../../../codex-rs/logos-runtime-types/src/events.rs`
  symbol: `InternalEvent`
  lines: `12-61`
- path: `../../../codex-rs/logos-runtime-types/src/message.rs`
  symbol: `InternalMessage`
  lines: `43-65`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `PendingToolCall`
  lines: `13-34`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolCallCollector`
  lines: `36-85`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `33-45`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `run_turn_with_tools_inner`
  lines: `153-266`
- path: `../../../codex-rs/logos-provider-openai/src/map_events.rs`
  symbol: `map_output_item_done`
  lines: `48-123`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn`
  lines: `438-500`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `denied_runtime_tool_is_reinjected_without_execution`
  lines: `556-598`
- path: `2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Core Decision`
  lines: `49-61`

## Links

- [[README.md]]
- [[2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[2026-04-02-logos-approval-control-architecture-spec.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
- [[../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
- [[../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md]]
- [[../implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
