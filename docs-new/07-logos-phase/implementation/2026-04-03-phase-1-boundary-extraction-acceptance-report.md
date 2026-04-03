---
id: logos-phase-1-boundary-extraction-acceptance-report
title: LOGOS Phase 1 Boundary Extraction Acceptance Report
type: report
status: active
tags: [implementation, report, phase-1, logos, openai]
related:
  [
    [README.md],
    [2026-04-02-phase-1-boundary-extraction-strict-spec.md],
    [2026-03-30-logos-phase-1-implementation-slice.md],
    [../architecture/2026-04-02-logos-provider-openai-adapter-spec.md],
    [../../../notes/2026-04-03-phase-1-boundary-extraction.md]
  ]
depends_on:
  [
    [2026-04-02-phase-1-boundary-extraction-strict-spec.md],
    [../architecture/2026-04-02-logos-adapter-kernel-architecture.md],
    [../architecture/2026-04-02-logos-tool-loop-architecture-spec.md]
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 1 Boundary Extraction Acceptance Report

## Scope

This report records the delivered phase-1 boundary extraction slice for the current branch and ties the result back to the strict spec acceptance gates.

## Delivered Crates

1. `codex-rs/logos-runtime-types/`
2. `codex-rs/logos-provider-api/`
3. `codex-rs/logos-provider-openai/`
4. `codex-rs/logos-session-engine/`
5. `codex-rs/logos-tool-runtime/`

## What Works

1. One text-only OpenAI turn runs through the new Logos-owned boundary.
2. One continuation flow preserves and reuses the OpenAI `previous_response_id` handle.
3. One normalized tool-call boundary preserves a stable `call_id` through normalized events and durable in-memory turn history.
4. Shared runtime crates do not export raw OpenAI wire structs as the shared runtime contract.

## Boundary Shape

### Shared Runtime

- `logos-runtime-types` owns normalized turn requests, messages, events, continuation handles, capabilities, and provider errors.
- `logos-provider-api` owns the adapter/session traits and event stream contract.
- `logos-tool-runtime` owns the minimal stable-id tool-call collection seam for phase 1.

### OpenAI Wrapper

- `logos-provider-openai` wraps `codex_core::ModelClientSession` instead of rewriting the Codex/OpenAI path.
- Request mapping stays adapter-local.
- Response and stream mapping stay adapter-local.
- The shared runtime only sees normalized `InternalEvent` and `InternalMessage` values.

### Session Engine

- `logos-session-engine` owns the turn loop facade for this slice.
- It accumulates assistant text, captures continuation updates, and records normalized tool-call history.

## Acceptance Mapping

### A1. Normalized runtime types exist without raw OpenAI leakage

- Met.
- Shared runtime exports are defined in `logos-runtime-types` and `logos-provider-api`.
- OpenAI-specific wire types remain adapter-local inside `logos-provider-openai`.

### A2. OpenAI text turn works through boundary

- Met.
- `text_only_openai_turn_runs_through_boundary` proves a text-only turn through the new adapter and session-engine seam.

### A3. OpenAI continuation handle survives boundary

- Met.
- `continuation_handle_is_preserved_for_follow_up_turn` proves the adapter emits a continuation update and the second turn reuses the preserved handle through the same provider session.

### A4. Tool boundary exists with stable IDs

- Met for the phase-1 proof target.
- `stable_tool_call_id_survives_normalized_boundary` proves a stable normalized tool-call boundary with preserved `call_id` and reinjected durable assistant-tool-call history.

## Known Limitation

`logos-provider-openai` does not yet map `InternalTurnRequest.tools` into `codex_core::Prompt::tools`.

Reason:

1. the current Codex seam exposes `Prompt` construction publicly but does not expose a safe additive builder around model-visible tool specs and runtime tool wiring that can be consumed from the new adapter crates;
2. phase 1 therefore preserves the canonical OpenAI path by refusing to invent a fake shared abstraction or by reaching deeper into Codex internals than the slice requires.

Current behavior:

1. if `request.tools` is non-empty, the adapter returns `ProviderErrorKind::UnsupportedCapability`;
2. inbound provider-emitted tool calls are still normalized successfully;
3. stable tool-call ID proof and `InternalMessage::AssistantToolCall` durability are already in place.

This limitation is explicit and accepted for the phase-1 slice because the strict-spec acceptance target requires a normalized tool-call boundary proof, not full runtime-owned tool definition export and reinjection yet.

## Why Codex-Base Changes Stayed Minimal

The implementation only touched `codex-rs/Cargo.toml` to register the new additive crates in the workspace. No deeper Codex-base file changes were required because:

1. `codex_core::ModelClientSession` already provides the thinnest stable OpenAI streaming seam;
2. the adapter can wrap and map `ResponseEvent` values without rewriting transport semantics;
3. the stable-id tool boundary proof can be delivered in additive crates for this phase.

## Tests Run

1. `cargo test -p logos-session-engine`
2. `just fix -p logos-runtime-types`
3. `just fix -p logos-provider-api`
4. `just fix -p logos-provider-openai`
5. `just fix -p logos-tool-runtime`
6. `just fix -p logos-session-engine`
7. `just fmt`

## Deferred Follow-Ups

1. Add outward mapping from `InternalTurnRequest.tools` into Codex/OpenAI tool specs through a thin additive seam.
2. Add runtime-owned tool result reinjection for the full portable loop, not only the boundary proof.
3. Decide whether the next seam should be a small public Codex helper for prompt tool wiring or a new Logos-owned bridge crate over existing tool registry/runtime code.

## Evidence

- path: `../../../codex-rs/logos-runtime-types/src/lib.rs`
  symbol: `public exports`
  lines: `1-22`
- path: `../../../codex-rs/logos-provider-api/src/provider.rs`
  symbol: `ProviderAdapter`
  lines: `1-34`
- path: `../../../codex-rs/logos-provider-openai/src/adapter.rs`
  symbol: `OpenAiAdapter`
  lines: `1-152`
- path: `../../../codex-rs/logos-provider-openai/src/map_request.rs`
  symbol: `map_turn_request`
  lines: `1-27`
- path: `../../../codex-rs/logos-provider-openai/src/map_events.rs`
  symbol: `map_response_event`
  lines: `1-207`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `1-109`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolCallCollector`
  lines: `1-71`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `phase1 boundary tests`
  lines: `1-225`
- path: `2026-04-02-phase-1-boundary-extraction-strict-spec.md`
  symbol: `Acceptance and Tests`
  lines: `109-121`

## Links

- [[README.md]]
- [[2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
- [[2026-03-30-logos-phase-1-implementation-slice.md]]
- [[../architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[../../../notes/2026-04-03-phase-1-boundary-extraction.md]]
