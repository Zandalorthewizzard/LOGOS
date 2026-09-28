---
id: logos-item-04-turn-lifecycle-implementation-reference
title: Item 04 Turn Lifecycle & Notification Dispatch Implementation Reference
type: implementation-reference
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    turn-lifecycle,
    notifications,
    implementation,
  ]
related:
  [
    [
      ../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.4-turn-lifecycle-notifications.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-17-item-04-turn-lifecycle-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.4-turn-lifecycle-notifications.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 04: Turn Lifecycle & Notification Dispatch Implementation Reference

## Purpose

Concrete code patterns extracted from Goose (Rust), OpenCode (TypeScript), and Codex (Rust) for each stage of the Item 04 draft implementation.

## Source Projects

| Project  | Language   | Key Files                                                                      | Lines |
| -------- | ---------- | ------------------------------------------------------------------------------ | ----- |
| Codex    | Rust       | `protocol/src/protocol.rs` — EventMsg enum (30+ variants), AgentStatus enum    | 4762  |
| Codex    | Rust       | `core/src/codex.rs` — EventMsg emission from turn execution                    | ~2300 |
| Goose    | Rust       | `crates/goose/src/tool_inspection.rs` — ToolInspector trait, InspectionManager | 302   |
| Goose    | Rust       | `crates/goose/src/tool_monitor.rs` — RepetitionInspector                       | 135   |
| OpenCode | TypeScript | `src/session/prompt.ts:105-134` — Runner pattern with BusyError                | 1906  |
| OpenCode | TypeScript | `src/session/processor.ts:185-213` — Doom loop detection                       | 523   |
| LOGOS OR | Rust       | `app-server/src/openrouter_beta_threads.rs:771-1044` — start_turn monolith     | 2426  |
| LOGOS OR | Rust       | `app-server/src/openrouter_beta_threads.rs:1973-1988` — send_notification      | 2426  |

---

## Stage A: Turn Lifecycle State Machine

### A.1 State Machine Enum

**Codex** (`protocol.rs:1606-1622`) — Directly relevant:

```rust
pub enum AgentStatus {
    PendingInit,
    Running,
    Interrupted,
    Completed(Option<String>),
    Errored(String),
    Shutdown,
    NotFound,
}
```

Codex already has a turn-status enum. LOGOS needs a similar but narrower enum focused on turn lifecycle (not agent lifecycle).

**LOGOS action:** Adapt Codex pattern. LOGOS `TurnLifecycleState`:

```rust
enum TurnLifecycleState {
    Idle,
    Running { turn_id: String, cancel: CancellationToken },
    Completed { turn_id: String },
    Failed { turn_id: String, error: TurnError },
    Interrupted { turn_id: String },
}
```

Key difference from Codex: LOGOS carries `turn_id` in each variant. This avoids separate `active_turn_id` field tracking. The `CancellationToken` in `Running` enables interrupt (Item 10).

### A.2 Atomic State Transition With Panic Safety

**No direct competitor equivalent.** Codex uses single-executor model (no concurrent turn risk). OpenCode uses Runner pattern but in TypeScript without Rust's panic concerns.

**LOGOS action:** Write from scratch.

```rust
struct TurnRunner {
    state: Mutex<TurnLifecycleState>,
}

impl TurnRunner {
    fn try_start(&self, turn_id: String) -> Result<CancellationToken, BusyError> {
        let mut state = self.state.lock().unwrap();
        match &*state {
            TurnLifecycleState::Idle => {
                let cancel = CancellationToken::new();
                *state = TurnLifecycleState::Running { turn_id, cancel: cancel.clone() };
                Ok(cancel)
            }
            _ => Err(BusyError),
        }
    }

    fn complete(&self, turn_id: &str) { /* Running → Completed */ }
    fn fail(&self, turn_id: &str, error: TurnError) { /* Running → Failed */ }
    fn reset(&self) { /* any terminal → Idle */ }
}
```

Panic safety: wrap spawned task with `AssertUnwindSafe` + catch_unwind. On panic: call `runner.fail(turn_id, panic_error)` + `runner.reset()`.

### A.3 Concurrency Guard (Runner Pattern)

**OpenCode** (`prompt.ts:105-134`) — Pattern-level reference:

```typescript
const getRunner = (runners, sessionID) => {
  const existing = runners.get(sessionID);
  if (existing) return existing;
  const runner = Runner.make(scope, {
    onIdle: () => {
      runners.delete(sessionID);
      status.set(sessionID, { type: "idle" });
    },
    onBusy: status.set(sessionID, { type: "busy" }),
    onInterrupt: lastAssistant(sessionID),
    busy: () => {
      throw new Session.BusyError(sessionID);
    },
  });
  runners.set(sessionID, runner);
  return runner;
};
```

Key properties:

- Per-session Runner instance
- `onIdle` callback cleans up runner from map
- `onBusy` publishes status
- `busy()` throws typed error

**LOGOS action:** Adapt pattern to Rust. LOGOS `TurnRunner` per thread (not per session like OpenCode). The `BusyError` maps directly to `invalid_request("thread already has an active turn")`. The callbacks (`onIdle`, `onBusy`) map to lifecycle state transitions that derive notifications.

### A.4 What LOGOS Must Write From Scratch

| Item                                                               | Why not refactorable                                                                     |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `TurnLifecycleState` with embedded `turn_id` + `CancellationToken` | Codex has `AgentStatus` but without embedded cancel. LOGOS needs turn-scoped state.      |
| Panic-safe state reset on spawn failure                            | No competitor handles Rust panic in this context.                                        |
| Atomic check-and-transition (Mutex-guarded)                        | Codex uses single-executor; OpenCode uses single-threaded JS. Rust needs explicit Mutex. |

---

## Stage B: Derived Notifications

### B.1 Typed Internal Events

**Codex** (`protocol.rs:1223-1302`) — Directly refactorable shape:

```rust
pub enum EventMsg {
    Error(ErrorEvent),
    TurnStarted(TurnStartedEvent),
    TurnComplete(TurnCompleteEvent),
    AgentMessage(AgentMessageEvent),
    AgentMessageDelta(AgentMessageDeltaEvent),
    McpToolCallBegin(McpToolCallBeginEvent),
    McpToolCallEnd(McpToolCallEndEvent),
    ContextCompacted(ContextCompactedEvent),
    // ... 30+ variants
}
```

Codex already has 30+ typed event variants. LOGOS needs a subset for stateless-provider lanes.

**LOGOS action:** Adapt. LOGOS `TurnEvent` enum covers the turn-scoped subset:

```rust
enum TurnEvent {
    TurnStarted { turn_id: String, thread_id: ThreadId },
    TurnCompleted { turn_id: String, thread_id: ThreadId, turn: Turn },
    TurnFailed { turn_id: String, thread_id: ThreadId, error: TurnError },
    ItemStarted { turn_id: String, thread_id: ThreadId, item: ThreadItem },
    ItemCompleted { turn_id: String, thread_id: ThreadId, item: ThreadItem },
    AgentMessageDelta { turn_id: String, thread_id: ThreadId, text: String },
}
```

This is smaller than Codex's `EventMsg` because LOGOS stateless lanes do not need all Codex events (realtime, MCP startup, model reroute, etc.).

### B.2 Centralized Event-to-Notification Mapper

**Codex** does this in the app-server listener: `EventMsg` → `ServerNotification`. The listener subscribes to the `EventMsg` channel and maps each variant.

**LOGOS action:** Adapt. Single function:

```rust
fn derive_notifications(event: TurnEvent) -> Vec<ServerNotification> {
    match event {
        TurnEvent::TurnStarted { turn_id, thread_id } => vec![
            ServerNotification::TurnStarted(TurnStartedNotification { thread_id, turn: ... }),
        ],
        TurnEvent::ItemStarted { item, thread_id, turn_id } => vec![
            ServerNotification::ItemStarted(ItemStartedNotification { item, thread_id, turn_id }),
        ],
        // ... exhaustiveness enforced by Rust compiler
    }
}
```

Key property: Rust's match exhaustiveness ensures every `TurnEvent` variant produces the correct notifications. Adding a new variant is a compile error until the mapper handles it.

### B.3 Transition-Triggers-Events Pattern

**No direct competitor equivalent** in the form LOGOS needs. Codex emits events from within the turn execution code. OpenCode derives from SyncEvent projectors. LOGOS needs transitions to produce events.

**LOGOS action:** Write from scratch. Each `TurnRunner` transition method produces events:

```rust
impl TurnRunner {
    fn start(&self, turn_id: String) -> Result<(CancellationToken, Vec<TurnEvent>), BusyError> {
        let mut state = self.state.lock().unwrap();
        match &*state {
            TurnLifecycleState::Idle => {
                let cancel = CancellationToken::new();
                *state = TurnLifecycleState::Running { turn_id: turn_id.clone(), cancel: cancel.clone() };
                let events = vec![
                    TurnEvent::TurnStarted { turn_id: turn_id.clone(), thread_id: ... },
                ];
                Ok((cancel, events))
            }
            _ => Err(BusyError),
        }
    }
}
```

The caller receives events and passes them to the notification bus. The state machine owns transition logic; notification delivery is separate.

---

## Stage C: Stream Processor Extraction

### C.1 Separation of Streaming From Lifecycle

**OpenCode** (`processor.ts`) — Pattern-level reference:

OpenCode has `SessionProcessor` as a dedicated component that:

- Manages streaming lifecycle of a single LLM call
- Maintains `ProcessorContext` tracking tool calls, reasoning, text parts
- Handles 14+ event types from AI SDK stream
- Cleans up incomplete parts on stream end

**Current LOGOS OR** does everything in one spawned task (`openrouter_beta_threads.rs:850-1034`):

- Runs `runtime.run_turn()` (streaming)
- Updates thread state (lifecycle)
- Persists snapshot (persistence)
- Emits notifications (dispatch)
- Handles errors (error lifecycle)

**LOGOS action:** Write from scratch. `StreamProcessor` component:

```rust
struct StreamProcessor {
    turn_id: String,
    thread_id: ThreadId,
}

impl StreamProcessor {
    async fn process_stream(
        &self,
        stream: impl Stream<Item = InternalEvent>,
        event_sink: mpsc::Sender<TurnEvent>,
    ) -> Result<StreamOutcome, TurnError> {
        // consume stream, emit TurnEvent per significant event
    }
}
```

The spawned task becomes:

```rust
tokio::spawn(async move {
    let processor = StreamProcessor::new(turn_id, thread_id);
    let stream = runtime.run_turn(messages).await;
    match processor.process_stream(stream, event_sink).await {
        Ok(outcome) => runner.complete(&turn_id),
        Err(error) => runner.fail(&turn_id, error),
    }
});
```

This is a thin orchestrator. All complexity lives in `StreamProcessor` and `TurnRunner`.

### C.2 Doom Loop Detection Inside Stream Processor

**Goose** (`tool_monitor.rs:34-80`) — Directly refactorable:

```rust
pub struct RepetitionInspector {
    max_repetitions: Option<u32>,
    last_call: Option<InternalToolCall>,
    repeat_count: u32,
    call_counts: HashMap<String, u32>,
}

impl RepetitionInspector {
    pub fn check_tool_call(&mut self, tool_call: CallToolRequestParams) -> bool {
        let internal_call = InternalToolCall::from_tool_call(&tool_call);
        let total_calls = self.call_counts.entry(internal_call.name.clone()).or_insert(0);
        *total_calls += 1;
        if self.max_repetitions.is_none() { /* skip */ return true; }
        if let Some(last) = &self.last_call {
            if last.matches(&internal_call) {
                self.repeat_count += 1;
                if self.repeat_count > self.max_repetitions.unwrap() { return false; }
            } else { self.repeat_count = 1; }
        } else { self.repeat_count = 1; }
        self.last_call = Some(internal_call);
        true
    }
}
```

**OpenCode** (`processor.ts:188-212`) — Pattern-level reference:

```typescript
const recentParts = parts.slice(-DOOM_LOOP_THRESHOLD)
if (recentParts.length !== DOOM_LOOP_THRESHOLD ||
    !recentParts.every(part => part.type === "tool" && part.tool === value.toolName &&
        JSON.stringify(part.state.input) === JSON.stringify(value.input))) {
    return  // not a doom loop
}
yield* permission.ask({ permission: "doom_loop", ... })
```

**LOGOS action:** Refactor Goose pattern. LOGOS `RepetitionInspector`:

```rust
struct RepetitionInspector {
    max_repetitions: u32,
    last_call: Option<(String, serde_json::Value)>,  // (tool_name, args)
    repeat_count: u32,
}
```

Differences from Goose:

- LOGOS uses `InternalMessage` tool call format, not `CallToolRequestParams`
- LOGOS emits `TurnEvent::DoomLoopDetected` instead of returning `bool`
- LOGOS threshold is configurable per-thread (from config snapshot)

Integration point: `StreamProcessor` calls `inspector.check()` before emitting tool call events. If check fails, `StreamProcessor` returns `Err(DoomLoop)`.

### C.3 Pluggable Inspector Framework

**Goose** (`tool_inspection.rs:33-54, 57-119`) — Directly refactorable:

```rust
#[async_trait]
pub trait ToolInspector: Send + Sync {
    fn name(&self) -> &'static str;
    async fn inspect(&self, session_id: &str, tool_requests: &[ToolRequest], ...) -> Result<Vec<InspectionResult>>;
    fn is_enabled(&self) -> bool { true }
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct ToolInspectionManager {
    inspectors: Vec<Box<dyn ToolInspector>>,
}

impl ToolInspectionManager {
    pub async fn inspect_tools(&self, ...) -> Result<Vec<InspectionResult>> {
        for inspector in &self.inspectors {
            if !inspector.is_enabled() { continue; }
            match inspector.inspect(...).await {
                Ok(results) => all_results.extend(results),
                Err(e) => { /* log, continue with other inspectors */ }
            }
        }
        Ok(all_results)
    }
}
```

**LOGOS action:** Adapt. LOGOS needs the same framework but with LOGOS types:

```rust
trait TurnInspector: Send + Sync {
    fn name(&self) -> &'static str;
    fn inspect(&self, context: &TurnContext) -> Vec<InspectionResult>;
}

struct InspectionManager {
    inspectors: Vec<Box<dyn TurnInspector>>,
}
```

`InspectionAction` enum (`Allow`, `Deny`, `RequireApproval`) is directly reusable from Goose.

---

## Cross-Cutting: What Codex Gets Right

| Pattern                                      | Source                  | Value for LOGOS                                |
| -------------------------------------------- | ----------------------- | ---------------------------------------------- |
| `EventMsg` typed enum with 30+ variants      | `protocol.rs:1223-1302` | Compile-time exhaustiveness for event handling |
| `AgentStatus` lifecycle enum                 | `protocol.rs:1606-1622` | Proven state model for agent/turn lifecycle    |
| Centralized `EventMsg` → notification mapper | app-server listener     | One place to update notification logic         |

## Cross-Cutting: What Goose Gets Right

| Pattern                                           | Source                      | Value for LOGOS                       |
| ------------------------------------------------- | --------------------------- | ------------------------------------- |
| `RepetitionInspector` with configurable threshold | `tool_monitor.rs:34-80`     | Simple, effective doom loop detection |
| `ToolInspector` trait with `InspectionAction`     | `tool_inspection.rs:33-54`  | Pluggable inspection framework        |
| `ToolInspectionManager` sequential execution      | `tool_inspection.rs:57-119` | Inspector chain with error isolation  |

## Cross-Cutting: What OpenCode Gets Right

| Pattern                                   | Source                 | Value for LOGOS                          |
| ----------------------------------------- | ---------------------- | ---------------------------------------- |
| Runner pattern with `BusyError`           | `prompt.ts:105-134`    | Clean concurrency guard                  |
| `onIdle`/`onBusy`/`onInterrupt` callbacks | `prompt.ts:121-131`    | Lifecycle hooks on state transitions     |
| `DOOM_LOOP_THRESHOLD` inline check        | `processor.ts:188-212` | Simpler alternative to Goose's inspector |

## Cross-Cutting: What Not To Import

| Pattern                             | Source           | Why not                                                  |
| ----------------------------------- | ---------------- | -------------------------------------------------------- |
| SyncEvent persistent event sourcing | OpenCode         | LOGOS has journal for canonical persistence              |
| `AgentEvent::HistoryReplaced`       | Goose (inferred) | LOGOS uses compaction events, not wholesale history swap |
| 30+ EventMsg variants               | Codex            | LOGOS stateless lanes need only turn-scoped subset       |

## Implementation Priority

| Step      | Action                                              | Source                 | Estimated Effort |
| --------- | --------------------------------------------------- | ---------------------- | ---------------- |
| 1         | `TurnLifecycleState` enum + `TurnRunner`            | Codex adapt + new      | 1.5 days         |
| 2         | Panic-safe spawn with state reset                   | Write from scratch     | 0.5 day          |
| 3         | `TurnEvent` typed enum (turn-scoped subset)         | Codex adapt            | 1 day            |
| 4         | `TurnEvent → ServerNotification` centralized mapper | Codex adapt            | 1 day            |
| 5         | Refactor `start_turn` to use transitions            | Adapt existing         | 2 days           |
| 6         | Refactor error path to use transitions              | Adapt existing         | 1 day            |
| 7         | `StreamProcessor` extraction                        | OpenCode pattern + new | 2 days           |
| 8         | `RepetitionInspector` (doom loop)                   | Goose refactor         | 1 day            |
| 9         | `TurnInspector` trait + `InspectionManager`         | Goose adapt            | 1 day            |
| 10        | Hook inspectors into StreamProcessor                | New                    | 0.5 day          |
| **Total** |                                                     |                        | **~11.5 days**   |

## Evidence

- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `EventMsg`
  lines: `1223-1302`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `AgentStatus`
  lines: `1606-1622`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `start_turn`
  lines: `771-1044`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `send_notification`
  lines: `1973-1988`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/tool_monitor.rs`
  symbol: `RepetitionInspector`
  lines: `34-80`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/tool_inspection.rs`
  symbol: `ToolInspector`
  lines: `33-54`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/tool_inspection.rs`
  symbol: `ToolInspectionManager`
  lines: `57-119`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/prompt.ts`
  symbol: `getRunner`
  lines: `105-134`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/processor.ts`
  symbol: `doom loop detection`
  lines: `185-213`
