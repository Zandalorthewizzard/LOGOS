---
id: logos-phase-3-mcp-tool-correlation-discussion
title: LOGOS Phase 3 MCP Tool Correlation Discussion
type: discussion
status: active
tags: [architecture, discussion, phase-3, mcp, tools, correlation, logos]
related:
  [
    [README.md],
    [2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [../implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md],
    [../implementation/2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-08-phase-3-tool-loop-boundary-discussion.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 3 MCP Tool Correlation Discussion

## Purpose

- close the last architecture blind spot identified by the phase-3 tool-loop boundary lock
- define the exact MCP/tool-correlation contract so implementation does not invent ad hoc fields or lifecycle rules
- make phase-3 implementation-ready on paper for the current OpenAI-plus-runtime-owned scope

## Starting Point

- the existing phase-3 boundary docs already locked that the remaining unresolved area was MCP/tool-correlation ownership and lifecycle terminal guarantees
- Codex upstream evidence already shows that MCP tool execution reuses the original request id for correlation and returns `threadId` in structured output so the caller can resume
- current LOGOS runtime code does not yet expose a dedicated MCP correlation payload beyond `call_id`, tool name, and argument payload

## Problem Statement

- `call_id` is necessary but not sufficient for MCP-backed runtime tool correlation
- upstream Codex MCP execution uses at least two additional protocol-grade correlation artifacts:
  1. original request id reused as Codex submission sub-id
  2. `threadId` returned in structured content for continuation
- if LOGOS leaves those artifacts implicit, later implementation could lose resume correlation, approval correlation, or host/daemon reconnect semantics even while preserving a stable `call_id`

## Core Decision

- LOGOS phase-3 must preserve two layers of tool correlation:
  1. universal runtime tool correlation keyed by stable `call_id`
  2. MCP-specific correlation metadata carried as an explicit opaque bundle on MCP-backed runtime calls only
- MCP-specific correlation is canonical for the runtime, but it is not promoted into a fake-neutral provider-wide ontology for all tools
- the daemon/runtime owns the MCP correlation bundle; provider adapters do not reinterpret or synthesize it unless they are the actual MCP boundary owner

## Canonical Correlation Model

### Universal Correlation Fields

Every normalized tool call must preserve:

1. `call_id`
2. tool name
3. tool execution kind

These fields are the minimum key for stream ordering, approval linkage, execution dispatch, and tool result reinjection.

### MCP Correlation Bundle

For `McpBackedRuntime` calls, the canonical correlation bundle is:

1. `request_id`
   - the original MCP `tools/call` request id
   - this is the correlation root for notifications, approval callbacks, and final response delivery
2. `submission_id`
   - the Codex submission id used inside the spawned Codex session
   - for the current Codex reference path this must equal the original `request_id`
3. `thread_id`
   - the Codex thread/session continuation identifier returned by the tool result when a resumable Codex tool session exists
   - optional before terminal tool completion, required in the terminal tool result branch when upstream Codex returns it

### Exact Ingress Seam

- the MCP correlation bundle does not originate from provider event normalization
- the exact ingress seam is the runtime-owned handoff between normalized pending-call collection and approval/execution dispatch for `McpBackedRuntime` calls
- required attachment sequence:
  1. provider events establish `call_id`, tool name, and arguments
  2. `logos-session-engine` identifies that the pending call targets an `McpBackedRuntime` tool
  3. before emitting `ApprovalRequested` or starting execution, the runtime-owned MCP ingress hook attaches `request_id` and `submission_id` to that call, keyed by the already-established `call_id`
  4. `thread_id` attaches later on the terminal execution result branch when a resumable Codex/MCP session actually yields it
- this ingress hook belongs above the provider boundary and below host/daemon approval/execution orchestration; it is not adapter-owned and it must not be reconstructed ad hoc from provider events

### Decision On Duplication

- upstream evidence directly proves that the current Codex MCP path assigns `sub_id = id.to_string()` for the spawned Codex submission
- LOGOS keeps both `request_id` and `submission_id` in the canonical phase-3 contract as an explicit LOGOS design choice, even though the current reference path makes them equal
- rationale: this future-proofs the runtime contract and prevents later code from silently assuming that internal submission routing is always derivable from external request identity without an explicit contract

## Lifecycle Ownership

### `codex-core`

- owns native Codex session execution once an MCP-backed Codex tool session is started
- owns native internal event production inside that spawned session
- does not own LOGOS-normalized correlation storage or phase-3 runtime policy

### MCP Boundary Owner

- for the current evidence-backed reference path this is the upstream MCP tool runner layer around Codex sessions
- it owns translating MCP request ids into Codex submission ids, returning `threadId`, and routing approval callbacks/responses through the original request identity

### `logos-tool-runtime`

- owns the normalized pending-call and execution payload shapes for runtime-owned and MCP-backed runtime tools
- must preserve the MCP correlation bundle unchanged once the bundle enters LOGOS-owned runtime state
- must not collapse MCP-backed runtime calls into plain `call_id` plus JSON arguments

### `logos-session-engine`

- owns lifecycle state transitions, approval branch handling, reinjection readiness, and terminality checks
- must treat MCP correlation artifacts as durable turn-state inputs, not logging extras
- owns the exact ingress handoff where `request_id` and `submission_id` are attached to a collected `McpBackedRuntime` call before approval/execution

### `logos-provider-openai`

- owns mapping OpenAI/Codex provider events into normalized tool lifecycle events
- does not own MCP correlation for runtime-owned or MCP-backed runtime tools, because that correlation originates above the provider adapter boundary

## Terminal Invariants

For the phase-3 scope, every normalized tool call that reaches `ToolCallStarted` must satisfy exactly one terminal runtime branch:

1. `CompletedWithResult`
   - execution finished and a correlated tool result is available for reinjection or external response delivery
2. `DeniedByApproval`
   - approval path resolved to denial and no execution occurred
3. `FailedBeforeResult`
   - execution or orchestration failed before a successful terminal result was produced
4. `Interrupted`
   - the turn or tool lifecycle stopped because the session was interrupted, cancelled, or disconnected before a terminal result was available

Additional invariants:

1. one started call must not produce more than one terminal branch
2. approval denial is terminal for execution, even if a synthetic error-shaped tool result is later emitted for model follow-up
3. result reinjection must always point to the original `call_id`
4. MCP `thread_id` may only be attached to the terminal branch that actually produced or resumed the MCP-backed Codex tool session

## Approval Correlation Rules

1. approval requests must correlate to the exact `call_id`
2. for MCP-backed runtime tools, approval records must also preserve the MCP correlation bundle
3. approval responses from host/daemon transport must route by `request_id` at the outer boundary and by `call_id` inside normalized runtime state
4. approval denial must not erase MCP continuation data that already exists for the call

## Persistence/Resume Handoff Requirements

Phase-4 implementation stays deferred, but the phase-3 contract now fixes the minimum future handoff payload for MCP-backed runtime tools:

1. `call_id`
2. tool name
3. tool execution kind
4. approval branch state
5. terminal lifecycle state
6. MCP correlation bundle: `request_id`, `submission_id`, optional `thread_id`
7. tool result presence plus error flag

This is enough to avoid reopening the tool-loop contract during persistence/resume design.

## OpenAI/Codex-Native Versus LOGOS-Owned Surfaces

### Remain OpenAI/Codex-Native

1. provider transport event ontology
2. provider-side request serialization and stream mechanics
3. native Codex thread execution inside the spawned MCP-backed tool session

### Become LOGOS-Owned For Phase 3

1. normalized storage of MCP correlation bundle once it enters runtime state
2. lifecycle and terminal invariants around started calls
3. approval correlation rules at normalized runtime and host/daemon boundaries
4. durability handoff requirements for later persistence/resume

## Seam Decision

- no new seam is assumed by this discussion doc
- the first implementation attempt must try to satisfy the phase-3 correlation contract in `logos-runtime-types`, `logos-tool-runtime`, and `logos-session-engine`
- only if the MCP boundary cannot preserve `request_id`, `submission_id`, or `thread_id` without a Codex-base hook may a new seam be proposed

## Implementation Readiness Verdict

- for the current documented scope, the architecture blind spot is now closed
- phase-3 is implementation-ready on paper after this narrowing slice
- remaining work is execution against the locked spec, not another discovery pass

## Evidence

- path: `2026-04-08-phase-3-tool-loop-boundary-discussion.md`
  symbol: `What Phase 3 Still Has To Solve`
  lines: `60-101`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-70`
- path: `2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Approval Flow Rules`
  lines: `78-102`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Daemon Owns`
  lines: `51-59`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Resume and approval are separate control surfaces`
  lines: `68-82`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Tool-Loop Findings`
  lines: `167-179`
- path: `../research/2026-03-30-logos-codex-evidence-appendix.md`
  symbol: `MCP call results carry thread continuation data`
  lines: `73-92`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `create_call_tool_result_with_thread_id`
  lines: `32-52`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `run_codex_tool_session`
  lines: `100-119`
- path: `../../../input-sources-temp/codex-upstream/codex-rs/mcp-server/src/codex_tool_runner.rs`
  symbol: `run_codex_tool_session_inner`
  lines: `192-310`
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
  symbol: `run_turn_with_tools_inner`
  lines: `153-266`

## Links

- [[README.md]]
- [[2026-04-08-phase-3-tool-loop-boundary-discussion.md]]
- [[2026-04-02-logos-tool-loop-architecture-spec.md]]
- [[2026-04-02-logos-approval-control-architecture-spec.md]]
- [[2026-04-02-logos-host-daemon-transport-spec.md]]
- [[../implementation/2026-04-08-phase-3-tool-loop-boundary-strict-spec.md]]
- [[../implementation/2026-04-08-phase-3-mcp-tool-correlation-strict-spec.md]]
