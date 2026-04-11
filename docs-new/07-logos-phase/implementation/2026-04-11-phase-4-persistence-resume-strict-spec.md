---
id: logos-phase-4-persistence-resume-strict-spec
title: LOGOS Phase 4 Persistence And Resume Strict Spec
type: spec
status: active
tags:
  [implementation, strict-spec, phase-4, persistence, resume, journal, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md],
    [2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md],
    [../architecture/2026-04-11-phase-4-persistence-resume-discussion.md],
    [../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
  ]
depends_on:
  [
    [../architecture/2026-04-11-phase-4-persistence-resume-discussion.md],
    [../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../architecture/2026-04-03-logos-compaction-strategy-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 4 Persistence And Resume Strict Spec

## Start Here

1. `../architecture/2026-04-11-phase-4-persistence-resume-discussion.md`
2. `../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
3. `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
4. `../architecture/2026-04-03-logos-compaction-strategy-discussion.md`
5. `../architecture/2026-04-02-logos-runtime-contract-spec.md`
6. `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
7. `../architecture/2026-04-02-logos-approval-control-architecture-spec.md`
8. `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
9. `2026-03-30-logos-global-roadmap.md`
10. `2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
11. `2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md`

## Intent

Implement phase-4 as the durable journal and restart-safe resume lane for the canonical OpenAI path and the stateless replay path, using one locked journal family without reopening continuation, tool-loop, approval, or compaction semantics.

Non-goals:

1. non-OpenAI adapter implementation work in this phase-4 lock pass
2. broad storage-engine yak shave beyond what is needed for the contract
3. fake-neutral continuation semantics
4. broad `codex-core` refactor
5. plugin work
6. host UI work

## Locked Inputs

1. explicit continuation handle exists and remains canonical
2. OpenAI turn-state preservation shape is already locked
3. compaction backend classification and OpenAI native remote fast path are already locked
4. `ToolCallRecord`, tool terminal states, result disposition, approval events, and MCP correlation bundle are already locked inputs
5. daemon-owned persistence authority and host transport non-authority are already locked

## Target Outcome

1. durable journal v0 exists as canonical runtime truth
2. OpenAI server-managed continuation can resume after process restart using journal-backed state
3. stateless replay adapters can reconstruct the next turn from the same journal family
4. approval state, tool terminal state, and compaction checkpoint transitions survive restart
5. optional raw provider snapshots remain debug-only and non-canonical

## Locked Decisions

1. journal v0 is append-oriented with a derived latest-session view
2. journal record families are the ones locked by the companion taxonomy discussion
3. canonical resume never depends on raw provider wire payloads
4. compaction is a journal/session policy surface, not a provider bool
5. OpenAI native remote compaction remains preserved as the canonical fast path, but journal truth remains normalized LOGOS data
6. any new Codex-base seam is conditional only

## Proof Targets

### Already Satisfied By Earlier Phases

1. explicit continuation handle exists
2. OpenAI turn-state preservation shape exists
3. compaction backend classification exists
4. approval events and tool terminal records exist
5. MCP correlation bundle exists

### New Work Required In Phase 4

1. durable journal record family implementation and persistence read/write ownership
2. restart-safe OpenAI resume using persisted canonical session state
3. stateless replay reconstruction from the journal for a validation adapter path
4. compaction checkpoint persistence and replay interaction
5. restart-safe approval and interrupted branch reconstruction
6. optional raw provider snapshot policy that stays non-canonical

## Required Journal Record Families

Phase-4 implementation must include durable support for these canonical record families:

1. `SessionOpened`
2. `TurnStarted`
3. `MessageAppended`
4. `ContinuationUpdated`
5. `ApprovalStateChanged`
6. `ToolCallTerminalRecorded`
7. `TurnFinished`
8. `CompactionApplied`

Optional debug-only family:

9. `ProviderSnapshotStored`

## Write Ordering And Terminality Rules

1. session open precedes all turn records
2. turn start precedes all turn-scoped durable records
3. turn finish is the only terminal turn record for one turn id
4. tool terminal record is the only terminal tool record for one `call_id`
5. approval state changes must persist in causal order relative to tool execution and turn finish
6. compaction applied must only occur at stable turn boundaries and after the compaction checkpoint message is already durable

## Resume Contract By Mode

### OpenAI Server-Managed Continuation

1. persist canonical continuation handle
2. persist enough normalized session state to recover if process restarts between turns
3. on resume, use persisted continuation handle when valid
4. if the handle is unusable, fallback must be explicit and journal-driven rather than hidden adapter magic

### Stateless Replay Adapters

1. rebuild replay input from canonical journal state
2. replay input must honor compaction checkpoints and post-checkpoint history boundaries
3. replay must include approval/tool-terminal outcomes needed for safe continuation

### Interrupted Tool Or Approval Branches

1. pending approval must be restart-visible
2. denied, approved, interrupted, and terminal tool states must be restart-visible
3. interrupted branches must not degrade into ordinary completed or failed paths on resume

### Compaction Checkpoints

1. checkpoint message becomes canonical only when durably written
2. compaction application changes future replay boundaries
3. raw compacted provider output is never the canonical instruction source

## Ordered Tasks

1. `T1`

- package: `docs-new/07-logos-phase/architecture`, `docs-new/07-logos-phase/implementation`
- output: completed phase-4 paper lock with companion taxonomy discussion

2. `T2`

- package: additive LOGOS runtime crates only
- output: journal v0 record types, persistence read/write seam, and restart-safe session reconstruction surfaces

3. `T3`

- package: OpenAI additive LOGOS crates plus journal integration
- output: restart-safe OpenAI resume using persisted continuation and normalized journal state

4. `T4`

- package: stateless replay-capable additive LOGOS crates
- output: journal-driven replay reconstruction path for stateless adapters without fake server-managed continuation

5. `T5`

- package: tests and acceptance docs
- output: restart, replay, compaction, approval, and interrupted-branch proofs plus phase-4 acceptance report

6. `T6`

- package: `codex-rs/core` only if proven necessary
- output: optional narrow seam, separately justified and registered, only if additive crates cannot preserve canonical OpenAI restart semantics without it

## Requirement -> Task -> Proof -> Gate

| Requirement                                                                                   | Task IDs               | Proof Source                             | Acceptance | Gates |
| --------------------------------------------------------------------------------------------- | ---------------------- | ---------------------------------------- | ---------- | ----- |
| Journal v0 record families are explicit and canonical                                         | `T1`, `T2`, `T5`       | record type review and persistence tests | `A1`       | `G1`  |
| OpenAI restart-safe resume uses explicit continuation plus journal state                      | `T2`, `T3`, `T5`       | restart tests on canonical OpenAI path   | `A2`       | `G2`  |
| Stateless replay resume works from the same journal family                                    | `T2`, `T4`, `T5`       | replay tests on validation adapter path  | `A3`       | `G3`  |
| Approval and tool terminal branches survive restart                                           | `T2`, `T3`, `T4`, `T5` | approval/tool resume tests               | `A4`       | `G4`  |
| Compaction checkpoints interact with the journal without becoming canonical instruction truth | `T2`, `T3`, `T4`, `T5` | compaction restart/replay tests          | `A5`       | `G5`  |
| Raw provider snapshots remain optional and non-canonical                                      | `T2`, `T5`             | type review and persistence tests        | `A6`       | `G6`  |
| Any new seam remains conditional and separately governed                                      | `T6`, `T5`             | seam index entry only if needed          | `A7`       | `G7`  |

## Acceptance And Gates

### A1. Journal v0 is canonical and minimal

- implementation must use the locked record families
- canonical correctness must not depend on raw provider snapshots

### A2. OpenAI restart-safe resume is faithful

- proofs must show process restart can recover canonical OpenAI continuation behavior without illegal hidden session reuse

### A3. Stateless replay is first-class

- proofs must show stateless replay reconstruction works from the same journal family, not a separate persistence model

### A4. Approval and tool branches are durable

- proofs must show pending approval, approval outcomes, tool terminal records, and interrupted branches survive restart correctly

### A5. Compaction remains policy-safe

- proofs must show compaction checkpoints do not become canonical instruction truth and do not corrupt replay/resume semantics

### A6. Raw provider snapshots stay optional

- turning snapshot storage off must not break correctness for canonical resume or replay

### A7. Seam governance remains conditional

- if no seam is needed, none is added
- if a seam becomes necessary, it must be narrow, evidence-backed, separately committed, and registered in `2026-04-03-codex-core-seam-index.md`

## Implementation Readiness Verdict

- verdict: implementation-ready
- required paper set: this strict spec, `2026-04-11-phase-4-persistence-resume-discussion.md`, and `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
- no further pre-implementation narrowing pass is required for the current phase-4 scope

## Deferred Risks Outside Phase 4

1. non-OpenAI provider implementation correctness beyond the stateless replay validation lane
2. storage-engine scaling, migration, and operational tuning beyond journal contract v0
3. plugin and host UI integration work
4. later provider-native fast paths that may require additional persistence nuance after validation phases

## Evidence

- path: `../architecture/2026-04-11-phase-4-persistence-resume-discussion.md`
  symbol: `Locked Inputs From Earlier Phases`
  lines: `37-49`
- path: `../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Journal Record Families`
  lines: `34-47`
- path: `../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Write Ordering Rules`
  lines: `88-97`
- path: `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Resume Modes`
  lines: `59-75`
- path: `../architecture/2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Safety Rules To Keep`
  lines: `95-101`
- path: `../architecture/2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Transport And Persistence Interaction`
  lines: `136-140`
- path: `../architecture/2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Logos Runtime Owns`
  lines: `85-97`
- path: `2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Phase-2 Completion State`
  lines: `80-96`
- path: `2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md`
  symbol: `Phase-3 Completion State`
  lines: `94-110`
- path: `../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationHandle`
  lines: `4-17`
- path: `../../codex-rs/logos-runtime-types/src/capabilities.rs`
  symbol: `ProviderCapabilitySet`
  lines: `13-24`
- path: `../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `ToolCallRecord`
  lines: `51-67`
- path: `../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `37-145`

## Links

- [[README.md]]
- [[2026-03-30-logos-global-roadmap.md]]
- [[../architecture/2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
