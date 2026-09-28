# RA-3.3 — Multi-Provider Session Handoff

## LOGOS Current Implementation

Not implemented. Sessions are bound to a provider at creation time. The `ContinuationHandle` enum is provider-aware:

- `OpenAiPreviousResponseId(String)` — OpenAI
- `ClientManagedHistory` — stateless providers
- `ProviderSessionId(String)` — future providers
- `Opaque(String)` — generic

The `ProviderCapabilitySet` declares provider capabilities at adapter level.

## OpenCode Analysis

### Provider Resolution at Call Time

**File:** `src/provider/provider.ts`, `src/session/llm.ts`

OpenCode resolves providers **per turn**, not per session:

- Session stores `providerID` and `modelID` in each user/assistant message
- Each message can use a different provider/model
- `LLM.stream()` resolves the provider SDK at call time
- No session-level provider binding

### How This Enables "Handoff"

Because each message records its own provider/model, OpenCode effectively supports multi-provider conversations without special handoff logic:

1. User starts conversation with OpenAI
2. User switches model to Claude mid-conversation
3. Next message uses Claude, full history replayed
4. Previous messages show they were from OpenAI (different `providerID`)
5. No session restart needed

### Provider Transform Pipeline

**File:** `src/provider/transform.ts`

`ProviderTransform` normalizes messages across providers:

- Message format adaptation (tool call IDs, content structure)
- Schema transformation (Gemini integer enums → string enums)
- Temperature control (per-model capability)
- Reasoning content handling (interleaved vs separate)
- Caching headers (Anthropic cache control)

This pipeline enables seamless provider switching by normalizing history to the target provider's expectations.

### Comparison

| Aspect             | LOGOS                     | OpenCode                       |
| ------------------ | ------------------------- | ------------------------------ |
| Provider binding   | At thread creation        | Per message (deferred)         |
| Provider switching | New thread required       | Switch mid-conversation        |
| Message tracking   | Provider-agnostic history | Per-message providerID/modelID |
| Normalization      | Per-adapter mapping       | ProviderTransform pipeline     |
| Continuation model | Provider-specific handle  | Full replay (stateless)        |

## Goose Analysis

### Multi-Provider Architecture

Goose supports **30+ providers** with a factory pattern and per-session provider/model storage.

**Key files:**

- `crates/goose/src/providers/base.rs` — Provider trait (1160 lines)
- `crates/goose/src/providers/init.rs` — Provider registry and factory (273 lines)
- `crates/goose/src/providers/provider_registry.rs` — Dynamic registration
- `crates/goose/src/providers/openrouter.rs` — OpenRouter with cross-provider routing
- `crates/goose/src/session/session_manager.rs` — Per-session provider_name, model_config
- `crates/goose/src/model.rs` — ModelConfig with canonical limits

### Provider Registry

**`init.rs`**: Global `REGISTRY: OnceCell<RwLock<ProviderRegistry>>` with ~30 providers registered:

- `create(provider_name, model, extensions)` — factory method
- `create_with_default_model(name, extensions)` — uses provider's default model
- `create_with_named_model(name, model_name, extensions)` — with canonical limit lookup

### Per-Session Provider Storage

**Session record** stores:

- `provider_name: Option<String>` — which provider
- `model_config: Option<ModelConfig>` — model config with context limits, temperature, etc.
- `goose_mode: GooseMode` — execution mode

These are persisted in SQLite and restored on session load.

### Mid-Session Provider Switching

**ACP server** (inferred from `goose-acp/src/server.rs`):

1. **Set model**: `on_set_model()` creates new `ModelConfig` → new provider via factory → `agent.update_provider(provider, session_id)`
2. **Set mode**: `on_set_mode()` calls `agent.update_goose_mode(mode, session_id)`

`Agent::update_provider()` swaps the provider at runtime — no session restart needed.

### Session Copy Preserves Provider

`copy_session()` preserves `provider_name`, `model_config`, and `goose_mode` from the original session (lines 1440-1448). Forked sessions continue with the same provider.

### OpenRouter Cross-Provider Routing

`openrouter.rs` adds Anthropic-specific cache control headers when routing to Anthropic models through OpenRouter. This demonstrates provider-aware optimization even within a single provider adapter.

### Canonical Model Registry

`crates/goose/src/providers/canonical/` provides:

- `canonical_models.json` — known models with context limits, output limits, modalities
- `map_to_canonical_model()` — normalize provider-specific model names
- Context limits auto-populated from registry when creating `ModelConfig`

### Comparison

| Aspect               | Goose                                             |
| -------------------- | ------------------------------------------------- |
| Provider binding     | Per session (stored in session record)            |
| Mid-session switch   | Yes (agent.update_provider)                       |
| Per-message tracking | No (provider at session level, not message level) |
| Provider count       | 30+ (OpenAI, Anthropic, Google, Ollama, etc.)     |
| Dynamic registration | Yes (ProviderRegistry with runtime additions)     |
| Transform pipeline   | Per-provider implementation (formats/ directory)  |
| Canonical model data | Yes (canonical_models.json)                       |
| Fast model fallback  | Yes (complete_fast tries fast model first)        |

## Cross-Project Comparison Matrix

| Aspect                        | LOGOS           | OpenCode                  | Goose                                 |
| ----------------------------- | --------------- | ------------------------- | ------------------------------------- |
| Provider binding              | Thread creation | Per message               | Per session (stored in session record |
| Mid-conversation switch       | No              | Yes                       | Yes (update_provider + set_model)     |
| Per-message provider tracking | No              | Yes                       | No (provider stored at session level) |
| Transform pipeline            | Per-adapter     | Unified ProviderTransform | Per-provider implementation           |

## Recommendations for LOGOS

### What to Borrow

1. **Per-message provider/model tracking**: Record which provider/model generated each message. This is useful for debugging, cost tracking, and enabling future handoff.

2. **Provider transform pipeline**: Centralized normalization of messages for each provider. OpenCode's `ProviderTransform` is a good model.

### Architecture Note

LOGOS should not implement full provider handoff for MVP. But the architecture should:

- Store provider/model per message (cheap to add now)
- Keep `ContinuationHandle` provider-agnostic
- Not assume single provider per thread

### Risks

1. **Capability mismatches**: Switching from a model with tool calling to one without breaks conversation flow. Need graceful degradation.

2. **Token counting differences**: History that fits in OpenAI context may not fit in another provider's. Need overflow handling on handoff.
