# Logos Phase 1 Implementation Slice

## 1. Purpose

- This document defines the first narrow implementation slice for Logos adapter architecture.
- It is designed to prove the OpenAI wrapper path without triggering broad Codex surgery.
- It is the practical execution companion to:
  - `docs-temp/logos-adapter-boundary-spec.md`
  - `docs-temp/logos-crate-map.md`
  - `docs-temp/logos-runtime-types-v0.md`

## 2. Objective

Phase 1 must do exactly this:

1. create the shared runtime types crate
2. create the provider adapter trait crate
3. wrap the current Codex/OpenAI path as `logos-provider-openai`
4. prove one end-to-end turn can run through the new boundary
5. keep Codex Responses mechanics intact while doing so

## 3. Non-goals

1. no native Gemini, Z.AI, or Anthropic adapters yet
2. no broad persistence redesign yet
3. no full agent tree migration yet
4. no CLI or TUI migration yet
5. no rewrite of Codex SSE/WS, `ResponseItem`, or `ToolSpec` into shared types

## 4. First 5 Rust Crates Or Modules To Create

### 1. `logos-runtime-types`

- Owns:
  - `ModelRef`
  - `InternalTurnRequest`
  - `InternalMessage`
  - `InternalToolDef`
  - `InternalEvent`
  - `ContinuationHandle`
  - capability structs
  - `ProviderError`

### 2. `logos-provider-api`

- Owns:
  - `ProviderAdapter` trait
  - provider/model capability contracts
  - shared adapter-facing result types if needed

### 3. `logos-provider-openai`

- Owns:
  - wrapping current Codex/OpenAI path
  - Responses-to-internal mapping
  - internal-to-Responses request mapping
  - OpenAI extension mapping

### 4. `logos-session-engine`

- Owns:
  - phase-1 turn loop facade
  - request handoff to provider adapter
  - normalized event consumption
  - tool result reinjection seam, even if stubbed initially

### 5. `logos-tool-runtime`

- Owns:
  - thin boundary around preserved Codex tool pipeline
  - initial adapter for normalized tool call events
  - phase-1 may keep this minimal if tool execution is not yet fully wired

## 5. First Files To Create

### `logos-runtime-types`

- `src/lib.rs`
- `src/model.rs`
- `src/message.rs`
- `src/tools.rs`
- `src/events.rs`
- `src/continuation.rs`
- `src/capabilities.rs`
- `src/errors.rs`

### `logos-provider-api`

- `src/lib.rs`
- `src/provider.rs`

### `logos-provider-openai`

- `src/lib.rs`
- `src/adapter.rs`
- `src/map_request.rs`
- `src/map_events.rs`
- `src/map_messages.rs`
- `src/extensions.rs`

### `logos-session-engine`

- `src/lib.rs`
- `src/turn.rs`
- `src/session.rs`
- `src/tool_loop.rs`

## 6. Minimal Working Path

The first working path should be:

1. build `InternalTurnRequest`
2. call `logos-provider-openai::stream_turn(...)`
3. internally map to existing Codex/OpenAI request path
4. receive Responses stream
5. map Responses events to `InternalEvent`
6. emit normalized text/tool/continuation events
7. complete one turn successfully

Tool execution may be stubbed in the first pass if the stream and event contract are validated cleanly.

## 7. Minimum Acceptance Plan

### Acceptance 1: OpenAI Wrapper Works

- Input: one simple text-only turn
- Expectation:
  - provider adapter returns normalized stream
  - text deltas arrive as `InternalEvent::TextDelta`
  - turn ends with `Completed`

### Acceptance 2: Continuation Is Preserved

- Input: one stateful follow-up turn on OpenAI path
- Expectation:
  - adapter emits `ContinuationUpdated`
  - second turn can consume persisted continuation handle
  - no fake client-history fallback is required on OpenAI path

### Acceptance 3: Tool Call Boundary Exists

- Input: one turn that triggers a tool call
- Expectation:
  - tool call enters normalized event surface
  - call id remains stable
  - tool result can be represented as `InternalMessage::ToolResult`

### Acceptance 4: No Shared Runtime Leakage

- Expectation:
  - `ResponseItem`, `ResponsesApiRequest`, and raw SSE enums are not exported from shared runtime crates

## 8. Failure Conditions

- Phase 1 fails if `logos-runtime-types` starts mirroring all OpenAI wire structs.
- Phase 1 fails if the OpenAI adapter requires rewriting Codex transport semantics.
- Phase 1 fails if session logic still fundamentally consumes raw Responses types outside the adapter.
- Phase 1 fails if tool execution becomes provider-owned by default.

## 9. Recommended Immediate Next Coding Tasks

1. create `logos-runtime-types` with the v0 types already drafted
2. create `logos-provider-api` with `ProviderAdapter`
3. implement `logos-provider-openai::OpenAiAdapter`
4. build a tiny integration test that runs one text-only turn through the adapter boundary
5. add a second test for continuation handle preservation

## 10. Guardrails

- If a change would make upstream sync with Codex materially harder, stop.
- If a type only exists to mirror an OpenAI JSON field one-to-one, it probably belongs in `logos-provider-openai`, not shared runtime.
- If a provider-specific feature cannot be normalized honestly, surface it through extensions and capabilities instead of flattening it.
