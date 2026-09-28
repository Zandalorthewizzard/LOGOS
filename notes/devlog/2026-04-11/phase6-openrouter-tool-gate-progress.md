# Phase 6 OpenRouter Tool Gate Progress

Date: 2026-04-11
Branch: `feat/phase6-openrouter-agent-beta`

## What Passed

The locked phase-6 OpenRouter tool-gate proof slice now passes through the existing LOGOS seams without widening shared runtime contracts or adding a new `codex-core` seam.

Implemented proof points:

1. adapter-local outbound runtime-tool declaration mapping for OpenRouter
2. adapter-local inbound provider tool-call event mapping into the existing `InternalEvent` contract
3. adapter-local replay mapping for `AssistantToolCall` and `ToolResult`
4. one end-to-end runtime-owned tool roundtrip through `logos-session-engine`
5. one journal-backed follow-up proof after tool execution
6. OpenAI continuation/tool/resume regressions stayed green in current scope

## Important Code Findings

1. `logos-provider-openrouter` was still hard-locked to phase-5 text-only behavior before this slice.
2. The existing shared LOGOS session/tool-runtime seam was already sufficient once the adapter mapped:
   - request `tools`
   - provider `function_call` output items
   - replayed `function_call` / `function_call_output` input items
3. No OpenRouter-specific logic had to move into `logos-session-engine`.
4. No fake `previous_response_id` or server-managed continuation was introduced.

## Remaining Phase-6 Reality

Phase 6 is not closed by this slice alone.

The repo still does not expose a real user-facing OpenRouter beta path through the main CLI/runtime surface, and the broader coexistence/auth-isolation/release-readiness work remains separate follow-on work after the tool gate.

## Checks Run

1. `cargo test -p logos-provider-openrouter`
2. `cargo test -p logos-session-engine --test phase5_openrouter_validation`
3. `cargo test -p logos-session-engine --test phase1_boundary`
4. `cargo test -p logos-session-engine --test phase4_persistence`
5. `cargo test -p logos-provider-openai`
