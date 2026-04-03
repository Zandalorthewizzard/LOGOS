---
id: logos-provider-zai-adapter-spec
title: LOGOS Z.AI Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, zai, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-preliminary-provider-research-scan.md]
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

# LOGOS Z.AI Adapter Architecture Spec

## Purpose

- Define the architecture posture for Z.AI as a mixed compat/native provider with strong provider-native semantics.
- Prevent an easy OpenAI-compat bootstrap from becoming a dishonest long-term architecture.
- Use Z.AI to test the extension-channel and capability model under richer provider-native behavior.

## Adapter Role

- mixed-mode provider adapter
- native-aware long-term design
- compat path allowed only as bounded bootstrap

## Why Z.AI Matters

- It combines OpenAI-compatible entrypoints with clearly differentiated native capabilities.
- It stresses multiple Logos concern areas at once: thinking, context caching, tool streaming, and MCP server calling.
- It is a strong test of whether Logos can support native capabilities without forcing false sameness.

## Adapter Modes

### Mode A: Z.AI OpenAI Compat

- useful for narrow bootstrap
- lower entry cost
- incomplete semantic picture

### Mode B: Z.AI Native

- architecture target for meaningful support
- required for honest treatment of caching, MCP server calling, and tool streaming

## Responses Reuse Evaluation

### Surface Type

- OpenAI-compatible path plus provider-native capability surface.

### What Logos Reuses

- compat-mode request/response patterns for bounded bootstrap work
- generic kernel concepts such as normalized tool events, continuation policy, durable journal, and capability flags

### Advantages

- faster initial adapter path
- easier comparison against OpenAI-shaped boundary
- useful when only narrow compat features are needed

### Disadvantages

- provider-native semantics around thinking, caching, MCP server calling, and tool streaming are too important to bury behind compat mode
- compat continuation semantics are not strong enough to define the long-term adapter shape
- official JS/TS SDK status remains unresolved, so bootstrap assumptions must stay conservative

### Decision

- Use `Responses`/OpenAI-compat only as bootstrap when needed.
- Keep the primary architecture native-aware and extension-capable.

## Required Adapter Behavior

### Native Capability Respect

- Keep MCP server calling distinct from application-owned MCP client/runtime semantics.
- Keep tool streaming semantics distinct from generic text streaming.
- Keep context caching explicit as provider-native capability rather than pretending it equals OpenAI continuation.

### Continuation And Persistence

- Persist provider mode explicitly.
- Treat caching and continuation as related but not identical concepts.
- Do not assume OpenAI-style `previous_response_id` semantics are enough to represent the native path.

### Tools And Streaming

- Normalize only the portable parts of tool lifecycle.
- Preserve provider-native detail via capability flags and extension payloads.

## Research Guardrails

- The current pack is strong enough to define architecture posture, but not to claim full field-level implementation readiness.
- Official JS/TS SDK remains `UNKNOWN`; do not design around an unverified SDK story.

## Non-goals

- no primary architecture based only on OpenAI compatibility examples
- no flattening of Z.AI-native MCP server calling into generic hosted tools
- no silent equivalence between context caching and OpenAI continuation handles

## Acceptance Gates

1. Compat and native modes remain explicitly separate in the design.
2. MCP server calling is treated as provider-native capability, not generic baseline behavior.
3. Context caching and continuation semantics are modeled separately.
4. `UNKNOWN` JS/TS SDK status remains explicit in planning.

## Open Questions

- whether Z.AI should enter Logos 0.1 only as profile-level planning or as a later narrow implementation spec
- how much native MCP server calling deserves first-class exposure
- what minimum native capability subset is worth supporting first

## Evidence

- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Confirmed Negative Findings`
  lines: `92-98`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `134-141`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `36-37`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `105-105`
- path: `../../input-sources-temp/provider-source-inventory.md`
  symbol: `Z.AI`
  lines: `104-141`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../research/2026-03-30-logos-provider-research-report.md]]
