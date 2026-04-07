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

- `logos-provider-openai` now maps runtime-owned tool definitions into `codex_core::Prompt` through a narrow Prompt helper seam in `codex-core`.
- `logos-session-engine` and `logos-tool-runtime` now execute runtime-owned tools with explicit approval events and reinject `ToolResult` follow-up turns.
- Provider-native tool declarations remain capability-gated and unsupported on the current OpenAI phase-1 path.
- OpenAI continuation now uses an explicit `previous_response_id` seam instead of relying on cross-turn `ModelClientSession` reuse.
- Adapter-local event mapping no longer double-emits tool/reasoning signals on `output_item.added` plus `output_item.done`.

## Why A Small Core Seam Was Added

- The current public Codex seam exposed `Prompt` but did not expose a safe additive way for external crates to populate configured tools.
- A narrow helper seam in `codex-core` was smaller and safer than duplicating request wiring outside core.
- The seam is intentionally registered so future upstream merges can track it explicitly.

## Next Safe Resume Point

- Treat phase-1 as logically complete for the current strict-spec scope.
- The next safe focus is either a later-phase spec or future durable approval/resume work rather than more boundary-slice churn.
