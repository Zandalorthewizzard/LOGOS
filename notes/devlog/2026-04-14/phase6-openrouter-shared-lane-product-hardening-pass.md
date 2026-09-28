# Phase 6 OpenRouter Shared Lane Product Hardening Pass

- Branch: `feat/phase6-openrouter-agent-beta`
- Scope: make the shared OpenRouter beta lane behave more like a real product path by restoring standard HTTP client semantics, honoring the relevant custom tuning knobs, and making app-server thread-management behavior explicit for OpenRouter beta threads.

## Closed in this pass

- The shared lane now builds OpenRouter requests from the standard LOGOS/Codex reqwest client path instead of `reqwest::Client::new()`.
- OpenRouter beta requests now inherit standard client behavior such as default `originator`, `User-Agent`, residency header handling, sandbox `no_proxy`, and shared custom-CA loading.
- The OpenRouter adapter now receives shared-lane transport policy explicitly: merged default headers, provider query params, request retry budget, and request timeout.
- Custom `openrouter-*` provider tuning now affects runtime where honest on the current non-websocket lane:
  - `request_max_retries` -> retry budget for retryable transport/provider failures
  - `stream_idle_timeout_ms` -> request timeout cap for the current HTTP turn request path
- The current non-websocket lane still ignores, but now explicitly records as ignored in transport policy/tests:
  - `stream_max_retries`
  - `websocket_connect_timeout_ms`
- App-server OpenRouter thread management is no longer half-visible:
  - `thread/list` now includes OpenRouter beta threads
  - `thread/name/set` works for OpenRouter beta threads
  - `thread/metadata/update` works for OpenRouter beta threads
  - `thread/increment_elicitation` and `thread/decrement_elicitation` work for OpenRouter beta threads
  - `thread/archive` remains explicitly unsupported
  - `thread/unarchive` remains explicitly unsupported

## Proofs added

- `logos-openrouter-beta` tests now prove:
  - standard default headers reach OpenRouter requests
  - custom retry tuning reaches the shared runtime lane and changes behavior
  - ignored non-websocket knobs are explicitly classified as ignored
- `codex-app-server` OpenRouter tests now prove:
  - thread management APIs behave consistently for OpenRouter beta threads
  - supported operations mutate visible thread state
  - unsupported archive remains explicit

## Environment caveats

- `turn_start_sends_originator_header` remained unreliable in this shell due ambient auth/runtime drift on the existing OpenAI-side app-server path, so it was not used as the pass verdict signal.
- `just bazel-lock-update` / `just bazel-lock-check` were still not usable in this shell environment for the same reasons recorded in the previous pass.
