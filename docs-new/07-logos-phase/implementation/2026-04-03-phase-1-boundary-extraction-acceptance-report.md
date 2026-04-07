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
4. Runtime-owned tool declarations can now be forwarded into the canonical OpenAI/Codex request path.
5. Runtime-owned tool results can now be reinjected through follow-up turns in the normalized session loop.
6. Approval decisions are now explicit normalized control-flow events instead of hidden execution side effects.
7. Shared runtime crates do not export raw OpenAI wire structs as the shared runtime contract.
8. OpenAI continuation now travels through an explicit request seam instead of cross-turn `ModelClientSession` reuse.

## Boundary Shape

### Shared Runtime

- `logos-runtime-types` owns normalized turn requests, messages, events, continuation handles, capabilities, approval artifacts, and provider errors.
- `logos-provider-api` owns the adapter/session traits and event stream contract.
- `logos-tool-runtime` owns the stable-id tool-call collection seam plus runtime-owned tool execution dispatch for phase 1.

### OpenAI Wrapper

- `logos-provider-openai` wraps `codex_core::ModelClientSession` instead of rewriting the Codex/OpenAI path.
- Request mapping stays adapter-local.
- Response and stream mapping stay adapter-local.
- The shared runtime only sees normalized `InternalEvent` and `InternalMessage` values.
- The adapter now creates a fresh `ModelClientSession` per turn and relies on explicit normalized continuation handles.

### Session Engine

- `logos-session-engine` owns the turn loop facade for this slice.
- It accumulates assistant text, captures continuation updates, records normalized tool-call history, executes registered runtime-owned tools, applies approval policy, and reinjects `ToolResult` follow-up turns.

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
- `declared_runtime_tools_are_forwarded_to_openai_request` proves outbound runtime-owned tool declaration forwarding through the new seam.
- `runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn` proves execution plus reinjection through the normalized loop.
- `runtime_tool_error_is_reinjected_as_tool_result` proves tool failure is reinjected as `ToolResult` with `is_error = true`.
- `denied_runtime_tool_is_reinjected_without_execution` proves approval-denied branches stay explicit and correlated.
- `provider_native_tool_declaration_is_capability_gated` proves provider-native declarations remain capability-gated and optional.
- `responses_websocket_v2_uses_explicit_previous_response_id_without_incremental_carryover` proves explicit continuation survives request serialization and bypasses implicit websocket carryover.
- `output_item_added_and_done_do_not_duplicate_tool_start` and `reasoning_item_added_and_done_do_not_duplicate_reasoning_text` prove adapter normalization no longer duplicates tool or reasoning signals.

## Completion State

The phase-1 boundary extraction slice is now logically complete for the scope defined by the strict spec and its tool-loop architecture dependencies.

Completed within phase-1 scope:

1. runtime-owned function-style tool definitions are forwarded into the canonical OpenAI/Codex request path;
2. inbound provider-emitted tool calls are normalized successfully and preserve stable `call_id` values;
3. registered runtime-owned tools can execute and reinject `ToolResult` follow-up messages;
4. approval requests and approval outcomes are explicit normalized events in the session loop;
5. provider-native tool declarations remain capability-gated and are rejected on the current OpenAI phase-1 path.
6. OpenAI continuation uses an explicit `previous_response_id` seam and no longer depends on reusing one turn-scoped `ModelClientSession` across multiple turns.

Out of scope for this phase and deferred to later phases or follow-on specs:

1. provider-native tool fast paths on non-canonical adapters;
2. durable approval journaling and resume reconstruction beyond in-memory loop state;
3. broader policy integration with future host/daemon control surfaces.

## Why Codex-Base Changes Stayed Minimal

The implementation uses one narrow registered Codex-core seam plus additive crates.

1. `codex_core::ModelClientSession` still provides the primary OpenAI streaming seam.
2. The only direct Codex-core intervention is the Prompt helper seam recorded in `2026-04-03-codex-core-seam-index.md`.
3. The adapter still owns request mapping, event mapping, and session-engine ownership outside `codex-core`.

## Tests Run

1. `cargo test -p codex-core client_common_tests`
2. `cargo test -p logos-tool-runtime`
3. `cargo test -p logos-session-engine`
4. `cargo test -p codex-api common::tests::websocket_conversion_preserves_previous_response_id --lib`
5. `cargo test -p codex-core responses_websocket_v2_uses_explicit_previous_response_id_without_incremental_carryover`
6. `cargo test -p logos-provider-openai --lib`
7. `just fix -p codex-core`
8. `just fix -p logos-runtime-types`
9. `just fix -p logos-provider-api`
10. `just fix -p logos-provider-openai`
11. `just fix -p logos-tool-runtime`
12. `just fix -p logos-session-engine`
13. `just fmt`

## Deferred Follow-Ups

1. Carry approval artifacts into future durable journal/resume layers when that phase is opened.
2. Define provider-native fast-path policy for later non-OpenAI adapters.
3. Revisit whether any second seam is needed only if later phases prove execution metadata cannot stay outside `codex-core`.
4. Replace `PENDING_LOCAL_COMMIT` in the seam index with the dedicated continuation-seam commit hash when the local stabilization commit is created.

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
  lines: `1-60`
- path: `2026-04-03-codex-core-seam-index.md`
  symbol: `SEAM-2026-04-03-01 Prompt Tool Injection`
  lines: `1-200`
- path: `2026-04-03-codex-core-seam-index.md`
  symbol: `SEAM-2026-04-03-02 Explicit Continuation Carryover`
  lines: `1-240`
- path: `../../../codex-rs/logos-provider-openai/src/map_events.rs`
  symbol: `map_response_event`
  lines: `1-210`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: `1-267`
- path: `../../../codex-rs/logos-tool-runtime/src/lib.rs`
  symbol: `ToolRuntime`
  lines: `1-171`
- path: `../../../codex-rs/logos-session-engine/tests/phase1_boundary.rs`
  symbol: `phase1 boundary tests`
  lines: `1-532`
- path: `2026-04-02-phase-1-boundary-extraction-strict-spec.md`
  symbol: `Acceptance and Tests`
  lines: `109-121`

## Links

- [[README.md]]
- [[2026-04-02-phase-1-boundary-extraction-strict-spec.md]]
- [[2026-03-30-logos-phase-1-implementation-slice.md]]
- [[../architecture/2026-04-02-logos-provider-openai-adapter-spec.md]]
- [[../../../notes/2026-04-03-phase-1-boundary-extraction.md]]
