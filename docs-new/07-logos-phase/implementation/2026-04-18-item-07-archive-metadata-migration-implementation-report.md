---
id: logos-item-07-archive-metadata-migration-implementation-report
title: Item 07 Archive Metadata Migration Implementation Report
type: implementation-report
status: completed
tags:
  [
    logos,
    phase-6,
    implementation-report,
    archive,
    metadata,
    migration,
    openrouter,
    registry,
  ]
related:
  [
    [2026-04-18-item-07-archive-metadata-migration-discussion.md],
    [2026-04-18-item-07-archive-metadata-migration-strict-spec.md],
    [2026-04-18-session-state-foundation-task-02-implementation-report.md],
  ]
depends_on:
  [
    [2026-04-18-item-07-archive-metadata-migration-discussion.md],
    [2026-04-18-item-07-archive-metadata-migration-strict-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 07 Archive Metadata Migration Implementation Report

## 1. Closed

1. Added the shared non-canonical archive and metadata surface in `logos-session-state`, so OpenRouter archive/unarchive, name updates, git metadata updates, and updated-at touches no longer need provider-lane final-owner methods.
2. Rewired `openrouter_beta_threads.rs` to call the shared registry surface for archive/unarchive, name, git, and list/read coherence while keeping only provider-specific validation, residency checks, and compatibility fallback selection in the lane.
3. Downgraded snapshot/archive residue in `openrouter_beta_persistence.rs` to compatibility-only recovery and import helpers instead of current archive truth.
4. Added focused proofs that restart-visible list/read behavior is driven by registry truth, not by snapshot location.
5. Closed the Item 07 ownership shift honestly: steady-state OpenRouter archive truth now lives in `logos-session-state`, while snapshot/archive residue remains only as explicit fallback.

## 2. Borrowed / Reused

1. Reused the existing `ThreadUpdateBuilder` pattern from `logos-session-state` instead of inventing a second partial-update path for non-canonical thread metadata.
2. Reused the existing `ThreadFilter` and registry-backed list/read model already established by the session-state foundation work.
3. Reused the existing OpenRouter beta test harness and persistence layout so the slice could prove restart coherence without changing the broader beta scaffolding.
4. No external reference was needed for this slice. The implementation stayed inside local LOGOS/Codex source and the existing Item 07 spec package.

## 3. Adapted In LOGOS

1. `SqliteThreadRegistry` now owns the narrow non-canonical archive/metadata operations needed by OpenRouter beta, including archive, unarchive, name, git-info, and updated-at writes.
2. `openrouter_beta_threads.rs` now treats the registry as the source of truth for restart-visible metadata coherence and uses provider-lane code only for lane-specific validation and runtime checks.
3. `openrouter_beta_persistence.rs` now reads as legacy residue: it can still recover payloads and legacy snapshot data, but it no longer pretends to define current archive state.
4. The tests now prove the ownership model directly: archive location can be moved around as residue without changing the archive truth seen after restart.

## 4. Stale Code Removed

1. Removed provider-lane final ownership of archive transitions from `OpenRouterBetaThreadManager::archive_thread`.
2. Removed provider-lane final ownership of unarchive transitions from `OpenRouterBetaThreadManager::unarchive_thread`.
3. Removed lane-local name, git, and updated-at registry helper duplication from `openrouter_beta_threads.rs`.
4. Removed current-archive inference from snapshot location as an accepted runtime truth path.
5. Removed the idea that `openrouter_beta_persistence.rs` archive helpers can be treated as steady-state owner logic.

## 5. Stale Code Still Remaining

1. `openrouter_beta_persistence.rs` still keeps legacy snapshot and directory helpers, but only as compatibility fallback and import residue.
2. Snapshot files can still be recovered or copied for transitional payload continuity, but they no longer decide current archive state.
3. `codex-state` remains the canonical metadata path for rollout-backed canonical ownership, but it is not required for the touched non-canonical OpenRouter ownership paths in this slice.
4. The OpenRouter lane still owns provider-specific validation and runtime residency checks, which is correct and intentionally remains outside the shared registry surface.

## 6. Tests / Checks Run

1. `cargo test -p logos-session-state`
   Result: passed.
2. `cargo test -p codex-app-server openrouter_beta::`
   Result: passed.
3. `cargo fmt`
   Result: passed after `just fmt` could not run because the repo shell wrapper is unavailable on this Windows environment.
4. `cargo clippy -p logos-session-state -p codex-app-server --tests --all-targets --no-deps -- -D warnings`
   Result: failed on an inherited `expect_used` in `logos-session-state/src/registry.rs:746`, which is outside this slice.

## 7. Residual Risks

1. Compatibility snapshot helpers still exist, so there is still some fallback surface to retire later.
2. The registry-backed ownership shift is now honest, but the broader OpenRouter beta lane still has provider-specific edges that should be watched as later cleanup work lands.
3. The clippy clean pass is not fully green because of pre-existing repository code outside this slice.

## 8. Explicit Answers

1. What is now the final OpenRouter archive truth?
`logos-session-state` `thread_records.archived_at`.
2. What is now the final OpenRouter metadata owner for list/read coherence?
`logos-session-state`, via the shared registry surface and the existing registry-backed list/read flow.
3. What snapshot/archive residue still remains?
Compatibility-only snapshot recovery and import helpers in `openrouter_beta_persistence.rs`, plus legacy snapshot files that can still be moved or copied for fallback.
4. Did any touched path still require `codex-state` for non-canonical ownership?
No.
5. Is Item 07 honestly closed after this slice, or still not?
Yes. It is closed, with the explicit caveat that compatibility residue still exists and is now bounded instead of being the owner.

## Evidence

- path: `codex-rs/logos-session-state/src/registry.rs`
  symbol: `ThreadFilter`, `ThreadGitInfoPatch`, `SqliteThreadRegistry` archive and metadata helpers
  lines: `105-118, 128-150, 344-401, 467-658, 806-917`
- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager` shared-owner rewiring
  lines: `491-1029`
- path: `codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: legacy snapshot and archive compatibility helpers
  lines: `19-23, 128-174`
- path: `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs`
  symbol: `app_server_openrouter_archive_unarchive_restart_uses_registry_state_not_snapshot_location`
  lines: `1249-1351, 1374-1606`

