# RA-2.3 — Turn Interrupt

## LOGOS Current Implementation

### Codex Path

**Where:** `codex-rs/core/src/codex.rs:4428` → `handlers::interrupt`

- `Op::Interrupt` received
- Calls `sess.interrupt_task()` → cancels via `CancellationToken`
- Streaming stops immediately
- Emits `EventMsg::TurnAborted`
- Background terminals NOT terminated (separate op)

### OR Path

**Where:** `codex-rs/app-server/src/codex_message_processor.rs:7742-7749`

Currently **rejected** with error: "Turn interrupt is not yet supported for OpenRouter beta threads."

**Why not implemented:** OR turn runs in spawned tokio task. The task calls `SessionEngine::run_turn()` which makes an HTTP request to OpenRouter. To interrupt:

1. Need to abort the HTTP connection
2. Clean up partial state in SessionEngine
3. Emit TurnAborted notification
4. Persist partial results (if any)

## OpenCode Analysis

### Cancel Mechanism

**File:** `src/session/prompt.ts:105-134` (Runner)

```typescript
runner.cancel(); // on interrupt
```

The Runner tracks the active operation and supports `cancel()`.

### Stream Interrupt

**File:** `src/session/processor.ts:367-414` (Cleanup)

On interrupt (via Effect's `onInterrupt`):

1. Finalize incomplete text parts (append "..." to indicate truncation)
2. Finalize incomplete reasoning parts
3. Mark running tool calls as error ("Tool execution aborted")
4. Set assistant message `time.completed`
5. Set session status to `idle`

### AbortController Lifecycle

**File:** `src/session/llm.ts:51-75`

The LLM stream is scoped to an `AbortController`:

- Created when stream starts
- `.abort()` cancels the HTTP request to the provider
- Scoped to Effect — automatic cleanup when scope ends
- On abort: the AI SDK `streamText()` throws, caught by processor's error handler

### Comparison

| Aspect           | LOGOS (Codex)                | LOGOS (OR)      | OpenCode                                           |
| ---------------- | ---------------------------- | --------------- | -------------------------------------------------- |
| Cancel mechanism | CancellationToken            | Not implemented | AbortController + Effect interrupt                 |
| HTTP abort       | SSE stream cancellation      | N/A             | AbortController.abort() → connection close         |
| Partial state    | TurnAborted event            | N/A             | Incomplete parts finalized with "..."              |
| Tool cleanup     | Tools continue to completion | N/A             | Running tools marked as "aborted"                  |
| Status update    | AgentStatus → Interrupted    | N/A             | Session status → idle                              |
| Persist partial  | No (turn discarded)          | N/A             | Yes (assistant message with partial content saved) |

### Reusable Patterns

1. **Scoped AbortController** (`llm.ts:51-75`): The abort lifecycle is tied to the streaming scope. Clean creation and cleanup. LOGOS should use tokio's `CancellationToken` similarly — create when turn starts, cancel on interrupt, cleanup when task ends.

2. **Partial state finalization** (`processor.ts:367-414`): OpenCode doesn't discard interrupted turns — it saves them with finalized partial content. This is better for UX: user can see what the model was generating before interrupt. LOGOS OR should save partial results too.

3. **Tool abort marking**: Running tool calls are explicitly marked as "aborted" rather than silently dropped. This provides clear history of what happened.

4. **Status reset on interrupt**: Session status returns to `idle` after interrupt. Simple but important for state consistency.

### What to Avoid

1. **Don't discard interrupted turns**: OpenCode saves partial content. LOGOS Codex discards via TurnAborted. The stateless layer should save partial results.

## Goose Analysis

### Cancellation Architecture

Goose uses `tokio_util::sync::CancellationToken` throughout for turn interruption.

**Key files:**

- `crates/goose/src/utils.rs:44-48` — `is_token_cancelled()` helper
- `crates/goose/src/scheduler.rs` — Job-level cancellation (lines 235-238, 619-622, 739-762)

### Agent-Level Cancellation (Inferred)

From `scheduler.rs:880`:

```rust
let stream = agent.reply(user_message, session_config, Some(cancel_token)).await?;
```

- `Agent::reply()` accepts `Option<CancellationToken>`
- The agent loop checks the token between iterations (inferred from ACP server code)
- Stream consumption checks `cancel_token.is_cancelled()` between events

### Scheduler Job Cancellation

**`kill_running_job()`** (`scheduler.rs:739-762`):

1. Verify job exists and is running
2. Look up `CancellationToken` in `running_tasks` map
3. Call `token.cancel()`
4. Job's execution loop detects cancellation and terminates

**Running tasks map** (`scheduler.rs:29`):

```rust
type RunningTasksMap = HashMap<String, CancellationToken>;
```

- Token inserted when job starts, removed when job completes
- `Mutex`-protected for concurrent access

### Cancellation Helper

```rust
pub fn is_token_cancelled(cancellation_token: &Option<CancellationToken>) -> bool {
    cancellation_token.as_ref().is_some_and(|t| t.is_cancelled())
}
```

### ACP Server Cancel (Inferred)

From exploration of `goose-acp/src/server.rs`:

- `on_cancel()` handler cancels the session's CancellationToken
- Session tracked in `sessions: Arc<RwLock<HashMap<String, ...>>>`
- Cancel triggers stream termination

### Key Differences

| Aspect           | LOGOS (Codex)                | LOGOS (OR)      | OpenCode                         | Goose                               |
| ---------------- | ---------------------------- | --------------- | -------------------------------- | ----------------------------------- |
| Cancel mechanism | CancellationToken            | Not implemented | AbortController + Effect         | CancellationToken                   |
| HTTP abort       | SSE stream cancellation      | N/A             | abort() → connection close       | Token checked between stream events |
| Partial state    | TurnAborted event            | N/A             | Incomplete parts finalized "..." | Not preserved (inferred)            |
| Tool cleanup     | Tools continue to completion | N/A             | Running tools marked "aborted"   | Not visible                         |
| Status update    | AgentStatus → Interrupted    | N/A             | idle                             | Job: currently_running → false      |
| Persist partial  | No                           | N/A             | Yes                              | No                                  |
| Scope            | Per CodexThread              | N/A             | Per streaming call               | Per reply call + per scheduler job  |

### Reusable Patterns

1. **CancellationToken per reply**: Goose passes `CancellationToken` to `Agent::reply()`. Simple, composable, tokio-native. LOGOS should use the same pattern for stateless providers.

2. **Running tasks map for job cancellation**: `HashMap<String, CancellationToken>` maps job IDs to cancel tokens. Clean pattern for managing multiple concurrent executions.

### What to Avoid

1. **No partial state preservation**: Goose appears to not save partial results on cancellation. OpenCode's pattern of finalizing incomplete content is better for UX.

2. **No tool abort handling visible**: No mechanism to mark in-progress tool calls as "aborted" in the available code.

## Cross-Project Comparison Matrix

| Aspect                | LOGOS (Codex)            | LOGOS (OR)      | OpenCode          | Goose                                |
| --------------------- | ------------------------ | --------------- | ----------------- | ------------------------------------ |
| Implementation        | CancellationToken        | Not implemented | AbortController   | CancellationToken (tokio_util)       |
| HTTP connection abort | SSE cancel               | N/A             | abort()           | Token checked between events         |
| Partial state saved   | No                       | N/A             | Yes               | No                                   |
| Tool abort handling   | N/A                      | N/A             | Mark as "aborted" | Not visible                          |
| Status reset          | AgentStatus::Interrupted | N/A             | idle              | currently_running → false            |
| Cleanup guarantee     | Manual                   | N/A             | Effect scoped     | Token cleanup in scheduler loop      |
| Per-job tracking      | N/A                      | N/A             | N/A               | RunningTasksMap (HashMap<Id, Token>) |

## Recommendations for LOGOS

### What to Borrow

1. **Save partial results on interrupt**: Don't discard. Save the interrupted assistant message with partial content. User can see what was generated. History remains coherent.

2. **Mark running tools as aborted**: Don't silently drop tool executions in progress. Mark them clearly in history.

3. **Abort HTTP connection**: The simplest way to interrupt a stateless provider turn. Close the HTTP connection. Provider stops generating. No special protocol needed.

### Implementation Plan

1. **CancellationToken per turn**: When `start_turn()` spawns the task, create a `CancellationToken`. Store it in thread state. On `turn/interrupt`, cancel the token.

2. **HTTP abort propagation**: SessionEngine's `run_turn()` must respect the cancellation token. When cancelled, abort the HTTP request to the provider.

3. **Partial finalization**: On interrupt:
   - Finalize any partial text content (mark as truncated)
   - Mark running tool calls as "aborted"
   - Save the assistant message with partial content
   - Set turn status to "interrupted"
   - Persist updated state

4. **Notification**: Emit TurnCompleted (with status=interrupted), not TurnAborted. This is more informative.

### Risks and Open Questions

1. **Provider behavior on abort**: OpenRouter may or may not gracefully handle connection closure. Need testing.

2. **Tool execution during interrupt**: If a tool is mid-execution (shell command running), should we wait for it or kill it? OpenCode kills. Codex keeps running. Decision needed.

3. **Race condition**: Cancel signal and stream completion can race. Need to handle the case where the turn completes just as interrupt is received.
