---
id: logos-session-state-foundation-task-02-session-persistence-strict-spec
title: Session-State Foundation Task 02 Session Persistence Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, sprint, session-state, persistence]
related:
  [
    [README.md],
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [2026-04-17-item-02-implementation-reference.md],
    [2026-04-17-session-state-foundation-slice-implementation-report-template.md],
  ]
depends_on:
  [
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md],
    [../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [2026-04-17-item-02-implementation-reference.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Task 02 Session Persistence Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
3. `docs-new/07-logos-phase/implementation/2026-04-17-item-02-implementation-reference.md`
4. the exact Goose and OpenCode snapshot files cited in that implementation reference evidence section
5. `docs-new/07-logos-phase/architecture/2026-04-11-phase-4-persistence-resume-discussion.md`
6. `docs-new/07-logos-phase/architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
7. `codex-rs/app-server/src/openrouter_beta_persistence.rs`
8. `codex-rs/app-server/src/openrouter_beta_threads.rs`

## Intent

Replace the monolithic non-canonical JSON snapshot model with a stronger metadata and state-reference persistence layer that wraps around the already-locked phase-4 journal truth instead of replacing it.

Non-goals:

1. no replacement of journal as canonical runtime truth
2. no archive migration sprint beyond what this task requires for the new metadata store
3. no fork/resume implementation yet
4. no compaction engine implementation

## Locked Decisions

1. Journal remains canonical runtime truth.
2. SQLite-backed metadata, migrations, partial updates, and legacy import are in scope.
3. Per-delta event sourcing and pure-SQL canonical history are rejected.
4. Production code naming must be LOGOS-native.

## Required External References

1. Goose session persistence and migration patterns cited in the Item 02 implementation reference
2. OpenCode metadata-store, WAL, and soft-archive patterns cited in the Item 02 implementation reference

## Current Invalid Or Stale Code

1. `PersistedOpenRouterThread` as one monolithic JSON owner for metadata, config, history, continuation, and elicitation state
2. full-file rewrite persistence in `write_snapshot`
3. file-by-file listing through `list_snapshots`
4. active/archived directory movement as the effective primary archive model for the non-canonical lane

## Transitional Code Allowed During This Task

1. temporary history payload files may remain outside SQLite while the journal boundary is preserved
2. temporary compatibility import from current JSON snapshots is allowed

## Code That Must Be Removed Or Downgraded Before Task Close

1. monolithic JSON snapshots must stop being the final owner of non-canonical session persistence
2. metadata listing and lookup must stop depending on loading every JSON snapshot file
3. any remaining JSON snapshot logic must be explicitly transitional and documented in the slice report

## Implementation Tasks

### T1. Metadata Store

1. create the metadata store and schema
2. add migration framework
3. define `session_state_ref` and `config_snapshot_ref` ownership concretely

### T2. Hybrid Boundary

1. keep journal as canonical history and continuation truth
2. move metadata, indexing, listing, and config snapshot ownership into the new persistence layer
3. define the temporary history/blob strategy honestly if journal-backed read model is not yet complete

### T3. Legacy Import

1. import current OpenRouter JSON snapshots into the new metadata layer
2. preserve recoverability during migration

### T4. Reporting

1. report exact Goose and OpenCode line ranges reused
2. report all adaptations made because of journal truth and LOGOS boundaries
3. report monolithic JSON code removed or left transitional

## Acceptance And Tests

1. targeted tests for metadata store and migrations
2. focused `cargo test -p codex-app-server` coverage for persistence-backed read/list behavior touched by this task
3. no claim of closure if monolithic JSON snapshots still remain the final owner of metadata and listing semantics

## Evidence

- path: `../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Current Code Diagnosis`
  lines: `60-117`
- path: `../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Two-Layer Persistence Architecture`
  lines: `131-144`
- path: `2026-04-17-item-02-implementation-reference.md`
  symbol: `Stage A: Metadata Store Foundation`
  lines: `52-148`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: `25-33`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `list_snapshots`
  lines: `185-210`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `build_persisted_thread_snapshot`
  lines: `2081-2104`
- path: `../architecture/2026-04-11-phase-4-persistence-resume-discussion.md`
  symbol: `Current Consensus`
  lines: `65-66`
- path: `../architecture/2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Core Record Families`
  lines: `43-61`

## Links

- [[README.md]]
- [[2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md]]
- [[2026-04-17-item-02-implementation-reference.md]]
- [[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
