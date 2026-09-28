# Stateless Session Layer — Discussion Plan

**Date:** 2026-04-16
**Status:** Draft
**Scope:** Architecture audit and implementation roadmap for extracting stateless session mechanics from the OpenRouter adapter into a provider-agnostic layer, and building missing capabilities.

---

## Context

The OpenRouter (OR) adapter currently implements session management mechanics that are not inherently OR-specific. These compensate for the lack of server-side state (which OpenAI provides via `previous_response_id` and `/responses/compact`). Any future stateless provider (Gemini, local models, etc.) will need the same mechanics.

The current `logos-session-engine` correctly owns core session mechanics (history accumulation, replay, tool loop, approvals, journal). But thread management, persistence, turn lifecycle, notification dispatch, and orchestration are trapped inside `openrouter_beta_threads.rs` (2426 lines) and `openrouter_beta_persistence.rs` (267 lines).

This document organises all required work into three tiers by complexity and risk, with mandatory competitive analysis.

---

## Tier 1: Migrate from OR Adapter + Audit

> Already works but lives at the wrong abstraction level. Must lift into a shared layer, audit against Codex (OpenAI path) and competitors (goose, opencode), and identify weaknesses and growth vectors.

### 1.1 Stateless Thread Manager

**What migrates:** `OpenRouterBetaThreadManager` → shared `StatelessThreadManager`

| Mechanic                                                | Current Location                     | Codex Equivalent                   |
| ------------------------------------------------------- | ------------------------------------ | ---------------------------------- |
| In-memory thread map (`HashMap<ThreadId, Arc<Thread>>`) | `openrouter_beta_threads.rs:73-76`   | `ThreadManagerState.threads`       |
| Thread state struct                                     | `openrouter_beta_threads.rs:84-90`   | `CodexThread` + session state      |
| Thread listing (in-memory + persisted merge)            | `openrouter_beta_threads.rs:264-296` | `list_threads_common()`            |
| Loaded thread tracking                                  | `openrouter_beta_threads.rs`         | `ThreadManager::list_thread_ids()` |

**Audit scope:**

- Compare with `ThreadManager` in Codex — what minimal trait allows both managers to coexist behind a unified dispatch?
- Goose: how do they manage sessions? Is there persistence?
- OpenCode: how is session lifecycle structured?
- Weaknesses: no TTL on in-memory threads, no eviction policy, no limit on concurrently loaded threads
- Growth vectors: LRU eviction for inactive threads, shared thread state for multiple connections

### 1.2 Session Persistence Layer

**What migrates:** `openrouter_beta_persistence.rs` → shared `StatelessSessionPersistence`

| Mechanic                                        | Current Location                       | Codex Equivalent               |
| ----------------------------------------------- | -------------------------------------- | ------------------------------ |
| JSON snapshot (history + continuation + config) | `openrouter_beta_persistence.rs:24-47` | Rollout JSONL + state DB       |
| Active/archived directory structure             | `persistence.rs:19-21`                 | Codex sessions directory       |
| Resume from persisted state                     | `openrouter_beta_threads.rs:413-479`   | `resume_thread_from_rollout()` |
| Config snapshot persistence                     | `persistence.rs:34-47`                 | Config in rollout SessionMeta  |

**Audit scope:**

- Codex uses append-only JSONL (rollout). OR uses monolithic JSON snapshot. Trade-offs: append-only is resilient to partial corruption; monolithic snapshot is simpler but catastrophic on corruption.
- Goose: how do they persist context? SQLite? Files? Not at all?
- OpenCode: how do they store sessions?
- Weaknesses: snapshot grows linearly with history, no incremental save, no compression
- Growth vectors: incremental persistence (append-only like rollout), compression of old history, shared persistence interface

### 1.3 Fork / Resume Orchestration

**What migrates:** fork and resume logic → shared layer

| Mechanic                                       | Current Location                       | Codex Equivalent                                    |
| ---------------------------------------------- | -------------------------------------- | --------------------------------------------------- |
| Fork (copy history → new SessionEngine)        | `openrouter_beta_threads.rs:1075-1178` | `ThreadManager::fork_thread()` + rollout truncation |
| Resume (load snapshot → rebuild SessionEngine) | `openrouter_beta_threads.rs:413-479`   | `resume_thread_from_rollout()`                      |
| Config restore on resume                       | `openrouter_beta_threads.rs:413-479`   | Config from rollout SessionMeta                     |

**Audit scope:**

- Codex resume via JSONL (resilient to partial corruption). OR resume via monolithic JSON (corruption = total loss). Can we use append-only for stateless too?
- Goose: fork/resume capabilities?
- OpenCode: fork/resume capabilities?
- Weaknesses: fork copies entire history array — no COW (copy-on-write)
- Growth vectors: COW for fork (shared history pointer + divergence marker), validation layer on resume (model unavailable, config stale, etc.)

### 1.4 Turn Lifecycle & Notification Dispatch

**What migrates:** turn tracking and notification emission → shared layer

| Mechanic                                                                 | Current Location                       | Codex Equivalent                      |
| ------------------------------------------------------------------------ | -------------------------------------- | ------------------------------------- |
| Active turn tracking (turn_id, status)                                   | `openrouter_beta_threads.rs:1930-1971` | `AgentStatus` watch channel           |
| Notification construction (ItemStarted/Completed, TurnStarted/Completed) | `openrouter_beta_threads.rs:809-977`   | EventMsg → ServerNotification mapping |
| Dynamic tool item mapping (ToolCallRecord → ThreadItem)                  | `openrouter_beta_threads.rs:1875-1928` | Inside CodexThread event handling     |

**Audit scope:**

- Codex emits EventMsg through listener, app-server maps to ServerNotification. OR constructs notifications manually. How much of the mapper can be unified?
- Goose: event pipeline architecture
- OpenCode: event streaming approach
- Weaknesses: manual emission in OR can skip events on errors (no transactional guarantee)
- Growth vectors: event sourcing (all producers write to unified event log), replay capability for reconnected clients

### 1.5 Shell Command Execution

**What migrates:** shell execution from OR → shared layer

| Mechanic                      | Current Location                     | Codex Equivalent                    |
| ----------------------------- | ------------------------------------ | ----------------------------------- |
| Shell exec + output streaming | `openrouter_beta_threads.rs:646-769` | `Op::RunUserShellCommand` → exec.rs |
| Output capture + persistence  | OR custom capture                    | Codex `StdoutStream::Record`        |

**Audit scope:**

- Codex exec.rs is mature (sandbox, timeout, background terminal support). OR shell is simplified. What is lost when switching to shared?
- Goose: how do they execute commands? Docker? Direct? Sandbox?
- OpenCode: command execution model
- Critical: shell execution is runtime-owned, not provider-dependent. Must be fully shared.

### 1.6 Review Orchestration

**What migrates:** review from OR → shared layer

| Mechanic                                   | Current Location                       | Codex Equivalent             |
| ------------------------------------------ | -------------------------------------- | ---------------------------- |
| Inline review (review turn on same thread) | `openrouter_beta_threads.rs:1180-1769` | `Op::Review` on CodexThread  |
| Detached review (fork + review on fork)    | `openrouter_beta_threads.rs:1180-1769` | Fork + Op::Review on fork    |
| Review prompt generation                   | `openrouter_beta_threads.rs:2327-2333` | `review_prompts.rs` in Codex |

**Audit scope:**

- Codex review spawns sub-agent CodexThread (own SQ/EQ). OR review runs turn through SessionEngine. Two different engines. How to unify?
- Goose: code review capability?
- OpenCode: code review capability?

### 1.7 Archive/Unarchive + Metadata

**What migrates:** simple local operations

| Mechanic                   | Current Location           | Codex Equivalent               | Migration Complexity                       |
| -------------------------- | -------------------------- | ------------------------------ | ------------------------------------------ |
| Archive (move file)        | OR: move JSON              | Codex: move rollout + state DB | Low — shared interface + different storage |
| Unarchive                  | Reverse of archive         | Reverse                        | Low                                        |
| Thread name set            | OR: mutate state + persist | `Op::SetThreadName`            | Low                                        |
| Metadata update (git info) | OR: mutate state + persist | state DB update                | Low                                        |
| Elicitation counter        | OR: local counter          | CodexThread counter            | Low                                        |

**Audit scope:**

- Minimal. These are CRUD operations. Main question: trait interface for storage backend.

---

## Tier 2: Build from Scratch + Mandatory Competitive Analysis

> Does not exist for stateless providers. Must be designed from scratch. Mandatory analysis of goose and opencode source code.

### 2.1 Rollback Engine

**What:** Roll back N last turns from history, rebuild session with truncated context.

**Codex does:** `Op::ThreadRollback` — drops last N turns from in-memory `ResponseItem` array. Does NOT touch files. Simple array truncation. After truncation, client continues with `previous_response_id` (server-side state is untouched).

**Why not portable:** Codex rollback relies on server-side state remaining intact. For stateless providers: truncate history → persist → rebuild SessionEngine.

**Must build:**

1. History truncation logic (remove last N user/assistant pairs + associated tool calls)
2. Session rebuild with truncated history
3. Persistence update after truncation
4. Ghost snapshot integration (optional — filesystem rollback)

**Competitive analysis (mandatory):**

- Goose: do they handle undo/rollback? Concept of turn boundaries?
- OpenCode: rollback capability? State restoration?
- What can be borrowed?

**Key discussion questions:**

- Truncation boundary: what counts as "one turn" for rollback? User message + all tool calls + assistant response? Or only user/assistant pairs?
- File changes: Codex rollback does NOT revert files. Should LOGOS offer filesystem rollback as an option?
- Partial tool executions: if a turn was interrupted mid-tool-chain — rollback entire turn or to interruption point?

### 2.2 Local Compaction Engine

**What:** Compress conversation history when context approaches model's token limit.

**Codex does:** Calls `/responses/compact` — server summarizes and returns compressed transcript. Also has auto-compact when tokens exceed threshold (`auto_compact_token_limit`).

**Why not portable:** OpenAI `/responses/compact` is a proprietary endpoint. No other provider offers this.

**Must build:**

1. Token counting / estimation for current history
2. Trigger threshold (auto-compact trigger)
3. Compaction strategy: send history to model with summarization prompt
4. History replacement: old history → summary + last N turns
5. Continuation handle update (reset — history changed)
6. Persist compacted state

**Competitive analysis (mandatory):**

- Goose: how do they manage growing context? Compaction? Truncation? Summarization?
- OpenCode: context window management? Automatic compression?
- Both projects work with multiple providers — how do they solve the context window problem?

**Key discussion questions:**

- Trigger threshold: percentage of context window? Absolute token count?
- Preserve last N turns without compression (as "recent context")?
- Summarization prompt: Codex uses server-side — we don't know the exact prompt. Need experimentation.
- What if compaction doesn't reduce context enough? Iterative compaction?
- How to notify user that context was compacted (ContextCompacted notification)?
- Keep pre-compaction history for possible rollback?

### 2.3 Turn Interrupt for Stateless Providers

**What:** Ability to abort the current model turn.

**Codex does:** `Op::Interrupt` → CancellationToken → streaming stops → TurnAborted.

**Why not portable for stateless:** For HTTP request to OR — can simply abort the connection. But must:

1. Correctly clean up state (partial history, partial tool calls)
2. Save what was already received (for possible resume)
3. Emit correct notifications

**Competitive analysis (mandatory):**

- Goose: how do they interrupt a turn? Abort HTTP request?
- OpenCode: interrupt implementation?

**Key discussion questions:**

- Interrupt during streaming: close connection immediately or wait for token/message boundary?
- Interrupt during tool execution: wait for completion or kill the tool?
- Partial state after interrupt: save received tokens to history? If yes, how to mark as "interrupted"?
- Interrupt during multi-tool turn (model called 3 tools, 2 completed, 3rd in progress): roll back all or keep results of first two?

### 2.4 Background Terminal Management (Unified)

**What:** Background process management for stateless provider path.

**Codex does:** `UnifiedExecProcessManager` — full-featured process manager with timeout, stdin/stdout, interaction.

**Why not portable directly:** `UnifiedExecProcessManager` is tightly coupled to `CodexThread` via Op dispatch. Needs adapter.

**Must build:**

1. Generic background process manager (not tied to CodexThread)
2. Integration with SessionEngine for tool execution
3. Clean/terminate operations
4. Output streaming and persistence

**Competitive analysis (mandatory):**

- Goose: background process management? Concept of background terminals?
- OpenCode: long-running process handling?

**Key discussion questions:**

- Processes bound to thread or to session (app-server)? What happens to processes when thread is evicted from memory?
- Timeout policy: configurable? Default?
- Persist list of background processes for resume?

---

## Tier 3: Build from Scratch — High Complexity and Risk

> Requires deep design work. High regression risk. Architectural decisions with long-term consequences.

### 3.1 Turn Steering (Mid-turn Input Injection)

**What:** Inject additional user input into the currently running model turn.

**Codex does:** `CodexThread::steer_input()` — places input into `pending_input` queue. On next iteration of sampling loop (`codex.rs:5948`), merges with primary input. Works because CodexThread owns the streaming loop.

**Why extremely complex:**

1. **Streaming loop ownership.** In Codex, steering is built into the `run_turn` sampling loop — an iterative process of "prompt → response → tool call → execute → prompt → ...". For stateless providers via SessionEngine, this loop exists too, but interrupt + re-prompt mid-turn requires:
   - Abort current HTTP request
   - Save partial results
   - Construct new request with merged input
   - Send again

2. **Provider limitations.** OpenRouter may not support clean partial cancellation — aborting a request may be "dirty". Results unpredictable.

3. **State consistency.** If model already returned tool calls and tool execution started — steering input may contradict current state.

**What to watch for:**

- Not all turns are steerable (review, compact — no). Must distinguish steerable vs non-steerable turn types.
- Timing: steering must queue and wait for next sampling iteration, not interrupt current one.
- Fallback strategy: if steering is impossible — queue input for next turn?

**Competitive analysis (mandatory):**

- Goose: mid-turn steering capability?
- OpenCode: similar capability?
- Likely absent — rare feature. Worth confirming.

### 3.2 Advanced Compaction Strategies

**Why high complexity (extension of 2.2):**

Basic compaction (summary) is tier 2. Advanced strategies are tier 3:

1. **Sliding window + summary.** Keep last N turns fully + summary of everything before. Requires accurate token counting.
2. **Hierarchical compaction.** Different compression levels for different recency. Recent turns — full, medium — partially compressed, old — summary.
3. **Compaction with tool call awareness.** Tool calls and results compress differently than regular messages (shell output can be summarized, arguments cannot).
4. **Model-specific compaction.** Different models have different context windows. Strategy must adapt.

**Risks:**

- Loss of critical context during compression → model loses understanding of the task
- Compaction prompt itself may consume many tokens → paradox
- Cascading compaction: if summary is too long, next compaction compresses the summary → quality degradation
- Testing: how to verify compaction doesn't break functionality? Need quality metrics.

### 3.3 Multi-Provider Session Handoff

**What:** Switch provider mid-conversation without losing context.

**Why extremely complex:**

1. Each provider has different capability flags. Transitioning from OpenAI (server-side state) to OR (stateless) mid-conversation requires full history reconstruction.
2. Reasoning tokens (encrypted) — specific to OpenAI. Lost when switching to OR.
3. Tool definitions may differ between providers.
4. Token counting differs between models — what fits in OpenAI context window may not fit in OR.

**What to watch for:**

- Post-MVP feature, but architecture must not preclude it.
- `ContinuationHandle` is already abstracted — good foundation.
- Discussion needed: is this worth doing at all, or should "new provider = new thread" be the model?

### 3.4 Streaming Optimization for Stateless Providers

**What:** Optimize full-history replay on every turn.

**Why complex:**

1. With long history, every turn resends megabytes of data. Need optimization:
   - Context caching (if provider supports prefix caching)
   - Delta compression
   - Transport-level compression
2. OpenRouter may support prompt caching — needs investigation.
3. During tool call loop (multiple prompt→response→tool iterations per turn) — each sub-request also replays full history.

**What to watch for:**

- Latency: full history replay adds delay on every turn.
- Cost: every token is re-sent → double payment for input tokens.
- Reliability: large payloads → higher chance of network errors → retry with even larger payload.

---

## Priority Order

### Tier 1 migration sequence:

```
1.7 Archive/Unarchive/Metadata        ← simplest, start here
1.1 Stateless Thread Manager           ← foundation for everything
1.2 Session Persistence Layer          ← depends on 1.1
1.3 Fork/Resume Orchestration          ← depends on 1.1 + 1.2
1.4 Turn Lifecycle & Notifications     ← depends on 1.1
1.5 Shell Command Execution            ← independent
1.6 Review Orchestration               ← depends on 1.1 + 1.4
```

### Tier 2 build sequence:

```
2.3 Turn Interrupt                     ← simplest new feature, high UX value
2.1 Rollback Engine                    ← depends on 1.2 (persistence)
2.4 Background Terminal Management     ← independent, moderate complexity
2.2 Local Compaction Engine            ← most complex in tier 2, depends on 1.1 + 1.2
```

### Tier 3 sequence (post-MVP):

```
3.1 Turn Steering                      ← high UX value but risky
3.2 Advanced Compaction Strategies     ← evolution of 2.2
3.4 Streaming Optimization             ← performance, post-MVP
3.3 Multi-Provider Session Handoff     ← far future, but don't break architecture
```

---

## Competitive Analysis Targets

For every item in tiers 1-3, the following must be analyzed before implementation:

| Project      | Repository                      | Focus Areas                                                                      |
| ------------ | ------------------------------- | -------------------------------------------------------------------------------- |
| **Goose**    | github.com/block/goose          | Session lifecycle, context management, tool execution, undo/rollback, compaction |
| **OpenCode** | github.com/opencode-ai/opencode | Multi-provider architecture, session persistence, streaming, interrupt           |

For each competitor, document:

1. How they solve the same problem (or whether they don't)
2. What patterns can be borrowed
3. What mistakes to avoid
4. What LOGOS can do better

---

## Evidence

- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: 72-2426
- path: `codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: 1-267
- path: `codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine`
  lines: 1-700
- path: `codex-rs/core/src/thread_manager.rs`
  symbol: `ThreadManager`
  lines: 192-600
- path: `codex-rs/core/src/codex_thread.rs`
  symbol: `CodexThread`
  lines: 46-100
- path: `codex-rs/protocol/src/protocol.rs`
  symbol: `Op`
  lines: 214-511
- path: `codex-rs/app-server-protocol/src/protocol/common.rs`
  symbol: `ClientRequest`
  lines: 230-578
