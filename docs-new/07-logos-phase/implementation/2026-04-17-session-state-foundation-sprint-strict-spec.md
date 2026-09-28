---
id: logos-session-state-foundation-sprint-strict-spec
title: Session-State Foundation Sprint Strict Spec
type: spec
status: active
tags:
  [logos, phase-6, strict-spec, sprint, session-state, foundation]
related:
  [
    [README.md],
    [../architecture/2026-04-17-session-state-foundation-sprint-discussion.md],
    [../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md],
    [../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md],
    [../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md],
    [2026-04-17-session-state-foundation-slice-implementation-report-template.md],
  ]
depends_on:
  [
    [../architecture/2026-04-17-session-state-foundation-sprint-discussion.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Session-State Foundation Sprint Strict Spec

## Start Here

1. `AGENTS.md`
2. `docs-new/07-logos-phase/architecture/2026-04-17-session-state-foundation-sprint-discussion.md`
3. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
4. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
5. `docs-new/07-logos-phase/implementation/2026-04-17-session-state-foundation-slice-implementation-report-template.md`

## Intent

Deliver the first modular execution sprint for the shared non-canonical session-state substrate by implementing Items `01`, `02`, `04`, and `03` in dependency order.

Non-goals:

1. no widening into shell execution or background terminals
2. no archive and metadata migration sprint
3. no review, rollback, interrupt, steering, or compaction sprint
4. no provider breadth expansion beyond the current phase-6 boundary
5. no broad `codex-core` rewrite for abstraction symmetry

## Target Outcome

1. Non-canonical thread identity is owned by a shared substrate instead of the OpenRouter proving-lane map.
2. Non-canonical persistence no longer depends on monolithic JSON snapshots as its only durable model.
3. Turn lifecycle and notifications for the targeted non-canonical path are driven by authoritative state transitions rather than manual notification assembly.
4. Fork and resume run on the new substrate rather than on live-thread-only proving-lane ownership.

## Locked Decisions

1. Task execution order is `Item 01 -> Item 02 -> Item 04 -> Item 03`.
2. Each task is a bounded slice with its own strict spec and its own implementation report.
3. Every task must use the corresponding implementation reference plus the exact local source snapshots cited by that reference.
4. Production code must use LOGOS-native naming. Reference naming remains in docs and reports only.
5. Every task must explicitly call out stale or invalid current code that must be removed or migrated.
6. A task is not closed if the target owner changed but stale provider-lane ownership still remains as final logic.
7. If implementation proves a draft verdict materially wrong, stop and route back to discussion instead of silently widening the slice.

## Architecture Fit

1. The shared LOGOS session-state layer owns non-canonical thread, state, lifecycle, and orchestration semantics above provider wires.
2. Provider lanes remain consumers of that substrate plus provider-specific policy hooks.
3. OpenAI remains the preserved canonical path and must not be weakened for shared-shape purity.

## Required References

Every task spec in this sprint must require:

1. the relevant Item draft implementation discussion
2. the relevant Item implementation reference
3. the relevant local source files in `codex-rs/`
4. the exact local OpenCode and Goose snapshot files cited in the implementation reference evidence section

## Borrow / Adapt / Rename Rules

1. Reference code may be refactored, adapted, or rejected, but the choice must be explicit in the slice report.
2. Imported names, class names, and variable names from OpenCode and Goose must not be preserved as final production identity unless they are already generic terms.
3. Slice reports must map:
   1. source file and line range
   2. LOGOS destination file and symbol
   3. adaptation or renaming rationale

## Implementation Tasks

1. `Task 01`
- spec: `2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md`
- item: `Item 01`
- purpose: shared non-canonical thread registry and ephemeral runtime boundary

2. `Task 02`
- spec: `2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md`
- item: `Item 02`
- purpose: metadata and persistence boundary around journal truth

3. `Task 03`
- spec: `2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md`
- item: `Item 04`
- purpose: authoritative lifecycle plus derived notifications

4. `Task 04`
- spec: `2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md`
- item: `Item 03`
- purpose: provider-policy-aware fork and resume on shared substrate

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| Shared non-canonical registry replaces proving-lane ownership | `Task 01` | targeted `codex-app-server` and new registry tests | task-level acceptance | no authoritative in-memory OR registry remains |
| Hybrid persistence boundary is locked around journal truth | `Task 02` | targeted persistence-layer tests | task-level acceptance | monolithic JSON snapshot is no longer the final owner |
| Derived notifications replace manual assembly on the targeted path | `Task 03` | targeted lifecycle and app-server tests | task-level acceptance | manual notification ownership is removed or reduced to transitional-only glue |
| Fork/resume works over durable state with provider-policy-aware continuation | `Task 04` | targeted resume/fork tests | task-level acceptance | no live-thread-only source dependency remains |

## Required Slice Reports

Each task must produce one report using `2026-04-17-session-state-foundation-slice-implementation-report-template.md`.

No task is considered complete without that report.

Current canonical delivery set:

1. `2026-04-18-session-state-foundation-task-01-implementation-report.md`
2. `2026-04-18-session-state-foundation-task-02-implementation-report.md`
3. `2026-04-18-session-state-foundation-checkpoint-a-reassessment.md`
4. `2026-04-18-session-state-foundation-task-03-implementation-report.md`
5. `2026-04-18-session-state-foundation-task-04-implementation-report.md`
6. `2026-04-18-session-state-foundation-post-task-04-reassessment.md`

## Acceptance And Tests

1. run targeted tests for every changed crate per task
2. do not run full-workspace validation as a substitute for slice-level proofs
3. after each task, produce the slice report before starting the next task
4. after `Task 02`, perform Checkpoint A re-review before starting `Task 03`
5. after `Task 04`, explicitly reassess `Items 06, 07, 08, 10`

## Risks And Rollback

1. Risk: the sprint may accidentally become a monolithic rewrite.
2. Mitigation: each task is separately spec-bound and separately reported.
3. Risk: stale proving-lane logic may be mistaken for valid substrate.
4. Mitigation: every task must enumerate stale and removable code explicitly.
5. Risk: imported OpenCode or Goose naming may leak into final production identity.
6. Mitigation: naming normalization is mandatory and borrow tracing stays in docs only.

## Agent Guardrails

1. Do not skip implementation references or source snapshots.
2. Do not silently widen a task into another item.
3. Do not preserve provider-lane stale code as final owner just because tests still pass.
4. Do not degrade the OpenAI canonical path.
5. Do not finalize a slice without the required implementation report.

## Evidence

- path: `../architecture/2026-04-17-session-state-foundation-sprint-discussion.md`
  symbol: `Why The Order Is 01 -> 02 -> 04 -> 03`
  lines: `43-50`
- path: `../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `The LOGOS-owned layer above provider wires should own`
  lines: `81-89`
- path: `../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Move Now`
  lines: `85-93`
- path: `../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md`
  symbol: `Stage A: Registry First`
  lines: `158-167`
- path: `../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Two-Layer Persistence Architecture`
  lines: `131-144`
- path: `../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md`
  symbol: `Turn Lifecycle State Machine`
  lines: `135-153`
- path: `../architecture/2026-04-17-item-03-fork-resume-draft-implementation-discussion.md`
  symbol: `Two Resume Paths Based On Continuation Mode`
  lines: `127-151`

## Links

- [[README.md]]
- [[../architecture/2026-04-17-session-state-foundation-sprint-discussion.md]]
- [[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]
- [[2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md]]
- [[2026-04-17-session-state-foundation-task-02-session-persistence-strict-spec.md]]
- [[2026-04-17-session-state-foundation-task-03-turn-lifecycle-strict-spec.md]]
- [[2026-04-17-session-state-foundation-task-04-fork-resume-strict-spec.md]]
- [[2026-04-18-session-state-foundation-task-01-implementation-report.md]]
- [[2026-04-18-session-state-foundation-task-02-implementation-report.md]]
- [[2026-04-18-session-state-foundation-checkpoint-a-reassessment.md]]
- [[2026-04-18-session-state-foundation-task-03-implementation-report.md]]
- [[2026-04-18-session-state-foundation-task-04-implementation-report.md]]
- [[2026-04-18-session-state-foundation-post-task-04-reassessment.md]]
