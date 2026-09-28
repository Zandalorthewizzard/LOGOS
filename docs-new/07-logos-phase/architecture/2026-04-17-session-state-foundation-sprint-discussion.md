---
id: logos-session-state-foundation-sprint-discussion
title: Session-State Foundation Sprint Discussion
type: discussion
status: active
tags:
  [logos, architecture, phase-6, sprint, session-state, foundation, non-canonical]
related:
  [
    [README.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
    [../implementation/2026-04-17-session-state-foundation-sprint-strict-spec.md],
  ]
depends_on:
  [
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Sprint Discussion

## Purpose

This document defines the modular foundation sprint that turns the already-validated session-layer Items `01`, `02`, `04`, and `03` into an execution package.

The goal is not to implement every deferred control surface now. The goal is to build the shared non-canonical substrate that later items depend on.

## Why This Sprint Exists

The current OpenRouter proving lane still concentrates too much ownership in one place:

1. thread identity and runtime residency
2. monolithic session persistence
3. manual turn lifecycle and notification dispatch
4. live-thread-only fork and resume behavior

These are the four substrate problems that make later items noisy, duplicated, or premature.

## Sprint Boundary

Included:

1. `Item 01` - thread manager
2. `Item 02` - session persistence
3. `Item 04` - turn lifecycle and notification dispatch
4. `Item 03` - fork and resume orchestration

Excluded:

1. `Item 05` shell execution
2. `Item 06` review orchestration
3. `Item 07` archive and metadata migration
4. `Item 08` rollback
5. `Item 09` and `Item 13` compaction
6. `Item 10` interrupt
7. `Item 11` background terminals
8. `Item 12` steering
9. `Item 14` multi-provider handoff
10. `Item 15` streaming optimization

## Why The Order Is `01 -> 02 -> 04 -> 03`

1. `Item 01` establishes durable thread ownership and removes the permanent in-memory proving-lane registry as the authoritative source.
2. `Item 02` establishes the persistence boundary around the existing phase-4 journal and removes monolithic JSON snapshots as the only non-canonical durability shape.
3. `Item 04` establishes authoritative turn lifecycle and derived notifications so later session controls stop relying on manually assembled event sequences.
4. `Item 03` is implemented last in the sprint because it depends on the new registry, persistence boundary, and lifecycle truth.

This order is not cosmetic. It is the dependency order that reduces false discovery in later items.

## Checkpoints

### Checkpoint A: After `Item 01` + `Item 02`

Canonical checkpoint artifact after closeout:

1. `../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md`

At this checkpoint the sprint must answer:

1. is the shared non-canonical registry shape stable enough
2. is the persistence boundary around journal truth stable enough
3. do `Item 07` and part of `Item 08` now simplify materially

### Checkpoint B: After `Item 04`

At this checkpoint the sprint must answer:

1. did manual notification assembly actually disappear from the targeted non-canonical path
2. is later review, rollback, and interrupt work now grounded in authoritative lifecycle state

### Checkpoint C: After `Item 03`

At this checkpoint the sprint must answer:

1. does non-canonical fork and resume now run on shared substrate instead of proving-lane ownership
2. which deferred items should now be re-reviewed immediately

## Artifact Map

This sprint package has one navigation-level discussion, one umbrella strict spec, four task specs, and one required report template.

### Sprint-Level Docs

1. `2026-04-17-session-state-foundation-sprint-discussion.md`
2. `2026-04-17-session-state-foundation-sprint-strict-spec.md`
3. `2026-04-17-session-state-foundation-slice-implementation-report-template.md`

### Task Specs

1. `2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md`
2. `2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md`
3. `2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md`
4. `2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md`

## Mandatory Inputs For Every Task

Each task in this sprint must treat these as required inputs:

1. the relevant Item draft implementation discussion
2. the relevant Item implementation reference
3. the relevant local LOGOS/Codex code evidence
4. the exact local OpenCode and Goose source snapshots cited by the implementation reference

The task must not rely on chat memory when the implementation reference already cites a local source snapshot.

## Borrowing And Adaptation Rules

Every task in this sprint must follow these rules:

1. production code must use LOGOS-native naming and identity
2. imported OpenCode or Goose names must not be copied literally into final production ownership unless they are already generic industry terms
3. the implementation report must preserve traceability back to the original source file and line range
4. if a mechanism is adapted rather than directly refactored, the report must explain what changed and why
5. if a mechanism is rejected after initial consideration, the report must say so explicitly

## Stale Code Rule

Every task spec in this sprint must include:

1. current stale or transitional code that is not final architecture
2. code that may stay temporarily during the slice
3. code that must be removed before the slice can be called closed

This is mandatory because the current proving lane contains temporary logic that is useful as evidence but invalid as final ownership.

## Required Implementation Reports

Each delivered slice in this sprint must produce its own implementation report using the sprint report template.

The report must include:

1. closed
2. borrowed from references
3. adapted or renamed
4. stale code removed or still remaining
5. tests and checks run
6. residual risks

Current canonical delivery artifacts:

1. `../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md`
2. `../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md`
3. `../implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md`
4. `../implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md`
5. `../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md`
6. `../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md`

## Execution Model

This sprint is intentionally modular.

1. the sprint doc is navigation only
2. each task is a bounded slice with its own strict spec
3. each task must close or stop honestly before the next task widens
4. later items must be re-reviewed against delivered substrate instead of being pre-judged from theory alone

## Expected Re-Review After Sprint

The first re-review set after this sprint should be:

1. `Item 06` review orchestration
2. `Item 07` archive and metadata
3. `Item 08` rollback
4. `Item 10` interrupt

## Evidence

- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `The LOGOS-owned layer above provider wires should own`
  lines: `81-89`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `85-93`
- path: `2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Accepted Import For Item 01`
  lines: `28-35`
- path: `2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Accepted Import For Item 02`
  lines: `32-40`
- path: `2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md`
  symbol: `Accepted Import For Item 04`
  lines: `24-31`
- path: `2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
  symbol: `Accepted Import For Item 03`
  lines: `25-31`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[2026-04-17-item-02-session-persistence-draft-implementation-discussion.md]]
- [[2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md]]
- [[2026-04-17-item-03-fork-resume-draft-implementation-discussion.md]]
- [[../implementation/2026-04-17-session-state-foundation-sprint-strict-spec.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-01-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-02-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-03-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-task-04-implementation-report.md]]
- [[../implementation/2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
- [[../implementation/2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
