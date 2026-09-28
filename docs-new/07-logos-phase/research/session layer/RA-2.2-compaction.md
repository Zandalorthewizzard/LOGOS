# RA-2.2 — Local Compaction Engine

## LOGOS Current Implementation

### Codex Path

**Where:** `codex-rs/core/src/tasks/compact.rs`, `codex-rs/core/src/client.rs` (`compact_conversation_history`)

Two strategies:

1. **Local compaction** (`compact.rs:54`): Sends history to model with summarization prompt, replaces with summary
2. **Remote compaction** (`compact_remote.rs:28`): Calls OpenAI `/responses/compact` endpoint — server returns compressed transcript
3. **Auto-compaction** (`codex.rs:5735`): Triggered pre-turn when token usage exceeds `auto_compact_token_limit`
4. **Mid-turn compaction** (`codex.rs:6042`): Triggered during turn when token limit reached

After compaction: `previous_response_id = None` (reset because history replaced).

### OR Path

**Where:** Rejected — compaction capability is `Disabled` in OR adapter flags.

**Why not implemented:** No server-side compaction endpoint available. Would need local implementation.

## OpenCode Analysis

### Compaction System

**File:** `src/session/compaction.ts`

### Trigger: Overflow Detection

**File:** `src/session/overflow.ts`

```typescript
isOverflow(count, model) {
  const usable = model.limit.input - reserved
  return count >= usable
}
```

`COMPACTION_BUFFER` = 20,000 tokens reserved for compaction output itself.

### Compaction Process

**File:** `src/session/compaction.ts:141-347`

Flow:

1. **Find compaction point**: Locates the user message to compact from
2. **Overflow handling**: If overflow, finds last non-compaction user message for replay after
3. **Model selection**: Uses `compaction` agent's model or falls back to user's model
4. **Prompt construction**: Sends conversation history + compaction template asking for structured summary
5. **Stream via processor**: Same streaming infrastructure as normal turns, with `summary: true` flag
6. **Post-compaction**:
   - On `"continue"` + auto mode: creates synthetic user message "Continue if you have next steps..."
   - On overflow replay: recreates original user message (stripped of media)
7. **Emits**: `session.compacted` bus event

### Tool Output Pruning

**File:** `src/session/compaction.ts:93-139`

`prune()` walks backwards through tool call parts:

- Protects last `PRUNE_PROTECT` (40,000) estimated tokens of tool outputs
- Marks older tool outputs as compacted (sets `time.compacted`)
- When compacted, tool output replaced with `[Old tool result content cleared]` during message reconstruction

### Compaction in Message Model

**File:** `src/session/message-v2.ts:903-919`

`filterCompacted()` walks messages backwards, stopping at compaction boundary (user message with compaction part, preceded by completed summary assistant message). Only messages after the boundary are loaded for the next turn.

### Agent Architecture

**File:** `src/agent/agent.ts`

Dedicated `compaction` agent:

- Mode: `primary` (hidden from user)
- Separate model possible (can use cheaper/faster model)
- Purpose: context summarization

### Comparison

| Aspect                   | LOGOS (Codex)                    | LOGOS (OR)    | OpenCode                                             |
| ------------------------ | -------------------------------- | ------------- | ---------------------------------------------------- |
| Compaction trigger       | Token threshold (auto) + manual  | Not supported | Token overflow + auto + manual                       |
| Compaction method        | Server-side `/responses/compact` | N/A           | Local model summarization                            |
| Compaction model         | Same as session                  | N/A           | Dedicated `compaction` agent (can use cheaper model) |
| Tool output handling     | Not specified                    | N/A           | Pruning with protection window (40K tokens)          |
| Post-compaction behavior | Continue session                 | N/A           | Synthetic "continue" message or replay original      |
| Compaction boundary      | Not explicit                     | N/A           | Explicit in message model (compaction parts)         |
| Buffer reservation       | Not explicit                     | N/A           | 20,000 tokens reserved for compaction output         |
| Compaction agent         | N/A                              | N/A           | Dedicated hidden agent with own config               |

### Reusable Patterns

1. **Two-phase compaction** (pruning + summarization): OpenCode first prunes old tool outputs (cheap, fast), then if still overflown, summarizes conversation (expensive, model call). This is practical — pruning alone may be enough.

2. **Dedicated compaction agent with cheaper model**: Use a smaller/cheaper model for summarization. The `compaction` agent can be configured independently. Smart cost optimization.

3. **Protection window for recent tool outputs**: Don't compact the last 40K tokens of tool output. Recent tool results are most relevant. Simple heuristic, effective in practice.

4. **Compaction boundary markers**: Explicit `compaction` parts in message model. Makes it clear where history was compressed. `filterCompacted()` uses this to only load post-compaction messages.

5. **Buffer reservation**: Reserve 20K tokens for compaction output itself. Prevents the compaction from triggering another overflow.

6. **Post-compaction synthetic message**: "Continue if you have next steps..." — keeps the conversation flowing naturally after compaction. User doesn't need to re-prompt.

7. **Overflow replay**: If overflow happened mid-task, the original user message is replayed after compaction so the model continues the task.

### What to Avoid

1. **Don't use server-side compaction**: Not available for stateless providers. Must be local.

2. **Don't summarize everything at once**: Pruning first, then summarization. Pruning is free and often sufficient.

## Goose Analysis

### Compaction Architecture

Goose has compaction support, but the core logic is in the **missing** `context_mgmt/` module. Evidence from available code:

**Key files:**

- `crates/goose/src/context_mgmt/` — **MISSING** (compaction logic)
- `crates/goose/src/prompt_template.rs:15-16` — `"compaction.md"` template registered: "Prompt for summarizing conversation history when context limits are reached"
- `crates/goose/src/providers/base.rs:675` — `fn manages_own_context(&self) -> bool` — flag to skip goose-side compaction
- `crates/goose/src/token_counter.rs` — `TokenCounter` with `count_chat_tokens()` for overflow detection
- `crates/goose/src/scheduler.rs:893` — `AgentEvent::HistoryReplaced(Conversation)` — compaction replaces full conversation

### Inferred Compaction Flow

1. **Token counting**: `TokenCounter::count_everything(system, messages, tools, resources)` counts total context tokens
2. **Overflow detection**: When token count approaches `ModelConfig::context_limit()`, trigger compaction
3. **Summarization**: Use `compaction.md` prompt template with the LLM to generate a summary
4. **History replacement**: Emit `AgentEvent::HistoryReplaced(new_conversation)` — the full conversation is replaced with a compacted version
5. **Persistence**: The replaced conversation is persisted via `session_manager.replace_conversation()`

### Provider-Specific Context Management

**`manages_own_context()`** (`base.rs:675`):

```rust
fn manages_own_context(&self) -> bool {
    false  // Default: goose handles compaction
}
```

Providers that return `true` (Claude Code, Gemini CLI) handle their own context management. Goose skips compaction for these providers because the provider's internal state is the source of truth.

### Token Counting Details

**`TokenCounter`** (`token_counter.rs`):

- Uses `tiktoken_rs::o200k_base()` (GPT-4o tokenizer)
- `count_chat_tokens(system, messages, tools)`: counts system + messages + tools + reply primer (3 tokens)
- Respects `message.metadata.agent_visible` — invisible messages excluded from count
- `count_tokens_for_tools(tools)`: estimates tool schema overhead with fixed constants
- `DashMap` concurrent cache (10K entries) for token count memoization
- Single tokenizer for all providers (approximation)

### Model Context Limits

**`ModelConfig::context_limit()`** (`model.rs:297-299`):

```rust
pub fn context_limit(&self) -> usize {
    self.context_limit.unwrap_or(DEFAULT_CONTEXT_LIMIT)  // 128,000
}
```

- Default: 128K tokens
- Set via: `GOOSE_CONTEXT_LIMIT` env var, provider-specific env var, or canonical model registry
- Canonical model data provides known limits per model (e.g., GPT-4o=128K, Claude Sonnet=200K)

### Key Differences

| Aspect                   | Goose (inferred)                               |
| ------------------------ | ---------------------------------------------- |
| Compaction trigger       | Token count vs context_limit                   |
| Compaction method        | Local summarization via `compaction.md` prompt |
| Compaction model         | Same model as session (no dedicated agent)     |
| Tool output handling     | Unknown (context_mgmt missing)                 |
| Post-compaction behavior | `AgentEvent::HistoryReplaced` → continue       |
| Provider opt-out         | `manages_own_context()` flag                   |
| Token counting           | tiktoken_rs (o200k_base), cached               |
| Context limit source     | Canonical model registry + env var override    |

### Reusable Patterns

1. **`manages_own_context()` flag**: Clean way to let providers opt out of client-side compaction. CLI wrappers (Claude Code, Gemini CLI) that handle their own context set this to `true`. LOGOS should adopt this pattern — stateful providers (OpenAI with `previous_response_id`) may not need client-side compaction.

2. **Canonical model registry for context limits**: Goose maintains a `canonical_models.json` with known context limits per model. This enables accurate overflow detection without manual configuration. LOGOS should maintain a similar registry.

3. **`HistoryReplaced` event**: Clean event type for compaction — consumers replace their entire conversation with the new one. Simple, atomic, unambiguous.

### What to Avoid

1. **Single tokenizer**: Goose uses one tokenizer for all providers. For accurate overflow detection, LOGOS should use provider-specific tokenizers or the provider's own token counting API.

2. **Full conversation replacement**: `replace_conversation()` does DELETE + INSERT all. For incremental compaction (only compact old messages), this is wasteful.

## Cross-Project Comparison Matrix

| Aspect               | LOGOS (Codex)    | LOGOS (OR) | OpenCode                     | Goose (inferred)                    |
| -------------------- | ---------------- | ---------- | ---------------------------- | ----------------------------------- |
| Compaction support   | Server-side      | None       | Local (prune + summarize)    | Local (summarize via compaction.md) |
| Trigger              | Token threshold  | N/A        | Overflow detection           | Token count vs context_limit        |
| Tool output pruning  | No               | N/A        | Yes (40K protection window)  | Unknown (context_mgmt missing)      |
| Dedicated model      | No               | N/A        | Yes (compaction agent)       | No (same model)                     |
| Buffer reservation   | Not explicit     | N/A        | 20K tokens                   | Not visible                         |
| Post-compaction flow | Reset + continue | N/A        | Synthetic message + continue | HistoryReplaced event               |
| Boundary markers     | No               | N/A        | Yes (compaction parts)       | No (full replacement)               |
| Provider opt-out     | N/A              | N/A        | No                           | Yes (manages_own_context)           |
| Token counting       | N/A              | None       | External (provider)          | tiktoken_rs with DashMap cache      |

## Recommendations for LOGOS

### What to Borrow

1. **Two-phase approach (prune → summarize)**: This is the most practical pattern. Start with tool output pruning (fast, free). Only summarize if still overflown (expensive, model call).

2. **Dedicated compaction agent**: Use a configurable agent for summarization. Allow cheaper model. This is a cost optimization that matters at scale.

3. **Protection window**: Preserve last N tokens of recent tool output. 40K is a reasonable default. Configurable.

4. **Buffer reservation**: Account for compaction output tokens in overflow calculation.

5. **Compaction boundary markers**: Mark where compaction happened in history. Enable loading only post-compaction history for next turn.

6. **Post-compaction synthetic message**: Keep conversation flowing naturally.

### Implementation Plan

1. **Token counting**: Implement or integrate token counting for the current history. Needed for overflow detection.

2. **Tool output pruning**: Walk history backwards, protect recent tool outputs, mark old ones as compacted.

3. **Summarization**: If pruning insufficient, send history to model with compaction prompt. Replace history with summary.

4. **Compaction agent**: Configure a dedicated agent (hidden, cheaper model) for summarization.

5. **Boundary markers**: Add compaction markers to history model.

6. **Auto-trigger**: Check token usage before each turn, trigger compaction if approaching limit.

### Risks and Open Questions

1. **Token counting accuracy**: Different providers count tokens differently. Need provider-aware counting or conservative estimation.

2. **Compaction quality**: How much context is lost? Need metrics. OpenCode uses a structured summary prompt — what structure works best?

3. **Cascading compaction**: If summary is too long, triggers another compaction. Need safeguards.

4. **Compaction during tool call chain**: If mid-turn (model called tools, context grew), compaction is harder. OpenCode handles this via overflow replay. Need to consider.
