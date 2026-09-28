---
id: logos-phase-6-openrouter-beta-hardening-and-release-readiness-strict-spec
title: LOGOS Phase 6 OpenRouter Beta Hardening And Release Readiness Strict Spec
type: spec
status: active
tags: [logos, phase-6, strict-spec, openrouter, beta, release]
related:
  [
    [README.md],
    [2026-03-30-logos-global-roadmap.md],
    [2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
    [../architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md],
    [../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md],
    [../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
    [../architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md],
  ]
depends_on:
  [
    [2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md],
    [../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 6 OpenRouter Beta Hardening And Release Readiness Strict Spec

## Objective

Implement phase 6 so that `LOGOS 0.1 beta` becomes releasable as:

1. preserved OpenAI canonical path
2. OpenRouter agent beta path
3. stable provider-foundation baseline for later adapters

## Locked Scope

Included:

1. Codex coexistence and auth isolation
2. OpenRouter configuration and live usability hardening
3. OpenRouter runtime-owned tool path hardening through existing LOGOS seams
4. provider-foundation stabilization
5. packaging, release docs, and beta support matrix
6. smoke and integration matrix for beta release readiness

Excluded:

1. new provider integrations
2. local validation implementation
3. Gemini implementation
4. GLM/Z.AI implementation
5. plugin-host implementation
6. broad structured-output implementation
7. storage-engine hardening beyond concrete beta blockers

## Required Reading

1. `AGENTS.md`
2. `docs-new/00-navigation/INDEX.md`
3. `docs-new/07-logos-phase/README.md`
4. `docs-new/07-logos-phase/implementation/README.md`
5. `docs-new/07-logos-phase/implementation/2026-03-30-logos-global-roadmap.md`
6. `docs-new/07-logos-phase/architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
7. `docs-new/07-logos-phase/architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
8. `docs-new/07-logos-phase/architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md`
9. `docs-new/07-logos-phase/architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
10. `docs-new/07-logos-phase/architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md`
11. `docs-new/07-logos-phase/architecture/2026-04-11-logos-structured-output-capability-discussion.md`
12. `docs-new/07-logos-phase/implementation/2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md`
13. `docs-new/07-logos-phase/implementation/2026-04-03-codex-core-seam-index.md`

## Implementation Slices

### Slice A: Codex Coexistence And Auth Isolation

Required outcomes:

1. separate binary identity
2. separate config, state, and cache paths
3. separate auth storage
4. no implicit use of Codex mutable state
5. explicit coexistence tests and docs

### Slice B: OpenRouter Config, Model Selection, And Live Usability

Required outcomes:

1. usable config flow
2. model selection surface
3. real-session OpenRouter usability
4. explicit degraded and unsupported behavior
5. readable provider-specific errors

### Slice C: OpenRouter Agent Path

Required outcomes:

1. runtime-owned tool support through existing LOGOS seams
2. no fake server-managed continuation
3. no degradation of OpenAI path
4. at least one meaningful OpenRouter tool roundtrip proof
5. explicit degraded behavior where OpenRouter or model capabilities do not support parity

Locked implementation order:

1. adapter-local outbound tool declaration mapping
2. adapter-local inbound tool-call event mapping
3. adapter-local replay mapping for `AssistantToolCall` and `ToolResult`
4. one end-to-end runtime-owned tool roundtrip proof through the existing LOGOS seams
5. one journal-backed restart-safe follow-up proof after tool execution
6. only after those pass, broader OpenRouter beta hardening may widen

### Slice D: Provider Foundation Baseline

Required outcomes:

1. capability truth remains honest
2. provider and model metadata contracts are stable
3. future adapter expectations are clearer after phase 6 than before it
4. support-tier language exists for later providers

### Slice E: Packaging, Release Docs, And Support Matrix

Required outcomes:

1. release artifacts and build instructions
2. install and run docs
3. support matrix and support-tier language
4. beta issue and reporting path

### Slice F: Smoke And Integration Matrix

Required outcomes:

1. install and first-run scenarios
2. auth-isolation scenarios
3. OpenAI canonical regression smoke
4. OpenRouter live smoke
5. journal and restart smoke
6. CLI and runtime behavior smoke

## Acceptance Targets

### A1. Coexistence Proofs

1. Codex and LOGOS can coexist on one machine without config or auth collision.
2. LOGOS does not implicitly reuse `~/.codex` mutable state.

### A2. OpenAI No-Regression Proofs

1. canonical OpenAI continuation remains intact
2. canonical tool loop remains intact
3. canonical compaction and resume remain intact

### A3. OpenRouter Live Agent Proofs

1. OpenRouter handles a real text turn
2. OpenRouter handles replay-backed follow-up
3. OpenRouter handles at least one runtime-owned tool roundtrip through the existing LOGOS tool-runtime seam
4. degraded and unsupported behavior remain explicit

### A4. Release Artifact And Docs Proofs

1. beta build and install path exist
2. release and support docs exist
3. coexistence and auth-isolation behavior are documented

### A5. OpenRouter Risk Walkthrough Proofs

1. the phase-6 acceptance report explicitly walks the OpenRouter risk register from `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
2. each recorded risk is classified as `proven`, `bounded limitation`, or `remaining blocker`
3. any bounded limitation is carried into the beta support statement explicitly rather than left implicit

## Required Tests

### Install And First Run

1. clean install
2. `logos --help`
3. first run with no config
4. first credential setup

### Auth Isolation

1. Codex first, then LOGOS
2. LOGOS first, then Codex
3. auth mutation in one tool does not corrupt the other

### OpenAI Canonical Smoke

1. text turn
2. continuation
3. restart and resume
4. tool roundtrip
5. compaction and resume

### OpenRouter Live Smoke

1. text turn
2. follow-up replay
3. model switch or override
4. auth failure
5. rate limit
6. context-limit failure
7. unsupported controls
8. runtime-owned tool roundtrip

### Journal And Restart

1. interrupted process
2. restart reconstruction
3. durable continuation correctness
4. approval and tool terminal correctness
5. compaction-boundary correctness

### CLI And Runtime Behavior

1. interactive TTY
2. non-interactive invocation
3. interrupted request
4. slow-network path
5. secret-safe error output

### Platform Tiers

Tier 1:

1. Linux x86_64
2. Linux aarch64
3. macOS arm64

Tier 2:

1. Windows x64
2. older x86_64 and low-end hardware where practical

## Release Checklist

1. beta binaries or build instructions exist
2. install and run docs exist
3. config and auth isolation are documented
4. support statement exists
5. issue and feedback path exists
6. OpenAI canonical support is described separately from OpenRouter beta support
7. OpenRouter is described as an agent beta path, not only a validation path
8. the documented OpenRouter beta scope reflects the explicit risk walkthrough results rather than a generic provider-wide claim

## Guardrails

1. no `codex-core` seam unless separately justified and registered
2. no new provider integration in phase 6
3. no fake capability claims
4. no OpenAI degradation
5. no fake OpenRouter server-managed continuation
6. no broad provider-neutral abstraction that hides OpenRouter-specific behavior

## OpenRouter Agent Beta Gate

This gate is mandatory.

Phase 6 does not close unless OpenRouter reaches agent-grade beta status through the existing LOGOS runtime seams.

If runtime-owned tools for OpenRouter cannot be implemented honestly without architectural distortion, phase 6 remains blocked and `LOGOS 0.1 beta` does not ship yet.

Blocking conditions for this gate are:

1. OpenRouter-specific logic would have to move into `logos-session-engine`
2. a new `codex-core` seam would be needed only for OpenRouter tool support
3. the adapter would need fake server-managed continuation semantics
4. shared runtime types would have to widen only for OpenRouter quirks
5. model- or route-specific limitations could not be represented honestly as degraded behavior

In addition, phase 6 does not close if the risk register in `2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md` remains unwalked.

## Exit Verdict Shape

1. `Phase 6 closed for LOGOS 0.1 beta scope`
2. or `Phase 6 blocked by X`

## Evidence

- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `LOGOS Global Roadmap`
  lines: `1-320`
- path: `../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md`
  symbol: `LOGOS Phase 6 OpenRouter Beta Hardening And Release Readiness Discussion`
  lines: `1-320`
- path: `../architecture/2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Current Consensus`
  lines: `22-37`
- path: `../architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md`
  symbol: `Decision Summary`
  lines: `27-46`
- path: `2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md`
  symbol: `LOGOS Phase 5 OpenRouter Validation Adapter Acceptance Report`
  lines: `1-220`
- path: `../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md`
  symbol: `LOGOS OpenRouter Adapter Architecture Spec`
  lines: `1-240`
- path: `../architecture/2026-04-11-logos-structured-output-capability-discussion.md`
  symbol: `LOGOS Structured Output Capability Discussion`
  lines: `1-260`
- path: `../architecture/2026-04-11-openrouter-stateless-replay-and-tool-gate-observations.md`
  symbol: `LOGOS OpenRouter Stateless Replay And Tool-Gate Observations`
  lines: `1-260`
- path: `../../../AGENTS.md`
  symbol: `LOGOS Agent Guide`
  lines: `1-120`
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
- [[2026-03-30-logos-global-roadmap.md]]
- [[2026-04-11-phase-5-openrouter-validation-adapter-acceptance-report.md]]
- [[../architecture/2026-04-11-code-main-fork-phase-6-triage-discussion.md]]
- [[../architecture/2026-04-11-phase-6-openrouter-beta-hardening-and-release-readiness-discussion.md]]
