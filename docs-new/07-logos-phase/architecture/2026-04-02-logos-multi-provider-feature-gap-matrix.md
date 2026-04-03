---
id: logos-multi-provider-feature-gap-matrix
title: LOGOS Multi-Provider Feature Gap Matrix
type: spec
status: active
tags: [architecture, providers, capabilities, matrix, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-semantics-and-guarantees-matrix.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md]
  ]
depends_on:
  [
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-semantics-and-guarantees-matrix.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [2026-04-02-logos-provider-adapter-profiles.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Multi-Provider Feature Gap Matrix

## Purpose

- Give one decision-grade view of what non-OpenAI providers may lack relative to the canonical OpenAI/Codex path.
- Prevent accidental promises of universal Codex parity.
- Help product and implementation planning decide where to use full support, partial support, fallback, or explicit non-support.
- This matrix now sits below the runtime contract and the semantics/guarantees matrix rather than carrying all meaning alone.

## Rating Legend

- `full`: target is close to canonical OpenAI behavior for Logos purposes
- `partial`: feature exists but with meaningful semantic or operational limits
- `fallback`: Logos can offer a substitute path, but not true provider parity
- `no`: not expected as a first-class capability in current planning
- `unknown`: current evidence is not strong enough

## Responses Reuse Evaluation

### What This Matrix Reuses From Responses

- OpenAI `Responses` is the baseline used to define the strongest known agent-runtime feature envelope.
- The matrix uses that baseline to measure where other providers are equivalent, partial, fallback-only, or non-equivalent.

### Advantages

- gives one canonical comparison target
- keeps OpenAI parity measurable instead of vague
- makes non-OpenAI limitations explicit before coding

### Disadvantages And Limits

- the baseline is necessarily OpenAI-shaped
- some provider-native strengths do not fit neatly into “gap from OpenAI” framing
- a lower score can still hide useful provider-native advantages outside the OpenAI envelope

### Decision

- Use the OpenAI `Responses` path as the comparison baseline.
- Treat lower ratings as parity gaps, not as claims that a provider is generally weaker in all dimensions.
- Use this matrix for capability-envelope planning, not as the sole semantic truth source.

## Relationship To Other Docs

- `2026-04-02-logos-runtime-contract-spec.md` defines what Logos aims to guarantee.
- `2026-04-02-logos-semantics-and-guarantees-matrix.md` explains how provider semantics differ even when a capability row looks similar.
- This document answers a narrower question: what likely ends up available, partial, emulated, or absent in product/runtime terms.

## Matrix

| Capability | OpenAI | OpenRouter | Gemini | Anthropic | Z.AI | LM Studio | Notes for Logos |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Stateful server-managed continuation | `full` | `no` | `partial` | `fallback` | `partial` | `partial` | OpenRouter is explicitly stateless; Gemini/Z.AI/LM Studio depend on mode; Anthropic uses different carryover patterns |
| Client-managed replay | `fallback` | `full` | `partial` | `full` | `partial` | `partial` | Replay is primary for OpenRouter and likely important for Anthropic/native flows |
| Typed streaming parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | All can stream, but event taxonomy and guarantees differ |
| Runtime-owned tool loop support | `full` | `full` | `partial` | `full` | `partial` | `partial` | Support means Logos can preserve the runtime-owned baseline; it does not mean identical semantic control models |
| Provider-native built-in tools parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | OpenAI hosted tools remain canonical; others are non-equivalent or mode-specific |
| MCP/connectors parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | Semantics differ sharply between runtime-owned MCP, provider-native MCP, and connectors |
| Streamed tool arguments | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | Capability should remain model/mode-gated |
| Structured outputs parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | Most providers support something here, but strictness and refusal behavior may differ |
| Reasoning/thinking parity | `full` | `unknown` | `partial` | `partial` | `partial` | `partial` | OpenRouter reasoning carryover remains unresolved; others have native semantics, not OpenAI parity |
| Durable resume after restart | `full` | `fallback` | `partial` | `partial` | `partial` | `partial` | Depends on Logos-native journal plus provider mode |
| Approval-flow parity | `full` | `full` | `full` | `full` | `full` | `full` | This is primarily Logos-owned control flow, though provider-native shortcuts may still be gated or disabled |
| Multimodal parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | Highly model/mode dependent across all non-OpenAI providers |
| Error/limit normalization parity | `full` | `partial` | `partial` | `partial` | `partial` | `partial` | Normalization is possible, but exact semantics differ |

## Provider-Specific Likely Gaps

### OpenRouter

- no true server-managed continuation
- no canonical OpenAI hosted-tool parity
- reasoning token carryover semantics still `unknown`
- model heterogeneity behind the facade can produce parameter drift

### Gemini

- no guaranteed OpenAI-equivalent semantics even in compat mode
- native thinking/tool behavior not fully representable through plain `Responses` compat
- exact hosted-tool and structured-output parity still not locked

### Anthropic

- no true `Responses` path
- no OpenAI-style continuation handle
- no reason to expect hosted-tool or connector parity as a native guarantee
- error normalization details are still less mature in current pack

### Z.AI

- native MCP server calling is not the same as OpenAI connectors or hosted tools
- context caching is not equivalent to OpenAI continuation
- JS/TS SDK status remains `unknown`

### LM Studio

- local runtime behavior is not a semantic authority for cloud-provider parity
- compatibility can move with releases and local-model behavior
- native local APIs and compat mode must stay separate

## Product Interpretation Rules

- `full` on a non-OpenAI provider still does not imply byte-for-byte wire compatibility.
- `partial` means the feature may be exposed, but only with capability checks and explicit user-facing limitations where needed.
- `fallback` means Logos can preserve user experience partly through runtime behavior rather than provider equivalence.
- `unknown` must not be silently treated as `full` or `partial` in implementation specs.
- A `full` or `partial` row here does not settle the semantic-shape or guarantee-strength question by itself.
- A non-OpenAI provider can still be competitive with OpenCode- or Goose-like UX without achieving canonical OpenAI semantic parity.

## Open Questions

- whether reasoning/thinking needs to be split into more than one row for product planning
- whether provider-native MCP should be split apart from runtime-owned MCP in a future version of the matrix
- whether LM Studio deserves separate rows for compat and native mode in later revisions

## Evidence

- path: `2026-04-02-logos-capability-model-architecture-spec.md`
  symbol: `Mandatory Capability Families`
  lines: `101-112`
- path: `2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `OpenRouter Validation Adapter`
  lines: `54-73`
- path: `2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `Gemini Adapter`
  lines: `75-94`
- path: `2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `Anthropic Adapter`
  lines: `96-114`
- path: `2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `Z.AI Adapter`
  lines: `116-135`
- path: `2026-04-02-logos-provider-adapter-profiles.md`
  symbol: `LM Studio Adapter`
  lines: `137-154`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `30-45`

## Links

- [[README.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-semantics-and-guarantees-matrix.md]]
- [[2026-04-02-logos-capability-model-architecture-spec.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
