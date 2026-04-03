---
id: logos-openrouter-validation-adapter-strict-spec
title: LOGOS OpenRouter Validation Adapter Strict Spec
type: spec
status: active
tags: [implementation, strict-spec, openrouter, validation, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md],
    [../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md]
  ]
depends_on:
  [
    [../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-02-phase-1-boundary-extraction-strict-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Validation Adapter Strict Spec

## Start Here

1. `../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md`
2. `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
3. `../architecture/2026-04-02-logos-capability-model-architecture-spec.md`
4. `2026-03-30-logos-global-roadmap.md`

## Intent

Implement the first non-canonical validation adapter so Logos can prove that the shared runtime works for a stateless `Responses`-shaped facade without contaminating the OpenAI path.

Non-goals:
1. no claim of semantic parity with OpenAI
2. no provider-native feature expansion beyond the bounded validation scope
3. no hidden server-managed continuation simulation
4. no broad native-provider rollout in this slice

## Target Outcome

1. `logos-provider-openrouter` exists as a validation adapter.
2. One stateless text turn works through the shared boundary.
3. One follow-up turn works through client-managed replay from the Logos journal.
4. Error, limit, and unsupported-parameter behavior are normalized instead of hidden.

## Locked Decisions

1. OpenRouter is validation adapter, not semantic truth source.
2. Continuation policy is `ClientManaged` unless later evidence proves otherwise.
3. OpenAI semantics must remain unchanged after the adapter is added.
4. Unknown reasoning-token semantics must remain `UNKNOWN`, not guessed.

## Architecture Fit

1. `logos-provider-openrouter` owns compat request mapping and compat event normalization.
2. `logos-session-engine` owns replay orchestration using the durable journal.
3. `logos-runtime-types` and `logos-provider-api` remain shared and provider-agnostic.
4. `logos-tool-runtime` remains runtime-owned even when tool calls come through OpenRouter.

## External References

1. OpenRouter Responses facade -> Adapt -> use for bounded validation only.
2. OpenAI adapter behavior -> Preserve -> OpenRouter must not force semantic downgrade in the canonical path.

## Implementation Tasks (ordered)

1. `T1`
- package: `logos-provider-openrouter`
- files: `src/lib.rs`, `src/adapter.rs`, request/event mapping files chosen by implementation
- output: first OpenRouter adapter skeleton with explicit stateless continuation policy
2. `T2`
- package: `logos-session-engine`
- files: replay and adapter dispatch files chosen by implementation
- output: client-managed replay path for stateless follow-up turns
3. `T3`
- package: persistence/journal layer in current implementation scope
- files: journal integration files chosen by implementation
- output: enough durable record support to reconstruct a follow-up OpenRouter turn
4. `T4`
- package: capability and error mapping surface
- files: adapter or shared mapping files chosen by implementation
- output: explicit unsupported-parameter, error, and limits behavior
5. `T5`
- package: tests
- files: adapter/session integration tests chosen by implementation
- output: text-turn, replay-turn, and normalization tests

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| OpenRouter text turn works through shared boundary | `T1`, `T5` | targeted integration test | `A1` | `G1` |
| Stateless follow-up works by client-managed replay | `T2`, `T3`, `T5` | replay integration test | `A2` | `G2` |
| Unsupported or ignored controls do not fail silently | `T4`, `T5` | parameter-compat tests | `A3` | `G3` |
| OpenAI path remains semantically unchanged | `T5` | OpenAI regression test set in current scope | `A4` | `G4` |

## Acceptance and Tests

1. One OpenRouter text-only integration test passes.
2. One OpenRouter follow-up test passes using Logos-owned replay rather than fake provider state.
3. One parameter-compatibility or unsupported-feature test proves explicit handling.
4. Existing OpenAI boundary tests continue to pass.

## Promotion Artifacts

1. OpenRouter adapter validation report
2. Replay-path acceptance report
3. Compatibility limitation report
4. OpenAI no-regression note

## Risks and Rollback

1. Risk: OpenRouter facade assumptions leak into shared runtime as if they were universal.
2. Mitigation: keep stateless policy and compatibility caveats adapter-local.
3. Risk: replay path becomes under-specified and silently diverges from real turn history.
4. Mitigation: require durable journal reconstruction tests.
5. Rollback: keep OpenRouter in validation-only status and avoid promoting any unstable shared abstractions.

## Agent Guardrails

1. Do not invent `previous_response_id` semantics for OpenRouter.
2. Do not widen shared runtime types solely to match OpenRouter quirks.
3. Do not weaken OpenAI parity to make OpenRouter feel more universal.
4. Do not upgrade `UNKNOWN` reasoning semantics into implementation assumptions.

## Evidence

- path: `../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md`
  symbol: `Required Adapter Behavior`
  lines: `80-115`
- path: `../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Resume Modes`
  lines: `59-76`
- path: `../architecture/2026-04-02-logos-multi-provider-feature-gap-matrix.md`
  symbol: `OpenRouter`
  lines: `63-68`
- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 5: Validation Adapter`
  lines: `163-180`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Unknowns and Prototype Targets`
  lines: `190-203`

## Links

- [[README.md]]
- [[../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[../architecture/2026-04-02-logos-persistence-resume-architecture-spec.md]]
- [[2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
