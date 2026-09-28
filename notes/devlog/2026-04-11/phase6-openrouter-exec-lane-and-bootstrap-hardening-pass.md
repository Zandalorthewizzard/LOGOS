# Phase 6 OpenRouter Exec Lane And Bootstrap Hardening Pass

Date: 2026-04-11
Branch: `feat/phase6-openrouter-agent-beta`

## What This Pass Closed

1. Added a real user-facing OpenRouter beta runtime path through `codex exec` that enters `logos-provider-openrouter` and `logos-session-engine` instead of the generic `codex-core` Responses client.
2. Added a reachable runtime-owned tool roundtrip on that exec beta lane with read-only workspace tools:
   - `workspace_list_dir`
   - `workspace_read_file`
3. Prevented interactive app-server and embedded TUI thread starts from silently using the generic Codex runtime for OpenRouter beta providers; those surfaces now fail explicitly instead of taking the wrong lane.
4. Hardened bootstrap dotenv filtering so `.logos/.env` cannot set or override internal `LOGOS_*` or `CODEX_*` variables.
5. Tightened the OpenRouter tuning posture so the built-in `openrouter` provider remains immutable and tuning is explicitly routed to separate custom `openrouter-*` provider ids.

## Important Reality

Phase 6 is still not closed.

This pass makes the real OpenRouter beta lane reachable from a user-facing product surface (`codex exec`) and removes the misleading dual-path state there, but interactive TUI/app-server OpenRouter support is still intentionally blocked rather than closed. Compaction replay also remains an explicit OpenRouter limitation.

## Checks Run

1. `cargo test -p codex-arg0`
2. `cargo test -p codex-model-provider-info`
3. `cargo test -p codex-exec openrouter_beta`
4. `cargo test -p codex-app-server openrouter_beta_provider_ids_are_marked_exec_only_for_app_server`
5. `cargo test -p codex-core openrouter_builtin_provider_can_be_selected_from_config`
6. `cargo test -p codex-core openrouter_custom_provider_can_tune_retry_and_timeout_knobs`
7. `cargo test -p codex-core reserved_openrouter_provider_override_rejected`
8. `cargo test -p logos-provider-openai`
9. `cargo test -p logos-provider-openrouter`
10. `cargo test -p logos-session-engine --test phase5_openrouter_validation openai_no_regression_scope_still_uses_previous_response_id`
