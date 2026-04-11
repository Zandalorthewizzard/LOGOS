---
id: logos-structured-output-capability-discussion
title: LOGOS Structured Output Capability Discussion
type: discussion
status: active
tags: [architecture, discussion, structured-output, capabilities, openai, openrouter, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [../implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
  ]
depends_on:
  [
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-provider-openai-adapter-spec.md],
    [2026-04-02-logos-provider-openrouter-adapter-spec.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Structured Output Capability Discussion

## Purpose

- record the structured-output gap that surfaced during the phase-5 OpenRouter validation review
- separate the immediate phase-5 honesty fix from the larger future feature lane
- prevent LOGOS from claiming schema-output support at the adapter boundary before the boundary can actually carry schema contracts

## Problem

The current LOGOS boundary is too coarse for structured output.

1. `logos-runtime-types` currently exposes only `OutputMode::Text` and `OutputMode::Structured`.
2. The current `InternalTurnRequest` shape carries no schema payload, no strictness flag, and no output-contract metadata beyond that enum.
3. The OpenRouter validation adapter currently rejects `OutputMode::Structured` in request mapping.
4. The same adapter also advertises `supports_structured_output: true`.

That combination is not an honest capability contract.

## Why This Surfaced In Phase 5

This did not come from a bad phase-5 scope decision. It came from three layers drifting apart.

1. The phase-5 strict spec intentionally locks a bounded text-only validation slice.
2. The OpenRouter architecture spec correctly leaves room for structured outputs where the facade and model really support them.
3. The current runtime capability shape still models structured output as a provider-level boolean instead of a richer mode-aware and model-aware contract.

So phase-5 implementation imported a future possibility into a current capability declaration even though the bounded slice never implemented the feature.

## Current Reality

### OpenAI Canonical Path

Canonical Codex/OpenAI already supports schema-constrained output through the older `final_output_json_schema` path.

That means structured output is real in LOGOS overall, but only inside the preserved OpenAI/Codex path today.

### OpenRouter Validation Path

OpenRouter documentation indicates that structured outputs exist on the facade for compatible models.

But phase-5 did not implement that feature through the new LOGOS boundary. The current adapter is text-only by design for this slice.

### LOGOS Boundary State

The new LOGOS runtime boundary has not yet imported a schema-bearing output contract.

Today, `OutputMode::Structured` is only a coarse marker. It is not enough to express:

1. `json_object` versus `json_schema`
2. schema payload
3. strict versus non-strict enforcement
4. provider-mode differences
5. model-specific support

## Design Position

### 1. Structured Output Is A First-Class Capability Family

Structured output must remain an explicit capability family in LOGOS.

It should not be dropped from the long-term capability model just because phase-5 is text-only.

### 2. Provider-Level Boolean Is Not Sufficient

The current provider-level `supports_structured_output: bool` is too weak for the real problem.

Structured output support depends on:

1. provider mode
2. model support
3. request-shape support at the LOGOS boundary
4. product policy for the current phase

### 3. OpenAI Remains Canonical Truth Source

The strongest current evidence and implementation already exist on the OpenAI/Codex path.

Future shared structured-output work should be shaped from that canonical path outward, not from OpenRouter compatibility marketing inward.

### 4. OpenRouter Must Stay Honest

OpenRouter should only advertise structured output once the LOGOS boundary can carry a real output schema contract and the adapter proves honest request/response behavior for that contract.

Until then, explicit rejection is correct.

## Immediate Phase-5 Decision

For phase-5 closeout, the correct action is narrow and non-ambiguous.

1. `logos-provider-openrouter` must advertise `supports_structured_output: false` in the current validation slice.
2. The existing explicit rejection of `OutputMode::Structured` should remain in place.
3. The phase-5 acceptance report must describe structured output as unsupported in the current bounded scope, not as a latent supported feature.

This is a capability-truth fix, not a feature rollback.

## Future Architecture Direction

When LOGOS opens real structured-output work, the boundary should be richer than the current enum.

The future direction should likely distinguish at least:

1. plain text output
2. unconstrained JSON object output
3. JSON-schema-constrained output with schema payload and strictness

That future lane must also decide where support truth lives:

1. provider-wide capability
2. model capability
3. request-time validation
4. product policy

The current answer is: all four matter, and the boundary has not encoded that yet.

## Phase Placement

### Phase 5

Phase-5 should only fix capability honesty and error normalization.

It should not widen into real structured-output implementation.

### After Phase 5

A separate discussion/spec pass is still required before the shared LOGOS boundary can claim structured-output support outside the preserved OpenAI path.

### Relation To Later Roadmap Phases

This future structured-output lane should land before broad non-OpenAI capability claims expand in later provider phases.

In roadmap terms:

1. phase-5 keeps the validation adapter honest
2. phase-6 and phase-7 should not claim structured-output parity through the shared boundary until this dedicated lane is locked
3. native provider expansion should consume a real structured-output contract rather than guessing from provider docs

## Non-goals

1. no attempt to force OpenRouter to pretend it has OpenAI semantic parity
2. no emergency refactor of the canonical OpenAI path
3. no new Codex-base seam
4. no broad multi-provider structured-output implementation in the phase-5 fix pass

## Implementation Readiness Verdict

This discussion does not make structured output implementation-ready.

It locks only two things:

1. phase-5 must stay honest and bounded
2. real shared structured-output work needs its own future discussion/spec lane

## Evidence

- path: `../../../codex-rs/logos-runtime-types/src/message.rs`
  symbol: `OutputMode`, `InternalTurnRequest`
  lines: `12-80`
- path: `../../../codex-rs/logos-runtime-types/src/capabilities.rs`
  symbol: `ProviderCapabilitySet`
  lines: `13-24`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `provider_capabilities`
  lines: `61-72`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `reject_unsupported_controls`
  lines: `76-123`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `UserTurnInput`
  lines: `2633-2640`
- path: `../../../codex-rs/app-server/tests/suite/v2/output_schema.rs`
  symbol: `request missing text.format field`
  lines: `83-97`
- path: `../../../codex-rs/core/tests/suite/json_result.rs`
  symbol: `final_output_json_schema`
  lines: `75-89`
- path: `2026-04-02-logos-capability-model-architecture-spec.md`
  symbol: `Layer 2: Model Capability Set`
  lines: `51-60`
- path: `2026-04-02-logos-provider-openrouter-adapter-spec.md`
  symbol: `Required Capability Posture`
  lines: `84-88`
- path: `../implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md`
  symbol: `Target Outcome`
  lines: `45-50`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 5: Validation Adapter`
  lines: `175-193`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 6: Local Validation Targets`
  lines: `194-210`
- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 7: Native Provider Expansion`
  lines: `212-228`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `130-136`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Provider / mode`
  lines: `30-39`
- equivalent: `https://developers.openai.com/api/docs/guides/structured-outputs`
- equivalent: `https://openrouter.ai/docs/guides/features/structured-outputs`

## Links

- [[README.md]]
- [[2026-04-02-logos-capability-model-architecture-spec.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[../implementation/2026-04-02-openrouter-validation-adapter-strict-spec.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
