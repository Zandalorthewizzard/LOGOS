---
id: logos-tier-0-semantic-surfaces-spec
title: LOGOS Tier-0 Semantic Surfaces Spec
type: spec
status: active
tags: [architecture, adapters, semantics, continuation, streaming, tools, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-runtime-tier-contract-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md]
  ]
depends_on:
  [
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Tier-0 Semantic Surfaces Spec

## Purpose

- Define the three semantic surfaces every adapter must specify before it is considered complete.
- Prevent adapter design from collapsing into plain request/response translation.

## Tier-0 Rule

- If any Tier-0 surface is unspecified, the adapter is incomplete.
- Tier-0 surfaces drive tier assignment, fallback design, and acceptance tests.

## Surface 1: Continuation Model

Each adapter must specify:

- source of state authority
- provider-managed continuation support
- replay requirements when continuation is absent
- determinism risks under replay
- resume source of truth
- state invalidation conditions

## Surface 2: Stream Ontology

Each adapter must specify:

- raw stream transport type
- actual event categories emitted by provider
- typed versus chunk-based behavior
- ordering guarantees
- interleaving behavior
- partial JSON and tool-argument behavior
- completion markers
- reasoning-related event behavior
- mapping rules into `InternalEvent`

## Surface 3: Tool-Loop Control Model

Each adapter must specify one primary control model:

- runtime-owned
- model-suggested, runtime-executed
- provider-mediated
- provider-owned
- unsupported

It must also specify:

- whether tool intent is advisory or authoritative
- whether approvals can intervene before execution
- how tool results are reinjected
- whether multi-step tool chaining is deterministic enough for the current tier
- fallback behavior when loop control is weak or opaque

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- the canonical OpenAI path demonstrates that continuation, typed streaming, and tool-call framing are the real semantic backbone of agent runtime behavior

### Advantages

- forces adapter specs to describe the surfaces that actually preserve runtime authority
- prevents false confidence from coarse capability labels alone

### Disadvantages And Limits

- some providers will require conservative language until prototypes exist

### Decision

- Treat continuation, stream ontology, and tool-loop control as mandatory Tier-0 surfaces for every adapter.

## Acceptance Gates

1. Every adapter spec documents all three Tier-0 surfaces.
2. Each surface has explicit fallback or downgrade behavior.
3. Provider-specific unknowns are surfaced rather than hidden under `partial`.

## Evidence

- path: `C:\Users\zand\Desktop\analysis.txt`
  symbol: `Tier-0 Semantic Control Surfaces`
  lines: `155-219`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Resume Modes`
  lines: `59-76`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Portable Tool Loop`
  lines: `39-47`

## Links

- [[README.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-runtime-tier-contract-spec.md]]
