---
id: logos-provider-openrouter-adapter-spec
title: LOGOS OpenRouter Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, openrouter, responses, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-03-30-logos-adapter-boundary-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS OpenRouter Adapter Architecture Spec

## Purpose

- Define the first non-canonical validation adapter for Logos.
- Use OpenRouter to pressure-test client-managed history, graceful degradation, and error normalization.
- Prove the boundary can survive a `Responses`-shaped facade that is not semantically equal to OpenAI.

## Adapter Role

- validation adapter
- stateless stress-test for continuation policy
- near-compatible facade, not semantic baseline

## Why OpenRouter Matters

- It is close enough to OpenAI to reduce early adapter cost.
- It is different enough to expose false assumptions immediately.
- It is the cleanest second adapter for proving the kernel does not secretly require OpenAI server state.

## Responses Reuse Evaluation

### Surface Type

- OpenAI-like `Responses` facade, explicitly stateless and beta.

### What Logos Reuses

- request vocabulary where compatible
- streaming envelope where compatible
- tool-calling envelope
- structured output controls where supported
- compat-mode error and limit mapping inputs

### Advantages

- fast second-adapter path
- validates that normalized event and message types are not OpenAI-only
- forces the journal/replay design to become real
- gives access to many models through one adapter surface

### Disadvantages

- no server-side state
- beta maturity and possible behavior churn
- model/provider heterogeneity behind the facade
- some parameters may be ignored or interpreted differently by downstream providers
- reasoning carryover semantics are still not fully locked in current research

### Decision

- Use `Responses` as a bounded validation surface only.
- Do not treat OpenRouter as semantic truth.
- Keep continuation policy explicitly `ClientManaged` unless later evidence proves a more nuanced mode.

## Required Adapter Behavior

### Request Policy

- The adapter may reuse OpenAI-like request assembly where it remains honest.
- Unsupported or ambiguous controls must be filtered, downgraded, or marked via capability/error policy rather than silently assumed.

### Continuation And Replay

- Every turn must be resumable through Logos-owned history and durable journal.
- The adapter must not invent fake `previous_response_id` semantics.
- Replay rules must be explicit enough to reconstruct the next turn from stored Logos records.

### Streaming

- Preserve normalized stream events when the facade provides them.
- Do not over-promise ordering or reasoning semantics beyond what current evidence supports.
- Unknown reasoning-token semantics must stay marked `UNKNOWN` until prototype-backed.

### Tools

- Tool calls are allowed through the compat surface.
- Tool execution still remains runtime-owned by default.
- If OpenRouter exposes convenience integrations such as web search, they must not be mislabeled as identical to OpenAI hosted tools.

## Required Capability Posture

- supports client-side history
- does not assume server-managed continuation
- exposes structured outputs only as model-capability-dependent
- exposes tool calling as compatibility-dependent rather than universal

## Failure Boundaries

- Any incompatibility between requested controls and downstream model/provider behavior must stay adapter-local.
- The shared runtime must receive normalized errors or capability failures, not hidden behavioral drift.

## Non-goals

- no semantic flattening of downstream model differences
- no attempt to treat OpenRouter as a drop-in replacement for all OpenAI behavior
- no promise of reasoning parity before prototype proof

## Acceptance Gates

1. One stateless text turn completes through the adapter boundary.
2. One follow-up turn works by replaying client-managed journal state.
3. Error and limit mapping produce normalized failures instead of silent misbehavior.
4. OpenAI path remains unchanged in semantics after OpenRouter is added.

## Open Questions

- exact reasoning-token and reasoning-details carryover semantics
- final model-capability policy for ignored or partially supported controls
- whether any OpenRouter-specific convenience features deserve first-class exposure in Logos 0.1

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `MVP Required Providers And Adapters`
  lines: `49-51`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 5: Validation Adapter`
  lines: `163-180`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `132-140`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Unknowns and Prototype Targets`
  lines: `190-203`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `102-102`
- path: `../../input-sources-temp/provider-source-inventory.md`
  symbol: `OpenRouter`
  lines: `49-75`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
