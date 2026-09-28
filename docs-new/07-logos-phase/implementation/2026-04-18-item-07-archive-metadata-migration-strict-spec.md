---
id: logos-item-07-archive-metadata-migration-strict-spec
title: Item 07 Archive Metadata Migration Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, session-state, archive, metadata, migration]
related:
  [
    [README.md],
    [../architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md],
    [../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md],
    [../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md],
  ]
depends_on:
  [
    [../architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-18-session-state-foundation-task-01-implementation-report.md],
    [2026-04-18-session-state-foundation-task-02-implementation-report.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 07 Archive Metadata Migration Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md`
3. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
4. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
5. `docs-new/07-logos-phase/research/2026-04-16-session-layer-research-transfer-ledger.md`
6. `docs-new/07-logos-phase/implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md`
7. `docs-new/07-logos-phase/implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md`
8. `docs-new/07-logos-phase/implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md`
9. `docs-new/07-logos-phase/implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md`
10. `codex-rs/logos-session-state/src/registry.rs`
11. `codex-rs/app-server/src/openrouter_beta_threads.rs`
12. `codex-rs/app-server/src/openrouter_beta_persistence.rs`
13. `codex-rs/state/src/lib.rs`
14. `codex-rs/state/src/model/thread_metadata.rs`
15. `codex-rs/state/src/runtime/threads.rs`
16. `codex-rs/app-server/src/codex_message_processor.rs`

## Intent

Finish the Item 07 owner shift by making `logos-session-state` the explicit non-canonical archive and metadata owner for OpenRouter beta, while downgrading stale snapshot-archive residue and removing final provider-lane ownership of archive/name/git/preview semantics.

Non-goals:

1. no rollback work
2. no interrupt work
3. no shell/background-terminal ownership work
4. no journal implementation work
5. no review workflow work
6. no broad `codex-state` redesign to absorb non-rollout provider-lane threads
7. no OpenAI canonical-path degradation or symmetry rewrite

## Target Outcome

1. OpenRouter archive truth is `logos-session-state.thread_records.archived_at`, not snapshot directory location and not provider-lane final logic.
2. OpenRouter list/read metadata truth remains `logos-session-state` for the non-canonical thread envelope: `name`, `git_*`, `preview`, `updated_at`, `forked_from_id`, `session_state_ref/path`, and `config_snapshot_ref`.
3. `openrouter_beta_threads.rs` becomes a caller of shared non-canonical archive/metadata operations rather than the final owner of those semantics.
4. Snapshot active/archived directory helpers are downgraded to explicit compatibility residue only.
5. `codex-state` remains the preserved rollout-backed canonical owner and is not forced to model non-canonical OpenRouter thread state.

## Locked Decisions

1. `logos-session-state` is the final owner for non-canonical OpenRouter archive and metadata semantics.
2. `codex-state` remains the owner for rollout-backed canonical thread metadata only.
3. OpenRouter archive truth stays soft-archive metadata (`archived_at`), not file movement.
4. OpenRouter list/read metadata truth stays registry-backed, not snapshot-discovered.
5. Thread name ownership for canonical threads remains outside `codex-state`; Item 07 must not pretend otherwise.
6. Any remaining snapshot helpers after this item must be explicitly transitional, not silent final owners.
7. Do not widen this item into journal, rollback, interrupt, shell/background-terminal, compaction, or review work.

## Architecture Fit

1. LOGOS Layer 2 owns non-canonical archive and metadata semantics above provider wires.
2. `logos-session-state` is the concrete additive store for that non-canonical thread envelope.
3. `openrouter_beta_threads.rs` is an OpenRouter lane consumer of that store, not the final owner.
4. `codex-state` remains the rollout-metadata mirror for canonical Codex/OpenAI flows.
5. `codex_message_processor.rs` may continue to merge canonical and non-canonical thread results, but owner truth for each side must stay explicit.

## Current Invalid Or Stale Code

1. `OpenRouterBetaThreadManager::archive_thread`, `unarchive_thread`, and `archive_persisted` still encode final archive transition semantics inside the provider lane.
2. `OpenRouterBetaThreadManager::set_thread_name` and `update_git_info` still encode final metadata mutation semantics inside the provider lane.
3. `openrouter_beta_persistence.rs` still exposes active/archived snapshot directory helpers that can be mistaken for archive truth.
4. `load_persisted_thread_from_record()` still consults legacy active/archived snapshot locations for fallback payload recovery.
5. Any implementation that would move OpenRouter archive truth into `codex-state` by inventing fake rollout ownership is invalid for this item.

## Transitional Code Allowed During This Task

1. JSONL `HistoryFile` may remain the transitional runtime/history payload store.
2. `config_snapshot_json` may remain as compatibility fallback while `config_snapshot_ref` stays authoritative.
3. legacy snapshot import and carefully bounded snapshot fallback may remain if tests still require compatibility.

## Code That Must Be Removed Or Downgraded Before Item 07 Close

1. provider-lane final ownership of archive and unarchive semantics
2. provider-lane final ownership of non-canonical name and git metadata mutation semantics used by list/read
3. any implication that active/archived snapshot directories are still the owner of OpenRouter archive state
4. any claim that `codex-state` is the OpenRouter non-canonical metadata owner without adding the missing non-rollout fields and references

## Implementation Tasks

### T1. Shared Non-Canonical Archive And Metadata Surface

1. add the narrow `logos-session-state` operations needed for OpenRouter archive/unarchive and metadata mutation ownership
2. keep the surface focused on non-canonical thread-record semantics already present in `ThreadRecord` and `ThreadUpdateBuilder`
3. cover at minimum:
   - archive
   - unarchive
   - name update
   - git update
   - preview and timestamp freshness where list/read depend on them

### T2. OpenRouter Lane Rewire

1. rewire `openrouter_beta_threads.rs` to call the shared `logos-session-state` archive/metadata surface
2. keep provider-lane logic limited to OpenRouter-specific validation and runtime state checks
3. remove lane-local final-owner methods or reduce them to thin delegating glue

### T3. Snapshot Residue Downgrade

1. downgrade `openrouter_beta_persistence.rs` archived snapshot helpers to explicit compatibility residue
2. ensure steady-state archive/unarchive no longer depends on active/archived snapshot location
3. keep only the minimal legacy import or fallback surface still required by compatibility tests

### T4. App-Server Read/List Coherence Proof

1. prove `thread/list` active and archived filtering still reflects `archived_at`
2. prove `thread/read(include_turns=false)` still reflects registry-backed `name`, `git_info`, and `preview`
3. prove restart-visible metadata remains coherent after archive/unarchive and after name/git mutation

### T5. Documentation And Closure Report

1. update the Item 07 discussion/spec if implementation reveals a real contradiction
2. produce an implementation report that explicitly lists stale owner logic removed
3. state what snapshot residue still remains transitional after closure

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| `logos-session-state` becomes the explicit non-canonical archive/metadata owner | `T1`, `T2` | focused `logos-session-state` and `codex-app-server` tests | `A1` | no final provider-lane owner remains |
| OpenRouter archive truth is metadata soft-archive, not snapshot location | `T2`, `T3`, `T4` | focused archive/unarchive tests plus restart proofs | `A2` | no active/archived snapshot location dependency remains in steady-state logic |
| OpenRouter list/read metadata stays coherent through the shared owner | `T1`, `T2`, `T4` | focused read/list/name/git tests and restart proofs | `A3` | list/read use shared metadata truth, not lane-local ad hoc state |
| Canonical `codex-state` path is preserved and not widened into non-rollout OpenRouter ownership | `T2`, `T5` | no-regression proof on touched canonical tests only if touched | `A4` | no fake rollout migration; no OpenAI degradation |

## Acceptance And Tests

### Acceptance Targets

#### A1. Owner Shift Proof

1. OpenRouter archive, unarchive, and relevant metadata mutations are expressed through `logos-session-state` operations.
2. `openrouter_beta_threads.rs` no longer contains the final archive/metadata transition semantics.

#### A2. Archive Truth Proof

1. Active versus archived OpenRouter thread visibility is driven by `archived_at`.
2. Snapshot directories are not required to determine current archive state.

#### A3. Metadata Coherence Proof

1. `thread/list` and `thread/read(include_turns=false)` return coherent `name`, `git_info`, `preview`, `updated_at`, and archive status after restart.
2. OpenRouter search/filter behavior remains coherent with the shared owner.

#### A4. Canonical Boundary Proof

1. `codex-state` remains the rollout-backed canonical owner only.
2. No new fake rollout or canonical-path degradation is introduced to make OpenRouter look symmetric.

### Expected Test Commands

1. `cargo test -p logos-session-state` if the crate changes
2. focused `cargo test -p codex-app-server` coverage for:
   - `thread_archive`
   - `thread_unarchive`
   - OpenRouter read/list/name/git restart proofs touched by the item
3. targeted archive/list/read tests only for any touched canonical `codex-state` or app-server path
4. `just fix -p <changed crate>` if the environment permits; otherwise run the narrow honest equivalent and report why
5. `just fmt`

Do not run a full workspace suite as a substitute for targeted proofs.

## Risks / Rollback

1. Risk: implementation may preserve provider-lane archive helpers as the real owner while only wrapping them in a new API.
2. Mitigation: closure requires explicit owner removal, not a cosmetic wrapper.
3. Risk: implementation may overreach and try to force OpenRouter into `codex-state` by fabricating rollout-backed semantics.
4. Mitigation: stop and return to discussion if a real `codex-state` redesign seems required.
5. Risk: snapshot fallback removal may break old-data compatibility unexpectedly.
6. Mitigation: keep legacy import and the minimum honest fallback surface until compatibility proofs say it can be retired.
7. Rollback: if the shared owner surface proves insufficient, stop with `DISCUSSION_ONLY_MORE_WORK_NEEDED` rather than landing a second final owner.

## Agent Guardrails

1. Do not move OpenRouter archive/metadata truth into `codex-state` without new code evidence that `codex-state` now models the non-canonical thread envelope honestly.
2. Do not keep provider-lane archive ownership and call it closure.
3. Do not widen into rollback, interrupt, shell/background terminals, journal, review, or compaction.
4. Do not degrade the preserved OpenAI canonical path.
5. Do not treat snapshot archive directories as the owner of current OpenRouter archive state.
6. Do not finalize the item without explicit restart-visible archive/list/read proofs.

## Evidence

- path: `../architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md`
  symbol: `Final Owner Decision`, `Current Stale Or Transitional Logic`, `Migration Shape`
  lines: `58-207`
- path: `../../../codex-rs/logos-session-state/src/registry.rs`
  symbol: `ThreadRecord`, `ThreadUpdateBuilder`
  lines: `50-75, 412-655`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `list_registry_threads`, `set_thread_name`, `update_git_info`, `archive_thread`, `unarchive_thread`, `archive_persisted`, `thread_from_record`
  lines: `458-502, 555-647, 980-1085, 2358-2438`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `list_archived_threads`, `save_archived_thread`
  lines: `133-173`
- path: `../../../codex-rs/state/src/lib.rs`
  symbol: `crate docs`
  lines: `1-5`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `update_thread_git_info`, `mark_archived`, `mark_unarchived`
  lines: `537-564, 763-807`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_archive`, `thread_metadata_update`, `thread_unarchive`, `thread_list`, `attach_thread_name`
  lines: `2526-2609, 2834-3015, 3150-3360, 3590-3701, 4669-4678`

## Links

- [[README.md]]
- [[../architecture/2026-04-18-item-07-archive-metadata-migration-discussion.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
- [[../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
