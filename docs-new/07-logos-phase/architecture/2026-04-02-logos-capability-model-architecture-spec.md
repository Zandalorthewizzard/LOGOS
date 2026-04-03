---
id: logos-capability-model-architecture-spec
title: LOGOS Capability Model Architecture Spec
type: spec
status: active
tags: [architecture, capabilities, providers, runtime, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-03-30-logos-runtime-types-v0.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-03-30-logos-runtime-types-v0.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Capability Model Architecture Spec

## Purpose

- Define the final capability model needed before implementation.
- Prevent Logos from deriving behavior from provider names or compat surfaces instead of explicit capability facts.
- Separate provider-level, mode-level, and model-level truths cleanly.

## Core Decision

- Capabilities are first-class architecture inputs.
- The session engine and tool runtime must make decisions from declared capability sets, not from hidden provider branching.
- Capability absence is as important as capability presence and must stay explicit.

## Capability Layers

### Layer 1: Provider Capability Set

- stable characteristics shared across a provider adapter or adapter mode
- examples:
  - server-side continuation support
  - client-managed replay support
  - provider-native tools support
  - context cache support
  - compaction support

### Layer 2: Model Capability Set

- model-specific properties within a provider or mode
- examples:
  - tool calling
  - streamed tool arguments
  - parallel tools
  - structured outputs
  - reasoning deltas
  - multimodal inputs

### Layer 3: Operational Policy

- Logos-owned execution choices layered on top of capabilities
- examples:
  - whether a feature is enabled in the current product milestone
  - whether provider-native tools are allowed for a session
  - whether fallback/replay is required

## Why Layer Separation Matters

- A provider can support a feature in native mode but not in compat mode.
- A model can support a feature while the provider family generally does not guarantee it.
- Logos product policy may intentionally disable a supported provider capability.

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- the lesson that capability-rich providers expose more than plain text generation
- the need to distinguish continuation, tools, reasoning, and schema outputs as separate concerns
- the insight that request compatibility is not the same thing as semantic parity

### Advantages Of Reusing Responses Concepts

- gives Logos a concrete baseline capability vocabulary from the strongest current path
- helps avoid under-specified provider contracts
- makes OpenAI fidelity measurable without forcing other providers into identical shapes

### Disadvantages And Limits

- a `Responses`-biased capability taxonomy could become too OpenAI-shaped
- providers with native caching, MCP server calling, or mixed-mode semantics need capability dimensions beyond OpenAI's default vocabulary

### Decision

- Start from the useful capability distinctions surfaced by `Responses`.
- Expand the capability model only where research proves the baseline is insufficient.
- Do not infer parity from any provider's OpenAI-compat marketing surface.

## Mandatory Capability Families

1. continuation and replay
2. streaming granularity
3. tool calling
4. streamed tool arguments
5. provider-native tools
6. structured outputs
7. reasoning/thinking exposure
8. multimodal input classes
9. context caching or similar provider-native carryover
10. MCP or connector-style integration surfaces

## Mode-Aware Capability Rules

- Compat mode and native mode must have independent capability declarations.
- Capability declarations must be persisted or derivable for replay/resume safety.
- The same provider ID is not enough when the mode changes semantics.

## Fallback Rules

- If a capability is absent, Logos must choose one of:
  1. portable fallback
  2. explicit downgrade
  3. explicit unsupported-capability error
- Silent downgrade is not acceptable unless documented as product policy.

## Non-goals

- no giant fake-neutral ontology for every possible provider feature
- no hidden behavior keyed off provider brand names
- no assumption that one compat endpoint means full capability equivalence

## Acceptance Gates

1. OpenAI, OpenRouter, Gemini compat/native, Z.AI compat/native, LM Studio compat/native, and Anthropic can all be described without hidden branching.
2. Session decisions can be written against capability flags rather than provider-specific if/else logic.
3. Fallback and unsupported-feature behavior are explicit.
4. Provider mode differences remain visible in the capability system.

## Open Questions

- whether reasoning needs additional sub-capabilities beyond delta visibility and carryover support
- whether MCP should be one capability family or split across runtime-owned and provider-native categories
- whether some provider-native carryover semantics deserve a distinct family separate from continuation and caching

## Evidence

- path: `2026-03-30-logos-runtime-types-v0.md`
  symbol: `v0 Rust API`
  lines: `127-162`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `128-143`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `30-45`
- path: `2026-04-02-logos-adapter-kernel-architecture.md`
  symbol: `Optimization Targets Before Coding`
  lines: `169-190`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-03-30-logos-runtime-types-v0.md]]
