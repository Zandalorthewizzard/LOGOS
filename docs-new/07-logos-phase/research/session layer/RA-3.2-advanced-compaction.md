# RA-3.2 — Advanced Compaction Strategies

## LOGOS Current Implementation

Not applicable — basic compaction not yet implemented. See RA-2.2 for base compaction analysis.

## OpenCode Analysis

### Current Compaction (Base)

OpenCode implements single-level compaction: summarize entire history → replace with summary + recent messages. See RA-2.2 for details.

### Advanced Patterns Present

1. **Tool output pruning** (`compaction.ts:93-139`):
   - Walk backwards through tool call parts
   - Protect last 40K tokens of tool outputs
   - Mark older ones as compacted
   - On reconstruction: `[Old tool result content cleared]`

2. **Compaction boundary in message model** (`message-v2.ts:903-919`):
   - `filterCompacted()` walks backwards, stops at compaction boundary
   - Only post-boundary messages loaded for next turn
   - Pre-boundary messages still exist in DB but are not sent to model

3. **Overflow replay** (`compaction.ts`):
   - If overflow occurred mid-task, original user message replayed after compaction
   - Ensures task continuity despite context compression

### Advanced Strategies NOT Present

OpenCode does NOT implement:

- Hierarchical compaction (different levels for different recency)
- Sliding window + summary
- Model-specific compaction adaptation
- Tool-aware differential compression
- Incremental compaction (only new messages since last compaction)

### What OpenCode's Approach Reveals

The pruning + summarize approach is pragmatic and sufficient for their use case. The key insight is that **tool output is the primary source of context bloat**, and pruning it is often enough without full summarization.

## Goose Analysis

### Advanced Compaction

Goose's compaction is **single-level summarization** only. The `context_mgmt/` module containing the actual algorithm is missing from the checkout.

**Evidence from available code:**

1. **`compaction.md` prompt template** (`prompt_template.rs:15-16`): A single template for summarizing conversation history. No evidence of multiple compaction strategies or levels.

2. **`AgentEvent::HistoryReplaced`** (`scheduler.rs:893`): Full conversation replacement — no evidence of selective pruning, hierarchical levels, or incremental compaction.

3. **`manages_own_context()`** (`base.rs:675`): Provider opt-out flag. CLI wrapper providers (Claude Code: `claude_code.rs:640`, Gemini CLI: `gemini_cli.rs:188`) set this to `true`, skipping goose-side compaction entirely.

4. **Token counting for overflow detection** (`token_counter.rs`): Single method `count_everything()` — no evidence of selective counting (e.g., counting only tool outputs, counting only recent messages).

### Advanced Strategies NOT Present

Goose does NOT implement:

- Tool output pruning (no protection window)
- Hierarchical compaction
- Sliding window + summary
- Model-specific compaction adaptation
- Incremental compaction (only new messages)
- Two-phase approach (prune first, then summarize)
- Compaction boundary markers in message model

### What Goose Has That Others Don't

1. **Provider opt-out via `manages_own_context()`**: Clean mechanism for providers that handle their own context. Neither OpenCode nor Codex has this.

2. **Canonical model registry for context limits**: `canonical_models.json` provides accurate context limits per model, enabling proper overflow detection.

## Cross-Project Comparison Matrix

| Strategy                  | LOGOS (Codex) | LOGOS (OR) | OpenCode    | Goose (inferred)          |
| ------------------------- | ------------- | ---------- | ----------- | ------------------------- |
| Tool output pruning       | No            | N/A        | Yes         | No                        |
| Full summarization        | Server-side   | N/A        | Local model | Local (compaction.md)     |
| Hierarchical levels       | No            | N/A        | No          | No                        |
| Sliding window            | No            | N/A        | No          | No                        |
| Model-specific adaptation | No            | N/A        | No          | No                        |
| Incremental compaction    | No            | N/A        | No          | No                        |
| Provider opt-out          | N/A           | N/A        | No          | Yes (manages_own_context) |

## Recommendations for LOGOS

### For MVP

Implement OpenCode's two-phase approach (prune + summarize) as documented in RA-2.2. This is sufficient for most use cases.

### For Advanced (Post-MVP)

1. **Tool-type-aware pruning**: Different tool types have different compression potential. Shell output can be aggressively pruned. File reads can be summarized. Error messages should be preserved. OpenCode treats all tool output uniformly.

2. **Incremental compaction**: Only summarize messages since last compaction, not entire history. Reduces cost and preserves older summary quality.

3. **Model-aware thresholds**: Different models have different context windows. Compaction thresholds should adapt per-model.

4. **Quality metrics**: Measure compaction effectiveness: how often does the model lose context after compaction? OpenCode doesn't measure this.
