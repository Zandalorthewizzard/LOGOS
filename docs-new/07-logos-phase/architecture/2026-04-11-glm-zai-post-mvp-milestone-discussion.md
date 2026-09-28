---
id: logos-glm-zai-post-mvp-milestone-discussion
title: LOGOS GLM Z.AI Post-MVP Milestone Discussion
type: discussion
status: active
tags: [logos, milestone, post-mvp, glm, zai, provider-expansion]
related:
  [
    [README.md],
    [2026-04-02-logos-provider-zai-adapter-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
  ]
depends_on:
  [
    [2026-04-02-logos-provider-zai-adapter-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS GLM Z.AI Post-MVP Milestone Discussion

## Status And Scope

This is a post-MVP milestone discussion.

GLM/Z.AI is not part of the current `LOGOS 0.1 beta` scope. The immediate product target is a preserved OpenAI path plus a beta-ready OpenRouter path, not broad provider coverage.

## Why This Moved Out Of MVP

1. Z.AI/GLM adds product breadth, but it does not materially improve concept validation compared with OpenRouter at the current stage.
2. There are still evidence gaps around public SDK posture, integration shape, and stable semantic guarantees.
3. Shipping Z.AI too early would increase adapter-surface complexity before the provider foundation is hardened through live OpenRouter use.
4. The current concept phase benefits more from deepening one wide validation path than from adding another partially understood provider lane.

## Milestone Intent

When this milestone becomes active, its job is to answer:

Can LOGOS add GLM/Z.AI in a way that is commercially useful and semantically honest without reopening preserved OpenAI contracts or reducing everything to fake OpenAI sameness?

## Locked Direction

1. Treat GLM/Z.AI as its own provider milestone, not as an automatic extension of the OpenRouter work.
2. Prefer evidence-backed native or mixed-mode evaluation over shallow OpenAI-compat assumptions.
3. Keep provider-specific semantics explicit, even if the first shipped mode is a narrower capability subset.
4. Do not let Z.AI-specific quirks rewrite the shared LOGOS runtime contract before provider foundation work is stable.

## Entry Conditions

This milestone should not move into implementation before:

1. `LOGOS 0.1 beta` has shipped with OpenAI plus OpenRouter.
2. OpenRouter beta feedback has clarified which provider-foundation abstractions were correct and which were too coarse.
3. Z.AI source availability, SDK posture, and integration surface have been refreshed enough to support a real strict-spec lane.

## Planned Questions

1. Is the first useful Z.AI path native, OpenAI-compatible, or mixed-mode?
2. Which features are worth first-class support versus explicit non-support?
3. What continuation, tool, and structured-output semantics are actually stable enough to promise?
4. Does Z.AI deserve concept-stage work before Gemini, or only after a broader provider-foundation pass?

## Non-goals

1. Shipping GLM/Z.AI in `LOGOS 0.1 beta`.
2. Assuming provider parity because an OpenAI-compatible endpoint exists.
3. Introducing a new Codex-base seam just to pre-prepare for Z.AI.

## Phase Placement

This milestone is post-MVP and follows the provider-foundation hardening lane.

## Evidence

- path: `2026-04-02-logos-provider-zai-adapter-spec.md`
  symbol: `LOGOS Z.AI Adapter Architecture Spec`
  lines: `1-240`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `LOGOS Provider Research Report`
  lines: `1-260`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `LOGOS Global Roadmap`
  lines: `1-280`

## Links

- [[README.md]]
- [[2026-04-02-logos-provider-zai-adapter-spec.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
