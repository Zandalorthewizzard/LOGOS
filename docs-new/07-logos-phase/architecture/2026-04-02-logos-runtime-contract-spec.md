---
id: logos-runtime-contract-spec
title: LOGOS Runtime Contract Spec
type: spec
status: active
tags: [architecture, runtime, guarantees, contract, logos]
related:
  [
    [README.md],
    [2026-04-02-logos-runtime-tier-contract-spec.md],
    [2026-04-02-logos-tier-0-semantic-surfaces-spec.md],
    [2026-04-02-logos-adapter-health-and-drift-policy.md],
    [2026-04-02-logos-minimal-survivable-agent-baseline-spec.md],
    [2026-04-02-logos-capability-model-architecture-spec.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md]
  ]
depends_on:
  [
    [2026-04-02-logos-adapter-kernel-architecture.md],
    [2026-04-02-logos-tool-loop-architecture-spec.md],
    [2026-04-02-logos-persistence-resume-architecture-spec.md],
    [2026-04-02-logos-capability-model-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Runtime Contract Spec

## Purpose

- Define the runtime contract Logos intends to guarantee before implementation starts.
- Separate provider support from runtime authority.
- Make downgrade behavior explicit instead of leaving weak-provider behavior undefined.

## Core Position

- Logos is not a provider router and not a thin API wrapper.
- Logos is a runtime normalizer with an OpenAI-canonical center of gravity.
- Provider adapters are semantic translators, downgrade boundaries, and drift observation points.
- The goal is runtime sovereignty, not fake uniformity.

## Responses Reuse Evaluation

### What This Spec Reuses From Responses

- OpenAI `Responses` is the canonical reference for the strongest current agent-runtime envelope.
- It anchors the baseline for continuation, event-oriented streaming, tool-call framing, and structured turn flow.

### Advantages

- gives Logos one stable comparison target
- keeps OpenAI parity measurable
- lets other providers be modeled as native, compat, translated, or degraded without pretending they are identical

### Disadvantages And Limits

- the top-level contract is intentionally OpenAI-shaped
- some provider-native strengths do not fit neatly into that baseline
- over-emulation would create false parity claims

### Decision

- Keep OpenAI `Responses` as the canonical benchmark.
- Guarantee Logos-owned runtime behavior where possible.
- Emulate, degrade, or reject explicitly when the provider cannot honestly support the canonical envelope.

## Adapter Architecture Boundary

### Adapter Owns

- provider transport binding
- auth, retry, timeout, and connection handling
- raw event ingestion and decoding
- request mapping into provider format
- provider event to `InternalEvent` normalization
- provider error to `ProviderError` normalization
- declared capability report
- observed capability hooks
- provider-specific fallback hooks
- drift watchpoints

### Logos Runtime Owns

- execution journal
- replay logic
- durable resume orchestration
- approvals and human gating
- tool execution and tool policy
- structured-output validation and repair
- downgrade policy
- planning semantics
- runtime telemetry
- feature gating based on capability and tier state

### Adapter Must Not Own

- approval policy
- canonical planning logic
- durable execution authority
- silent provider-specific behavior in the common runtime path
- silent bypass of runtime-owned tools
- silent elevation of runtime tier

## Logos Runtime Guarantees

These are the guarantees Logos should target as runtime behavior, subject to explicit provider capability limits.

### 1. Normalized Event Stream

- Host-visible runtime flow remains event-oriented.
- Unknown provider events are surfaced, not dropped.

### 2. Runtime-Owned Tool Policy

- The portable default tool path is runtime-owned.
- Provider-native tools are optional, isolated, and capability-gated.

### 3. Explicit Approval Control Flow

- Approval state is first-class, durable, and auditable.
- Tool suggestions are advisory until accepted by Logos runtime.

### 4. Resumable Execution

- Logos maintains a durable journal sufficient for resume or replay according to provider mode.
- This does not imply universal server-managed continuation.

### 5. Capability-Driven Behavior

- Runtime decisions come from declared and observed capability state plus policy, not hidden provider-name branching.

### 6. Explicit Downgrade Semantics

- Missing semantics lead to emulation, constrained operation, or explicit non-support.
- Weak providers must degrade into lower tiers rather than undefined behavior.

## What Logos Does Not Guarantee Universally

- universal OpenAI-level server-managed continuation
- universal hosted-tool parity
- universal identical reasoning semantics
- universal identical streaming ontology
- universal identical provider-native MCP or connector behavior
- universal byte-level wire parity across providers

## Adapter Classes

### Type A: Compat Adapters

- Examples: OpenRouter, LM Studio compat mode
- Main value: bounded compatibility surface and validation value
- Main risk: false semantic confidence

### Type B: Native Adapters

- Examples: Anthropic, Gemini native, Z.AI native
- Main value: honest preservation of provider-native semantics
- Main risk: larger semantic divergence and mapping cost

### Type C: Mixed-Mode Adapters

- Examples: Gemini, Z.AI, LM Studio overall provider story
- Main value: support more than one operational mode honestly
- Main risk: hidden mode drift if mode is not explicit and persisted

## Product Positioning Rule

- OpenAI targets canonical Logos agent behavior.
- Non-OpenAI providers target the strongest honest runtime experience available under their capability envelope.
- Product messaging must distinguish canonical parity from capability-constrained support.
- Competitive parity with tools like OpenCode and Goose is a valid target for non-canonical providers.
- Canonical OpenAI `Responses` semantic parity remains exclusive to the OpenAI path unless another provider demonstrably exposes the same semantics.

## Competitive Parity Rule

- `Competitive parity` means user-visible runtime UX and primary capability surface can remain competitive on non-canonical providers.
- `Canonical parity` means invisible OpenAI `Responses` semantic parity.
- Logos may target competitive parity broadly.
- Logos must reserve canonical parity claims for the OpenAI path or any future provider proven equivalent at the semantic level.

## Invariants

- unknown provider events must never be silently dropped
- missing continuation transfers state authority to Logos journal and replay
- provider-native tools must never silently bypass approvals or tool policy
- structured outputs are untrusted until accepted by Logos validation policy
- provider extensions remain opt-in and capability-gated
- observed runtime degradation can override static claims
- durable state authority must remain visible and auditable

## Non-goals

- no promise that all providers behave like OpenAI
- no requirement to emulate every missing feature
- no replacement of provider-native strengths with lowest-common-denominator abstractions

## Acceptance Gates

1. The runtime contract can be stated without promising universal OpenAI parity.
2. Each adapter can be described as canonical, compat, native, or mixed-mode against this contract.
3. Missing provider behavior maps cleanly to emulate, constrained, or unsupported outcomes.
4. Implementation specs can cite this contract as the runtime baseline.

## Open Questions

- whether product UX should expose canonical versus constrained provider classes directly
- which fallback behaviors are worth promising in 0.1 versus leaving provider-specific
- whether reasoning should get its own narrower runtime contract addendum later

## Evidence

- path: `2026-04-02-logos-capability-model-architecture-spec.md`
  symbol: `Core Decision`
  lines: `33-37`
- path: `2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Core Decision`
  lines: `33-37`
- path: `2026-04-02-logos-persistence-resume-architecture-spec.md`
  symbol: `Core Decision`
  lines: `39-43`
- path: `C:\Users\zand\Desktop\analysis.txt`
  symbol: `Final Architectural Rule`
  lines: `676-690`
- path: `../research/2026-04-02-opencode-goose-architecture-reference.md`
  symbol: `Parity Decision For LOGOS`
  lines: `270-330`

## Links

- [[README.md]]
- [[2026-04-02-logos-runtime-tier-contract-spec.md]]
- [[2026-04-02-logos-tier-0-semantic-surfaces-spec.md]]
- [[2026-04-02-logos-adapter-health-and-drift-policy.md]]
- [[2026-04-02-logos-minimal-survivable-agent-baseline-spec.md]]
