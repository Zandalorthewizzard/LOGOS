# RA-3.4 — Streaming Optimization for Stateless Providers

## LOGOS Current Implementation

### Current State

OR adapter sends **full history on every turn** via HTTP POST. Each `SessionEngine::run_turn()`:

1. Accumulates full `Vec<InternalMessage>` history
2. Adds new user messages
3. Sends everything to OpenRouter API
4. Receives response

No optimization for reducing payload size or leveraging caching.

### Codex Path Optimization

- WebSocket with incremental input (`prepare_websocket_request()` in `client.rs:844-874`)
- Sends only delta items when input is a strict extension of previous
- Uses `previous_response_id` for server-side continuation
- Zstd compression for ChatGPT-authenticated requests

## OpenCode Analysis

### Full Replay Architecture

OpenCode also uses **full message replay** for every provider. No provider has server-side state in OpenCode's model.

### Optimization: Anthropic Cache Control

**File:** `src/provider/transform.ts:192-238`

`applyCaching()` adds `cacheControl: { type: "ephemeral" }` to:

- First 2 system messages
- Last 2 messages in conversation

This enables Anthropic's prompt caching — the provider caches the annotated prefix, reducing cost and latency on repeated requests.

### Optimization: LiteLLM Compatibility

**File:** `src/session/llm.ts:208-228`

When a proxy (LiteLLM) is used, tool call history may reference tools no longer active. OpenCode injects a dummy `_noop` tool to prevent API errors. Not a performance optimization but a compatibility fix that prevents failed requests and retries.

### No Other Optimizations

OpenCode does NOT implement:

- Request compression
- Delta/incremental requests
- Client-side caching of model responses
- Payload size reduction strategies
- Context caching for non-Anthropic providers

### Comparison

| Aspect              | LOGOS (Codex)                        | LOGOS (OR)  | OpenCode                |
| ------------------- | ------------------------------------ | ----------- | ----------------------- |
| Payload model       | Incremental via previous_response_id | Full replay | Full replay             |
| Request compression | Zstd (ChatGPT auth)                  | None        | None                    |
| Provider caching    | Server-side continuation             | None        | Anthropic cache_control |
| Delta requests      | WebSocket incremental                | None        | None                    |

## Goose Analysis

### Streaming Architecture

**Key files:**

- `crates/goose/src/providers/base.rs:514-521, 790-849` — Provider trait, MessageStream type
- Individual provider files (anthropic.rs, openai.rs, etc.) — SSE stream consumption

### MessageStream Type

```rust
pub type MessageStream = Pin<
    Box<dyn Stream<Item = Result<(Option<Message>, Option<ProviderUsage>), ProviderError>> + Send>,
>;
```

Each stream item yields:

- `Option<Message>` — partial text or complete tool calls
- `Option<ProviderUsage>` — token usage (usually in final chunk)

### Streaming Flow

1. **Provider trait**: `stream()` is the primary method all providers must implement
2. **`complete()`** (default impl): calls `stream()` then `collect_stream()` internally
3. **`complete_fast()`**: tries fast model first, falls back to regular on error
4. **`collect_stream()`** (`base.rs:800-849`): Consumes stream into `(Message, ProviderUsage)`, coalescing consecutive text blocks

### Text Coalescing

`collect_stream()` merges consecutive text content blocks:

- If previous chunk ends with text and new chunk starts with text → concatenate
- Non-text content (tool calls, images) breaks coalescing
- Tool calls are emitted once complete (not streamed token-by-token)

### Provider-Specific Streaming

- **OpenAI/Anthropic/etc.**: SSE-based streaming from HTTP APIs
- **CLI wrappers (Claude Code, Gemini CLI)**: Parse CLI stdout as message stream
- **ACP providers**: Stream from ACP protocol
- **Local inference**: Direct model output stream

### No Optimization Strategies

Goose does NOT implement:

- Request compression (no zstd/gzip)
- Provider caching hints (no Anthropic cache_control — though OpenRouter provider adds it conditionally)
- Incremental/delta requests (full replay every turn)
- WebSocket streaming for provider calls (HTTP SSE only)
- Prefix caching optimization

### OpenRouter-Specific Optimization

**File:** `crates/goose/src/providers/openrouter.rs:75`

OpenRouter provider conditionally adds Anthropic-specific cache control headers when using Anthropic models through OpenRouter. This is a provider-level optimization, not a framework-level one.

### Comparison

| Aspect              | Goose                                 |
| ------------------- | ------------------------------------- |
| Full replay         | Yes (every turn sends full history)   |
| Incremental/delta   | No                                    |
| Request compression | No                                    |
| Provider caching    | OpenRouter: conditional cache headers |
| Streaming model     | HTTP SSE for most providers           |
| Text coalescing     | Yes (consecutive text merged)         |
| Fast model fallback | Yes (`complete_fast()` with fallback) |

## Cross-Project Comparison Matrix

| Strategy            | LOGOS (Codex)        | LOGOS (OR) | OpenCode                | Goose                          |
| ------------------- | -------------------- | ---------- | ----------------------- | ------------------------------ |
| Full replay         | No (server state)    | Yes        | Yes                     | Yes                            |
| Incremental/delta   | WebSocket            | None       | None                    | None                           |
| Request compression | Zstd                 | None       | None                    | None                           |
| Provider caching    | previous_response_id | None       | Anthropic cache_control | OpenRouter conditional headers |
| Text coalescing     | N/A                  | N/A        | N/A                     | Yes (in collect_stream)        |
| Fast model fallback | N/A                  | N/A        | N/A                     | Yes (complete_fast)            |

## Recommendations for LOGOS

### What to Borrow

1. **Provider-specific cache headers**: Like OpenCode's Anthropic cache_control, LOGOS should add provider-specific optimization hints where available. OpenRouter may support prefix caching — investigate.

2. **Request compression**: Zstd compression (as Codex does for ChatGPT) is straightforward to add and reduces bandwidth for large payloads.

### Implementation Priority (Post-MVP)

1. **Investigate OpenRouter caching**: Does OpenRouter support any form of prompt caching or prefix caching? If yes, use it.
2. **Add request compression**: Zstd or gzip for large payloads. Low effort, high impact.
3. **Token estimation before send**: Estimate payload size before sending. If approaching limits, trigger pre-emptive compaction instead of sending and getting an error.

### Risks

1. **Latency**: Full replay means every turn is slower as history grows. The main mitigation is compaction, not transport optimization.

2. **Cost**: Every token is sent again on every turn. Transport optimization doesn't reduce token cost — only compaction does.
