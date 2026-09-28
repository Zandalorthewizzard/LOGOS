---
id: logos-session-state-foundation-task-01-thread-manager-strict-spec
title: Session-State Foundation Task 01 Thread Manager Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, sprint, session-state, thread-manager]
related:
  [
    [README.md],
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-16-item-01-implementation-reference.md],
    [2026-04-17-session-state-foundation-slice-implementation-report-template.md],
  ]
depends_on:
  [
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-16-item-01-implementation-reference.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Task 01 Thread Manager Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
3. `docs-new/07-logos-phase/implementation/2026-04-16-item-01-implementation-reference.md`
4. the exact Goose and OpenCode snapshot files cited in that implementation reference evidence section
5. `codex-rs/app-server/src/openrouter_beta_threads.rs`
6. `codex-rs/app-server/src/openrouter_beta_persistence.rs`
7. `codex-rs/app-server/src/thread_state.rs`
8. `codex-rs/logos-session-engine/src/session.rs`
9. `codex-rs/core/src/thread_manager.rs`

## Intent

Create the shared non-canonical thread registry and loaded-versus-active ownership boundary so stateless-provider lanes stop using a permanent proving-lane in-memory map as the authoritative owner of thread identity.

Non-goals:

1. no full persistence schema work beyond what is needed to establish registry ownership
2. no lifecycle or notification rewrite
3. no fork or resume implementation in this task
4. no shell or background-terminal extraction in this task

## Locked Decisions

1. The accepted imported core is storage-backed registry, load on demand, and no permanent in-memory thread ownership between turns for stateless-provider lanes.
2. `ThreadStateManager` remains separate and must not be re-absorbed into the new registry owner.
3. Production naming must be LOGOS-native even when refactoring Goose-inspired shapes.
4. The implementation must use the Item 01 implementation reference and its cited local source snapshots, not chat memory.

## Required External References

1. Goose `session_manager.rs` patterns cited in the implementation reference
2. OpenCode `storage/db.ts` and `session/session.sql.ts` patterns cited in the implementation reference
3. any additional OpenCode files only if they are already cited in the implementation reference evidence section

## Current Invalid Or Stale Code

These current shapes are evidence, not final architecture:

1. `OpenRouterBetaThreadManager.threads` as the authoritative registry in `codex-rs/app-server/src/openrouter_beta_threads.rs`
2. `openrouter_beta_persistence` active and archived JSON directories as the durable identity source in `codex-rs/app-server/src/openrouter_beta_persistence.rs`
3. app-server dispatch branching that asks whether the OpenRouter lane currently contains a thread

## Transitional Code Allowed During This Task

1. JSON-backed session-state payloads may remain temporarily while registry ownership moves first
2. OpenRouter runtime bootstrap may remain provider-lane-specific
3. shell/background process ownership may remain outside this task

## Code That Must Be Removed Or Downgraded Before Task Close

1. the permanent proving-lane `threads` map must stop being the authoritative durable registry
2. read/list/archive-unarchive identity resolution must stop relying on in-memory presence as source of truth
3. any temporary compatibility path must be explicitly marked transitional in the slice report

## Implementation Tasks

### T1. Registry Substrate

1. introduce shared non-canonical thread registry ownership
2. define durable thread record shape for non-canonical lanes
3. keep the registry narrow and metadata-focused

### T2. Load-On-Demand Boundary

1. separate durable thread identity from active runtime residency
2. make runtime construction re-creatable from durable state references
3. keep `ThreadStateManager` separate

### T3. Dispatch Simplification

1. move app-server ownership tests away from `OpenRouterBetaThreadManager.threads.contains(...)`
2. resolve non-canonical thread identity through durable registry metadata

### T4. Reporting

1. produce a slice report with exact borrowed code paths and line ranges
2. explicitly list stale proving-lane code that was removed
3. explicitly list stale proving-lane code that remains and why

## Acceptance And Tests

1. targeted tests for the new registry crate or module
2. focused `cargo test -p codex-app-server` coverage for read/list behavior touched by this task
3. no claim of closure if the durable registry still depends on the OpenRouter in-memory map as final owner

## Evidence

- path: `../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Current Code Diagnosis`
  lines: `49-100`
- path: `../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Stage A: Registry First`
  lines: `158-173`
- path: `2026-04-16-item-01-implementation-reference.md`
  symbol: `Stage A: Durable Non-canonical Thread Registry`
  lines: `47-156`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `73-103`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: `25-33`
- path: `../../../codex-rs/app-server/src/thread_state.rs`
  symbol: `ThreadStateManager`
  lines: `141-145`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `seed_history`
  lines: `135-156`
- path: `../../../codex-rs/core/src/thread_manager.rs`
  symbol: `ThreadManager`
  lines: `1-80`

## Links

- [[README.md]]
- [[2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[2026-04-16-item-01-implementation-reference.md]]
- [[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
