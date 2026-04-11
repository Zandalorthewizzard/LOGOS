---
id: logos-phase-3-mcp-tool-correlation-strict-spec
title: LOGOS Phase 3 MCP Tool Correlation Strict Spec
type: spec
status: active
tags: [implementation, strict-spec, phase-3, mcp, tools, correlation, logos]
related:
  [
    [README.md],
    [2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
    [../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md],
  ]
depends_on:
  [
    [../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md],
    [2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 3 MCP Tool Correlation Strict Spec

## Start Here

1. `../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md`
2. `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
3. `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
4. `../architecture/2026-04-02-logos-approval-control-architecture-spec.md`
5. `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
6. `2026-04-08-phase-3-tool-loop-boundary-strict-spec.md`

## Intent

Remove the last pre-implementation ambiguity in phase-3 by locking the exact MCP/tool-correlation payload, lifecycle rules, ownership split, and proof gates.

Non-goals:

1. implementing persistence/resume
2. implementing non-OpenAI adapters
3. broad provider-neutral abstraction work
4. assuming a new Codex-base seam before evidence proves one is needed

## Locked Decisions

1. stable `call_id` remains the universal normalized tool key
2. MCP-backed runtime tools carry an additional explicit correlation bundle
3. the MCP correlation bundle is `request_id`, `submission_id`, and optional terminal `thread_id`
4. `request_id` and `submission_id` must both be preserved explicitly in phase-3 as a conscious LOGOS design choice even though the current Codex reference path makes them equal
5. every started tool call must end in exactly one terminal lifecycle branch
6. approval and host/daemon routing use `request_id` at the outer MCP boundary and `call_id` inside normalized runtime state
7. phase-3 is now implementation-ready for the current scope once code follows this spec

## Required Data Contract

### Universal Tool Call State

Every normalized pending or persisted tool call must expose:

1. `call_id`
2. tool name
3. execution kind
4. arguments payload or argument delta accumulation result

### MCP Correlation State

Every `McpBackedRuntime` pending or persisted tool call must additionally expose:

1. `request_id: String`
2. `submission_id: String`
3. `thread_id: Option<String>`

Rules:

1. `request_id` is the original outer MCP request identity
2. `submission_id` is the inner Codex submission identity
3. `thread_id` is absent before terminal completion unless a resumable Codex thread is already known
4. if the current upstream Codex path derives `submission_id` from `request_id`, LOGOS still stores both explicitly

### Exact Ingress Contract

1. provider event normalization creates the pending call with `call_id`, tool name, and arguments only
2. when `logos-session-engine` recognizes that the call targets an `McpBackedRuntime` tool, it must invoke the runtime-owned MCP ingress hook before approval or execution begins
3. that ingress hook attaches `request_id` and `submission_id` to the pending call, keyed by `call_id`
4. `thread_id` must not be fabricated at ingress time; it attaches only when the terminal execution result actually yields resumable session identity
5. implementation must not derive the MCP bundle by re-reading provider events after collection

## Lifecycle Contract

### Non-terminal States

1. `Started`
2. `AwaitingApproval`
3. `ApprovedPendingExecution`
4. `Executing`

### Terminal States

1. `CompletedWithResult`
2. `DeniedByApproval`
3. `FailedBeforeResult`
4. `Interrupted`

Rules:

1. a call may enter at most one non-terminal chain and exactly one terminal state
2. `ToolCallFinished` from provider event mapping is not the same as runtime terminal completion; it only closes provider argument emission
3. a denied approval must not execute the tool
4. a reinjected synthetic error payload after denial does not change the terminal state from `DeniedByApproval`
5. a runtime execution error produces terminal state `FailedBeforeResult`
6. interruption, cancellation, or disconnect before a result is available produces terminal state `Interrupted`

## Ownership Split

### `codex-core`

- no new responsibility is assumed here by default
- only participates if an evidence-backed seam is later proven necessary

### `logos-provider-openai`

- maps provider-native tool events into normalized lifecycle events
- must not invent or own MCP request/thread correlation for runtime-owned tools

### `logos-tool-runtime`

- owns canonical normalized payload shapes for pending runtime-owned and MCP-backed runtime calls
- owns transport of MCP correlation bundle from collection through execution result production

### `logos-session-engine`

- owns lifecycle transitions, approval flow, terminal state assignment, and reinjection readiness
- owns the rule that all tool-call branches must be terminally classified exactly once

### Host/Daemon Boundary

- outer routing for approvals, notifications, and final MCP responses uses `request_id`
- reconnect, replay, and later persistence handoff must preserve `request_id` plus optional `thread_id`

## Persistence/Resume Handoff Contract

Without implementing phase-4, phase-3 code must leave enough structure so later persistence can store:

1. universal tool call state
2. MCP correlation bundle
3. approval branch outcome
4. terminal lifecycle state
5. result presence and error flag

No other phase-4 behavior is required in phase-3.

## Ordered Tasks

1. `T1`

- package: `codex-rs/logos-runtime-types`
- output: explicit normalized contract for MCP correlation bundle and lifecycle state

2. `T2`

- package: `codex-rs/logos-tool-runtime`
- output: pending-call collection and execution payloads preserve MCP correlation bundle instead of flattening to only `call_id`, tool name, and arguments

3. `T3`

- package: `codex-rs/logos-session-engine`
- output: session engine assigns explicit terminal lifecycle state and preserves approval/MCP correlation through denial, failure, and completion branches

4. `T4`

- package: `codex-rs/logos-provider-openai`
- output: adapter remains scoped to provider event normalization and does not absorb MCP boundary ownership

5. `T5`

- package: `codex-rs/core` only if needed
- output: optional narrow seam, separately justified and registered, only if `T1`-`T4` cannot preserve the contract without it

6. `T6`

- package: tests, docs, acceptance report
- output: proof that the full phase-3 contract is implemented without reopening accepted phase-1/phase-2 outcomes

## Requirement -> Task -> Proof -> Gate

| Requirement                                                                                        | Task IDs         | Proof                                          | Acceptance | Gates |
| -------------------------------------------------------------------------------------------------- | ---------------- | ---------------------------------------------- | ---------- | ----- |
| MCP-backed runtime calls preserve explicit `request_id`, `submission_id`, and optional `thread_id` | `T1`, `T2`, `T6` | focused payload tests                          | `A1`       | `G1`  |
| Started tool calls end in exactly one terminal lifecycle state                                     | `T1`, `T3`, `T6` | lifecycle branch tests                         | `A2`       | `G2`  |
| Approval routing preserves both `call_id` and MCP outer-boundary correlation                       | `T2`, `T3`, `T6` | approval-flow tests                            | `A3`       | `G3`  |
| OpenAI adapter stays out of MCP boundary ownership                                                 | `T4`, `T6`       | code review plus targeted tests                | `A4`       | `G4`  |
| Persistence/resume handoff payload is fully defined without implementing phase-4                   | `T1`, `T3`, `T6` | docs plus type review                          | `A5`       | `G5`  |
| Any new seam is conditional and separately governed                                                | `T5`, `T6`       | seam index and dedicated commit only if needed | `A6`       | `G6`  |

## Acceptance And Gates

### A1. MCP correlation bundle is explicit

- `McpBackedRuntime` calls must no longer rely on `call_id` alone
- proofs must show `request_id`, `submission_id`, and terminal `thread_id` survive normalized runtime ownership
- proofs must also show the exact ingress hook where `request_id` and `submission_id` first attach before approval/execution

### A2. Terminality is exact

- proofs must show one started call yields exactly one terminal state
- proofs must cover approved result, denied approval, runtime error, and interruption branch shapes

### A3. Approval correlation is lossless

- proofs must show approval requests and outcomes remain linked to `call_id`
- for MCP-backed runtime calls, proofs must also show the preserved outer-boundary `request_id`

### A4. Ownership boundaries stay honest

- no MCP boundary logic should be quietly moved into `logos-provider-openai`
- no broad `codex-core` refactor is allowed under this spec

### A5. Future persistence/resume does not need another correlation redesign

- after phase-3 implementation, later persistence/resume may consume the locked payload without reopening the tool-loop contract

### A6. Seam governance remains conditional

- if no seam is needed, none is added
- if a seam is needed, it must be narrow, evidence-backed, separately committed, and registered in `2026-04-03-codex-core-seam-index.md`

## Readiness Verdict

- verdict: phase-3 pre-implementation spec coverage is now complete for the current scope
- next step: implementation may begin against the locked phase-3 docs

## Evidence

- path: `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Canonical Correlation Model`
  lines: `59-96`
- path: `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Exact Ingress Seam`
  lines: `85-96`
- path: `../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md`
  symbol: `Terminal Invariants`
  lines: `111-128`
- path: `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-70`
- path: `../architecture/2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Approval Flow Rules`
  lines: `78-102`
- path: `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Daemon Owns`
  lines: `51-59`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `create_call_tool_result_with_thread_id`
  lines: `32-52`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `run_codex_tool_session`
  lines: `100-119`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `run_codex_tool_session_inner`
  lines: `216-310`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `PendingToolCall`
  lines: `13-34`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `run_turn_with_tools_inner`
  lines: `153-266`
- path: `2026-04-08-phase-3-tool-loop-boundary-strict-spec.md`
  symbol: `New Work Required In Phase 3`
  lines: `87-93`

## Links

- [[README.md]]
- [[2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
- [[../architecture/2026-04-08-phase-3-mcp-tool-correlation-discussion.md]]
- [[../architecture/2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
