---
id: logos-global-roadmap
title: LOGOS Global Roadmap
type: roadmap
status: active
tags: [roadmap, logos, implementation, architecture]
related:
  [
    [README.md],
    [
      ../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md,
    ],
    [../architecture/2026-04-11-local-validation-post-mvp-milestone-discussion.md],
    [../architecture/2026-04-11-glm-zai-post-mvp-milestone-discussion.md],
    [../architecture/2026-04-11-gemini-post-mvp-milestone-discussion.md],
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

The first meaningful LOGOS MVP is reached when the adapter boundary is real, OpenAI remains full-fidelity, OpenRouter is usable as a live beta path, and the provider foundation is stable enough to onboard future providers without reopening the already-closed runtime phases.

### MVP Required Providers And Adapters

1. `logos-provider-openai`
   - canonical provider
   - preserves Codex/OpenAI Responses behavior
   - must support OpenAI continuation/resume
   - must support the first working tool roundtrip boundary
2. `logos-provider-openrouter`
    - first non-canonical beta provider
    - proves client-managed history and graceful degradation
    - supplies the live model surface for concept-stage testing

### MVP Non-goals

1. LM Studio is not required for MVP ship and is now explicitly a post-MVP local-validation milestone.
2. Ollama is not required for MVP ship.
3. Gemini is not required for MVP ship and is now explicitly a post-MVP provider milestone.
4. GLM/Z.AI is not required for MVP ship and is now explicitly a post-MVP provider milestone.
5. Provider-native feature parity across all providers is not required for MVP ship.

### MVP Exit Criteria

1. OpenAI path works through the new boundary without semantic regression.
2. OpenRouter works as a real agent beta path with explicit degraded-capability behavior, live-session usability, and runtime-owned tool support where the provider path can honestly sustain it.
3. LOGOS has a working minimal persistence/resume story for canonical and stateless modes.
4. Tool roundtrip exists at least for the OpenAI path and the shared runtime seam is proven.
5. Provider foundation is stable enough that later provider milestones do not need to reopen phase-1 through phase-5 runtime contracts.
6. A releasable `LOGOS 0.1 beta` build and documentation posture exist.

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

## Phase 6: OpenRouter Beta Hardening And Provider Foundation

### Goals

1. Harden OpenRouter from a bounded validation slice into a live beta path.
2. Stabilize provider capability, metadata, and adapter-conformance foundations for future providers.
3. Produce the release-ready posture for `LOGOS 0.1 beta`.
4. Prove OpenRouter as an agent-grade beta path rather than only a text and replay path.

### Deliverables

1. OpenRouter beta-hardening work around configuration, credentials, model selection, explicit degradation, and runtime-owned tools.
2. A provider-foundation baseline that future adapters can target without reopening closed OpenAI and runtime semantics.
3. A release checklist and install/run guidance for `LOGOS 0.1 beta`.
4. Live OpenRouter validation findings that feed later provider milestones.

### Exit Criteria

1. OpenRouter is usable for real agent beta testing, not only test-harness proofs.
2. `LOGOS 0.1 beta` is buildable and releasable.
3. Future provider milestones can start from a stable provider foundation instead of reopening MVP runtime design.

### Canonical Planning Inputs

1. `../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
2. `2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec.md`

## Post-MVP Provider Milestones

### Local Validation Milestone

1. Local and offline validation are now explicitly post-MVP.
2. LM Studio remains the first preferred local target.
3. Ollama remains deferred until LM Studio findings justify a second local path.
4. See `../architecture/2026-04-11-local-validation-post-mvp-milestone-discussion.md`.

### GLM Z.AI Milestone

1. GLM/Z.AI is now a post-MVP provider-expansion milestone.
2. Its first implementation should be evidence-backed and may be native, compatible, or mixed-mode depending on the narrowest honest path.
3. See `../architecture/2026-04-11-glm-zai-post-mvp-milestone-discussion.md`.

### Gemini Milestone

1. Gemini is now a post-MVP provider-expansion milestone.
2. The first Gemini path should follow provider-foundation hardening and use capability-first narrowing.
3. See `../architecture/2026-04-11-gemini-post-mvp-milestone-discussion.md`.

## Post-MVP Future Direction: Plugin System And Extension Host

### Position

1. A LOGOS plugin system is a future direction, not part of the initial MVP.
2. The plugin host should live around the preserved Codex kernel, not inside `codex-core`.
3. First-party "core plugins" should prove the extension contracts before community plugins are opened broadly.

### Earliest Useful Timing

1. Architecture shaping can happen earlier.
2. Real host implementation should wait until phase 4 makes runtime authority around persistence, resume, and compaction stable enough.
3. The first serious pluginization move is more realistic after phase 6 or the first post-MVP provider milestone, once LOGOS has one live non-canonical provider path and one durable runtime path.

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

1. Lock the discussion/spec package for phase-6 OpenRouter beta hardening and provider foundation work.
2. Define the release checklist for `LOGOS 0.1 beta`.
3. Keep local validation, Gemini, and GLM/Z.AI as post-MVP milestone planning until OpenRouter beta feedback exists.

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
