---
id: logos-phase-6-openrouter-beta-hardening-and-release-readiness-discussion
title: LOGOS Phase 6 OpenRouter Beta Hardening And Release Readiness Discussion
type: discussion
status: active
tags: [logos, phase-6, beta, openrouter, release, provider-foundation]
related:
  [
    [README.md],
    [2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [2026-04-11-code-main-fork-phase-6-triage-discussion.md],
    [2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-11-logos-structured-output-capability-discussion.md],
    [2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
    [../implementation/2026-03-30-logos-global-roadmap.md],
    [../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
  ]
depends_on:
  [
    [../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md],
    [../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md],
    [../implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md],
    [../implementation/2026-04-11-phase-4-persistence-resume-acceptance-report.md],
    [../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 6 OpenRouter Beta Hardening And Release Readiness Discussion

## Status And Scope

This is the active next phase.

Phase 6 is now the beta-hardening and release-readiness lane for `LOGOS 0.1 beta`.

Out of scope:

1. local validation implementation
2. Gemini implementation
3. GLM/Z.AI implementation
4. plugin-host implementation
5. broad structured-output implementation

## Why Phase 6 Exists

Phases 1 through 5 closed the runtime baseline:

1. the boundary is real
2. OpenAI semantics are preserved
3. tool loop and MCP correlation are explicit
4. journal and restart-safe resume exist
5. OpenRouter exists as a bounded validation adapter

Phase 6 exists because that state is still architecture-complete rather than release-complete.

The project now needs to cross the gap between:

1. a test-proven implementation slice
2. a releasable beta that can be installed, configured, run, and evaluated on real machines

OpenRouter is no longer treated as only a validation artifact. It is now the first live non-canonical beta path.

## Phase-6 Goals

1. Harden OpenRouter into a real beta provider path.
2. Stabilize provider-foundation contracts for later adapters.
3. Make `LOGOS 0.1 beta` release-ready.
4. Prove coexistence with Codex on the same machine.

## Locked Inputs

1. phase-1 acceptance
2. phase-2 acceptance
3. phase-3 acceptance
4. phase-4 acceptance
5. phase-5 acceptance
6. structured-output capability discussion
7. current roadmap and `AGENTS.md` MVP boundary

## Workstreams

1. `WS1: Codex Coexistence And Auth Isolation`
2. `WS2: OpenRouter Live Usability`
3. `WS3: Provider Foundation`
4. `WS4: Release Packaging And Beta Docs`
5. `WS5: Live Validation And Smoke Matrix`

## Coexistence Requirements

Phase 6 must prove that LOGOS can live on the same machine as Codex without storage or auth collision.

Required properties:

1. separate binary identity
2. separate config, state, and cache directories
3. separate auth storage
4. no implicit reuse of `~/.codex`
5. no OpenAI auth collision
6. optional import only as an explicit one-shot flow, never shared mutable state

This requirement is product-critical, not polish work. A Codex-derived fork that corrupts Codex auth or runtime state on the same host is not beta-ready.

## OpenRouter Beta Hardening Scope

Phase 6 should harden the OpenRouter path across:

1. credential and config flow
2. model-selection surface
3. honest unsupported and degraded behavior
4. live replay and journal behavior
5. readable provider-specific errors
6. no fake server-managed continuation

The target is not generic “provider support”. The target is one honest third-party provider path that users can run in real sessions.

## Provider Foundation Scope

Phase 6 should stabilize the provider layer enough that future adapters do not reopen already-closed runtime decisions.

This includes:

1. capability truth
2. provider and model metadata contract
3. adapter conformance expectations
4. support-tier language for later adapters

## Session-State Ownership Baseline

Phase 6 now also has an explicit ownership baseline:

1. LOGOS owns session, state, context, replay, resume, and thread-control semantics above provider wires.
2. Provider-native continuation or compaction remains valid as a provider-first backend, but not as the owner of product semantics.
3. Thin adapters remain the target; provider-specific thread-control stacks are a proving tactic, not the final ownership model.

The `code-main` fork research may inform this work, but only through the bounded triage recorded in [[2026-04-11-code-main-fork-phase-6-triage-discussion.md]]. Phase 6 should not import a second architecture wholesale.

## Release Readiness Scope

Phase 6 must also cover:

1. packaging and build targets
2. install and run docs
3. support matrix
4. beta issue and reporting workflow
5. minimal operational checklist

## Test Matrix

### Install And First Run

1. clean-machine install
2. `logos --help`
3. first run without config
4. first-run auth or credential setup
5. Codex and LOGOS installed side-by-side

### Auth Isolation

1. Codex already authenticated, then LOGOS first run
2. LOGOS auth change does not mutate Codex state
3. Codex auth change does not mutate LOGOS state
4. shared OpenAI key value is allowed, shared storage is not

### OpenAI Canonical Smoke

1. text turn
2. continuation
3. restart and resume
4. tool roundtrip
5. compaction and resume

### OpenRouter Live Smoke

1. text turn against a real model
2. follow-up replay from journal-backed history
3. model selection and override
4. auth failure
5. rate limit
6. context-limit failure
7. unsupported-control path

### Journal And Restart

1. hard process interruption
2. restart reconstruction
3. continuation correctness
4. approval and tool terminal correctness
5. compaction-boundary correctness

### CLI And Runtime Behavior

1. interactive TTY use
2. non-interactive invocation
3. interrupted request
4. slow network path
5. readable errors without secret leakage

### Platform Tiers

Tier 1:

1. Linux x86_64
2. Linux aarch64
3. macOS arm64

Tier 2:

1. Windows x64
2. older x86_64 and low-end hardware where practical

## OpenRouter Agent Beta Gate

This gate is now resolved as a hard requirement.

`LOGOS 0.1 beta` is an `Agent Beta`, not only a live-model beta. That means the OpenRouter path must reach the maximum available parity that can be achieved honestly through the existing LOGOS runtime contracts.

For phase 6 this means:

1. runtime-owned tools for OpenRouter are a release requirement, not an optional stretch goal
2. the OpenRouter path must be tested as an agent path, not only as a text and replay path
3. if this cannot be done without architectural distortion, phase 6 remains open and `0.1 beta` does not ship

Locked constraints for this gate:

1. no degradation of the canonical OpenAI path
2. no fake server-managed continuation
3. no provider-neutral fiction that hides OpenRouter differences
4. no broad `codex-core` refactor just to force parity
5. runtime-owned tool support must route through the existing LOGOS session and tool-runtime seams

The point of the beta is one fully meaningful third-party provider path, not a second text-only demo path.

## OpenRouter Tool Gate Proof Plan

Phase 6 must not begin with broad OpenRouter hardening. It must begin with one narrow proof slice that tests whether the OpenRouter agent-beta gate is realistically achievable through the existing LOGOS seams.

Locked proof order:

1. adapter-local outbound tool declaration mapping for a validated OpenRouter model or route
2. adapter-local inbound tool-call event mapping into the existing LOGOS runtime event contract
3. adapter-local replay mapping for `AssistantToolCall` and `ToolResult` through client-managed history
4. one end-to-end runtime-owned tool roundtrip through `logos-session-engine` and `logos-tool-runtime`
5. one journal-backed follow-up proof after tool execution from the same durable state family
6. explicit degraded behavior for OpenRouter models or routes that do not sustain the same tool path honestly

This proof plan exists to answer one question early:

1. can OpenRouter reach useful agent-grade parity through adapter-local work
2. or would phase 6 require architecture distortion to force it

Failure conditions for this proof plan are phase blockers:

1. OpenRouter-specific behavior would need to move into `logos-session-engine`
2. a new `codex-core` seam would be needed only for OpenRouter tool support
3. the adapter would need fake OpenAI-style server-managed continuation
4. shared runtime types would need widening only for OpenRouter quirks
5. model- or route-specific limitations could not be represented honestly as degraded behavior

If the proof slice fails for one of those reasons, phase 6 remains open and `LOGOS 0.1 beta` does not ship yet as an OpenRouter agent beta.

The proof slice is not sufficient by itself for closeout. Phase-6 closeout must also explicitly walk the risk register in [[2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md]].

## Non-goals

1. adding new providers in phase 6
2. plugin-host implementation
3. local validation implementation
4. shared structured-output implementation
5. broad storage hardening beyond beta blockers

## Exit Conditions

Phase 6 closes only when all of the following are true:

1. OpenRouter is usable as a live agent beta path
2. Codex coexistence and auth isolation are proven
3. provider foundation is stable enough for later milestones
4. release artifacts and release docs are ready
5. support tiers are explicitly documented
6. the OpenRouter stateless replay and tool-gate risk register has been explicitly walked and either proven or bounded in the beta support statement

## Evidence

- path: `../implementation/2026-03-30-logos-global-roadmap.md`
  symbol: `LOGOS Global Roadmap`
  lines: `1-320`
- path: `../../../AGENTS.md`
  symbol: `LOGOS Agent Guide`
  lines: `1-120`
- path: `../implementation/2026-04-03-phase-1-boundary-extraction-acceptance-report.md`
  symbol: `LOGOS Phase 1 Boundary Extraction Acceptance Report`
  lines: `1-260`
- path: `../implementation/2026-04-07-phase-2-openai-canonical-turn-loop-acceptance-report.md`
  symbol: `LOGOS Phase 2 OpenAI Canonical Turn Loop Acceptance Report`
  lines: `1-240`
- path: `../implementation/2026-04-11-phase-3-tool-loop-boundary-acceptance-report.md`
  symbol: `LOGOS Phase 3 Tool Loop Boundary Acceptance Report`
  lines: `1-240`
- path: `../implementation/2026-04-11-phase-4-persistence-resume-acceptance-report.md`
  symbol: `LOGOS Phase 4 Persistence Resume Acceptance Report`
  lines: `1-240`
- path: `../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md`
  symbol: `LOGOS Phase 5 OpenRouter Validation Adapter Acceptance Report`
  lines: `1-220`
- path: `2026-04-02-logos-provider-openrouter-adapter-spec.md`
  symbol: `LOGOS OpenRouter Adapter Architecture Spec`
  lines: `1-240`
- path: `2026-04-11-logos-structured-output-capability-discussion.md`
  symbol: `LOGOS Structured Output Capability Discussion`
  lines: `1-260`
- path: `2026-04-11-code-main-fork-phase-6-triage-discussion.md`
  symbol: `Decision Summary`
  lines: `27-46`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Current Consensus`
  lines: `22-37`
- path: `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `LOGOS OpenRouter Stateless Replay And Tool-Gate Observations`
  lines: `1-260`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine::run_turn_with_tools_inner`
  lines: `240-420`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolRuntime`
  lines: `1-360`
- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `OpenRouterAdapter`
  lines: `1-240`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `map_turn_request`
  lines: `1-240`
- path: `../../../codex-rs/logos-provider-openai/src/map_request.rs`
  symbol: `map_turn_request`
  lines: `1-260`
- path: `../../../codex-rs/logos-provider-openai/src/map_events.rs`
  symbol: `map_response`
  lines: `1-320`
- path: `../../../codex-rs/logos-provider-openai/src/map_messages.rs`
  symbol: `map_message`
  lines: `1-240`

## Links

- [[README.md]]
- [[2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[2026-04-11-code-main-fork-phase-6-triage-discussion.md]]
- [[2026-04-11-logos-structured-output-capability-discussion.md]]
- [[../implementation/2026-03-30-logos-global-roadmap.md]]
- [[../implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md]]
