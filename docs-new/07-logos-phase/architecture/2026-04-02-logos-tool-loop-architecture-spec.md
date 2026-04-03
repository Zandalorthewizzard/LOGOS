---
id: logos-tool-loop-architecture-spec
title: LOGOS Tool Loop Architecture Spec
type: spec
status: active
tags: [architecture, tools, mcp, approvals, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-logos-codex-evidence-appendix.md]
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

# LOGOS Tool Loop Architecture Spec

## Purpose

- Lock the portable tool-loop architecture before implementation begins.
- Preserve Codex strengths in tool routing, approvals, sandboxing, and MCP correlation.
- Prevent provider-native tools from quietly replacing the safe runtime-owned baseline.

## Core Decision

- The default Logos tool path is runtime-owned.
- Provider-native tools are optional, explicit, capability-gated fast paths.
- The adapter boundary normalizes tool-call lifecycle events, but does not erase provider-specific tool semantics when they matter.

## Portable Tool Loop

The portable baseline tool loop is:

1. provider emits normalized tool-call events
2. session engine preserves call ordering and approval flow
3. tool runtime dispatches executable action
4. result returns as `InternalMessage::ToolResult`
5. next model turn continues from normalized history

## Mandatory Invariants

### Stable Call Identity

- Every tool call must have a stable call ID across stream, approval, execution, reinjection, and persistence.

### Ordering

- Tool argument deltas must remain ordered for a given call ID.
- Terminal events must remain explicit.
- Tool result reinjection must occur against the intended call, not merely by tool name.

### Approval Separation

- Approval requests are distinct control-flow objects, not just tool metadata.
- Approval outcomes must remain explicit in the session loop and durable journal.

### MCP Correlation

- MCP `tools/call` roundtrip correlation must survive normalization.
- Continuation-critical identifiers such as thread correlation must not be lost.

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- typed tool-call event framing
- streamed tool arguments pattern
- distinction between model output and tool result reinjection
- lesson that built-in tools can exist but should not define the whole runtime

### Advantages Of Reusing Responses Concepts

- preserves the most mature known tool-loop reference in the current pack
- lowers translation cost for the canonical OpenAI path
- gives Logos a concrete baseline for event framing and reinjection semantics

### Disadvantages And Limits

- provider-native tool ecosystems are not the same as OpenAI hosted tools
- over-modeling tools after `Responses` can hide distinct MCP or provider-managed behavior
- a shared runtime designed too tightly around OpenAI hosted tools would become dishonest for native adapters

### Decision

- Reuse `Responses` tool-loop concepts where they improve the portable baseline.
- Keep provider-native tools behind explicit capability and policy gates.
- Do not let hosted-tool semantics become the universal Logos tool ontology.

## Tool Classes

### Class A: Runtime-Owned Tools

- default path
- portable baseline
- approval and sandbox policy owned by Logos runtime

### Class B: Provider-Native Tools

- explicit fast path
- provider capability dependent
- never the only way to complete a normal safe tool loop

### Class C: MCP-Backed Runtime Tools

- runtime-owned orchestration using MCP transport and correlation rules
- portable when the runtime owns the interaction

## Provider-Native Tool Policy

- Provider-native tool exposure must be explicit per provider.
- If a provider-native tool bypasses runtime execution, the architecture must document why that bypass is acceptable.
- The kernel must still have a portable fallback when product scope requires it.

## Capability Requirements

- tool calling support must be modeled separately from provider-native tool support
- streamed tool argument support must be a separate capability from simple tool calling
- parallel tool support must be capability-gated, not assumed

## Non-goals

- no attempt to normalize every provider-native tool into one fake shape
- no replacement of approval logic with provider convenience features
- no loss of MCP correlation or thread identity through normalization

## Acceptance Gates

1. One normalized tool-call roundtrip works through the OpenAI path.
2. Stable call IDs survive stream, approval, execution, and persistence.
3. Provider-native tools remain capability-gated and optional.
4. MCP correlation-critical identifiers remain recoverable after normalization.

## Open Questions

- which provider-native tools, if any, deserve first-class Logos exposure in 0.1
- how much parallel-tool behavior should be part of the first portable baseline
- whether some provider-native tool classes require richer extension payloads than current runtime types provide

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 3: Tool Loop Boundary`
  lines: `125-142`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Tool-Loop Findings`
  lines: `167-179`
- path: `2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Tool Loop Invariants`
  lines: `234-246`
- path: `../research/2026-03-30-logos-codex-evidence-appendix.md`
  symbol: `Code-backed Evidence Blocks`
  lines: `22-120`

## Links

- [[README.md]]
- [[2026-04-02-logos-adapter-kernel-architecture.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
