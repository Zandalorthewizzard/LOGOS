# 2026-04-03 Phase 1 Boundary Extraction

## Context

- Implemented the first additive Logos phase-1 boundary extraction slice inside `codex-rs/`.
- Goal was to prove the wrapper-first OpenAI boundary without degrading the canonical Codex/OpenAI path.

## Confirmed Facts

- Added five new crates:
  - `logos-runtime-types`
  - `logos-provider-api`
  - `logos-provider-openai`
  - `logos-session-engine`
  - `logos-tool-runtime`
- The OpenAI adapter wraps `codex_core::ModelClientSession` rather than replacing transport logic.
- `logos-session-engine/tests/phase1_boundary.rs` now proves:
  - one text-only turn through the new boundary
  - one continuation flow with preserved `previous_response_id`
  - one normalized tool-call boundary with stable `call_id`

## Current Limitation

- `logos-provider-openai` does not yet map `InternalTurnRequest.tools` into `codex_core::Prompt::tools`.
- Current behavior is explicit: non-empty `request.tools` returns `UnsupportedCapability`.
- Inbound provider-emitted tool calls are still normalized, so the phase-1 stable-id boundary proof is already covered.

## Why This Was Left Deferred

- The current public Codex seam exposes `Prompt` but not a clean additive bridge for external crates to populate runtime-owned tool wiring without reaching deeper into `codex-core` internals.
- For this slice it was safer to keep the limitation explicit than to widen the abstraction dishonestly or rewrite Codex mechanics.

## Next Safe Resume Point

- Extend the OpenAI adapter with a thin tool-definition bridge so `InternalTurnRequest.tools` can map into Codex model-visible tool specs.
- Then add full runtime-owned tool result reinjection, not just inbound tool-call normalization.
