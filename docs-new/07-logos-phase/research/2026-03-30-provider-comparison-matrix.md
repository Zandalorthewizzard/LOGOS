---
id: logos-provider-comparison-matrix
title: LOGOS Provider Comparison Matrix
type: research
status: active
tags: [research, matrix, providers, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-provider-research-report.md],
    [2026-03-30-logos-codex-evidence-appendix.md],
    [2026-03-30-logos-research-requirements.md]
  ]
depends_on: [[2026-03-30-logos-provider-research-report.md]]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Provider Comparison Matrix

## Purpose

This matrix is a compact working view of provider semantics relevant to Logos 0.1.
It is intentionally narrower than the full research report and is meant to help
implementation planning without flattening provider-specific behavior.

## Matrix

| Provider / mode | State model | Streaming | Tool calling | Structured outputs | Reasoning / thinking | Continuation / resume | Built-in / native tools | Error / limits | Notes for Logos |
|---|---|---|---|---|---|---|---|---|---|
| OpenAI Responses | Stateful-by-design | Native event streaming | Built-in tools + MCP/connectors | Yes | Responses-oriented reasoning surfaces | `previous_response_id` and stateful continuation | Yes | Separate error/rate-limit docs | Canonical baseline |
| OpenRouter Responses | Stateless facade | Streaming docs captured | Tool calling docs captured | Yes | Reasoning docs exist, but tokens semantics remain `UNKNOWN` | Client-owned journal required | Mostly facade behavior | Dedicated errors/limits docs | Validation adapter, not truth source |
| Gemini native | Mixed / provider-native | Native streaming docs captured | Native function calling | Yes | Thinking docs captured | Native continuation differs from OpenAI | Native features vary by mode | Separate native docs | Keep native and compat modes separate |
| Gemini OpenAI compat | Compatibility layer | Compatibility streaming | Compatibility tool calling | Yes | Thinking may be gated via provider-specific fields | Not identical to native Gemini | Limited by compat surface | Compat-specific error handling | Treat as adapter mode |
| Z.AI native | Mixed / provider-native | Stream-tool docs captured | Function calling + MCP calling | Yes | Thinking mode / deep thinking docs captured | Context caching docs captured | Native MCP server calling | Rate-limit docs captured | Strong provider-native semantics |
| Z.AI OpenAI compat | Compatibility layer | Compatibility streaming | Compatibility tool calling | Yes | Provider-specific thinking exposure uncertain | Compatibility continuation uncertain | Compatibility-limited | Rate-limit docs captured | Keep separate from native mode |
| LM Studio OpenAI compat | Stateful in docs | Responses streaming docs captured | OpenAI-compatible tools | Yes | Local-model dependent | `previous_response_id` support captured | MCP docs captured | Endpoint comparison docs | Useful local validation target |
| LM Studio native REST/chat | Native local mode | Native REST/docs captured | Native MCP docs captured | Mode-dependent | Model-dependent | Native stateful chat docs captured | Core + app MCP docs | REST/docs captured | Best for offline prototyping |

## Open Questions

- OpenRouter reasoning token semantics
- Verified Z.AI JS/TS SDK repository status, which currently appears absent from public official docs
- Minimal persistence schema remains Logos design work rather than a public canonical artifact

## Suggested Use

Use this matrix as a checklist when deciding whether a provider capability should be:

1. implemented directly in Logos runtime,
2. preserved as a provider-specific adapter feature, or
3. deferred to a later phase.
