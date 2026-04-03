---
id: logos-adapter-project-docs-spec
title: LOGOS Adapter Project Documentation Spec
type: spec
status: active
tags: [architecture, adapters, documentation, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-adapter-boundary-spec.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md]
  ]
depends_on:
  [
    [2026-03-30-logos-adapter-boundary-spec.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Adapter Project Documentation Spec

## Purpose

- Define the canonical documentation set that must exist before adapter implementation broadens.
- Force provider architecture work to become decision-grade before code work starts.
- Standardize the mandatory `Responses` evaluation section across all adapter project documents.
- Standardize the minimum runtime-tier, Tier-0 semantic, health, and fallback sections across adapter specs.

## Why This Exists Now

- Current research is strong enough to drive planning, but several adapter decisions still need narrower architecture docs before implementation.
- The current pack already says the next step is stricter planning and prototype targeting rather than broad coding.

## Required Document Set

### 1. Kernel Architecture Doc

- One canonical document for the shared adapter boundary, session seam, continuation policy, capability policy, extension channels, and failure boundaries.
- This document must answer what every adapter is allowed to depend on and what must remain provider-owned.

### 2. Provider Adapter Profile Doc

- One canonical document that profiles the first provider classes and target adapters.
- It must cover at minimum:
  - OpenAI canonical adapter
  - OpenRouter validation adapter
  - Gemini mixed compat/native adapter
  - Anthropic native adapter
  - Z.AI mixed compat/native adapter
  - LM Studio dev/local validation adapter

### 3. Future Narrow Docs

- Add provider-specific deep-dive docs only when one provider reaches implementation-spec readiness or when one provider's semantics are too large for the shared profile document.

## Mandatory Section In Every Adapter Project Document

Every new adapter project document must contain a dedicated section named `Responses Reuse Evaluation` or a more specific equivalent.

That section must explicitly answer all of these:

1. Does this provider expose a real `Responses` surface, an OpenAI-compatible facade, or no `Responses` surface at all?
2. Which `Responses` concepts, if any, can Logos safely reuse for this provider?
3. What are the advantages of using that `Responses` surface for this provider?
4. What are the disadvantages, semantic lies, lock-in risks, or missing features if Logos uses that `Responses` surface?
5. Final decision:
   - use `Responses` heavily,
   - use `Responses` only as bootstrap or compat mode,
   - or avoid `Responses` for the primary adapter path.

If the answer is incomplete, the document must mark the missing part `UNKNOWN` and name the prototype or research gap.

## Mandatory Sections In Every Provider Adapter Spec

Every provider adapter spec must also document all of these explicitly:

1. `Provider Role`
2. `Canonical Deviation Summary`
3. `Runtime Tier Target`
4. `Continuation Model`
5. `Stream Ontology`
6. `Tool-Loop Control Model`
7. `Persistence / Replay Requirements`
8. `Capability Declaration`
9. `Error / Limit Normalization`
10. `Fallback / Downgrade Policy`
11. `Operational Risk Notes`
12. `Change Watchpoints`
13. `Acceptance Test Matrix`

## Document Quality Bar

- Do not repeat the fake-neutral mistake of pretending all providers are just OpenAI with different URLs.
- Do not drift into provider-native maximalism that weakens the canonical OpenAI path.
- Keep every design choice attached to evidence from the research pack or source snapshots.
- Keep unknowns explicit when the current research does not justify a stronger claim.
- Treat adapters as semantic translators and downgrade boundaries, not JSON pass-through layers.

## Recommended Order

1. Lock the shared kernel architecture.
2. Lock the provider profile and classification document.
3. Only then split into narrow provider implementation specs.

## Immediate Output From This Spec

- `2026-04-02-logos-adapter-kernel-architecture.md`
- `2026-04-02-logos-provider-adapter-profiles.md`
- `2026-04-02-logos-runtime-contract-spec.md`
- `2026-04-02-logos-runtime-tier-contract-spec.md`
- `2026-04-02-logos-tier-0-semantic-surfaces-spec.md`

## Evidence

- path: `../research/2026-03-30-research-pack-audit.md`
  symbol: `Verdict`
  lines: `22-27`
- path: `../research/2026-03-30-research-pack-audit.md`
  symbol: `Recommended Next Use Of This Pack`
  lines: `67-72`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Locked Decisions`
  lines: `23-40`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Recommended Decisions for Logos 0.1`
  lines: `181-188`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Unknowns and Prototype Targets`
  lines: `190-205`

## Links

- [[README.md]]
- [[2026-03-30-logos-adapter-boundary-spec.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../research/2026-03-30-logos-provider-research-report.md]]
