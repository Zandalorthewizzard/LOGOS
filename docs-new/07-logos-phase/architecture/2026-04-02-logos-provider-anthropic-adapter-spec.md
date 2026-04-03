---
id: logos-provider-anthropic-adapter-spec
title: LOGOS Anthropic Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, anthropic, claude, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../research/2026-03-30-preliminary-provider-research-scan.md],
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

# LOGOS Anthropic Adapter Architecture Spec

## Purpose

- Define the native-first architecture posture for Anthropic.
- Ensure Logos can support an important non-Responses provider without fake OpenAI semantics.
- Use Anthropic to validate that the kernel is genuinely adapter-modular.

## Adapter Role

- native-first provider adapter
- non-Responses mapping path
- proof that the kernel can serve materially different message and tool models

## Why Anthropic Matters

- It is one of the clearest examples of a strong provider whose semantics are not honestly described by OpenAI `Responses`.
- It exposes different message/content-block, tool-use, and caching patterns.
- It is a future architecture-regret source if Logos designs only for OpenAI-shaped providers.

## Responses Reuse Evaluation

### Surface Type

- No real `Responses` surface in the current plan.

### What Logos Reuses

- only the shared kernel concepts that are not OpenAI-specific:
  - normalized turn request shape
  - normalized tool events
  - continuation policy abstraction
  - durable journal contract
  - capabilities and extension channels

### Advantages

- avoids semantic distortion
- keeps Anthropic-native tool and message semantics visible
- pressures the kernel to stay honest instead of secretly depending on OpenAI wire assumptions

### Disadvantages

- higher adapter design cost
- less bootstrap reuse from Codex/OpenAI internals
- error normalization and some provider details remain less fully researched than OpenAI/OpenRouter

### Decision

- Do not use `Responses` as the primary Anthropic integration path.
- Treat Anthropic as a native-first adapter from the start.

## Required Adapter Behavior

### Message Mapping

- Map Anthropic-native message/content-block structures into Logos runtime messages without pretending they are OpenAI items.
- Preserve tool-use and tool-result lifecycle in a way that survives persistence and replay.

### Continuation And Caching

- Treat continuation as client-managed history plus provider-specific caching policy unless future evidence proves a stronger durable handle.
- Keep prompt caching and other provider-native carryover semantics adapter-owned until justified otherwise.

### Streaming

- Normalize stream output into `InternalEvent`, but do not assume OpenAI-like event granularity where the provider differs.
- Any Anthropic-specific event detail needed for correctness should go through extension channels instead of widening the kernel immediately.

### Tools

- Preserve the distinction between provider-native tool behavior and runtime-owned tools.
- Do not collapse Anthropic tool-use blocks into fake OpenAI hosted-tool semantics.

## Capability Posture

- native message semantics
- native tool-use semantics
- provider-specific caching/thinking behavior
- no dependency on `Responses` compatibility surface

## Research Limits To Respect

- Current error normalization details remain only partially specified.
- The current pack is sufficient for architecture posture, but not yet for a narrow implementation spec.
- Any stronger claim about exact field-level parity should be marked `UNKNOWN` until backed by deeper direct evidence or prototypes.

## Non-goals

- no translation-proxy design as the canonical long-term architecture
- no force-fit into OpenAI item/event vocabulary
- no early shared-runtime widening around Anthropic-specific details

## Acceptance Gates

1. Anthropic mode is clearly marked native-first in Logos docs.
2. No part of the design requires fake `previous_response_id` semantics.
3. Tool-use and caching semantics remain explicit and adapter-owned.
4. Unknowns stay documented rather than silently resolved.

## Open Questions

- exact durable continuation/caching posture Logos should expose for Anthropic
- exact error taxonomy mapping for normalized provider errors
- whether any Anthropic-native tool or thinking features deserve first-class Logos exposure in 0.1

## Evidence

- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `103-103`
- path: `../research/2026-03-30-logos-research-requirements.md`
  symbol: `Provider Semantics Matrix`
  lines: `69-93`
- path: `../research/2026-03-30-research-pack-audit.md`
  symbol: `Practical Readiness Assessment`
  lines: `51-65`
- path: `../../input-sources-temp/anthropic-sdk-typescript/README.md`
  symbol: `Claude SDK for TypeScript`
  lines: `1-28`
- path: `../../input-sources-temp/anthropic-sdk-python/README.md`
  symbol: `Anthropic Python SDK`
  lines: `1-40`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../research/2026-03-30-preliminary-provider-research-scan.md]]
