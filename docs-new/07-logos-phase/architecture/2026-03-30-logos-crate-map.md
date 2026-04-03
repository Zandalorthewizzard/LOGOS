# Codex to Logos Crate Map

## 1. Purpose

- This document maps current Codex areas to the proposed Logos adapter-first architecture.
- It is a planning artifact for boundary extraction, not a promise that every Codex crate will be copied 1:1.
- It assumes the locked rules from `docs-temp/logos-adapter-boundary-spec.md` remain in force.

## 2. Mapping Rules

- Keep Codex as-is where it is already strong and upstream-sensitive.
- Wrap OpenAI/Responses-specific stacks before attempting generalization.
- Move only the minimum set of runtime contracts into Logos-owned crates.
- Prefer adapter-owned provider logic over polluting the shared session engine.

## 3. Keep / Wrap / Move / Defer Matrix

| Codex area | Current responsibility | Logos destination | Action |
|---|---|---|---|
| `codex-rs/core/src/tools/` | tool registry, router, handlers, orchestration | `logos-tool-runtime` | Keep mostly as-is, wrap boundary |
| `codex-rs/core/src/agent/` | agent lifecycle and control patterns | `logos-session-engine` or `logos-agent-runtime` later | Keep concepts, selective reuse |
| `codex-rs/mcp-server/` | expose Codex as MCP server | `logos-mcp-bridge` later | Defer direct migration |
| `codex-rs/rmcp-client/` | MCP client transport | `logos-tool-runtime` support layer | Keep or vendor with minimal changes |
| `codex-rs/codex-client/` | HTTP transport and retry | `logos-provider-openai` internals | Keep as implementation detail |
| `codex-rs/codex-api/` | Responses HTTP/WS clients, SSE parsing | `logos-provider-openai` internals | Wrap, do not generalize first |
| `codex-rs/core/src/client.rs` | model client orchestration for OpenAI path | `logos-provider-openai` facade + `logos-session-engine` seam | Wrap first |
| `codex-rs/core/src/model_provider_info.rs` | provider config for OpenAI-compatible providers | split across `logos-provider-api` and provider crates | Wrap then replace config surface gradually |
| `codex-rs/protocol/src/models.rs` | OpenAI wire items | `logos-provider-openai` only | Do not promote as shared runtime type |
| `codex-rs/protocol/src/protocol.rs` | event/submission protocol | reference for host/daemon event model | Selective borrowing only |
| `codex-rs/protocol/src/openai_models.rs` | model metadata and capabilities | split across `logos-runtime-types` and provider adapters | Selective copy/adapt |
| `codex-rs/lmstudio/` | local bootstrap for LM Studio | `logos-provider-openai-compat` or provider-specific crate later | Defer until after boundary proof |
| `codex-rs/ollama/` | local bootstrap for Ollama | `logos-provider-openai-compat` or provider-specific crate later | Defer until after boundary proof |
| `codex-rs/core/compact*` | OpenAI-oriented compaction | `logos-provider-openai` optional path | Keep provider-specific |
| `codex-rs/cli/`, `tui_app_server/` | CLI/TUI shell | none for first daemon wave | Defer |
| `codex-rs/app-server/` | app server protocol | reference only for daemon transport | Defer direct adoption |

## 4. Proposed Logos Crates

### `logos-runtime-types`

- Purpose:
  - small shared runtime contract
  - portable messages/events/tool definitions
  - continuation and capability shapes
- Must not contain:
  - raw Responses request/response structs
  - provider HTTP logic
  - OpenAI-only event enums

### `logos-provider-api`

- Purpose:
  - provider adapter trait
  - provider/model capability contract
  - provider error contract
- Must not contain:
  - OpenAI wire types
  - session orchestration policy

### `logos-provider-openai`

- Purpose:
  - canonical adapter over current Codex/OpenAI path
  - uses existing Responses stack internally
  - maps OpenAI-specific features into internal runtime events and extensions
- Internal borrow candidates:
  - `codex-client`
  - `codex-api`
  - pieces of `core/client.rs`
  - OpenAI model metadata and config mapping

### `logos-provider-openrouter`

- Purpose:
  - prove the boundary against a near-compatible but stateless provider
  - force client-managed-history correctness
- Strategy:
  - thin variant over openai-like adapter where possible
  - separate continuation policy

### `logos-session-engine`

- Purpose:
  - turn loop
  - retry and cancellation policy
  - continuation policy resolution
  - tool loop ownership
- Inputs:
  - `logos-runtime-types`
  - `logos-provider-api`
  - `logos-tool-runtime`

### `logos-tool-runtime`

- Purpose:
  - preserve Codex strength in tool routing, approvals, sandboxing, orchestration
  - consume normalized tool-call events
- Internal borrow candidates:
  - `codex-rs/core/src/tools/`
  - `codex-rs/rmcp-client/`
  - approval/sandbox pieces already proven in Codex

## 5. Phase-1 Concrete Ownership

### Logos-Owned From Day 1

- `logos-runtime-types`
- `logos-provider-api`
- `logos-provider-openai`
- `logos-session-engine` boundary layer

### Reused From Codex With Minimal Surgery

- Responses transport stack
- HTTP retry and transport stack
- tool runtime core
- MCP client building blocks

### Explicitly Deferred

- full agent tree migration
- CLI/TUI migration
- native Gemini/Z.AI/Anthropic adapters
- generalized persistence layer beyond what phase 1 needs

## 6. Recommended Extraction Order

1. Create `logos-runtime-types`.
2. Create `logos-provider-api`.
3. Wrap existing OpenAI path into `logos-provider-openai`.
4. Move the session loop to consume internal events.
5. Reconnect preserved Codex tool runtime through the new boundary.
6. Add `logos-provider-openrouter` as the first stress test.

## 7. Red Lines

- Do not move `ResponseItem` into shared Logos runtime APIs.
- Do not rewrite Responses SSE/WS mechanics just to make them look generic.
- Do not let provider-specific continuation semantics leak as ad hoc branching across the session engine.
- Do not let provider-native built-in tools become the only execution path.

## 8. Decision Summary

- Codex contributes its strongest subsystems as preserved implementation islands.
- Logos owns the runtime seam, capability model, and adapter contract.
- OpenAI remains first-class and upstream-friendly.
- Other providers enter through adapters, not through mutation of Codex internals into fake universality.
