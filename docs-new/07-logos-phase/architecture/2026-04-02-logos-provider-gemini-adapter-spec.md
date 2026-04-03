---
id: logos-provider-gemini-adapter-spec
title: LOGOS Gemini Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, gemini, google, logos]
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

# LOGOS Gemini Adapter Architecture Spec

## Purpose

- Define the architecture posture for Gemini as a commercially important mixed-mode provider.
- Prevent Logos from accidentally treating Gemini compat mode as the whole provider.
- Decide where compat bootstrap ends and native-first architecture begins.

## Adapter Role

- mixed-mode provider adapter
- native-first long-term design
- compat surface allowed only as a bounded bootstrap mode

## Why Gemini Matters

- It is explicitly called out in the roadmap as an MVP-relevant provider.
- It has both OpenAI-compatible entrypoints and native semantics that differ materially from OpenAI.
- It is the best early proof that mixed-mode providers need explicit mode modeling, not one flattened adapter story.

## Adapter Modes

### Mode A: Gemini OpenAI Compat

- lowest-friction bootstrap path
- useful for narrow experiments and selective early integration
- must stay clearly marked as compatibility mode

### Mode B: Gemini Native

- long-term primary architecture target
- required for honest handling of native thinking, tool, and continuation semantics

## Responses Reuse Evaluation

### Surface Type

- OpenAI-compatible layer plus native Gemini APIs.

### What Logos Reuses

- compat-mode request and stream shape for bounded scenarios
- schema/output patterns where compat behavior is sufficient
- normalized kernel concepts such as tool events, continuation policy, and capabilities

### Advantages

- reduces early integration cost
- allows quick comparison against the OpenAI-shaped boundary
- gives Logos a practical bootstrap path before native adapter design is complete

### Disadvantages

- compat mode is not equivalent to native Gemini orchestration
- Google-specific fields can leak through compat mode and prove that the surface is only partial
- hosted tools, thinking, multimodal behavior, and continuation semantics are not faithfully described by `Responses` alone
- overcommitting to compat mode would bias the kernel toward OpenAI semantics

### Decision

- Do not make Gemini a `Responses`-first provider in the long-term architecture.
- Permit compat mode as bootstrap only.
- Treat native Gemini as the architecture target for any meaningful provider-specific capability exposure.

## Required Adapter Behavior

### Mode Separation

- The adapter must expose compat and native mode as distinct operational modes.
- Logs, persisted records, and capabilities must always record which mode was used.

### Continuation Policy

- Do not assume OpenAI-style continuation semantics in native Gemini mode.
- Compat mode may look OpenAI-like, but its continuation semantics must still be treated as provider-specific until prototype-backed.

### Tools And Reasoning

- Native function calling and thinking behavior must not be collapsed into generic OpenAI assumptions.
- Compat mode may be used only where its feature coverage is sufficient and explicit.
- Provider-specific behavior should go through capabilities and extensions before any shared-runtime widening.

### Structured Output

- Structured outputs should be modeled as supported but mode-sensitive.
- If parity between compat and native modes is incomplete, that difference must remain explicit.

## Implementation Posture

- phase 1: no Gemini code yet
- pre-implementation: lock mode split and minimum supported capability set
- first implementation target: one narrowly scoped Gemini mode, chosen explicitly rather than implicitly

## Non-goals

- no pretending Gemini compat is sufficient proof for native Gemini support
- no flattening of Google-specific behavior into generic `Responses` semantics
- no provider-native feature commitments without stronger evidence or prototypes

## Acceptance Gates

1. The chosen first Gemini mode is explicitly declared before coding.
2. Capabilities differ cleanly between compat and native mode.
3. Continuation policy is recorded per mode and does not assume OpenAI semantics by default.
4. Any provider-specific thinking/tool controls stay adapter-owned unless deliberately promoted.

## Open Questions

- whether Gemini should first ship in compat mode, native mode, or split milestone form
- exact structured-output parity between compat and native modes
- exact tool and hosted-feature overlap worth exposing in Logos 0.1

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `MVP Required Providers And Adapters`
  lines: `52-55`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `134-142`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `34-35`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `104-104`
- path: `../../input-sources-temp/provider-source-inventory.md`
  symbol: `Google Gemini`
  lines: `77-102`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
