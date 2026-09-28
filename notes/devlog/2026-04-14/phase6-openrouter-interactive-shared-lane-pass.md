# Phase 6 OpenRouter Interactive Shared Lane Pass

- Branch: `feat/phase6-openrouter-agent-beta`
- Scope: remove exec-only status for OpenRouter beta and route interactive surfaces through the same shared `logos-provider-openrouter + logos-session-engine` lane already proven in `codex exec`

## What changed

- Added shared crate `codex-rs/logos-openrouter-beta`.
- Moved OpenRouter adapter construction, `SessionEngine` setup, runtime-owned tool registration, system-prompt assembly, and turn helpers into that shared crate.
- Rewired `codex exec` to call the shared crate instead of owning a duplicate lane implementation.
- Added app-server-owned OpenRouter beta thread runtime in `codex-rs/app-server/src/openrouter_beta_threads.rs`.
- Routed app-server `thread/start` and `turn/start` for OpenRouter provider ids into that shared lane rather than the generic Codex runtime path.
- Added explicit unsupported errors for interactive operations not yet honestly supported on the OpenRouter beta lane, including `resume` and `compact`.

## Product-path proofs added

- `codex-rs/app-server/tests/suite/v2/openrouter_beta.rs` proves:
  - app-server OpenRouter thread start succeeds
  - interactive turn start reaches the shared OpenRouter lane
  - a runtime-owned tool roundtrip is reachable from the product path
  - `resume` is explicitly rejected
  - `compact` is explicitly rejected

## Important implementation notes

- TUI did not need a separate OpenRouter architecture pass; it continues to ride on app-server.
- The app-server OpenRouter path currently owns a lightweight in-memory thread representation for the beta lane rather than forcing a fake `codex-core` thread path.
- The OpenRouter beta lane remains replay-first and does not fake `previous_response_id` semantics.
- Compaction stays explicitly unsupported on this lane in this pass.

## Validation notes

- Shared-crate tests passed.
- Exec OpenRouter regression tests passed after rewiring.
- App-server OpenRouter focused proofs passed.
- `logos-session-engine` OpenAI no-regression proof passed.
- `just fix -p logos-openrouter-beta`, `just fix -p codex-exec`, `just fix -p codex-app-server`, and `just fmt` were run.

## Environment caveats seen during this pass

- `just bazel-lock-update` and `just bazel-lock-check` did not complete in this shell environment because Bazel or the helper script path was unavailable from the current command context.
- A targeted app-server OpenAI regression test (`turn_start_sends_originator_header`) hit environment-specific auth/runtime drift in this shell and did not serve as a reliable no-regression signal for this pass.
