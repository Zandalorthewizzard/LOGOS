---
id: logos-global-roadmap
title: LOGOS Global Roadmap
type: roadmap
status: active
tags: [roadmap, logos, implementation, architecture]
related:
  [
    [README.md],
    [../architecture/2026-04-08-logos-plugin-system-future-discussion.md],
    [../architecture/2026-04-08-logos-plugin-system-architecture-direction.md],
    [../architecture/2026-04-03-logos-compaction-strategy-discussion.md],
    [../architecture/2026-03-30-logos-adapter-boundary-spec.md],
    [../architecture/2026-03-30-logos-crate-map.md],
    [../architecture/2026-03-30-logos-runtime-types-v0.md],
    [../research/2026-03-30-logos-provider-research-report.md],
    [../research/2026-03-30-provider-comparison-matrix.md],
    [2026-03-30-logos-phase-1-implementation-slice.md],
  ]
depends_on: [[../research/2026-03-30-logos-provider-research-report.md]]
generated: false
source_of_truth: planning
outline: []
---

# LOGOS Global Roadmap

## Mission

Build LOGOS as a Codex-derived, adapter-first runtime that preserves the full-fidelity OpenAI Responses path while establishing a narrow internal boundary for additional providers.

## Locked Roadmap Rules

1. OpenAI Responses remains the canonical path and must not be degraded for abstraction purity.
2. Additional providers are integrated through honest capability-aware adapters, not fake sameness.
3. Tool runtime, approvals, and session semantics stay explicit.
4. Persistence and resume are first-class design work, not incidental storage details.

## MVP Definition

### MVP Scope

The first meaningful LOGOS MVP is reached when the adapter boundary is real, OpenAI remains full-fidelity, and at least two non-canonical provider paths validate the boundary in different ways.

### MVP Required Providers And Adapters

1. `logos-provider-openai`
   - canonical provider
   - preserves Codex/OpenAI Responses behavior
   - must support OpenAI continuation/resume
   - must support the first working tool roundtrip boundary
2. `logos-provider-openrouter`
   - stateless validation adapter
   - proves client-managed history and graceful degradation
3. `logos-provider-gemini`
   - compatibility-important MVP provider
   - proves Logos can handle a provider that matters commercially but has both OpenAI-compat and native semantic differences

### MVP Non-goals

1. LM Studio is not required for MVP ship, but remains a strong dev/local validation target.
2. Ollama is not required for MVP ship.
3. Z.AI is not required for MVP ship.
4. Provider-native feature parity across all providers is not required for MVP ship.

### MVP Exit Criteria

1. OpenAI path works through the new boundary without semantic regression.
2. OpenRouter works as a stateless validation adapter.
3. Gemini works in at least one supported mode with explicit capability differences documented.
4. Logos has a working minimal persistence/resume story for canonical and stateless modes.
5. Tool roundtrip exists at least for the OpenAI path and the shared runtime seam is proven.

## Phase 0: Research And Source Stabilization

### Goals

1. Finish source capture and evidence-backed provider research.
2. Lock current architecture direction and known unknowns.
3. Prepare implementation without re-opening broad discovery.

### Exit Criteria

1. Research pack exists and is considered roadmap-grade.
2. Source inventory is populated enough for later implementation agents.
3. Prototype targets are explicit.

## Phase 1: Boundary Skeleton

### Goals

1. Create `logos-runtime-types`.
2. Create `logos-provider-api`.
3. Create `logos-provider-openai` as a wrapper, not a rewrite.
4. Create `logos-session-engine` seam.

### Deliverables

1. Crate skeletons.
2. Initial type contract.
3. First adapter trait.
4. One text-only end-to-end turn through the new boundary.

### Risks

1. Over-designing the runtime contract.
2. Pulling raw OpenAI wire types into shared APIs.

## Phase 2: OpenAI Canonical Turn Loop

### Goals

1. Preserve OpenAI stateful continuation.
2. Normalize streamed events into internal event types.
3. Keep Codex transport and session behavior intact.

### Deliverables

1. `stream_turn()` working for OpenAI.
2. Continuation handle preservation.
3. Completion and ordering invariants enforced.
4. Canonical OpenAI compaction behavior preserved without promoting provider-name backend checks into shared LOGOS architecture.

### Exit Criteria

1. OpenAI text-only and structured-output turns work.
2. Stateful follow-up turn works through the boundary.

### Canonical Planning Inputs

1. `../architecture/2026-04-03-phase-2-openai-canonical-turn-loop-discussion.md`
2. `2026-04-03-phase-2-openai-canonical-turn-loop-strict-spec.md`

## Phase 3: Tool Loop Boundary

### Goals

1. Connect normalized tool-call events to a Logos-owned runtime seam.
2. Preserve Codex approval behavior and MCP correlation.
3. Make `ToolResult` reinjection explicit in internal message history.

### Deliverables

1. Runtime-owned tool dispatch path.
2. Stable `call_id` handling.
3. Approval and completion branches preserved.

### Exit Criteria

1. One tool-call roundtrip works through the adapter boundary.
2. MCP tool-call correlation survives.

## Phase 4: Persistence And Resume

### Goals

1. Define Logos-native durable journal.
2. Support OpenAI stateful continuation and stateless replay mode.
3. Avoid storing raw provider wire formats as the canonical record model.
4. Lock a LOGOS-owned compaction policy that can use native or portable backends without making compaction output the canonical instruction source.

### Deliverables

1. Persistence schema v0.
2. Resume flow for OpenAI path.
3. Client-managed replay flow for stateless adapters.
4. Compaction policy and backend contract aligned with [[../architecture/2026-04-03-logos-compaction-strategy-discussion.md]].

### Exit Criteria

1. OpenAI resume works after process restart.
2. Stateless replay works for a validation adapter.
3. Compaction checkpointing preserves canonical instruction reinjection and does not corrupt resume semantics.

## Phase 5: Validation Adapter

### Goals

1. Add `logos-provider-openrouter`.
2. Prove stateless mode does not contaminate the OpenAI path.
3. Exercise capability flags and graceful degradation.

### Deliverables

1. OpenRouter adapter.
2. Parameter compatibility tests.
3. Error and limit mapping.

### Exit Criteria

1. OpenRouter is usable as a validation adapter.
2. OpenAI path remains unchanged in semantics.

## Phase 6: Local Validation Targets

### Goals

1. Add LM Studio as a dev/local validation path.
2. Decide whether Ollama belongs in 0.1 or later.
3. Validate local continuation and tool behavior where practical.

### Deliverables

1. LM Studio compatibility notes and adapter mode decisions.
2. Optional dev-only local adapter(s).

### Exit Criteria

1. Local validation path exists for offline/dev workflows.
2. Local behavior is clearly documented as canonical or non-canonical.

## Phase 7: Native Provider Expansion

### Goals

1. Evaluate Gemini native path.
2. Evaluate Z.AI native path.
3. Decide which provider-native features deserve first-class exposure.

### Deliverables

1. Capability-driven expansion plan.
2. Prototype-backed decisions for native adapters.

### Exit Criteria

1. Provider expansion is driven by evidence and product value.
2. No native provider support is added by pretending it is just OpenAI compat.

## Post-MVP Future Direction: Plugin System And Extension Host

### Position

1. A LOGOS plugin system is a future direction, not part of the initial MVP.
2. The plugin host should live around the preserved Codex kernel, not inside `codex-core`.
3. First-party "core plugins" should prove the extension contracts before community plugins are opened broadly.

### Earliest Useful Timing

1. Architecture shaping can happen earlier.
2. Real host implementation should wait until phase 4 makes runtime authority around persistence, resume, and compaction stable enough.
3. The first serious pluginization move is more realistic after phase 5 or phase 6, once LOGOS has at least one non-canonical provider path and one durable runtime path.

### Companion Planning Inputs

1. `../architecture/2026-04-08-logos-plugin-system-future-discussion.md`
2. `../architecture/2026-04-08-logos-plugin-system-architecture-direction.md`

## Cross-Cutting Workstreams

### 1. Contract Tests

1. Streaming invariants.
2. Tool-call ordering.
3. Resume correctness.
4. Error mapping.

### 2. Security And Approvals

1. Provider-native tool policy.
2. MCP allowlist and approval flow.
3. Secrets separation from logs and journals.

### 3. Documentation

1. Keep architecture, implementation, and research docs synchronized.
2. Promote stable findings out of working docs as the codebase forms.

## Immediate Next Milestones

1. Author the first strict implementation spec.
2. Create crate skeletons for phase 1.
3. Define prototype checklist for OpenAI resume, OpenRouter replay, and MCP tool roundtrip.
4. Decide the persistence schema design approach before broad coding.

## Evidence

- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Recommended Decisions for Logos 0.1`
  lines: `181-188`
- path: `../research/2026-03-30-logos-provider-research-report.md`
  symbol: `Unknowns and Prototype Targets`
  lines: `190-205`
- path: `../architecture/2026-03-30-logos-adapter-boundary-spec.md`
  symbol: `Migration Phases`
  lines: `253-277`
- path: `../architecture/2026-03-30-logos-crate-map.md`
  symbol: `Recommended Extraction Order`
  lines: `126-133`
- path: `../architecture/2026-03-30-logos-runtime-types-v0.md`
  symbol: `Companion Provider Trait v0`
  lines: `236-258`

## Links

- [[README.md]]
- [[../architecture/2026-03-30-logos-adapter-boundary-spec.md]]
- [[../architecture/2026-03-30-logos-crate-map.md]]
- [[../architecture/2026-03-30-logos-runtime-types-v0.md]]
- [[../research/2026-03-30-logos-provider-research-report.md]]
- [[../research/2026-03-30-provider-comparison-matrix.md]]
- [[2026-03-30-logos-phase-1-implementation-slice.md]]
