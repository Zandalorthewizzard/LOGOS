---
id: logos-phase-4-journal-record-taxonomy-discussion
title: LOGOS Phase 4 Journal Record Taxonomy Discussion
type: discussion
status: active
tags: [architecture, discussion, phase-4, journal, taxonomy, persistence, logos]
related:
  [
    [README.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [../implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 4 Journal Record Taxonomy Discussion

## Purpose

- close the last architecture-critical blind spot for phase-4 by fixing the exact journal record families, canonical field classes, and ordering rules
- prevent persistence implementation from inventing ad hoc record shapes that later phases would have to unpick

## Core Decision

- phase-4 journal v0 is an append-oriented canonical record log with a derived latest-session view
- the append log is the durable source of truth; derived session state is a convenience index or materialized view, not an alternative authority
- record families are limited, explicit, and normalized; optional raw provider snapshots stay out-of-band and non-canonical

## Journal Record Families

Phase-4 journal v0 uses the following canonical record families.

1. `SessionOpened`
   - starts the durable session identity and captures provider/model/mode baseline
2. `TurnStarted`
   - records the logical turn boundary and request policy baseline
3. `MessageAppended`
   - records normalized durable messages, including user, assistant, assistant-tool-call, tool-result, and compaction-checkpoint messages
4. `ContinuationUpdated`
   - records the current canonical continuation handle after a completed provider turn or equivalent update
5. `ApprovalStateChanged`
   - records pending approval, resolved approval, and any terminal approval branch needed for restart-safe reconstruction
6. `ToolCallTerminalRecorded`
   - records durable `ToolCallRecord` terminal facts
7. `TurnFinished`
   - records turn terminal outcome: completed or failed, with restart-safe classification
8. `CompactionApplied`
   - records the compaction policy decision and the journal-visible replacement boundary when a compaction checkpoint becomes durable
9. `ProviderSnapshotStored` (optional, non-canonical)
   - references stored raw provider snapshots for debugging or compatibility analysis only

## Canonical Field Classes

### Required Canonical Fields

Every canonical record family must carry, as applicable:

1. session id
2. monotonic durable sequence id
3. logical turn id when the record is turn-scoped
4. record family discriminator
5. timestamp

### Domain-Specific Canonical Fields

1. `SessionOpened`
   - provider id
   - model ref
   - continuation mode
   - provider mode/capability markers needed for resume policy
2. `TurnStarted`
   - turn id
   - request policy/config baseline needed for deterministic replay boundaries
3. `MessageAppended`
   - normalized `InternalMessage`
4. `ContinuationUpdated`
   - normalized `ContinuationHandle`
5. `ApprovalStateChanged`
   - stable approval correlation id
   - correlated `call_id` when applicable
   - normalized approval branch state
   - MCP correlation bundle when approval is tied to an MCP-backed runtime call
6. `ToolCallTerminalRecorded`
   - full `ToolCallRecord`
7. `TurnFinished`
   - turn terminal state
   - normalized error summary when failed
8. `CompactionApplied`
   - compaction backend kind
   - journal sequence boundary before/after checkpoint application
   - canonical checkpoint summary reference

### Optional Debug-Only Fields

These may be stored, but are never required for correctness:

1. raw provider request snapshot reference
2. raw provider response or event snapshot reference
3. provider extension payload reference
4. verbose diagnostics metadata used only for observability

## Prohibited As Canonical Fields

1. raw provider SSE enum values as primary replay truth
2. raw `ResponseItem` or provider wire structs as canonical message history
3. opaque provider blob dumps that are required to reconstruct approval or tool terminal state

## Write Ordering Rules

1. `SessionOpened` must exist before any turn record
2. `TurnStarted` must exist before any turn-scoped message, approval, continuation, tool terminal, or turn-finished record
3. `ApprovalStateChanged` and `ToolCallTerminalRecorded` must preserve journal order relative to the turn in which they occurred
4. `TurnFinished` is the only terminal turn record for a given turn id
5. `ToolCallTerminalRecorded` is the only terminal tool record for a given `call_id`
6. `CompactionApplied` must occur only at stable turn boundaries and must reference a durable checkpoint already written through `MessageAppended`

## Compaction Interaction Rules

1. a compaction checkpoint is first durably represented as a normalized `InternalMessage::CompactionCheckpoint`
2. `CompactionApplied` then records the policy boundary that makes the checkpoint authoritative for future replay input
3. provider-native compacted history is not itself the canonical durable record set
4. replay after compaction uses journal state after the latest `CompactionApplied` boundary plus subsequent records, not the full pre-compaction transcript every time

## Approval And Interrupted Branch Rules

1. pending approval must be journal-visible before the daemon waits on host response
2. approval resolution must be journal-visible before any resumed execution branch continues
3. interrupted approvals and interrupted tool calls must remain reconstructible after restart from canonical approval/tool terminal records rather than raw provider artifacts

## Implementation Readiness Contribution

- this companion discussion closes the exact journal-taxonomy blind spot left open in the earlier persistence architecture spec
- with this taxonomy locked, phase-4 no longer needs another pre-implementation design pass for record families or field classes

## Evidence

- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Durable Journal Minimum Record Model`
  lines: `45-58`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Canonical Storage Boundaries`
  lines: `103-124`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Session And Branching Rules`
  lines: `126-131`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Safety Rules To Keep`
  lines: `95-101`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Open Questions`
  lines: `147-152`
- path: `2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Approval Flow Rules`
  lines: `78-92`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-70`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Persistence / Resume Findings`
  lines: `144-165`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationHandle`
  lines: `4-17`
- path: `../../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `ToolCallRecord`
  lines: `51-67`

## Links

- [[README.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[../implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md]]
