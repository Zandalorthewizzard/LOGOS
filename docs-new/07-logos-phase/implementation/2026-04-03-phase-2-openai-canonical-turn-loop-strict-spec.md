---
id: logos-phase-2-openai-canonical-turn-loop-strict-spec
title: LOGOS Phase 2 OpenAI Canonical Turn Loop Strict Spec
type: spec
status: active
tags: [implementation, strict-spec, phase-2, openai, turn-loop, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [
      ../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md,
    ],
    [../architecture/2026-04-03-logos-compaction-strategy-discussion.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md,
    ],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 2 OpenAI Canonical Turn Loop Strict Spec

## Start Here

1. `../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md`
2. `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
3. `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
4. `../architecture/2026-04-03-logos-compaction-strategy-discussion.md`
5. `2026-03-30-logos-global-roadmap.md`

## Intent

Preserve the canonical OpenAI/Codex turn loop as the reference LOGOS path while making continuation, stream ordering, turn-state behavior, and compaction boundaries explicit enough for later phases to extend safely.

Non-goals:

1. implementing non-OpenAI adapters
2. broad persistence/resume redesign
3. broad tool-loop redesign beyond canonical turn preservation
4. CLI/TUI migration

## Target Outcome

1. OpenAI canonical turn behavior remains preserved through the LOGOS boundary without hidden lifecycle regressions.
2. The OpenAI path has an evidence-backed ownership split for continuation, websocket incrementality, turn-state, and compaction.
3. Later persistence/resume and provider phases can build on the phase-2 boundary without another broad refactor.

## Locked Decisions

1. OpenAI remains the canonical turn-loop reference path.
2. Turn-scoped transport/session behavior stays in Codex/OpenAI layers unless a narrow registered seam is strictly required.
3. Continuation remains explicit and capability-driven.
4. OpenAI remote compaction stays preserved as an adapter-native fast path, not a fake-neutral kernel primitive.

## Architecture Fit

1. `codex-rs/core` owns canonical OpenAI turn transport/session mechanics and websocket request behavior.
2. `codex-rs/logos-provider-openai` owns OpenAI request/event mapping and capability translation.
3. `codex-rs/logos-session-engine` owns normalized policy orchestration and future durable journal integration points.

## Implementation Tasks (ordered)

1. `T1`

- package: `docs-new/07-logos-phase/architecture`
- files: `2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md`, `2026-04-03-logos-compaction-strategy-discussion.md`
- output: architecture lock for phase-2 ownership and preservation boundaries

2. `T2`

- package: `codex-rs/core`, `codex-rs/codex-api`
- files: continuation/turn-state/compaction touch points identified by the phase-2 discussion
- output: only the minimum registered seams needed for canonical turn preservation, if any, plus honest compaction or turn-state handoff surfaces where no seam is required

3. `T3`

- package: `codex-rs/logos-provider-openai`
- files: OpenAI adapter request/event/compaction integration points
- output: canonical OpenAI turn behavior preserved through the adapter without hidden provider state leaks

4. `T4`

- package: `codex-rs/logos-session-engine`
- files: normalized turn orchestration integration points
- output: phase-2-safe orchestration that preserves canonical OpenAI behavior and exposes later persistence hooks such as capability-backed compaction handoff points

5. `T5`

- package: `codex-rs/core/tests`, `codex-rs/logos-provider-openai`, `codex-rs/logos-session-engine`
- files: focused tests for continuation, ordering, compaction preservation, and any registered seam behavior
- output: explicit proof that the canonical OpenAI turn loop still holds through the LOGOS boundary

## Requirement -> Task -> Test -> Gate

| Requirement                                                                | Task IDs               | Tests                                 | Acceptance | Gates |
| -------------------------------------------------------------------------- | ---------------------- | ------------------------------------- | ---------- | ----- |
| Canonical continuation remains explicit and lifecycle-correct              | `T2`, `T3`, `T5`       | focused continuation tests            | `A1`       | `G1`  |
| Turn-state preservation remains canonical within logical turns             | `T2`, `T3`, `T4`, `T5` | focused turn-state tests              | `A2`       | `G2`  |
| Stream ordering and terminal semantics remain canonical                    | `T3`, `T4`, `T5`       | focused event-order tests             | `A3`       | `G3`  |
| OpenAI compaction path is preserved without shared-architecture distortion | `T1`, `T2`, `T3`, `T5` | focused compaction preservation tests | `A4`       | `G4`  |
| Later persistence/resume work has explicit handoff hooks                   | `T1`, `T4`             | docs + code ownership review          | `A5`       | `G5`  |

## Acceptance and Tests

1. focused OpenAI continuation tests pass
2. focused turn-state preservation tests pass
3. focused OpenAI stream ordering and completion tests pass
4. focused OpenAI compaction preservation tests pass
5. any new Codex-base seam is registered in the seam index and introduced in a dedicated commit

## Promotion Artifacts

1. phase-2 architecture discussion
2. phase-2 strict spec
3. phase-2 acceptance report when implementation starts
4. seam register updates if additional Codex-base hooks are required

## Risks and Rollback

1. Risk: phase-2 reimplements canonical Codex/OpenAI turn behavior in `logos-*` crates.
2. Mitigation: keep ownership split explicit and require evidence-backed seam justification before touching `codex-core`.
3. Risk: compaction or turn-state handling leaks provider-name logic into shared runtime design.
4. Mitigation: keep OpenAI-preservation logic adapter-owned and document portable versus adapter-native ownership explicitly.
5. Rollback: stop at the discussion/spec stage if the planned ownership split still requires broad Codex transport surgery.

## Agent Guardrails

1. Do not treat phase-2 as permission for broad `codex-core` refactoring.
2. Do not promote raw OpenAI request/response types into shared LOGOS runtime contracts.
3. Do not widen persistence scope beyond explicit phase-2 handoff points.
4. Do not let compaction preservation become a provider-name switch embedded in shared runtime policy.

## Evidence

- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 2: OpenAI Canonical Turn Loop`
  lines: `107-126`
- path: `../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md`
  symbol: `Core Decision`
  lines: `46-88`
- path: `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Required Adapter Behavior`
  lines: `80-147`
- path: `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Durable Journal Minimum Record Model`
  lines: `45-58`
- path: `../architecture/2026-04-03-logos-compaction-strategy-discussion.md`
  symbol: `Phase Placement`
  lines: `118-137`

## Links

- [[README.md]]
- [[2026-03-30-logos-global-roadmap.md]]
- [[../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md]]
- [[../architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[../architecture/2026-04-03-logos-compaction-strategy-discussion.md]]
