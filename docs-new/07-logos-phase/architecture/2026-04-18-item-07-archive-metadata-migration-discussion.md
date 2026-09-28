---
id: logos-item-07-archive-metadata-migration-discussion
title: Item 07 Archive Metadata Migration Discussion
type: discussion
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    archive,
    metadata,
    migration,
    openrouter,
  ]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-14-openrouter-thread-archive-support-discussion.md],
    [2026-04-14-openrouter-thread-unarchive-support-discussion.md],
    [2026-04-17-session-state-foundation-sprint-discussion.md],
    [../implementation/2026-04-18-item-07-archive-metadata-migration-strict-spec.md],
    [../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md],
    [../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md],
    [../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 07 Archive Metadata Migration Discussion

## Purpose

This document replaces the earlier broad defer posture for `Item 07` with a code-backed migration decision after the real `Task 02` persistence closeout.

The question is no longer whether LOGOS needs archive and metadata support. The question is which owner is now honest for non-canonical OpenRouter archive, unarchive, and list/read metadata semantics.

## Current Transfer Status

1. The research ledger still records `Item 07` as `DEFERRED` because the earlier honest posture was: wait for `Item 01` and `Item 02` to settle the real registry and persistence boundary.
2. That dependency is now satisfied for the targeted non-canonical lane:
   - `Task 01` made `logos-session-state` the durable non-canonical registry owner.
   - `Task 02` made registry metadata and registry-backed config truth authoritative for restart-visible OpenRouter list/read state.
3. The research ledger is now promoted into dual-consensus alignment with this migration package, so `Item 07` is no longer in a defer-only state.
4. `Checkpoint A` and the post-`Task 04` reassessment both say `Item 07` is now mainly migration and owner cleanup, not missing-substrate design.

## Final Owner Decision

### Chosen Model

`logos-session-state` remains the honest owner for non-canonical archive and metadata semantics.

This is not a hand-wave about what crate already exists. It follows directly from the shapes that now exist in code.

### Why `logos-session-state` Wins

`logos-session-state` already owns the exact non-canonical fields and references that OpenRouter list/read/archive flows consume today:

1. `thread_records` already stores `name`, `archived_at`, `forked_from_id`, `config_snapshot_ref`, `session_state_ref`, `session_state_path`, `git_*`, `preview`, and `updated_at` in one registry row shape.
2. `config_snapshots` already gives that registry a separate config-truth surface.
3. `ThreadUpdateBuilder` already supports partial mutation of the exact fields used by archive/name/git/preview update paths.
4. `openrouter_beta_threads.rs` already reconstructs not-loaded threads directly from `ThreadRecord` via `thread_from_record()` and serves list/read from registry-backed records through `list_registry_threads()` and `read_persisted_thread()`.

After `Task 02`, that is not provisional anymore. Restart-visible OpenRouter metadata truth already lives there.

### Why `codex-state` Loses

`codex-state` is real and mature, but it is the owner of rollout-backed canonical metadata, not the owner of the non-canonical thread envelope now used by OpenRouter beta.

Code evidence shows four concrete mismatches:

1. `codex-state` declares itself as "SQLite-backed state for rollout metadata" and mirrors JSONL rollout metadata into SQLite. OpenRouter beta now persists registry rows plus history/config references, not rollout-backed thread state.
2. `ThreadMetadata` requires a canonical `rollout_path` and exposes canonical rollout-derived fields such as `title`, `first_user_message`, and `tokens_used`; it does not own `name`, `forked_from_id`, `session_state_ref`, `session_state_path`, or provider-lane config references.
3. Canonical app-server read/list still attaches names separately with `find_thread_name_by_id()` and `find_thread_names_by_ids()`, which proves `codex-state` is not the name owner even on the preserved path.
4. `mark_archived()` and `mark_unarchived()` in `codex-state` both require a rollout path and update canonical rollout metadata rows. Moving OpenRouter into that model would either require fake rollout files or a broader `codex-state` redesign that this item does not justify.

So `codex-state` is the wrong target not because it is weak, but because it would create false discovery: a rollout-backed canonical store would be asked to own a non-rollout provider-lane thread envelope that it does not model today.

### Why A Split Owner Model Loses

A bounded split is already present at the product level: canonical rollout-backed threads use `codex-state`, while non-canonical provider-lane threads use `logos-session-state`.

But inside `Item 07` itself, a new split owner for OpenRouter archive/metadata would be worse than choosing one owner:

1. archive truth would drift if `archived_at` lived in one store while name/git/preview truth still lived in another
2. list/read filtering would keep stitching together two non-canonical metadata owners
3. OpenRouter lane code would continue deciding which store is "really" authoritative for each field

That is exactly the ambiguity this item is supposed to remove.

## Comparison Against Current App-Server Needs

### What OpenRouter List/Read/Archive Needs Today

For non-canonical OpenRouter threads, the app-server needs:

1. active versus archived filtering
2. restart-visible `updated_at` ordering
3. `preview` for search and list rendering
4. `name` for list/read coherence
5. `git_info` mutation and reload
6. `forked_from_id`
7. `session_state_path` / `session_state_ref` to reopen non-loaded history payloads
8. `config_snapshot_ref` for config truth during resume/read

That exact shape already exists in `logos-session-state::ThreadRecord` plus `config_snapshots` and is consumed directly by `openrouter_beta_threads.rs`.

### What `codex-state` Provides Today

`codex-state` already provides:

1. canonical rollout metadata rows
2. rollout-backed `list_threads()` and `get_thread()`
3. canonical archive/unarchive row updates through `mark_archived()` and `mark_unarchived()`
4. canonical git mutation through `update_thread_git_info()`

That is useful evidence, but it is not the same owner shape.

## Current Stale Or Transitional Logic

The remaining stale logic is not in the wire adapter. It is in the OpenRouter session/thread lane and in leftover snapshot helpers.

### Lane-Local Archive Ownership Still Present

`openrouter_beta_threads.rs` still owns archive state transitions through lane-local methods:

1. `archive_thread()` decides archive eligibility and then calls `archive_persisted()`.
2. `archive_persisted()` directly mutates `registry.update(thread_id).archived_at(Some(now))`.
3. `unarchive_thread()` directly mutates `registry.update(thread_id).archived_at(None)`.

That logic now targets the right storage owner, but the transition semantics still live in the provider lane. That is not the intended final architecture.

### Lane-Local Metadata Mutation Still Present

The OpenRouter lane still owns metadata mutations that list/read consume:

1. `set_thread_name()` updates the registry directly for not-loaded threads and persists loaded-thread mutations through the lane.
2. `update_git_info()` applies patch logic in the lane and writes registry git fields directly for not-loaded threads.
3. completion persistence still refreshes `preview` and `updated_at` through lane-owned `persist_thread()`.

These are valid working mechanics, but they still leave final metadata semantics in `openrouter_beta_threads.rs` rather than in a LOGOS-owned metadata surface.

### Snapshot Archive Residue Still Present

`openrouter_beta_persistence.rs` still carries active/archived snapshot directory helpers:

1. `ARCHIVED_THREADS_DIR`
2. `archived_thread_path()`
3. `list_archived_threads()`
4. `save_archived_thread()` and `remove_archived_thread()`

After `Task 02`, those are no longer the owner of archive truth. They are migration residue and legacy compatibility helpers only.

### Canonical Name Handling Proves `codex-state` Is Not The Name Owner

Canonical list/read paths still call `attach_thread_name()` which reads names through `find_thread_name_by_id()` or `find_thread_names_by_ids()`. That means even preserved Codex does not ask `codex-state` to own thread names.

This matters because a forced OpenRouter migration into `codex-state` would still leave name ownership elsewhere, which would preserve duplicate owner logic instead of removing it.

## Migration Shape

### 1. Existing OpenRouter Records

No broad data migration into `codex-state` is required.

The migration target is the already-delivered `logos-session-state` registry:

1. existing imported registry rows remain the durable non-canonical metadata owner
2. existing `config_snapshot_ref` and `session_state_ref` remain the durable non-canonical references
3. existing `archived_at` stays the archive truth

So the main migration work is owner cleanup in code, not another persistence-store migration.

### 2. Snapshot-Based Archive Residue

Snapshot-based archive residue should be downgraded further:

1. archived snapshot directories remain legacy import and compatibility residue only
2. steady-state OpenRouter archive and unarchive must not depend on file movement between active/archived snapshot directories
3. steady-state OpenRouter list/read must not rediscover archive truth from snapshot location

### 3. Metadata Truth For List/Read

For OpenRouter beta, list/read metadata truth remains in `logos-session-state`:

1. `archived_at`
2. `name`
3. `git_*`
4. `preview`
5. `updated_at`
6. `forked_from_id`
7. `session_state_path` / `session_state_ref`
8. `config_snapshot_ref`

`codex_message_processor` should continue to merge canonical `codex-state` summaries with non-canonical registry-backed threads, but it should not continue to treat provider-lane methods as the final owner of non-canonical metadata semantics.

### 4. Archive Truth

Archive truth for OpenRouter stays soft-archive metadata, not file location:

1. archive = set `archived_at`
2. unarchive = clear `archived_at`
3. list/read filters follow `archived_at`

This is already the data model in the registry. `Item 07` closes the remaining owner cleanup so the provider lane stops being the long-term owner of that transition.

### 5. What Remains Transitional After Item 07

Even after Item 07, these pieces may still remain transitional:

1. JSONL `HistoryFile` as the pre-journal runtime/history payload store
2. `config_snapshot_json` row duplication fallback in `thread_records`
3. legacy snapshot import from `openrouter_beta_persistence.rs`
4. transitional snapshot fallback for recovery/debug, if still required by compatibility coverage

Those are Task 02 follow-on residues. They do not require `Item 07` to keep provider-lane archive ownership.

## Item Boundary

### Included

1. archive owner cleanup for OpenRouter beta
2. unarchive owner cleanup for OpenRouter beta
3. metadata mutation ownership needed for list/read/archive coherence:
   - name
   - git info
   - preview and timestamp freshness where archive/list/read depend on it
4. migration of stale snapshot-archive logic into explicit compatibility-only residue

### Excluded

1. rollback
2. interrupt
3. shell/background process ownership
4. journal implementation
5. review workflow
6. broad `codex-state` redesign to accept non-rollout provider-lane threads

## Implementation Readiness

`IMPLEMENTATION_READY`

The final owner is no longer ambiguous:

1. `logos-session-state` already owns the non-canonical thread envelope actually consumed by OpenRouter list/read/archive paths
2. `codex-state` is clearly rollout-backed canonical metadata, not the non-canonical provider-lane metadata owner
3. the remaining work is a narrow migration and owner-cleanup slice, not unresolved architecture design

## Evidence

- path: `../../../codex-rs/logos-session-state/src/registry.rs`
  symbol: `THREAD_RECORD_COLUMNS`, `ThreadRecord`, `ThreadRegistry`, `ThreadUpdateBuilder`
  lines: `24-25, 50-75, 119-136, 412-655`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `list_registry_threads`, `read_persisted_thread`, `set_thread_name`, `update_git_info`, `archive_thread`, `unarchive_thread`, `archive_persisted`, `thread_from_record`
  lines: `458-502, 555-647, 748-785, 980-1085, 2358-2438`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `OPENROUTER_BETA_THREADS_DIR`, `list_archived_threads`, `save_active_thread`, `save_archived_thread`, `build_persisted_thread`
  lines: `19-22, 133-173, 278-293`
- path: `../../../codex-rs/state/src/lib.rs`
  symbol: `crate docs`
  lines: `1-5`
- path: `../../../codex-rs/state/src/model/thread_metadata.rs`
  symbol: `ThreadMetadata`, `ThreadMetadataBuilder`
  lines: `55-141, 143-212`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `get_thread`, `list_threads`, `update_thread_git_info`, `mark_archived`, `mark_unarchived`
  lines: `5-40, 329-400, 537-564, 763-807`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_archive`, `thread_set_name`, `thread_metadata_update`, `thread_unarchive`, `thread_list`, `attach_thread_name`, `summary_to_thread`
  lines: `2526-2609, 2732-3015, 3150-3360, 3590-3701, 4669-4678, 9419-9459`
- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 07`
  lines: `357-402`
- path: `../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md`
  symbol: `1. Closed`
  lines: `29-40`
- path: `../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md`
  symbol: `1. Closed`, `6. Stale Code Still Remaining`, `9. Explicit Answers`
  lines: `26-35, 117-132, 185-200`
- path: `../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md`
  symbol: `6. Item 07 Reassessment At Checkpoint A`
  lines: `75-87`
- path: `../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md`
  symbol: `4. Item 07: Archive / Unarchive And Metadata`, `7. Recommended Next Order`
  lines: `63-78, 111-116`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-14-openrouter-thread-archive-support-discussion.md]]
- [[2026-04-14-openrouter-thread-unarchive-support-discussion.md]]
- [[../implementation/2026-04-18-item-07-archive-metadata-migration-strict-spec.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
- [[../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
