---
id: logos-phase-5-openrouter-validation-adapter-acceptance-report
title: LOGOS Phase 5 OpenRouter Validation Adapter Acceptance Report
type: report
status: active
tags: [implementation, report, phase-5, openrouter, validation, logos]
related:
  [
    [README.md],
    [2026-04-02-openrouter-validation-adapter-strict-spec.md],
    [../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [2026-04-11-phase-4-persistence-resume-acceptance-report.md],
    [2026-04-03-codex-core-seam-index.md],
  ]
depends_on:
  [
    [2026-04-02-openrouter-validation-adapter-strict-spec.md],
    [../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md],
    [2026-04-11-phase-4-persistence-resume-acceptance-report.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# LOGOS Phase 5 OpenRouter Validation Adapter Acceptance Report

## Scope

This report records the completed phase-5 validation-adapter slice for `logos-provider-openrouter` in the bounded stateless text and replay scope.

## Delivered In Phase 5

1. A new additive crate `codex-rs/logos-provider-openrouter/` now implements the first non-canonical validation adapter.
2. The adapter sends one stateless text turn through OpenRouter's `/api/v1/responses` beta endpoint without inventing `previous_response_id` semantics.
3. Follow-up turns now replay client-managed history from the LOGOS journal family introduced in phase-4, including required assistant `id` and `status` fields.
4. Unsupported controls stay explicit: the adapter rejects tools, structured output, non-default reasoning controls, non-client-managed continuations, and non-text message parts in the current scope.
5. Current capability truth is now honest: the validation slice advertises `supports_structured_output: false` and keeps `OutputMode::Structured` as an explicit `UnsupportedCapability` path.
6. Malformed non-`2xx` HTTP bodies now fall back to status-based normalization instead of collapsing into generic retryable transport errors.
7. The canonical OpenAI path remains unchanged in semantics; no OpenAI adapter code or Codex-base seam changed in this slice.

## Acceptance Mapping

### A1. OpenRouter text turn works through the shared boundary

- Met.
- `openrouter_text_only_turn_runs_through_shared_boundary` proves a session-engine turn can call the new adapter, receive normalized text/usage/completion events, and persist `ClientManagedHistory` continuation state.

### A2. Stateless follow-up replay works from the same journal family

- Met.
- `openrouter_follow_up_replays_history_from_journal_without_fake_provider_state` proves the second turn is reconstructed from journal-backed LOGOS history and sent without any fake `previous_response_id`.

### A3. Unsupported controls and limit behavior are explicit

- Met.
- `openrouter_unsupported_parameter_and_limit_behavior_is_explicit` now proves a malformed/plain-text `429` still normalizes to `ProviderErrorKind::RateLimited` through the real adapter path, and structured output requests fail as `UnsupportedCapability`.
- `map_turn_request` also rejects tools, non-default reasoning controls, non-client-managed continuations, and non-text content parts in the current validation slice.
- `reports_client_managed_validation_capabilities` proves the current validation mode advertises `supports_structured_output == false`.
- `malformed_400_falls_back_to_invalid_request`, `malformed_401_falls_back_to_authentication`, and `malformed_429_falls_back_to_rate_limited` prove malformed non-`2xx` fallback now follows explicit HTTP-status mapping.

### A4. OpenAI path remains unchanged in semantics

- Met for the current scope.
- `openai_no_regression_scope_still_uses_previous_response_id` proves the canonical server-managed continuation path still carries `OpenAiPreviousResponseId`.
- Existing OpenAI regression suites also continue to pass unchanged.

## Phase-5 Completion State

The strict phase-5 validation-adapter scope is now logically complete.

Completed in this phase:

1. `logos-provider-openrouter` exists as a real validation adapter;
2. one stateless text turn works through the shared boundary;
3. one follow-up turn replays LOGOS-owned journal state;
4. unsupported controls are explicit and current structured-output capability truth is honest;
5. OpenAI semantics remain preserved.

Still out of scope for this phase:

1. OpenRouter tool-calling support beyond the text-only validation slice;
2. OpenRouter structured output enablement beyond explicit rejection in the current model/scope posture;
3. any claim that OpenRouter reasoning carryover semantics are known or parity-complete.

## Codex-Base Seams Used

Phase-5 uses only the earlier registered seams.

1. Prompt tool injection seam
2. explicit continuation carryover seam
3. turn-state transfer hook seam

No new seam was required for the phase-5 OpenRouter validation-adapter slice.

## Tests Run

1. `cargo test -p logos-provider-openrouter`
2. `cargo test -p logos-session-engine --test phase5_openrouter_validation`
3. `cargo test -p logos-session-engine --test phase1_boundary`
4. `cargo test -p logos-session-engine --test phase4_persistence`
5. `cargo test -p logos-provider-openai`
6. `just fix -p logos-provider-openrouter`
7. `just fix -p logos-session-engine`
8. `just fmt`

## Verdict

Phase-5 is closed for the current strict validation-adapter scope.

## Evidence

- path: `../../../codex-rs/logos-provider-openrouter/src/adapter.rs`
  symbol: `OpenRouterAdapter`, `OpenRouterSession`
  lines: `14-174`
- path: `../../../codex-rs/logos-provider-openrouter/src/map_request.rs`
  symbol: `map_turn_request`
  lines: `49-193`
- path: `../../../codex-rs/logos-provider-openrouter/src/responses.rs`
  symbol: `map_response`, `map_http_error_response`, `map_http_error_fallback`
  lines: `68-246`
- path: `../../../codex-rs/logos-session-engine/tests/phase5_openrouter_validation.rs`
  symbol: `openrouter_text_only_turn_runs_through_shared_boundary`
  lines: `33-89`
- path: `../../../codex-rs/logos-session-engine/tests/phase5_openrouter_validation.rs`
  symbol: `openrouter_follow_up_replays_history_from_journal_without_fake_provider_state`
  lines: `91-197`
- path: `../../../codex-rs/logos-session-engine/tests/phase5_openrouter_validation.rs`
  symbol: `openrouter_unsupported_parameter_and_limit_behavior_is_explicit`
  lines: `199-253`
- path: `../../../codex-rs/logos-provider-openrouter/src/responses.rs`
  symbol: `malformed_400_falls_back_to_invalid_request`, `malformed_401_falls_back_to_authentication`, `malformed_429_falls_back_to_rate_limited`
  lines: `307-336`
- path: `../../../codex-rs/logos-session-engine/tests/phase5_openrouter_validation.rs`
  symbol: `openai_no_regression_scope_still_uses_previous_response_id`
  lines: `255-279`
- path: `2026-04-02-openrouter-validation-adapter-strict-spec.md`
  symbol: `Acceptance and Tests`
  lines: `103-108`

## Links

- [[README.md]]
- [[2026-04-02-openrouter-validation-adapter-strict-spec.md]]
- [[../architecture/2026-04-02-logos-provider-openrouter-adapter-spec.md]]
- [[2026-04-03-codex-core-seam-index.md]]
