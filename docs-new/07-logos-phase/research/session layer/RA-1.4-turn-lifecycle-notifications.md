# RA-1.4 — Turn Lifecycle & Notification Dispatch

## LOGOS Current Implementation

### Turn Lifecycle (OR path)

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:771-1044`

Flow:

1. `start_turn()` — validate input, spawn tokio task
2. Task runs `OpenRouterBetaSession::run_turn()` via SessionEngine
3. Manual notification emission: ItemStarted, ItemCompleted, TurnStarted, TurnCompleted
4. `active_turn_id` tracking in `OpenRouterThreadState`
5. Persist thread after turn completion

### Notification Dispatch

**Where:** `openrouter_beta_threads.rs:809-977, 1973-1988`

- `send_notification()` helper constructs and sends `ServerNotification` variants
- Notifications emitted manually: TurnStarted, TurnCompleted, ItemStarted, ItemCompleted, AgentMessageDelta, CommandExecutionOutputDelta
- No event sourcing — notifications are fire-and-forget

### Codex Equivalent

- CodexThread owns the event channel — emits `EventMsg` variants through `tx_event`
- App-server listener translates `EventMsg` → `ServerNotification`
- AgentStatus watch channel tracks status transitions
- Events are typed: TurnStarted, TurnComplete, ExecCommandBegin, etc.

**Known weaknesses:**

- Manual notification construction in OR — easy to miss events, no compile-time guarantee
- No transactional notification delivery — partial state visible to clients
- Duplicate logic between OR and Codex paths

## OpenCode Analysis

### Turn Lifecycle

**File:** `src/session/prompt.ts:1337-1565` (runLoop)

The agent loop:

1. Set session status to `busy`
2. Load history via `MessageV2.filterCompactedEffect()`
3. Scan for pending tasks (compaction, subtask)
4. If nothing pending and last assistant finished → break
5. Otherwise: resolve tools, build system prompt, call LLM
6. Process stream via `SessionProcessor`
7. On completion: trigger `compaction.prune()` in background
8. Set session status to `idle`

### Runner (Concurrency Guard)

**File:** `src/session/prompt.ts:105-134`

Each session has a `Runner<MessageV2.WithParts>`:

- Tracks busy/idle state
- Throws `BusyError` if session already processing
- Supports `cancel()` — interrupts current operation
- On cancel: returns last assistant message

### Stream Processing (SessionProcessor)

**File:** `src/session/processor.ts`

Manages streaming lifecycle of a single LLM call:

- Captures filesystem snapshot before streaming
- Maintains `ProcessorContext` tracking tool calls, reasoning, text parts
- Handles 14+ event types from AI SDK stream
- Doom loop detection (3 consecutive identical tool calls)
- Error handling with retry policy
- Cleanup on stream end: finalize incomplete parts, mark aborted tools

### Event/Notification Model

**Two separate systems:**

1. **SyncEvent** (persistent, transactional):
   - `message.updated` — message creation/update
   - `message.removed` — message deletion
   - `message.part.updated` — part creation/update
   - `message.part.removed` — part deletion

2. **Bus** (in-process, async):
   - `session.status` — busy/idle transitions
   - `session.compacted` — compaction done
   - `session.error` — processing errors
   - `message.part.delta` — streaming text/reasoning deltas

### Key Differences

| Aspect                    | LOGOS (OR)                          | LOGOS (Codex)                      | OpenCode                     |
| ------------------------- | ----------------------------------- | ---------------------------------- | ---------------------------- |
| Turn execution            | Spawn tokio task + manual lifecycle | CodexThread Op dispatch loop       | Runner + runLoop iteration   |
| Concurrency guard         | None                                | Single executor                    | Runner with BusyError        |
| Status tracking           | active_turn_id in thread state      | AgentStatus watch channel          | In-memory Map + bus events   |
| Notification construction | Manual per-event                    | EventMsg→ServerNotification mapper | SyncEvent + Bus dual system  |
| Stream processing         | Inside spawned task                 | Inside CodexThread                 | SessionProcessor (dedicated) |
| Partial state handling    | Manual cleanup                      | CancellationToken + cleanup        | Effect interrupt + cleanup   |
| Event persistence         | None                                | None (transient)                   | SyncEvent (transactional)    |

### Reusable Patterns

1. **Runner-based concurrency** (`prompt.ts:105-134`): Clean pattern for preventing concurrent turn submissions. Should be adopted in LOGOS stateless layer.

2. **SessionProcessor as dedicated streaming handler** (`processor.ts`): Separates streaming concerns from session lifecycle. LOGOS OR mixes them in the spawned task. Extracting a processor-like component would be cleaner.

3. **Dual event system** (SyncEvent + Bus): Persistent events for data mutations, transient bus events for real-time notifications. This is a clean separation — LOGOS could use it to ensure notifications are derived from persisted state, not constructed manually.

4. **Doom loop detection** (`processor.ts:188-213`): If the same tool with same args is called 3 times consecutively, prompt user for permission. Prevents infinite tool call loops. LOGOS doesn't have this for OR path.

5. **Step boundaries** (step-start/step-finish events): OpenCode marks multi-step boundaries with snapshots and token accounting. This enables per-step diff tracking. LOGOS could use this for turn-level file change tracking.

### What to Avoid

1. **Manual notification construction**: OpenCode proves that notifications should be derived from state changes (SyncEvent projectors), not manually constructed. LOGOS OR's approach is error-prone.

2. **Mixing stream processing with lifecycle management**: OR's spawned task does both. OpenCode separates SessionProcessor (stream) from runLoop (lifecycle). Cleaner separation.

## Goose Analysis

### Architecture

Goose's turn lifecycle is split between available code (session, scheduler, tool inspection) and missing code (agent loop, event emission).

**Key files:**

- `crates/goose/src/session/session_manager.rs` — Session CRUD, message persistence
- `crates/goose/src/scheduler.rs` — Scheduled job execution with CancellationToken
- `crates/goose/src/tool_inspection.rs` — Tool inspection framework
- `crates/goose/src/tool_monitor.rs` — RepetitionInspector (doom loop detection)
- `crates/goose/src/action_required_manager.rs` — Permission request/response flow
- `crates/goose/src/session_context.rs` — Task-local session ID propagation
- `crates/goose/src/agents/` (MISSING) — Agent loop, AgentEvent stream, turn orchestration

### Inferred Turn Lifecycle

From usage in `scheduler.rs:880-901`:

```
Agent::reply(user_message, SessionConfig, CancellationToken) → Stream<AgentEvent>
```

1. Consumer calls `agent.reply(msg, session_config, cancel_token)` — returns event stream
2. Stream yields `AgentEvent::Message(Message)` — incremental messages (assistant text, tool calls, tool responses)
3. Stream may yield `AgentEvent::HistoryReplaced(Conversation)` — compaction event
4. Consumer iterates with `futures::StreamExt`, collecting messages into `Conversation`
5. On stream end: persist accumulated conversation + session metadata

### Concurrency

- **No explicit concurrency guard** in available code
- SQLite `BEGIN IMMEDIATE` provides transaction-level isolation for persistence
- No Runner-like busy/idle tracking visible
- ACP server may have session-level locking (not available for inspection)

### Notification / Event Model

**AgentEvent** (inferred from scheduler.rs):

- `AgentEvent::Message(Message)` — each message from the agent loop
- `AgentEvent::HistoryReplaced(Conversation)` — full conversation swap after compaction

**ActionRequiredManager** (`action_required_manager.rs`):

- Global singleton for pending user action requests
- `request_and_wait(message, schema, timeout)` → blocks until user responds or timeout
- Uses oneshot channels + timeout: `tokio::sync::oneshot` + `tokio::time::timeout`
- Sends `MessageContent::ActionRequired` with elicitation ID
- Response matched by request ID, returned via oneshot channel
- **Timeout**: Default timeout from caller (not configurable in manager)

### Doom Loop Detection

**RepetitionInspector** (`tool_monitor.rs`):

- Tracks consecutive identical tool calls (same name + same parameters)
- Configurable `max_repetitions` threshold
- When exceeded: returns `InspectionAction::Deny` with `finding_id: "REP-001"`
- Also tracks per-tool total call counts
- Implements `ToolInspector` trait (pluggable inspector framework)
- **Inspectors are read-only during inspection** (creates temp clone to check without modifying state)

### Session ID Propagation

**`session_context.rs`**:

- Task-local `SESSION_ID: Option<String>` via `tokio::task_local!`
- `with_session_id(id, future)` — scopes session ID for async context
- `current_session_id()` — retrieves from task-local
- Used for tracing/logging correlation

### Key Differences

| Aspect                    | LOGOS (OR)                            | Goose                                            |
| ------------------------- | ------------------------------------- | ------------------------------------------------ |
| Turn execution            | Spawned tokio task + manual lifecycle | `Agent::reply()` → Stream<AgentEvent> (inferred) |
| Concurrency guard         | None                                  | None visible (SQLite tx isolation only)          |
| Status tracking           | active_turn_id in thread state        | Not visible                                      |
| Notification construction | Manual per-event                      | AgentEvent stream (inferred)                     |
| Doom loop detection       | None                                  | RepetitionInspector (configurable threshold)     |
| Permission requests       | Elicitation counter                   | ActionRequiredManager (oneshot + timeout)        |
| Session ID propagation    | Thread-local                          | Task-local (`tokio::task_local!`)                |
| Tool inspection framework | None                                  | ToolInspector trait + ToolInspectionManager      |

### Reusable Patterns

1. **Pluggable ToolInspector framework** (`tool_inspection.rs`): Clean inspector trait with `inspect()` returning `Vec<InspectionResult>`. Multiple inspectors run sequentially. Results merged by `apply_inspection_results_to_permissions()` — Deny overrides Allow, RequireApproval overrides auto-Allow. LOGOS should adopt this pattern for a shared approval/inspection layer.

2. **RepetitionInspector with configurable threshold** (`tool_monitor.rs`): Configurable `max_repetitions`, per-tool tracking, finding IDs (`REP-001`). More configurable than OpenCode's hardcoded 3x threshold.

3. **ActionRequiredManager** (`action_required_manager.rs`): Clean request-response pattern with oneshot channels + timeout. Global singleton. LOGOS's elicitation counter is cruder — Goose's approach is more structured.

4. **Task-local session ID** (`session_context.rs`): Scoping session ID via `tokio::task_local!` is a good pattern for tracing. Available across await points without explicit parameter passing.

### What to Avoid

1. **No concurrency guard**: Goose has no protection against concurrent turns on the same session. LOGOS must add one.

2. **Global singleton for action requests**: `ActionRequiredManager::global()` is a global singleton. This doesn't scale for multi-session concurrent operation. LOGOS should scope it per-session or per-thread-manager.

## Cross-Project Comparison Matrix

| Aspect              | LOGOS (OR)          | LOGOS (Codex)         | OpenCode                   | Goose                                 |
| ------------------- | ------------------- | --------------------- | -------------------------- | ------------------------------------- |
| Turn execution      | Spawned tokio task  | CodexThread Op loop   | Runner + runLoop           | Agent::reply() → Stream<AgentEvent>   |
| Concurrency guard   | None                | Single executor       | Runner + BusyError         | None (SQLite tx only)                 |
| Status model        | active_turn_id      | AgentStatus enum      | idle/busy/retry Map        | Not visible                           |
| Notification source | Manual construction | EventMsg mapper       | SyncEvent + Bus            | AgentEvent stream (inferred)          |
| Event persistence   | None                | None                  | Yes (SyncEvent)            | None (transient stream)               |
| Stream processor    | Inline in task      | Inline in CodexThread | Dedicated SessionProcessor | Agent stream consumer (scheduler/ACP) |
| Doom loop detection | None                | None                  | Yes (3x threshold)         | RepetitionInspector (configurable)    |
| Tool inspection     | None                | None                  | None                       | Pluggable ToolInspector framework     |
| Permission flow     | Elicitation counter | Per-command approval  | Ruleset-based              | ActionRequiredManager + ACP protocol  |

## Recommendations for LOGOS

### What to Borrow

1. **Runner pattern**: Add concurrency guard to stateless thread manager. Prevent concurrent turns with clear error.

2. **Dedicated stream processor**: Extract streaming lifecycle from the OR spawned task into a reusable component. This processor handles event mapping, tool state tracking, and notification emission.

3. **Doom loop detection**: Add threshold-based detection for repeated identical tool calls. Simple and effective safety measure.

4. **Derive notifications from state changes**: Instead of manually constructing notifications, derive them from session state transitions. This reduces bugs and ensures consistency.

### What to Avoid

1. **Don't keep manual notification construction**: It's the biggest source of bugs in the OR path. Every state change should automatically produce the correct notification.

### What LOGOS Can Do Better

1. **Typed notification guarantee**: Use the type system to ensure every state transition produces the correct notification. Rust's enum exhaustiveness can enforce this at compile time.

2. **Step-level granularity**: OpenCode's step-start/step-finish markers enable per-step tracking. LOGOS could add similar granularity for better UI feedback.

### Risks and Open Questions

1. **Notification ordering**: If notifications are derived from state changes asynchronously, ordering guarantees must be preserved. OpenCode handles this via SyncEvent sequence numbers.

2. **Migration from manual to derived notifications**: Changing the OR notification system is invasive. Need to ensure no regressions in TUI behavior.
