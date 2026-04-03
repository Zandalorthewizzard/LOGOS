---
id: logos-semantics-and-guarantees-matrix
title: LOGOS Semantics And Guarantees Matrix
type: spec
status: active
tags: [architecture, providers, semantics, guarantees, matrix, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-multi-provider-feature-gap-matrix.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
depends_on:
  [
    [2026-04-02-logos-runtime-contract-spec.md],
    [2026-04-02-logos-capability-model-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Semantics And Guarantees Matrix

## Purpose

- Add the missing second layer above the capability matrix.
- Distinguish feature presence from semantic shape and guarantee strength.
- Prevent adapter design from rotting under labels like `partial` that hide major behavioral differences.

## Why This Exists

- A provider may "support streaming" while exposing a very different stream ontology.
- A provider may "support tools" while shifting who actually controls the tool loop.
- A provider may "support continuation" while offering only replay, caching, or mixed carryover rather than a canonical durable handle.

## Responses Reuse Evaluation

### What This Matrix Reuses From Responses

- OpenAI `Responses` provides the canonical semantic reference for event-oriented streaming, turn continuation, tool-call framing, and completion semantics.

### Advantages

- makes semantic distance from the canonical path visible
- avoids flattening all provider behavior into capability checkboxes
- helps decide where fallback or downgrade is architecture-safe

### Disadvantages And Limits

- semantics and guarantees are partly interpretive unless backed by prototypes
- some rows remain intentionally conservative where evidence is incomplete

### Decision

- Use this matrix to complement, not replace, the capability matrix.
- Treat uncertain rows conservatively until prototype-backed.

## Rating Legend

- `canonical`: aligns closely with the OpenAI/Codex baseline for Logos
- `compatible`: close enough to reuse substantial runtime behavior, but not identical
- `translated`: Logos can normalize it, but only by explicit mapping across different semantics
- `emulated`: Logos can offer a runtime substitute, not real provider equivalence
- `unknown`: evidence is insufficient

## Matrix

| Surface | OpenAI | OpenRouter | Gemini | Anthropic | Z.AI | LM Studio | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Streaming semantics | `canonical` | `compatible` | `translated` | `translated` | `translated` | `compatible` | OpenRouter and LM Studio are closer on compat path; native providers expose different event ontologies |
| Streaming guarantees | `canonical` | `compatible` | `translated` | `translated` | `translated` | `compatible` | Ordering and event completeness need provider/mode-specific care outside OpenAI |
| Tool-loop control model | `canonical` | `compatible` | `translated` | `translated` | `translated` | `compatible` | Runtime-owned baseline remains mandatory, but provider-native semantics differ sharply |
| Tool-loop determinism | `canonical` | `compatible` | `translated` | `translated` | `translated` | `translated` | Determinism is strongest where Logos owns more of the loop and provider semantics are simpler |
| Continuation semantics | `canonical` | `emulated` | `translated` | `emulated` | `translated` | `translated` | OpenRouter is replay-first; Anthropic is history/caching-oriented; mixed-mode providers vary by mode |
| Resume guarantees after restart | `canonical` | `emulated` | `translated` | `translated` | `translated` | `translated` | Depends heavily on Logos journal quality outside OpenAI |
| Structured output semantics | `canonical` | `compatible` | `translated` | `translated` | `translated` | `translated` | Capability exists broadly, but strictness and refusal semantics differ |
| Reasoning/thinking carryover | `canonical` | `unknown` | `translated` | `translated` | `translated` | `translated` | Current pack does not justify stronger OpenRouter claim |
| MCP/connectors semantics | `canonical` | `translated` | `translated` | `translated` | `translated` | `translated` | Provider-native MCP or connector stories are not identical to OpenAI connectors |

## Interpretation

### Streaming

- "supports streaming" is not enough.
- The important distinction is whether the provider emits a stream ontology Logos can treat as event-native, compat-shaped, or only translation-shaped.

### Tool Loop

- The key question is not only whether tools exist.
- The key questions are:
  - who appears to control the loop semantically,
  - how much Logos can keep runtime-owned,
  - and how deterministic the boundary remains.

### Continuation

- Continuation is backbone runtime behavior, not just another feature row.
- Non-OpenAI providers are generally not canonical here and often require translation or emulation through replay and journal logic.

## Conservative Claims Only

- This matrix intentionally avoids stronger statements like "reliability medium" or "determinism weak" unless the research pack or prototypes justify them directly.
- Where evidence is not solid, Logos should prefer `translated`, `emulated`, or `unknown` rather than overclaiming.

## Open Questions

- whether this matrix should later split guarantees into separate dimensions such as ordering, completeness, and determinism
- whether mixed-mode providers should get separate native and compat rows in the next revision
- whether a dedicated reasoning guarantees matrix becomes necessary after prototypes

## Evidence

- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `128-143`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Core Decision`
  lines: `33-37`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Resume Modes`
  lines: `59-76`
- path: `C:\Users\zand\Desktop\analysis.txt`
  symbol: `4.1 Введи второй слой матрицы`
  lines: `260-276`

## Links

- [[README.md]]
- [[2026-04-02-logos-runtime-contract-spec.md]]
- [[2026-04-02-logos-multi-provider-feature-gap-matrix.md]]
