---
id: logos-provider-openai-adapter-spec
title: LOGOS OpenAI Adapter Architecture Spec
type: spec
status: active
tags: [architecture, adapters, openai, responses, logos]
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

# LOGOS OpenAI Adapter Architecture Spec

## Purpose

- Define the canonical provider adapter for Logos.
- Preserve Codex/OpenAI fidelity while moving it behind the Logos boundary.
- Establish the reference behavior all other adapters are measured against.

## Adapter Role

- canonical adapter
- stateful reference implementation
- wrapper over existing Codex/OpenAI path, not a semantic rewrite

## Why OpenAI Is Different

- It is the only provider in the current pack whose `Responses` surface is both canonical and stateful-by-design.
- It is the source path that already carries the richest combination of streaming, continuation, built-in tools, structured outputs, and MCP/connectors.
- If this adapter loses fidelity, the whole adapter-first strategy loses its anchor.

## Responses Reuse Evaluation

### Surface Type

- Real canonical `Responses` protocol.

### What Logos Reuses

- request/turn lifecycle
- typed streaming event model
- `previous_response_id`
- built-in tools and provider-native tool surfaces
- structured outputs controls
- reasoning-oriented surfaces
- MCP/connectors path

### Advantages

- maximum parity with Codex behavior
- lowest semantic translation loss
- strongest reference path for continuation, stream ordering, and tool-loop design
- fastest path to phase-1 proof without destructive refactor

### Disadvantages

- can over-shape shared abstractions if the adapter is not kept behind a narrow boundary
- encourages accidental dependence on raw OpenAI wire types
- vendor-specific features such as built-in tools and compaction can distort the shared runtime if promoted too early

### Decision

- Use `Responses` heavily.
- Keep OpenAI as the canonical upstream path.
- Reuse as much existing Codex/OpenAI machinery as possible, but keep raw wire types out of shared runtime crates.

## Required Adapter Behavior

### Request Mapping

- `InternalTurnRequest` must map to the existing OpenAI/Codex request path with minimal semantic loss.
- OpenAI-only request features may remain adapter-owned rather than shared-runtime fields.
- The adapter may emit `ProviderExtension` for meaningful OpenAI-specific behavior rather than widening the kernel prematurely.

### Streaming

- The adapter must preserve typed stream semantics and ordering.
- Text deltas, tool-call framing, reasoning deltas, usage events, completion, and failure must remain explicit.
- The adapter must not degrade the stream into a plain text pipe.

### Continuation And Resume

- `previous_response_id` is the canonical continuation handle.
- The adapter must support stateful follow-up turns without forcing client-managed history as the normal path.
- If Logos later supports alternative OpenAI continuation modes, those remain adapter-owned policies layered over the canonical path.

### Tools And MCP

- Provider-native OpenAI built-in tools remain allowed as explicit fast paths.
- Runtime-owned tools remain the portable baseline and must still work through the normalized seam.
- MCP/connectors behavior must not be erased merely to simplify the abstraction.

## Mapping Boundaries

### Shared Runtime Must See

- normalized messages
- normalized stream events
- continuation handle updates
- provider and model capabilities
- normalized provider errors

### Shared Runtime Must Not See As Canonical Truth

- raw `ResponseItem`
- raw `ResponsesApiRequest`
- raw OpenAI SSE or websocket event enums
- OpenAI-only request details that have no honest cross-provider meaning

## Capability Expectations

The OpenAI adapter is the baseline provider expected to support, at minimum:

- server-managed continuation
- structured outputs
- tool calling
- streamed tool arguments
- reasoning-oriented surfaces
- provider-native built-in tools
- MCP/connectors path

## Non-goals

- no redesign of OpenAI semantics to look provider-neutral
- no early promotion of every OpenAI field into shared runtime types
- no weakening of upstream sync by rewriting stable Codex behavior unnecessarily

## Acceptance Gates

1. One text-only turn passes through the adapter boundary without semantic regression.
2. One stateful follow-up turn works using persisted continuation handle.
3. One tool-call roundtrip passes through normalized events while preserving stable call IDs.
4. Shared runtime crates do not export raw OpenAI wire types.

## Open Questions

- exact boundary for OpenAI provider-native tool exposure in Logos 0.1
- whether OpenAI compaction remains fully adapter-owned in early phases
- how much OpenAI-specific reasoning payload should be preserved via `ProviderExtension`

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `MVP Required Providers And Adapters`
  lines: `42-49`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Hard Rule 1`
  lines: `25-29`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Current Codex Modules That Stay As-Is`
  lines: `48-87`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Provider Semantics Matrix`
  lines: `128-133`
- path: `../research/2026-03-30-preliminary-provider-research-scan.md`
  symbol: `Провайдер/режим`
  lines: `101-101`
- path: `../../input-sources-temp/provider-source-inventory.md`
  symbol: `OpenAI`
  lines: `31-47`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
