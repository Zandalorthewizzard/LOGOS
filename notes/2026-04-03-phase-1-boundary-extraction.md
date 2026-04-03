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
- Provider-native tool declarations are still unsupported from the normalized boundary.
- Full runtime execution plus tool-result reinjection is still deferred; the current slice covers request-side declaration forwarding and inbound normalized tool-call handling.

## Why A Small Core Seam Was Added

- The current public Codex seam exposed `Prompt` but did not expose a safe additive way for external crates to populate configured tools.
- A narrow helper seam in `codex-core` was smaller and safer than duplicating request wiring outside core.
- The seam is intentionally registered so future upstream merges can track it explicitly.

## Next Safe Resume Point

- Extend the session/tool runtime so declared tools can execute and reinject `ToolResult` messages through the normalized loop.
- Keep any further Codex-base interventions indexed separately if another seam turns out to be unavoidable.
