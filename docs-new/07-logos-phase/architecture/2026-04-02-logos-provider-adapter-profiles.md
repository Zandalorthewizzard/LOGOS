---
id: logos-provider-adapter-profiles
title: LOGOS Provider Adapter Profiles
type: spec
status: active
tags: [architecture, adapters, providers, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-project-docs-spec.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md],
    [../implementation/2026-03-30-logos-global-roadmap.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Provider Adapter Profiles

## Purpose

- Classify the first provider targets before implementation.
- Record the recommended adapter mode for each provider.
- Force an explicit `Responses` decision for each provider instead of leaving compat mode implicit.

## Parity Rule

- Only the OpenAI adapter should currently claim canonical parity with the strongest Codex/Responses path.
- Non-OpenAI adapters may still target competitive parity at the user-experience and capability-surface level when the runtime owns the common control surfaces honestly.
- No non-OpenAI adapter should claim invisible `Responses` semantic parity without direct evidence.

## OpenAI Canonical Adapter

### Recommended Adapter Mode

- Canonical adapter
- highest-fidelity path
- wrapper, not rewrite

### Responses Reuse Evaluation

- Surface type: real canonical `Responses` protocol
- What Logos should reuse: request/stream model, `previous_response_id`, built-in tools, MCP/connectors, structured outputs, typed events
- Advantages: best parity with Codex behavior, richest stateful continuation model, strongest reference path for kernel design
- Disadvantages: OpenAI-specific semantics can easily over-shape the shared runtime if not bounded
- Decision: use `Responses` heavily and treat it as the canonical upstream path

### Notes

- This is the only provider that should define the gold path for boundary fidelity.

## OpenRouter Validation Adapter

### Recommended Adapter Mode

- validation adapter
- Responses-compatible facade
- stateless by policy

### Responses Reuse Evaluation

- Surface type: OpenAI-like `Responses` facade
- What Logos should reuse: request/response vocabulary where it lowers adapter cost, tool-calling envelope, schema-oriented output controls, streaming shape where compatible
- Advantages: cheap second adapter for stress-testing the boundary, useful for validating client-managed history and error normalization
- Disadvantages: no server-side state, beta maturity, some parameters may be ignored or vary by model, not safe as semantic source of truth
- Decision: use `Responses` as a validation and bootstrap surface only; do not let it define shared semantics

### Notes

- This provider is valuable precisely because it exposes the limits of a `Responses`-shaped facade.
- `UNKNOWN`: exact reasoning-token carryover semantics still need tighter prototype proof.

## Gemini Adapter

### Recommended Adapter Mode

- mixed-mode adapter
- native-first for long-term design
- compat mode allowed for early bootstrap or selective scenarios

### Responses Reuse Evaluation

- Surface type: OpenAI-compatible layer plus native Gemini APIs
- What Logos should reuse: compat mode only where it lowers initial integration cost or preserves a narrow feature subset
- Advantages: faster entry path for selected use cases, may reduce initial request/stream mapping work
- Disadvantages: Google explicitly maintains native semantics beyond compat mode, provider-specific thinking and tool behavior leak through compat extensions, compat mode is not a faithful substitute for native Gemini orchestration
- Decision: do not make Gemini a `Responses`-first adapter; use compat mode only as a bounded bootstrap path

### Notes

- Keep native Gemini and Gemini compat documented as distinct modes.
- `UNKNOWN`: exact structured-output parity and hosted-tool overlap still need prototype confirmation.

## Anthropic Adapter

### Recommended Adapter Mode

- native-first adapter
- no fake `Responses` contract

### Responses Reuse Evaluation

- Surface type: no real `Responses` surface in the current plan
- What Logos should reuse: only abstract kernel concepts such as normalized tool events, continuation policy, and durable journal requirements
- Advantages: avoiding `Responses` reuse prevents semantic distortion around content blocks, tool use, caching, and thinking
- Disadvantages: mapping cost is higher because Anthropic needs direct native translation instead of cheap compat wrapping
- Decision: do not use `Responses` as the primary Anthropic integration path

### Notes

- Anthropic should be the clearest proof that the kernel is not secretly OpenAI-only.
- Error normalization remains only partially specified in the current research pack.

## Z.AI Adapter

### Recommended Adapter Mode

- mixed-mode adapter
- native-first long-term
- compat mode optional for bootstrap

### Responses Reuse Evaluation

- Surface type: OpenAI-compatible path plus provider-native capability surface
- What Logos should reuse: compat mode for narrow bootstrap scenarios; not for full MCP, caching, or thinking semantics
- Advantages: easier early access path through familiar OpenAI-compatible tooling
- Disadvantages: strong provider-native semantics around tool streaming, MCP server calling, thinking, and caching would be under-modeled if compat mode became primary
- Decision: use `Responses` only as an optional bootstrap path; keep the primary architecture native-aware

### Notes

- Z.AI is a good test of whether the extension-channel policy is honest enough.
- `UNKNOWN`: official JS/TS SDK status remains unresolved and must not be silently assumed.

## LM Studio Adapter

### Recommended Adapter Mode

- dev/local validation adapter
- split-mode between OpenAI-compatible `/v1/responses` and native local APIs

### Responses Reuse Evaluation

- Surface type: OpenAI-compatible `Responses` endpoint plus native local REST/chat surfaces
- What Logos should reuse: OpenAI-compatible `Responses` path for local validation of continuation and tool-loop behavior; native mode for local-only experiments when needed
- Advantages: local validation of `previous_response_id`, useful MCP experimentation, lower cloud dependency during prototyping
- Disadvantages: behavior still depends on local model/runtime specifics, release churn can affect compatibility, should not become the semantic baseline for cloud providers
- Decision: use `Responses` for dev validation value, not as a canonical architecture source

### Notes

- LM Studio should stay explicitly non-canonical even when it is highly useful for offline testing.

## Current Priority Order

1. OpenAI
2. OpenRouter
3. Gemini
4. LM Studio
5. Z.AI
6. Anthropic

## Why This Order

- The first two providers prove the kernel boundary against canonical stateful and strict stateless modes.
- Gemini is commercially important and exposes mixed compat/native pressure early.
- LM Studio helps prototype continuation and tool behavior locally.
- Z.AI and Anthropic remain important, but their deeper native semantics benefit from a stronger kernel first.

## Open Questions

- whether Gemini should ship first in compat mode, native mode, or a split milestone
- how much Z.AI MCP server calling belongs in Logos 0.1
- whether LM Studio local continuation is close enough to be a stable regression target
- when Anthropic should move from profile status into a narrow implementation spec

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `MVP Required Providers And Adapters`
  lines: `42-55`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Executive Summary`
  lines: `36-47`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `128-143`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Recommended Decisions for Logos 0.1`
  lines: `181-188`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `30-45`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `99-107`
- path: `../research/2026-03-30-research-pack-audit.md`
  symbol: `Highest-Risk Gaps Still Open`
  lines: `43-49`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Parity Decision For LOGOS`
  lines: `270-330`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-project-docs-spec.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[../research/2026-03-30-logos-provider-research-report.md]]
- [[../research/2026-03-30-provider-comparison-matrix.md]]
