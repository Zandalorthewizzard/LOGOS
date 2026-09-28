---
id: logos-item-03-fork-resume-draft-implementation-discussion
title: Item 03 Fork / Resume Orchestration Draft Implementation Discussion
type: discussion
status: active
tags:
  [logos, architecture, phase-6, session-state, fork, resume, research-transfer]
related:
  [
    [README.md],
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-resume-support-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../research/session layer/RA-1.3-fork-resume.md],
  ]
depends_on:
  [
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-resume-support-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 03: Fork / Resume Orchestration Draft Implementation Discussion

## Purpose

This document turns the validated `Item 03: Fork / Resume Orchestration` research verdict into a practical LOGOS implementation draft.

## Current Transfer Status

1. `Item 03` verdict is `ADAPT` with `DUAL_CONSENSUS_VALID`.
2. The ADAPT arises because fork/resume ownership belongs to LOGOS Layer 2, but continuation semantics differ fundamentally between provider types: OpenAI uses server-managed `previous_response_id`, stateless providers use client-managed replay. No competitor has this duality, so no competitor pattern maps directly.
3. This draft exists so future spec work can start from code-aware implementation shape instead of only from theory.

## Accepted Import For Item 03

1. Fork and resume are LOGOS session-state layer concerns, not adapter responsibilities.
2. When persistence is robust enough, resume should approach "load from durable state → reconstruct runtime → continue" (from RA-1.3 and OpenCode's stateless model).
3. Parent tracking as first-class metadata (from Codex `thread_spawn_edges` + OpenCode `parentID`).
4. Fork as an operation over durable state, not only over live in-memory threads.
5. Fork granularity: whole-history for MVP, message/turn boundary post-MVP.

## What Requires Adaptation (Not Direct Accept)

1. **Resume orchestration must be provider-policy-aware.** OpenAI canonical: continuation handle. Stateless: replay from journal/persistence. Cannot flatten to one path.
2. **Fork must work over durable state.** Current OR fork requires live `Arc<OpenRouterBetaThread>`. With ephemeral runtime (Item 01), source thread may not be in memory.
3. **Config and model validation on fork/resume.** No competitor validates model availability or tool compatibility. LOGOS should.
4. **Listener reattachment after resume.** LOGOS has persistent SSE connections; competitors do not. This is a LOGOS-specific concern.
5. **Continuation backend policy.** Execution policy from ownership doc: provider-native first → LOGOS-managed fallback → degraded/unsupported.

## Explicitly Rejected

1. Resume as "just load from DB" as universal truth (works for stateless only).
2. Fake-neutral continuation model that hides provider differences.
3. Flat SQL sessions without provider continuation policy hooks.

## What This Draft Covers

1. Fork orchestration ownership and flow.
2. Resume orchestration ownership and flow.
3. Parent/lineage tracking.
4. Config/model validation on fork/resume.
5. Continuation backend selection policy.
6. Active listener reattachment.

## What This Draft Does Not Cover

These remain separate items and must not be silently folded into `Item 03`:

1. Thread registry and ephemeral runtime (Item 01).
2. Session persistence schema and migrations (Item 02).
3. Turn lifecycle and notification dispatch (Item 04).
4. Review orchestration (Item 06) — review builds on top of fork/resume but is a separate product workflow.
5. Rollback engine (Item 08).
6. Compaction (Items 09/13).
7. Journal record family implementation (phase-4 T2).
8. OpenAI canonical-path fork/resume changes.

## Current Code Diagnosis

### OR Fork Requires Live In-Memory Thread

`fork_thread` (`openrouter_beta_threads.rs:1075-1178`):

1. Looks up `source_thread` in `self.threads` HashMap — **requires live in-memory presence**.
2. Locks runtime, clones entire history array.
3. Creates `OpenRouterBetaSession::forked_from(source_history, source_continuation, settings)`.
4. Creates new `OpenRouterBetaThread` with `forked_from_id: Some(source_thread_id)`.
5. Persists new thread. Inserts into in-memory map. Attaches listener.

Problem: if source thread is not loaded (ephemeral model, process restart), fork fails. With Item 01 Stage B (ephemeral runtime), the source thread will commonly not be in memory.

### OR Resume From Persistence Is Monolithic-JSON Based

`resume_persisted_thread` (`openrouter_beta_threads.rs:413-479`):

1. Loads `PersistedOpenRouterThread` from JSON snapshot.
2. Rebuilds `OpenRouterBetaSession::forked_from(persisted.history, persisted.continuation, settings)`.
3. Creates new `OpenRouterBetaThread`. Inserts into in-memory map. Attaches listener.

Problem: tied to monolithic JSON format. After Item 02, history lives in JSONL/journal, metadata in SQLite. Resume must adapt.

### Codex Has Structured Fork/Resume With Lineage

Codex uses `InitialHistory` enum (`protocol.rs:2259-2263`):

- `New` — fresh session
- `Resumed(ResumedHistory)` — resume with `conversation_id`, `history: Vec<RolloutItem>`, `rollout_path`
- `Forked(Vec<RolloutItem>)` — fork with copied rollout items

`ThreadManager::fork_thread` (`thread_manager.rs:596`):

- Reads source rollout JSONL
- Truncates at Nth user message (turn boundary granularity)
- Creates new CodexThread with `InitialHistory::Forked(truncated_items)`

`thread_spawn_edges` table (`state/src/runtime/threads.rs:92-106`):

- `parent_thread_id`, `child_thread_id`, `status`
- Supports descendant queries via recursive CTE
- Upsert pattern: `ON CONFLICT(child_thread_id) DO UPDATE`

This is the most structured fork/resume implementation in the codebase. LOGOS stateless path should borrow the shape but adapt the backing store.

### OR Has No Validation On Fork/Resume

Neither `fork_thread` nor `resume_persisted_thread` validates:

- Is the model still available?
- Are tools referenced in history still registered?
- Is the config snapshot still valid?
- Has the provider changed capabilities?

RA-1.3 confirms: "None of the projects do this." LOGOS should.

### Thread.forked_from_id Already Exists

`Thread` struct already has `forked_from_id: Option<String>`. The wire protocol already carries it. Tests verify it (`openrouter_beta.rs:983`). This field is the existing lineage surface.

### Listener Reattachment Already Works

`ThreadStateManager::try_add_connection_to_thread` is called in both `fork_thread` and `resume_persisted_thread`. This pattern already works and should be preserved.

## Draft Target Model

### 1. Two Resume Paths Based On Continuation Mode

```
┌─────────────────────────────────────────────┐
│  Resume Request                              │
│  ↓ resolve thread from registry              │
│  ↓ load durable session state                │
│  ↓ validate config/model/tools               │
│                                              │
│  ┌─ Continuation Mode? ──────────────────┐   │
│  │                                       │   │
│  │  ServerManaged (OpenAI)               │   │
│  │  → use persisted continuation handle  │   │
│  │  → if handle invalid, journal fallback│   │
│  │                                       │   │
│  │  ClientManaged (stateless)            │   │
│  │  → replay from journal/persistence    │   │
│  │  → seed_history → run_turn            │   │
│  │                                       │   │
│  │  Hybrid                               │   │
│  │  → try provider-native first          │   │
│  │  → fallback to client-managed         │   │
│  └───────────────────────────────────────┘   │
│  ↓                                           │
│  build SessionEngine                         │
│  ↓                                           │
│  attach listener                             │
└─────────────────────────────────────────────┘
```

This follows the execution policy from the ownership doc: provider-native first → LOGOS-managed fallback → degraded/unsupported.

### 2. Fork Over Durable State

Fork must work whether source thread is in memory or not:

1. Resolve source thread from registry (Item 01).
2. Load source session state from persistence (Item 02).
3. Copy history/config to new thread record.
4. Record lineage (`forked_from_id` + edge record).
5. Persist new thread.
6. Optionally construct runtime if immediate turn is needed.

This removes the dependency on live `Arc<OpenRouterBetaThread>`.

### 3. Lineage Tracking

Two complementary surfaces:

1. **`forked_from_id` field** on `Thread` struct — already exists, used by wire protocol.
2. **Edge records** in session-state layer — modeled after Codex `thread_spawn_edges` but stored in LOGOS session-state SQLite (Item 02), not Codex state DB.

Edge record schema:

- `parent_thread_id TEXT NOT NULL`
- `child_thread_id TEXT NOT NULL`
- `fork_type TEXT NOT NULL` — `fork`, `review`, `scheduled` (extensible)
- `status TEXT NOT NULL` — `active`, `completed`, `abandoned`
- `created_at INTEGER NOT NULL`

This supports descendant queries (recursive CTE) for future review orchestration (Item 06) and scheduler features.

### 4. Validation Step

Before constructing runtime on fork/resume:

1. **Model availability:** check that `config_snapshot.model` is still reachable through the thread's provider.
2. **Tool compatibility:** check that tools referenced in history still exist in current tool registry. If not, mark as degraded.
3. **Config freshness:** check that approval policy, sandbox policy, and other config have not been globally overridden in ways that conflict with the snapshot.
4. **Provider capability:** check that the provider still supports the continuation mode the session expects.

If validation fails:

- Recoverable (model changed): return warning, allow resume with current model.
- Unrecoverable (provider gone): return explicit error, do not silently degrade.

This is a LOGOS-specific improvement over all competitors.

## Practical Draft Shape

### Stage A: Fork Over Durable State

1. Refactor `fork_thread` to accept a thread ID and resolve source from registry instead of requiring live `Arc<OpenRouterBetaThread>`.
2. Load source history/config from Item 02 persistence (JSONL transitional, later journal).
3. Create new thread with `forked_from_id` lineage.
4. Insert edge record in session-state SQLite.
5. Persist new thread metadata and history.

This stage removes the live-thread dependency from fork.

### Stage B: Provider-Policy-Aware Resume

1. Refactor `resume_persisted_thread` to use Item 02 persistence layer.
2. Add continuation mode dispatch: `ServerManaged` → use handle, `ClientManaged` → replay, `Hybrid` → try handle then fallback.
3. Add validation step (model, tools, config, provider capability).
4. Preserve listener reattachment through `ThreadStateManager`.

This stage makes resume work correctly for both provider types.

### Stage C: Review Integration Surface

1. Review orchestration (Item 06) consumes fork/resume as substrate.
2. `fork_type: "review"` edge records.
3. Detached review = fork + run review turn + report back.
4. Inline review = same-thread review turn (no fork).

This stage is gated on Item 06 design and must not block Stages A and B.

## Minimal Draft Data Model

### Edge Records (Additive to Item 02 Schema)

```sql
CREATE TABLE thread_edges (
    parent_thread_id TEXT NOT NULL,
    child_thread_id  TEXT NOT NULL PRIMARY KEY,
    fork_type        TEXT NOT NULL DEFAULT 'fork',
    status           TEXT NOT NULL DEFAULT 'active',
    created_at       INTEGER NOT NULL,
    FOREIGN KEY (parent_thread_id) REFERENCES thread_metadata(thread_id),
    FOREIGN KEY (child_thread_id) REFERENCES thread_metadata(thread_id)
);

CREATE INDEX idx_thread_edges_parent ON thread_edges(parent_thread_id);
CREATE INDEX idx_thread_edges_status ON thread_edges(status);
```

### Validation Result (In-Memory, Not Persisted)

```rust
struct ResumeValidation {
    model_available: bool,
    tools_compatible: bool,
    config_fresh: bool,
    provider_capable: bool,
    warnings: Vec<String>,
}
```

### Continuation Mode Dispatch

Existing `ContinuationMode` enum (`continuation.rs:13-17`):

- `ServerManaged` — use persisted `ContinuationHandle::OpenAiPreviousResponseId`
- `ClientManaged` — replay from journal/persistence via `SessionEngine::seed_history`
- `Hybrid` — try server-managed, fallback to client-managed

## Why This Still Supports The ADAPT Verdict

1. **Borrowed patterns:** load-from-durable-state resume posture, parent tracking, fork-as-durable-operation, trivialize-resume direction — all from competitors, all accepted.
2. **Custom LOGOS layer:** continuation mode dispatch, validation step, journal-as-source, listener reattachment — none of these exist in competitors, all required by LOGOS constraints.
3. **Provider-native backend where available:** execution policy preserved. OpenAI continuation stays native-first. Stateless providers use replay-first.
4. **The draft does not widen scope** into review orchestration, compaction, or notification dispatch.
5. **The ADAPT is strong, not weak:** we know exactly what we borrow, what we build custom, and where each piece lives.

## Dependencies On Earlier Items

| Dependency                    | What Item 03 Needs From It                                                  |
| ----------------------------- | --------------------------------------------------------------------------- |
| Item 01 (Thread Manager)      | Durable registry to resolve threads without live `Arc<>`                    |
| Item 02 (Session Persistence) | SQLite metadata + JSONL/journal to load history and config                  |
| Phase-4 Journal               | Canonical `MessageAppended`, `ContinuationUpdated`, `SessionOpened` records |

Item 03 cannot ship fully until Items 01 and 02 provide the registry and persistence substrate. However, the orchestration logic (continuation dispatch, validation, lineage tracking) can be designed and tested against mocks.

## Revisit Triggers

The current ADAPT verdict should be re-opened only if:

1. Validation proves that provider-policy-aware resume is unnecessary because all future providers will use client-managed history — would simplify toward ACCEPT.
2. Codex upstream introduces a shared fork/resume abstraction that LOGOS can reuse directly — would reduce custom work.
3. Phase-4 journal proves that no SQLite metadata is needed for fork/resume and all lineage can be derived from journal alone — would simplify persistence.

## Closeout Expectation

The later strict spec derived from this draft should be considered successful only if:

1. Fork works without requiring source thread to be in memory.
2. Resume works for both server-managed (OpenAI) and client-managed (stateless) continuation modes.
3. Parent lineage is queryable through edge records.
4. Validation catches model unavailability, tool incompatibility, and config drift on fork/resume.
5. Listener reattachment after resume preserves subscription correctness.
6. Review orchestration (Item 06) can consume fork/resume as substrate without redesigning it.
7. OpenAI canonical fork/resume path is not affected.

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 03`
  lines: `153-188`
- path: `../research/session layer/RA-1.3-fork-resume.md`
  symbol: `LOGOS Current Implementation`
  lines: `1-39`
- path: `../research/session layer/RA-1.3-fork-resume.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `171-182`
- path: `../research/session layer/RA-1.3-fork-resume.md`
  symbol: `Recommendations for LOGOS`
  lines: `184-208`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Ownership Layers`
  lines: `59-106`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Execution Policy`
  lines: `107-119`
- path: `2026-04-14-openrouter-thread-resume-support-discussion.md`
  symbol: `Ownership Baseline`
  lines: `46-50`
- path: `2026-04-14-openrouter-thread-resume-support-discussion.md`
  symbol: `Current Likely Path`
  lines: `65-69`
- path: `2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Stage B: Runtime Residency Shrinks`
  lines: `191-199`
- path: `2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Draft Target Model`
  lines: `101-165`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `fork_thread`
  lines: `1075-1178`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `resume_persisted_thread`
  lines: `413-479`
- path: `../../../codex-rs/core/src/thread_manager.rs`
  symbol: `fork_thread`
  lines: `596-620`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `InitialHistory`
  lines: `2259-2263`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `upsert_thread_spawn_edge`
  lines: `84-108`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationHandle`
  lines: `4-10`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationMode`
  lines: `12-17`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-14-openrouter-thread-resume-support-discussion.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[2026-04-17-item-02-session-persistence-draft-implementation-discussion.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../research/session layer/RA-1.3-fork-resume.md]]
