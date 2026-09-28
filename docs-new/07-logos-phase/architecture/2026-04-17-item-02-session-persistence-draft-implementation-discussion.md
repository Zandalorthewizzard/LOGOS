---
id: logos-item-02-session-persistence-draft-implementation-discussion
title: Item 02 Session Persistence Draft Implementation Discussion
type: discussion
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    persistence,
    session-persistence,
    research-transfer,
  ]
related:
  [
    [README.md],
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../research/session layer/RA-1.2-session-persistence.md],
  ]
depends_on:
  [
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-phase-4-persistence-resume-discussion.md],
    [2026-04-11-phase-4-journal-record-taxonomy-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 02: Session Persistence Draft Implementation Discussion

## Purpose

This document turns the validated `Item 02: Session Persistence` research verdict into a practical LOGOS implementation draft.

## Current Transfer Status

1. `Item 02` verdict is `ADAPT` with `DUAL_CONSENSUS_VALID`.
2. The ADAPT arises because LOGOS already has a locked phase-4 durability contract where the journal is canonical runtime truth. The new session persistence layer must enhance the non-canonical session-state layer around that journal, not replace it.
3. This draft exists so future spec work can start from code-aware implementation shape instead of only from theory.

## Accepted Import For Item 02

The ADAPT imports are:

1. SQLite metadata store for thread/session metadata, archive state, listing, indexes, provider/model refs, timestamps.
2. WAL mode + busy timeout + foreign keys for transactional integrity.
3. Versioned schema migrations following the Goose pattern.
4. Partial update builder following the Goose `SessionUpdateBuilder` pattern.
5. Legacy import path for current JSON snapshots into the new store.
6. Soft-archive field (`archived_at`) instead of file-move between directories as the primary archive model.
7. Stable message/thread record IDs adapted to LOGOS runtime semantics.

## What Requires Adaptation (Not Direct Accept)

1. **Hybrid persistence model.** SQLite for metadata/index/state refs, LOGOS journal remains canonical runtime truth, heavy history/blob payloads live separately or are derived from journal.
2. **Session persistence boundary.** The new store wraps around journal ownership rather than replacing it.
3. **Compaction-aware storage.** Must respect LOGOS journal `CompactionApplied` boundaries, not introduce its own model.
4. **History durability.** Journal `MessageAppended` records are canonical history. The SQLite store may index or cache derived views but must not become the primary durability owner for message content.

## Explicitly Rejected

1. Per-delta event sourcing for streaming (OpenCode pattern).
2. Full transfer of canonical truth into SQLite blobs.
3. Single tokenizer for all providers (Goose pattern).
4. Replacement of append/replay model with a pure SQL model.

## What This Draft Covers

1. SQLite metadata store schema for non-canonical thread/session records.
2. Write/read ownership boundary between the new metadata store and the phase-4 journal.
3. Legacy import path from current JSON snapshots.
4. History/blob separation strategy.
5. Migration framework structure.
6. Query surfaces for list/read/archive/filter.

## What This Draft Does Not Cover

These remain separate items and must not be silently folded into `Item 02`:

1. Thread manager registry ownership (Item 01).
2. Turn lifecycle and notification dispatch (Item 04).
3. Fork/resume orchestration (Item 03).
4. Shell command execution (Item 05).
5. Compaction engine implementation (Items 09/13).
6. Journal record family implementation (phase-4 T2).
7. OpenAI canonical-path persistence changes.

## Current Code Diagnosis

### OR Persistence Is A Monolithic JSON Snapshot

`openrouter_beta_persistence.rs` (267 lines) stores `PersistedOpenRouterThread`:

- `version` — snapshot format version
- `config_snapshot` — full config (model, provider, policies, cwd, etc.)
- `thread` — `Thread` metadata (id, turns, name, status, preview, path, git_info)
- `history: Vec<InternalMessage>` — complete message history
- `continuation: Option<ContinuationHandle>` — resume context
- `elicitation_count: u64` — approval gating state

Everything is serialized as one JSON file. `write_snapshot` does full rewrite on every persist. `list_snapshots` loads every file to enumerate threads.

Weaknesses:

- Monolithic JSON: corruption = total session loss.
- No incremental writes: full rewrite on every persist.
- Snapshot grows linearly with history.
- No transactional guarantee during write.
- Listing requires loading every snapshot file.
- Different format from Codex rollout (no shared tooling).

### Phase-4 Journal Owns Canonical Runtime Truth

Phase-4 (`2026-04-11-phase-4-persistence-resume-discussion.md:65-66`) locks:

> "a LOGOS-native journal becomes the canonical durable store for replayable state"
> "the journal must be sufficient to recover... stateless replay turns without relying on UI memory or raw provider transcripts"

Journal record families (`2026-04-11-phase-4-journal-record-taxonomy-discussion.md:43-61`):

- `MessageAppended` — canonical message history
- `ContinuationUpdated` — canonical continuation handle
- `SessionOpened` — provider/model/mode baseline
- `TurnStarted`, `TurnFinished` — turn boundaries
- `ApprovalStateChanged`, `ToolCallTerminalRecorded` — control flow
- `CompactionApplied` — compaction boundaries
- `ProviderSnapshotStored` (optional, non-canonical)

The journal is append-oriented with a derived latest-session view. Journal is canonical. SQLite store is non-canonical.

### Overlap Between OR Snapshot And Journal

The current `PersistedOpenRouterThread` stores data that the journal will also store canonically:

| OR Snapshot Field                          | Journal Equivalent                          |
| ------------------------------------------ | ------------------------------------------- |
| `history: Vec<InternalMessage>`            | `MessageAppended` records                   |
| `continuation: Option<ContinuationHandle>` | `ContinuationUpdated` records               |
| `config_snapshot` (model, provider, mode)  | `SessionOpened` record                      |
| `elicitation_count`                        | Derived from `ApprovalStateChanged` records |

This overlap is the fundamental reason Item 02 is ADAPT rather than ACCEPT: the new SQLite store cannot become the durability owner for data that journal already owns canonically.

### Where Persistence Is Called

`openrouter_beta_threads.rs` calls persistence at these points:

- `load_active_thread` / `load_archived_thread` — on thread construction (line 212, 219, 242, 420, 510, 528, 550)
- `save_active_thread` — after persisting snapshot (line 408, 569, 889, 999, 2241)
- `save_archived_thread` — on archive (line 602)
- `build_persisted_thread_snapshot` — constructs snapshot from live runtime state (line 2079-2104)

The snapshot is built from runtime state (`history()`, `continuation()`, state fields). After a turn completes (lines 880, 990), the snapshot is rebuilt and saved.

### Item 01 Dependency

Item 01 draft introduces `thread_records` with `session_state_ref` and `config_snapshot_ref` fields that are intentionally deferred to Item 02. Item 02 must define what these references resolve to.

## Draft Target Model

### 1. Two-Layer Persistence Architecture

```
┌─────────────────────────────────────────────┐
│  SQLite Metadata Store (non-canonical)       │
│  - thread metadata, archive state, listings  │
│  - config snapshot references                │
│  - provider/model refs, timestamps           │
│  - query surfaces for list/filter/search     │
├─────────────────────────────────────────────┤
│  Phase-4 Journal (canonical runtime truth)   │
│  - MessageAppended, ContinuationUpdated      │
│  - SessionOpened, TurnStarted, TurnFinished  │
│  - ApprovalStateChanged, ToolCallTerminal    │
│  - CompactionApplied                         │
└─────────────────────────────────────────────┘
```

SQLite is the metadata/index layer. Journal is the canonical durability layer.

For resume/replay, the journal is the source of truth. SQLite metadata accelerates listing and administrative operations but must not be required for correctness.

### 2. SQLite Metadata Schema (Draft)

The metadata store holds non-canonical records that accelerate operations and provide structured query surfaces.

`thread_metadata`:

- `thread_id TEXT PRIMARY KEY`
- `provider_lane TEXT NOT NULL`
- `provider_id TEXT NOT NULL`
- `name TEXT`
- `archived_at INTEGER` — NULL = active, non-NULL = archived (soft-archive)
- `status TEXT NOT NULL` — coarse lifecycle status
- `config_snapshot_ref TEXT` — reference to stored config snapshot
- `session_state_ref TEXT` — reference to journal session or legacy history payload
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`
- `git_branch TEXT`
- `git_commit TEXT`
- `preview TEXT` — last message preview for listing

`config_snapshots` (optional separate table or file-backed):

- stores the full `ThreadConfigSnapshot` JSON for each thread
- referenced by `config_snapshot_ref`

This schema extends the Item 01 `thread_records` draft by concretizing the `session_state_ref` and `config_snapshot_ref` fields.

### 3. History/Blob Separation Strategy

Current state: `history: Vec<InternalMessage>` lives inside the monolithic JSON snapshot.

Target state depends on journal implementation timeline:

- **If journal is implemented first:** history lives in journal `MessageAppended` records. SQLite does not store message content. Resume reconstructs from journal.
- **If Item 02 ships before journal:** transitional JSONL append-only file per thread for history (like Codex rollout format). SQLite stores only metadata. When journal lands, JSONL files become the import source.

The draft recommends the second path as the safer sequencing: JSONL for history payloads now, journal as canonical owner when phase-4 T2 ships. This avoids blocking Item 02 on journal implementation.

### 4. Legacy Import Path

Goose's `import_legacy()` pattern maps directly:

1. On first startup with new SQLite store, scan `logos-openrouter-beta/active-threads/` and `archived-threads/`.
2. For each JSON snapshot:
   - Extract metadata into `thread_metadata` table.
   - Extract config into `config_snapshots`.
   - Extract history into JSONL file (transitional) or leave as-is if journal will import later.
3. Mark imported snapshots to avoid re-import on next startup.
4. Original JSON files remain untouched until a cleanup pass confirms successful import.

### 5. Migration Framework

Following Goose's version-tracked migration pattern:

1. `user_version` PRAGMA for schema version tracking.
2. Each migration is a numbered step in a match arm.
3. Migrations run inside `BEGIN IMMEDIATE` transactions.
4. Forward-only: no downgrade path.
5. Migration 0 = empty database.
6. Migration 1 = initial schema (`thread_metadata`, `config_snapshots`).
7. Future migrations add indexes, new columns, new tables.

### 6. Write/Read Ownership

- **SQLite writes:** thread metadata changes (create, rename, archive, unarchive, status update, preview update).
- **SQLite reads:** list, filter, search, administrative queries.
- **Journal writes:** canonical runtime truth (messages, continuation, approvals, tool terminals, compaction). Journal ownership is phase-4's responsibility.
- **JSONL writes (transitional):** history append after turn completion, only until journal takes over.
- **Config snapshot writes:** on thread creation and config change. Stored as JSON in `config_snapshots` table or file-backed store.

The key invariant: **SQLite must never be the only source of truth for data that journal owns canonically.** If SQLite and journal diverge, journal wins.

## Practical Draft Shape

### Stage A: Metadata Store Foundation

1. Add `rusqlite` (or equivalent) dependency to `logos-session-state` crate.
2. Implement SQLite metadata store with WAL mode, busy timeout, foreign keys.
3. Implement migration framework (version 0 → version 1).
4. Implement CRUD for `thread_metadata`.
5. Implement `list_threads` / `get_thread` / `filter_threads` query surfaces.
6. Implement legacy import from current JSON snapshots.

At this point, `openrouter_beta_persistence.rs` metadata functions (`list_active_threads`, `list_archived_threads`, thread metadata load) can begin delegating to SQLite instead of scanning JSON files.

### Stage B: History/Blob Separation

1. Extract `Vec<InternalMessage>` from the monolithic JSON snapshot into separate JSONL append-only files per thread.
2. SQLite `session_state_ref` points to the JSONL file path.
3. `build_persisted_thread_snapshot` splits into metadata persist (SQLite) + history append (JSONL).
4. Config snapshot moves to `config_snapshots` store.

At this point, the monolithic JSON snapshot is no longer the primary format. History is append-only, metadata is queryable.

### Stage C: Journal Integration

1. When phase-4 journal ships, `session_state_ref` transitions from JSONL file path to journal session identifier.
2. JSONL files become import source for journal on first journal-backed startup.
3. SQLite metadata store remains unchanged — only the reference target changes.
4. `ContinuationHandle` resume path reads from journal instead of JSONL.

This stage is gated on phase-4 journal implementation and must not block Stages A and B.

## Minimal Draft Data Model

### SQLite Schema (Migration 1)

```sql
CREATE TABLE thread_metadata (
    thread_id        TEXT PRIMARY KEY NOT NULL,
    provider_lane    TEXT NOT NULL,
    provider_id      TEXT NOT NULL,
    name             TEXT,
    archived_at      INTEGER,
    status           TEXT NOT NULL DEFAULT 'active',
    config_snapshot_ref TEXT,
    session_state_ref   TEXT,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    git_branch       TEXT,
    git_commit       TEXT,
    preview          TEXT
);

CREATE INDEX idx_thread_metadata_archived ON thread_metadata(archived_at);
CREATE INDEX idx_thread_metadata_provider ON thread_metadata(provider_lane, provider_id);
CREATE INDEX idx_thread_metadata_updated ON thread_metadata(updated_at);
```

### JSONL History File Format (Transitional)

Each line is a JSON object:

```json
{"type":"message","seq":1,"data":{...InternalMessage...}}
{"type":"continuation","seq":2,"data":{...ContinuationHandle...}}
{"type":"elicitation","seq":3,"data":{"count":5}}
```

Append-only. No rewrite. Corruption loses at most the last line.

### Config Snapshot Storage

Two options:

1. **Inline JSON blob** in SQLite `config_snapshots` table (simpler, acceptable for small configs).
2. **File-backed** JSON in a dedicated directory (keeps DB leaner, easier to inspect).

Draft recommends option 1 for MVP. Config snapshots are small (~1KB). SQLite handles this comfortably.

## Why This Still Supports The ADAPT Verdict

1. SQLite metadata, WAL, migrations, partial updates, legacy import, and soft-archive are all ACCEPT-level imports taken directly from competitors.
2. The hybrid model (SQLite metadata + journal canonical + separate history) is the ADAPT — it reinterprets the competitor "everything in SQLite" pattern to fit the locked phase-4 journal contract.
3. The draft does not widen scope into compaction, fork/resume, turn lifecycle, or shell execution.
4. The draft explicitly preserves journal authority: "if SQLite and journal diverge, journal wins."
5. History/Blob separation is additive and does not conflict with either competitor patterns or journal contract.

## Revisit Triggers

The current ADAPT verdict should be re-opened only if:

1. Journal implementation proves that no non-canonical metadata store is needed (all metadata derivable from journal alone) — would simplify toward journal-only.
2. Journal implementation timeline slips far enough that SQLite must temporarily serve as canonical history store — would temporarily widen Item 02 scope.
3. Performance testing shows SQLite metadata queries on thread listing are not materially faster than JSON file scanning for the expected dataset size — would question the migration value.
4. rusqlite dependency introduces unacceptable build complexity or platform compatibility issues.

## Closeout Expectation

The later strict spec derived from this draft should be considered successful only if:

1. Monolithic JSON snapshots are no longer the primary non-canonical persistence format.
2. Thread metadata (list, filter, archive state) is queryable through SQLite without loading full snapshots.
3. History payloads are stored separately from metadata (JSONL transitional or journal).
4. Config snapshots are stored separately from history.
5. Legacy JSON snapshots are importable into the new store.
6. Journal canonical authority is preserved: resume/replay reads from journal (or transitional JSONL), not from SQLite.
7. Item 01 `session_state_ref` and `config_snapshot_ref` fields are concretized.
8. OpenAI canonical path persistence is not affected.

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 02`
  lines: `117-151`
- path: `../research/session layer/RA-1.2-session-persistence.md`
  symbol: `LOGOS Current Implementation`
  lines: `1-27`
- path: `../research/session layer/RA-1.2-session-persistence.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `202-216`
- path: `../research/session layer/RA-1.2-session-persistence.md`
  symbol: `Recommendations for LOGOS`
  lines: `218-250`
- path: `2026-04-11-phase-4-persistence-resume-discussion.md`
  symbol: `What Phase 4 Now Means`
  lines: `60-75`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Journal Record Families`
  lines: `41-63`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Write Ordering Rules`
  lines: `120-128`
- path: `2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Minimal Draft Data Model`
  lines: `212-229`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `85-94`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: `24-47`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `write_snapshot`
  lines: `210-234`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `list_snapshots`
  lines: `185-208`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `build_persisted_thread_snapshot`
  lines: `2079-2104`
- path: `../../../codex-rs/logos-runtime-types/src/continuation.rs`
  symbol: `ContinuationHandle`
  lines: `4-10`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-11-phase-4-persistence-resume-discussion.md]]
- [[2026-04-11-phase-4-journal-record-taxonomy-discussion.md]]
- [[2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../research/session layer/RA-1.2-session-persistence.md]]
