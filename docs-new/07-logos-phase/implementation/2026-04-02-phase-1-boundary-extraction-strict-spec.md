---
id: logos-phase-1-boundary-extraction-strict-spec
title: LOGOS Phase 1 Boundary Extraction Strict Spec
type: spec
status: active
tags: [implementation, strict-spec, phase-1, adapters, logos]
related:
  [
    [README.md],
    [2026-03-30-logos-phase-1-implementation-slice.md],
    [2026-03-30-logos-global-roadmap.md],
    [../architecture/2026-04-02-logos-adapter-kernel-architecture.md],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [../architecture/2026-04-02-logos-host-daemon-transport-spec.md]
  ]
depends_on:
  [
    [../architecture/2026-04-02-logos-adapter-kernel-architecture.md],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [../architecture/2026-04-02-logos-tool-loop-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 1 Boundary Extraction Strict Spec

## Start Here

1. `../architecture/2026-04-02-logos-adapter-kernel-architecture.md`
2. `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
3. `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
4. `2026-03-30-logos-phase-1-implementation-slice.md`

## Intent

Create the first production-usable boundary extraction slice for Logos so that the canonical OpenAI path runs through Logos-owned runtime seams without semantic degradation.

Non-goals:
1. no non-OpenAI provider implementation yet
2. no broad persistence engine implementation yet
3. no CLI/TUI migration
4. no final UI transport implementation beyond the minimum daemon seam needed for the slice

## Target Outcome

1. `logos-runtime-types`, `logos-provider-api`, `logos-provider-openai`, `logos-session-engine`, and `logos-tool-runtime` exist with minimally viable ownership boundaries.
2. One text-only OpenAI turn works through the new boundary.
3. One OpenAI continuation flow works through the new boundary.
4. One normalized tool-call boundary exists, even if execution remains minimally wired.

## Locked Decisions

1. OpenAI remains canonical and must not be degraded for abstraction purity.
2. Shared runtime crates must not export raw OpenAI wire structs as canonical types.
3. Tool runtime remains Logos-owned by default.
4. Host/daemon transport remains outside provider adapter design.

## Architecture Fit

1. `logos-runtime-types` owns normalized messages, events, continuation handles, capabilities, and provider errors.
2. `logos-provider-api` owns the provider adapter trait and capability contracts.
3. `logos-provider-openai` owns request and event mapping to the existing Codex/OpenAI path.
4. `logos-session-engine` owns turn loop orchestration across normalized events.
5. `logos-tool-runtime` owns the portable tool boundary.

## External References

1. Existing Codex/OpenAI path -> Adapt -> preserve strongest behavior and wrap at the seam instead of rewriting it.
2. Existing Codex tool pipeline -> Adapt -> preserve stable IDs and approval semantics while normalizing the boundary.

## Implementation Tasks (ordered)

1. `T1`
- package: `logos-runtime-types`
- files: `src/lib.rs`, `src/model.rs`, `src/message.rs`, `src/tools.rs`, `src/events.rs`, `src/continuation.rs`, `src/capabilities.rs`, `src/errors.rs`
- output: normalized runtime type crate with no raw OpenAI wire exports
2. `T2`
- package: `logos-provider-api`
- files: `src/lib.rs`, `src/provider.rs`
- output: provider adapter trait and capability interfaces
3. `T3`
- package: `logos-provider-openai`
- files: `src/lib.rs`, `src/adapter.rs`, `src/map_request.rs`, `src/map_events.rs`, `src/map_messages.rs`, `src/extensions.rs`
- output: OpenAI wrapper adapter over preserved Codex/OpenAI path
4. `T4`
- package: `logos-session-engine`
- files: `src/lib.rs`, `src/turn.rs`, `src/session.rs`, `src/tool_loop.rs`
- output: phase-1 turn loop consuming normalized events
5. `T5`
- package: `logos-tool-runtime`
- files: boundary files chosen by implementation
- output: minimal portable tool boundary with stable call-id handling
6. `T6`
- package: integration tests
- files: provider/session integration test files chosen by implementation
- output: one text turn, one continuation turn, and one tool-boundary test

## Requirement -> Task -> Test -> Gate

| Requirement | Task IDs | Tests | Acceptance | Gates |
| --- | --- | --- | --- | --- |
| Normalized runtime types exist without raw OpenAI leakage | `T1`, `T2` | crate compile + export audit | `A1` | `G1` |
| OpenAI text turn works through boundary | `T3`, `T4`, `T6` | targeted integration test | `A2` | `G2` |
| OpenAI continuation handle survives boundary | `T3`, `T4`, `T6` | targeted continuation test | `A3` | `G3` |
| Tool boundary exists with stable IDs | `T4`, `T5`, `T6` | targeted tool-boundary test | `A4` | `G4` |

## Acceptance and Tests

1. Workspace build or crate build commands for all new crates must pass.
2. One integration test for text-only OpenAI turn must pass.
3. One integration test for OpenAI continuation handle preservation must pass.
4. One integration test or focused boundary test for tool-call ID stability must pass.

## Promotion Artifacts

1. Phase-1 crate skeleton summary
2. Boundary acceptance report
3. OpenAI parity notes for the slice
4. Known limitations list for unfinished persistence/tool execution depth

## Risks and Rollback

1. Risk: raw OpenAI wire types leak into shared runtime crates.
2. Mitigation: review public exports and keep mapping code adapter-local.
3. Risk: boundary extraction forces large Codex transport surgery.
4. Mitigation: stop and revert to a thinner wrapper approach for the slice.
5. Rollback: keep OpenAI integration wrapper-local and avoid promoting unstable shared abstractions.

## Agent Guardrails

1. Do not rewrite Codex transport semantics merely to satisfy abstraction neatness.
2. Do not add non-OpenAI provider code in this slice.
3. Do not store raw provider wire types as the shared runtime contract.
4. Do not make tool execution provider-owned by default.

## Evidence

- path: `../architecture/2026-04-02-logos-adapter-kernel-architecture.md`
  symbol: `Kernel Boundary`
  lines: `35-45`
- path: `../architecture/2026-04-02-logos-provider-openai-adapter-spec.md`
  symbol: `Required Adapter Behavior`
  lines: `80-105`
- path: `../architecture/2026-04-02-logos-tool-loop-architecture-spec.md`
  symbol: `Mandatory Invariants`
  lines: `49-69`
- path: `2026-03-30-logos-phase-1-implementation-slice.md`
  symbol: `Objective`
  lines: `12-20`
- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 1: Boundary Skeleton`
  lines: `85-100`
- path: `2026-03-30-logos-global-roadmap.md`
  symbol: `Phase 2: OpenAI Canonical Turn Loop`
  lines: `106-123`

## Links

- [[README.md]]
- [[2026-03-30-logos-phase-1-implementation-slice.md]]
- [[../architecture/2026-04-02-logos-adapter-kernel-architecture.md]]
- [[../architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
