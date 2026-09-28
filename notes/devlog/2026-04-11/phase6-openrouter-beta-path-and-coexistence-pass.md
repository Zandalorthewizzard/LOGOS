# Phase 6 OpenRouter Beta Path And Coexistence Pass

Date: 2026-04-11
Branch: `feat/phase6-openrouter-agent-beta`

## What This Pass Closed

1. Added a real built-in `openrouter` provider to the main config/runtime surface so users can select it through existing product paths such as `config.toml` and `-c model_provider=openrouter`.
2. Reused the existing provider-config retry and timeout knobs for OpenRouter-oriented custom provider ids, while leaving the built-in `openrouter` provider immutable.
3. Reserved the built-in `openrouter` provider ID so user config cannot silently override it.
4. Switched LOGOS home/state defaults away from Codex-owned storage:
   - `LOGOS_HOME`
   - default `~/.logos`
   - `LOGOS_SQLITE_HOME`
   - keyring service renamed to `LOGOS Auth`

## Important Reality

This pass did not close phase 6.

The main product config/runtime surface can now select the built-in OpenRouter provider honestly, but this pass did not re-architect the main Codex-derived runtime around `logos-session-engine`. The previously proven OpenRouter tool gate still exists in the dedicated LOGOS session-engine proof lane, and live OpenRouter replay behavior in the real wired product path remains unvalidated here.

## Support Statement Tightening

The built-in OpenRouter path now emits an explicit startup warning that beta support is validated only for `openai/o4-mini`, that replay-first history plus missing compaction replay remain current limitations, and that tuning should use a separate custom `openrouter-*` provider id.

## Checks Run

1. `cargo test -p codex-utils-home-dir`
2. `cargo test -p codex-state`
3. `cargo test -p codex-model-provider-info`
4. `cargo test -p codex-login`
5. `cargo test -p logos-provider-openrouter`
6. `cargo test -p codex-core openrouter_builtin_provider_can_be_selected_from_config`
7. `cargo test -p codex-core openrouter_custom_provider_can_tune_retry_and_timeout_knobs`
8. `cargo test -p codex-core reserved_openrouter_provider_override_rejected`
9. `cargo test -p logos-provider-openai`
10. `cargo test -p logos-session-engine --test phase5_openrouter_validation openai_no_regression_scope_still_uses_previous_response_id`
