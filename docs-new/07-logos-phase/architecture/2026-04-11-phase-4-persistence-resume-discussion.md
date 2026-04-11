---
id: logos-phase-4-persistence-resume-discussion
title: LOGOS Phase 4 Persistence And Resume Discussion
type: discussion
status: active
tags: [architecture, discussion, phase-4, persistence, resume, journal, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [
      ../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md,
    ],
    [
      ../implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md,
    ],
    [../implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-03-logos-compaction-strategy-discussion.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-host-daemon-transport-spec.md],
    [2026-04-02-logos-approval-control-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 4 Persistence And Resume Discussion

## Purpose

- turn phase-4 into an exact execution lane after the now-accepted phase-1, phase-2, and phase-3 work
- lock durable journal and resume semantics without reopening OpenAI preservation, tool-loop boundary design, or host UI work
- define what must become durable canonical truth and what must remain optional debug-only provider residue

## Locked Inputs From Earlier Phases

Phase-4 does not start from a blank persistence concept. It inherits concrete locked inputs from earlier accepted work.

1. explicit continuation handles already exist, including canonical OpenAI `previous_response_id`
2. OpenAI turn-state preservation already has a defined within-turn carryover shape and reset boundary
3. compaction backend classification already exists, and OpenAI native remote rewrite is already the canonical fast path
4. `ToolCallRecord`, `ToolCallTerminalState`, and `ToolCallResultDisposition` already exist as normalized tool-loop terminal artifacts
5. approval requests and outcomes are already explicit normalized control-flow artifacts
6. MCP-backed runtime calls already preserve an explicit MCP correlation bundle
7. current ownership split is already locked: adapters normalize provider behavior, session engine owns orchestration, tool runtime owns runtime execution seams, and durable state authority belongs to LOGOS runtime rather than provider wire payloads

## What Phase 4 Now Means

Phase 4 is the first durable-state implementation lane, not a general storage exploration phase.

It means:

1. define and implement a durable journal v0 as canonical runtime truth
2. define write/read ownership and ordering guarantees for that journal
3. support resume after process restart for the canonical OpenAI path
4. support replay-based resume for stateless adapters from the same journal family
5. define compaction checkpoint interaction with the journal without promoting provider compact output into canonical truth
6. durably persist approval state and tool terminal records so restart does not erase control-flow truth

It does not mean:

1. broad storage-engine experimentation detached from runtime semantics
2. provider-neutral continuation fiction
3. widening into non-OpenAI implementation work in the same phase-4 lock pass

## Canonical Phase-4 Scope

### 1. Durable Journal v0

- a LOGOS-native journal becomes the canonical durable store for replayable state
- the journal must be sufficient to recover the OpenAI canonical path after process restart and to reconstruct stateless replay turns without relying on UI memory or raw provider transcripts

### 2. Write/Read Ownership

- daemon owns all journal writes and reads
- adapters do not write canonical journal records directly
- host transport consumes derived ordered events and session views, not journal authority

### 3. Resume After Process Restart

- OpenAI resume means the daemon can recover a persisted session, reload canonical state, and continue using the preserved OpenAI continuation handle when it is still valid
- replay adapters resume by reconstructing the next turn from canonical journal state rather than by pretending to own server-managed continuation

### 4. Compaction Interaction

- compaction checkpoints become journal-visible state transitions
- provider-native compaction output remains non-canonical input to a journal update, not durable instruction truth by itself
- after compaction, resume must rebuild the next turn from canonical initial context plus journal state plus the checkpointed conversation shape, not from raw compacted provider artifacts alone

### 5. Approval And Tool-Terminal Persistence

- pending approvals, approval outcomes, tool terminal records, interrupted branches, and MCP correlation data must become durable so restart does not erase safety-critical control-flow facts

### 6. Source-Of-Truth Boundary

- canonical truth lives in normalized journal records
- optional raw provider snapshots may be stored for diagnostics, drift analysis, or later compatibility work
- raw provider snapshots are never the primary source for resume or replay correctness

## Required Decisions Locked In Phase 4

### Durable Journal Record Families

Phase-4 must explicitly lock the journal record families needed for restart and replay. The companion taxonomy discussion narrows the exact families and field classes.

### Canonical Versus Optional Fields

- canonical fields are the ones required for runtime correctness, replay, resume reconstruction, safety, and auditability
- optional fields are debug-only observability residue and must never be required for correctness on the canonical path

### Write Ordering And Terminality

- journal writes must preserve enough ordering to reconstruct turn progression, approval control flow, tool terminal state, and compaction checkpoints deterministically
- turn completion and failure remain terminal journal facts, not inferred side effects

### Resume Semantics By Mode

1. OpenAI server-managed continuation:
   - primary resume path uses persisted continuation handle plus canonical journal state
   - if the persisted handle is invalid, fallback behavior must be explicit and journal-driven rather than silent provider magic
2. stateless replay adapters:
   - next turn is reconstructed from journal state
   - replay is a first-class mode, not fallback-only repair
3. interrupted tool or approval branches:
   - resume must reconstruct whether execution is pending, denied, interrupted, terminal, or waiting on host/daemon approval response
4. compaction checkpoints:
   - resume must treat checkpoint records as journal state transitions that change future replay input, not as whole-session replacement truth

### Minimum Data Needed For Later Phases

- phase-4 must leave phase-5 and later provider validation work with a closed persistence contract so validation adapters do not reopen journal family design, approval durability, tool terminal taxonomy, or compaction checkpoint meaning

## Non-goals

1. no non-OpenAI adapter implementation in this pass
2. no broad storage-engine yak shave beyond what is needed to lock the contract
3. no fake-neutral continuation model
4. no broad `codex-core` refactor
5. no plugin work
6. no host UI work

## Conditional Seam Position

- phase-4 does not assume a new Codex-base seam
- if later implementation proves that OpenAI resume correctness after process restart cannot be preserved outside additive LOGOS crates, any seam must be documented as conditional, narrow, additive, separately committed, and immediately registered
- until such proof exists, persistence design must assume the journal, replay logic, and resume orchestration stay outside `codex-core`

## Remaining Blind Spot

- the earlier persistence architecture spec leaves the smallest durable record set open
- that is architecture-critical because phase-4 cannot be implementation-ready until record families, field classes, and write ordering are fully locked
- this blind spot is clearly bounded, so this pass creates one companion narrowing discussion for journal record taxonomy rather than deferring the ambiguity to implementation

## Implementation Readiness Verdict

- phase-4 can go directly to implementation after this phase-4 discussion/spec pair together with the companion `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
- no further pre-implementation narrowing pass is required if those three docs are treated as one locked paper set

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 4: Persistence And Resume`
  lines: `153-173`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Durable Journal Minimum Record Model`
  lines: `45-58`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Resume Modes`
  lines: `59-75`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Canonical Storage Boundaries`
  lines: `103-124`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Session And Branching Rules`
  lines: `126-131`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Primary Implementation Phase`
  lines: `131-138`
- path: `2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Open Questions`
  lines: `147-152`
- path: `2026-04-02-logos-runtime-contract-spec.md`
  symbol: `Adapter Architecture Boundary`
  lines: `70-106`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Daemon Owns`
  lines: `51-59`
- path: `2026-04-02-logos-host-daemon-transport-spec.md`
  symbol: `Transport And Persistence Interaction`
  lines: `136-140`
- path: `2026-04-02-logos-approval-control-architecture-spec.md`
  symbol: `Approval Flow Rules`
  lines: `78-92`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-70`
- path: `../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Delivered Preservation Work`
  lines: `36-43`
- path: `../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `Phase-2 Completion State`
  lines: `78-96`
- path: `../implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md`
  symbol: `Delivered In Phase 3`
  lines: `32-39`
- path: `../implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md`
  symbol: `Phase-3 Completion State`
  lines: `94-110`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationHandle`
  lines: `4-17`
- path: `../../../codex-rs/logos-runtime-types/src/capabilities.rs`
  symbol: `ProviderCapabilitySet`
  lines: `13-24`
- path: `../../../codex-rs/logos-runtime-types/src/tools.rs`
  symbol: `ToolCallRecord`
  lines: `51-67`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `37-145`
- path: `../../../codex-rs/logos-provider-openai/src/adapter.rs`
  symbol: `provider_capabilities`
  lines: `64-76`

## Links

- [[README.md]]
- [[2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-03-logos-compaction-strategy-discussion.md]]
- [[2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
- [[../implementation/2026-04-11-phase-4-persistence-resume-strict-spec.md]]
