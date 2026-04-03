---
id: logos-adapter-kernel-architecture
title: LOGOS Adapter Kernel Architecture
type: spec
status: active
tags: [architecture, adapters, runtime, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-adapter-boundary-spec.md],
    [2026-03-30-logos-runtime-types-v0.md],
    [2026-04-02-logos-adapter-project-docs-spec.md],
    [2026-04-02-logos-provider-adapter-profiles.md],
    [../implementation/2026-03-30-logos-global-roadmap.md]
  ]
depends_on:
  [
    [2026-03-30-logos-adapter-boundary-spec.md],
    [2026-03-30-logos-runtime-types-v0.md],
    [../research/2026-03-30-logos-provider-research-report.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Adapter Kernel Architecture

## Intent

- Refine the adapter-first direction into a tighter kernel architecture that can survive multiple providers without weakening the canonical OpenAI path.
- Improve the design on paper before implementation begins.
- Minimize future architecture regret around continuation, tool ownership, and provider-native feature exposure.

## Kernel Boundary

The shared Logos kernel should be only this:

1. `logos-runtime-types`
2. `logos-provider-api`
3. `logos-session-engine`
4. `logos-tool-runtime`
5. durable journal contract for resume and replay

Everything else should stay provider-owned, provider-mapped, or provider-extension-only.

## Shared Kernel Responsibilities

### Session Engine

- own turn lifecycle
- own retry, cancellation, and timeout policy
- own continuation policy resolution
- own tool dispatch policy
- consume only normalized internal events

### Provider Adapter

- map provider request shape into `InternalTurnRequest`
- map provider stream and terminal events into `InternalEvent`
- publish capability differences explicitly
- preserve provider-native extensions without making them part of kernel control flow

### Tool Runtime

- stay runtime-owned by default
- preserve stable call IDs
- return tool results as `InternalMessage::ToolResult`
- support provider-native tools only as explicit optional fast paths

### Durable Journal

- persist Logos-native messages, tool actions, ordering markers, approvals, and continuation handles
- never depend on raw provider wire objects as the canonical stored model

## Design Rules

### Rule 1: Canonical Is Not Universal

- OpenAI `Responses` remains the reference upstream protocol.
- The kernel must not make raw `Responses` structs the internal ontology.

### Rule 2: Continuation Is A Policy Surface

- The session engine must treat continuation as capability-driven policy, not as one hard-coded `previous_response_id` assumption.
- Server-managed, client-managed, and hybrid continuation must remain explicit.

### Rule 3: Tooling Must Stay Honest

- Runtime-owned tools are the portable baseline.
- Provider-native tools may exist, but the kernel must not require them to complete a normal tool loop.

### Rule 4: Extension Channels Must Be Bounded

- `ProviderExtension` is the supported escape hatch.
- `RawProviderEvent` is only for observability, compatibility work, and debugging.
- Kernel control flow must not silently drift onto raw provider payloads.

## Provider Classes

The kernel should classify adapters into provider classes instead of treating all of them as one shape.

### Class A: Canonical Stateful Responses

- Example: OpenAI
- Characteristics:
  - stateful continuation is first-class
  - typed streaming is rich
  - built-in tools and MCP/connectors exist
  - highest fidelity path for Codex-derived behavior

### Class B: Responses-Compatible Validation Adapter

- Example: OpenRouter
- Characteristics:
  - OpenAI-like facade
  - useful for boundary validation
  - not safe as semantic source of truth
  - requires client-owned history and stronger fallback discipline

### Class C: Mixed Compat Plus Native Provider

- Examples: Gemini, Z.AI, LM Studio
- Characteristics:
  - some OpenAI-compatible entrypoints exist
  - important provider-native semantics remain outside compat mode
  - adapter strategy must choose compat, native, or split-mode explicitly

### Class D: Native Non-Responses Provider

- Example: Anthropic
- Characteristics:
  - no real `Responses` contract to rely on
  - message/content block and tool semantics need native mapping
  - should not be forced into OpenAI wire assumptions

## Responses Reuse Evaluation

### What The Kernel Reuses From Responses

The shared architecture intentionally reuses these ideas from the canonical OpenAI path:

- turn-oriented request/stream lifecycle
- typed stream events instead of plain text streaming
- explicit continuation handle semantics
- tool-call event framing with stable call IDs
- schema-aware output controls

### Advantages Of Reusing Responses Concepts

- preserves the strongest proven Codex path
- reduces phase-1 blast radius
- gives Logos a high-fidelity reference protocol for stream, tool, and continuation behavior
- helps validate whether normalized types are honest enough

### Disadvantages And Risks

- third-party providers can look more compatible than they really are
- `Responses` vocabulary can bias the kernel toward OpenAI assumptions
- provider-native orchestration features can be hidden or under-modeled if compat mode is overused
- stateful OpenAI semantics can leak into session policy unless continuation remains capability-driven

### Kernel Decision

- Reuse `Responses` concepts as the canonical reference model for the OpenAI adapter.
- Do not require third-party adapters to implement a fake full `Responses` contract.
- Use `Responses`-shaped compat surfaces only when they simplify integration without lying about semantics.

## Optimization Targets Before Coding

These are the parts that should be improved on paper before implementation starts.

### 1. Continuation Modes

- Lock exact meanings for `ServerManaged`, `ClientManaged`, and `Hybrid`.
- Define legal fallback transitions between them.

### 2. Tool Ownership

- Lock which built-in/provider-native tools can bypass runtime-owned tools, if any.
- Require explicit policy flags for every such bypass.

### 3. Durable Journal Contract

- Define the minimum event/message record set that resumes OpenAI faithfully and replays stateless adapters honestly.

### 4. Capability Partition

- Keep provider-level and model-level capabilities split.
- Prevent the session engine from inferring unsupported features from the presence of compat endpoints.

## Current Best Shape

At this stage the best architecture shape is:

- OpenAI as canonical adapter
- OpenRouter as validation adapter
- Gemini and Z.AI as mixed-mode providers that need explicit compat/native decisions
- Anthropic as native-first mapping work
- LM Studio as dev/local validation target, not canonical truth

## Open Questions

- exact durable journal schema for mixed continuation modes
- final policy for provider-native tools in Logos 0.1
- whether `ReasoningDelta` and extension channels are sufficient for all early providers
- how much of LM Studio native state should be modeled versus treated as dev-only

## Evidence

- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Locked Decisions`
  lines: `23-40`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Invariants Required For Session, Resume, and Tool Loop`
  lines: `216-246`
- path: `2026-03-30-logos-runtime-types-v0.md`
  symbol: `v0 Rust API`
  lines: `33-234`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Executive Summary`
  lines: `36-47`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Persistence / Resume Findings`
  lines: `144-166`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Tool-Loop Findings`
  lines: `167-179`
- path: `../research/2026-03-30-provider-comparison-matrix.md`
  symbol: `Matrix`
  lines: `28-45`

## Links

- [[README.md]]
- [[2026-03-30-logos-adapter-boundary-spec.md]]
- [[2026-03-30-logos-runtime-types-v0.md]]
- [[2026-04-02-logos-adapter-project-docs-spec.md]]
- [[2026-04-02-logos-provider-adapter-profiles.md]]
