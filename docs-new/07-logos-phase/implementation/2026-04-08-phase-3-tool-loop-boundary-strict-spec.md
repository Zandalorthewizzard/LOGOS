---
id: logos-phase-3-tool-loop-boundary-strict-spec
title: LOGOS Phase 3 Tool Loop Boundary Strict Spec
type: spec
status: active
tags: [implementation, strict-spec, phase-3, tools, mcp, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [2026-04-03-phase-1-boundary-extraction-acceptance-report.md],
    [2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md],
    [../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md],
  ]
depends_on:
  [
    [../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [../architecture/2026-04-02-logos-tool-loop-architecture-spec.md],
    [../architecture/2026-04-02-logos-approval-control-architecture-spec.md],
    [../architecture/2026-04-02-logos-host-daemon-transport-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 3 Tool Loop Boundary Strict Spec

## Start Here

1. `../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md`
2. `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
3. `../architecture/2026-04-02-logos-approval-control-architecture-spec.md`
4. `../architecture/2026-04-02-logos-runtime-contract-spec.md`
5. `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
6. `2026-03-30-logos-global-roadmap.md`
7. `2026-04-03-phase-1-boundary-extraction-acceptance-report.md`
8. `2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
9. `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
10. `2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md`

## Intent

Lock phase-3 as the remaining tool-loop boundary execution lane after the accepted phase-1 and phase-2 slices, while explicitly separating proof that is already satisfied from proof that still requires new work.

Non-goals:

1. persistence implementation
2. non-OpenAI adapter implementation work
3. broad `codex-core` refactor
4. fake-neutral rewrite of OpenAI or MCP semantics
5. restating closed phase-1 or phase-2 acceptance items as if they were unfinished

## Target Outcome

1. phase-3 has a narrow, evidence-backed scope instead of an open-ended "tool loop" label
2. ownership is explicit for tool lifecycle, MCP correlation, approval/control flow, and follow-up turn boundaries
3. future persistence/resume work has defined handoff requirements without phase-4 implementation leakage
4. implementation now proceeds against a fully narrowed phase-3 paper spec set

## Locked Decisions

1. the current normalized tool-call baseline is already delivered and accepted
2. runtime-owned tool execution and `ToolResult` reinjection are already delivered and accepted
3. approval requested/resolved events are already delivered and accepted
4. canonical OpenAI follow-up turn preservation around runtime tool turns is already delivered and accepted
5. phase-3 is about the remaining boundary contract, not a fresh baseline tool-loop prototype
6. any new Codex-base seam is conditional and must not be assumed in advance

## Architecture Fit

1. `codex-rs/core` keeps canonical OpenAI/Codex-native transport and any provider-native semantics that cannot be honestly lifted out
2. `codex-rs/logos-provider-openai` keeps OpenAI request/event/history normalization and OpenAI-specific preservation logic
3. `codex-rs/logos-session-engine` keeps normalized orchestration, approval flow, and follow-up turn control
4. `codex-rs/logos-tool-runtime` keeps runtime tool registration, pending-call accumulation, approval inputs, and execution dispatch
5. phase-3 must only move ownership when evidence shows the current split cannot preserve the required invariant outside `codex-core`

## Proof Targets

### Already Satisfied By Earlier Phases

1. one normalized tool-call boundary exists
2. stable `call_id` survives the current normalized path
3. runtime-owned tool execution works
4. `ToolResult` reinjection works
5. approval requested/resolved events are explicit
6. canonical OpenAI continuation and within-turn preservation around tool follow-up turns work

### New Work Required In Phase 3

1. publish the canonical ownership contract for MCP correlation artifacts
2. publish the canonical lifecycle ordering and terminal invariants for started, approved, denied, executed, failed, and completed tool calls
3. classify which tool-loop surfaces remain OpenAI/Codex-native versus LOGOS-owned
4. define the minimum future persistence/resume hooks required by the tool loop without implementing phase-4 storage or replay
5. identify whether any narrow Codex-base seam is truly required for the remaining invariants; if yes, document it as conditional and register it only when proven necessary

## Ordered Tasks

1. `T1`

- package: `docs-new/07-logos-phase/architecture`, `docs-new/07-logos-phase/implementation`
- output: completed boundary and MCP/tool-correlation spec lock

2. `T2`

- package: `codex-rs/logos-runtime-types`, `codex-rs/logos-tool-runtime`, `codex-rs/logos-session-engine`
- output: code changes that implement the locked phase-3 correlation and lifecycle contracts

3. `T3`

- package: `codex-rs/logos-provider-openai`
- output: adapter remains aligned with the locked ownership split and does not absorb MCP boundary ownership

4. `T4`

- package: `codex-rs/core` only if required
- output: optional narrow seam, separately justified and registered, only if additive crates cannot preserve the contract without it

5. `T5`

- package: focused test/report/docs updates
- output: evidence-backed proofs for the new phase-3 requirements plus an acceptance report that clearly separates newly satisfied items from earlier accepted work

## Requirement -> Task -> Proof -> Gate

| Requirement                                                                                                                | Task IDs         | Proof Source                                                  | Acceptance | Gates |
| -------------------------------------------------------------------------------------------------------------------------- | ---------------- | ------------------------------------------------------------- | ---------- | ----- |
| Earlier baseline tool-loop outcomes remain treated as already satisfied                                                    | `T1`             | docs alignment against phase-1 and phase-2 acceptance reports | `A1`       | `G1`  |
| MCP correlation ownership and guarantees are now fully narrowed before implementation                                      | `T1`, `T2`, `T5` | correlation payload tests plus acceptance report              | `A2`       | `G2`  |
| Tool lifecycle ordering and terminal invariants are made canonical                                                         | `T1`, `T2`, `T5` | invariant tests plus acceptance report                        | `A3`       | `G3`  |
| Ownership split across `codex-core`, `logos-provider-openai`, `logos-session-engine`, and `logos-tool-runtime` is explicit | `T1`, `T2`, `T3` | docs plus code review against locked boundaries               | `A4`       | `G4`  |
| Future persistence/resume hooks are identified without opening phase-4 implementation                                      | `T1`, `T2`, `T5` | docs and acceptance review                                    | `A5`       | `G5`  |
| Any new seam is conditional, narrow, and separately governed                                                               | `T4`, `T5`       | seam index plus dedicated commit only if required             | `A6`       | `G6`  |

## Acceptance And Gates

### A1. Earlier delivered scope is not reopened

- phase-3 artifacts must explicitly mark the following as already satisfied by earlier phases:
  1. normalized tool-call boundary
  2. stable `call_id` baseline
  3. runtime-owned tool execution
  4. `ToolResult` reinjection
  5. approval requested/resolved events
  6. canonical OpenAI turn-loop preservation around tool follow-up turns

### A2. MCP/tool-correlation contract is fully explicit

- phase-3 implementation must follow the companion MCP/tool-correlation discussion/spec pair
- no phase-3 code may invent alternative correlation fields or rely on `call_id` alone for MCP-backed runtime tools

### A3. Lifecycle invariants are explicit

- the implementation lane must prove the canonical ordering and terminal guarantees still missing from the current baseline
- proofs must distinguish provider event completion, approval denial, runtime failure, and follow-up turn completion

### A4. Ownership split is explicit and enforced

- artifacts must state which tool-loop concerns stay in `codex-core`
- artifacts must state which concerns are adapter-owned, session-owned, and tool-runtime-owned
- no broad ownership drift into `codex-core` is allowed without a separately justified seam

### A5. Persistence/resume stays deferred but prepared

- only handoff hooks, record-shape requirements, or invariant requirements may be added
- no durable journal implementation, resume orchestration, or replay engine work is allowed in phase-3

### A6. Seam governance remains conditional

- if no seam is proven necessary, none should be added
- if a seam becomes necessary, it must be narrow, evidence-backed, separately committed, and added to `2026-04-03-codex-core-seam-index.md`

## Implementation Readiness Verdict

- verdict: ready for implementation
- required input set: this phase-3 boundary spec plus the companion MCP/tool-correlation discussion/spec pair
- no further pre-implementation narrowing pass is required for the current scope

## Promotion Artifacts

1. phase-3 discussion doc
2. phase-3 strict spec
3. companion MCP/tool-correlation discussion/spec pair
4. future phase-3 acceptance report once code work actually starts
5. seam index update only if a new seam is proven necessary

## Risks And Rollback

1. Risk: phase-3 code still smuggles undefined MCP correlation through ad hoc fields despite the new docs.
2. Mitigation: require implementation to use the companion MCP/tool-correlation spec as a hard gate.
3. Risk: lifecycle ordering gets hard-coded in one layer without a canonical ownership statement.
4. Mitigation: require explicit ownership and invariant proofs in `A3` and `A4`.
5. Risk: persistence/resume scope leaks into phase-3 under the label of tool reliability.
6. Mitigation: treat all durable state work as handoff-only and defer real implementation to phase-4.
7. Rollback: if implementation reveals an evidence-backed gap that the new docs still did not lock, stop and promote one narrower correction doc before widening code.

## Agent Guardrails

1. do not write `codex-rs` code that violates the locked MCP/tool-correlation contract
2. do not assume a new Codex-base seam
3. do not redefine already accepted phase-1 or phase-2 outcomes as unfinished
4. do not open persistence, replay, or resume implementation
5. do not open non-OpenAI provider work from this phase-3 entry point

## Evidence

- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 3: Tool Loop Boundary`
  lines: `134-152`
- path: `../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md`
  symbol: `What Phase 3 Still Has To Solve`
  lines: `53-103`
- path: `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Canonical Correlation Model`
  lines: `52-83`
- path: `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Terminal Invariants`
  lines: `111-128`
- path: `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Acceptance Gates`
  lines: `135-146`
- path: `../architecture/2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Acceptance Gates`
  lines: `109-120`
- path: `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Daemon Owns`
  lines: `51-59`
- path: `2026-04-03-phase-1-boundary-extraction-acceptance-report.md`
  symbol: `Acceptance Mapping`
  lines: `73-101`
- path: `2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Acceptance Mapping`
  lines: `45-77`
- path: `../../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `ToolExecutionKind`
  lines: `9-41`
- path: `../../../codex-rs/logos-runtime-types/src/events.rs`
  symbol: `InternalEvent`
  lines: `12-61`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `PendingToolCall`
  lines: `13-34`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `run_turn_with_tools_inner`
  lines: `153-266`
- path: `2026-04-03-codex-core-seam-index.md`
  symbol: `Rules`
  lines: `30-36`

## Links

- [[README.md]]
- [[2026-03-30-logos-global-roadmap.md]]
- [[2026-04-03-phase-1-boundary-extraction-acceptance-report.md]]
- [[2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md]]
- [[../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
- [[../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
- [[2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md]]
