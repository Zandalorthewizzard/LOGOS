---
id: logos-session-state-foundation-task-04-fork-resume-strict-spec
title: Session-State Foundation Task 04 Fork Resume Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, sprint, session-state, fork, resume]
related:
  [
    [README.md],
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
    [2026-04-17-item-03-implementation-reference.md],
    [2026-04-17-session-state-foundation-slice-implementation-report-template.md],
  ]
depends_on:
  [
    [2026-04-17-session-state-foundation-sprint-strict-spec.md],
    [2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md],
    [2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md],
    [2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md],
    [../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
    [2026-04-17-item-03-implementation-reference.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Task 04 Fork Resume Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
3. `docs-new/07-logos-phase/implementation/2026-04-17-item-03-implementation-reference.md`
4. the exact Goose, OpenCode, and Codex snapshot files cited in that implementation reference evidence section
5. `docs-new/07-logos-phase/architecture/2026-04-14-openrouter-thread-resume-support-discussion.md`
6. `codex-rs/app-server/src/openrouter_beta_threads.rs`
7. `codex-rs/core/src/thread_manager.rs`
8. `codex-rs/protocol/src/protocol.rs`
9. `codex-rs/state/src/runtime/threads.rs`

## Intent

Move non-canonical fork and resume onto the new shared substrate so they operate over durable state, explicit lineage, and provider-policy-aware continuation behavior instead of live-thread-only proving-lane ownership.

Non-goals:

1. no review orchestration implementation
2. no rollback implementation
3. no compaction implementation
4. no fake one-size-fits-all continuation model

## Locked Decisions

1. Fork and resume belong to the LOGOS session-state layer, not to provider-specific lanes.
2. Continuation semantics remain provider-policy-aware.
3. Fork must work over durable state, not only over currently loaded threads.
4. Production naming must be LOGOS-native.

## Required External References

1. Goose fork/copy-session patterns cited in the Item 03 implementation reference
2. OpenCode parent tracking and fork patterns cited in the Item 03 implementation reference
3. Codex `InitialHistory`, `thread_manager`, and `thread_spawn_edges` patterns cited in the Item 03 implementation reference

## Current Invalid Or Stale Code

1. `fork_thread` in the OpenRouter lane requires the source thread to be present in `self.threads`
2. `resume_persisted_thread` is tied to monolithic JSON persistence
3. lineage is only partially represented through `forked_from_id`
4. config and model validation are missing from current non-canonical fork and resume

## Transitional Code Allowed During This Task

1. provider-specific runtime bootstrap may remain in the provider lane
2. listener reattachment may reuse current `ThreadStateManager` integration

## Code That Must Be Removed Or Downgraded Before Task Close

1. live-thread-only fork ownership must be removed as the final behavior
2. monolithic JSON-specific resume ownership must be removed as the final behavior
3. any remaining provider-lane-only lineage handling must be explicitly transitional

## Implementation Tasks

### T1. Durable-State Fork

1. resolve source threads from the shared registry
2. fork over durable state references rather than only live runtime state
3. persist lineage explicitly

### T2. Provider-Policy-Aware Resume

1. implement resume over the new persistence boundary
2. select continuation backend honestly by provider mode
3. keep validation explicit for model, tools, and config compatibility

### T3. Lineage Surfaces

1. preserve `forked_from_id`
2. add edge-record ownership in the shared substrate where required

### T4. Reporting

1. trace every borrowed fork/resume pattern to exact source lines
2. list any provider-native continuation behavior preserved
3. list every stale proving-lane fork/resume path removed

## Acceptance And Tests

1. focused `cargo test -p codex-app-server` coverage for fork and resume
2. targeted tests for lineage persistence and provider-policy-aware continuation selection
3. no claim of closure if fork still fundamentally requires a loaded in-memory source thread

## Evidence

- path: `../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
  symbol: `Current Code Diagnosis`
  lines: `58-110`
- path: `../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
  symbol: `Two Resume Paths Based On Continuation Mode`
  lines: `127-151`
- path: `2026-04-17-item-03-implementation-reference.md`
  symbol: `Stage A: Fork Over Durable State`
  lines: `30-128`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `resume_persisted_thread`
  lines: `413-479`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `fork_thread`
  lines: `1075-1178`
- path: `../../../codex-rs/core/src/thread_manager.rs`
  symbol: `fork_thread`
  lines: `596-618`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `InitialHistory`
  lines: `2259-2262`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `thread_spawn_edges`
  lines: `92-106`

## Links

- [[README.md]]
- [[2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md]]
- [[2026-04-17-item-03-implementation-reference.md]]
- [[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
