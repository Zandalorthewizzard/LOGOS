---
id: logos-item-04-turn-lifecycle-draft-implementation-discussion
title: Item 04 Turn Lifecycle & Notification Dispatch Draft Implementation Discussion
type: discussion
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    turn-lifecycle,
    notifications,
    research-transfer,
  ]
related:
  [
    [README.md],
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../research/session layer/RA-1.4-turn-lifecycle-notifications.md],
  ]
depends_on:
  [
    [
      2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md,
    ],
    [2026-04-14-logos-session-state-layer-extraction-discussion.md],
    [2026-04-16-item-01-thread-manager-draft-implementation-discussion.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 04: Turn Lifecycle & Notification Dispatch Draft Implementation Discussion

## Purpose

This document turns the validated `Item 04: Turn Lifecycle & Notification Dispatch` research verdict into a practical LOGOS implementation draft.

## Current Transfer Status

1. `Item 04` verdict is `ACCEPT` with `DUAL_CONSENSUS_VALID`.
2. The ACCEPT arises because the imported pattern — authoritative lifecycle state with derived notifications — directly matches LOGOS's existing ownership model (Layer 2 owns thread/turn lifecycle) and does not conflict with any locked decision (journal, OpenAI path, provider semantics).
3. This draft exists so future spec work can start from code-aware implementation shape instead of only from theory.

## Accepted Import For Item 04

1. Authoritative lifecycle state machine for turns and items.
2. Notifications derived from state transitions, not manually constructed.
3. Concurrency guard (Runner pattern) preventing concurrent turns on one thread.
4. Dedicated stream processor boundary separating streaming from lifecycle management.
5. Typed notification guarantee using Rust's enum exhaustiveness.
6. Doom loop detection for repeated identical tool calls.

## Explicitly Not Imported

1. **OpenCode SyncEvent (persistent event sourcing).** LOGOS already has phase-4 journal for canonical event persistence. SyncEvent would be a duplicate. Runtime lifecycle events (busy/idle, deltas) are transient and do not need persistent event sourcing.

## What This Draft Covers

1. Turn lifecycle state machine.
2. Notification derivation from state transitions.
3. Concurrency guard.
4. Stream processor boundary.
5. Doom loop detection.

## What This Draft Does Not Cover

These remain separate items:

1. Thread registry and ephemeral runtime (Item 01).
2. Session persistence schema (Item 02).
3. Fork/resume orchestration (Item 03).
4. Review orchestration (Item 06).
5. Shell command execution (Item 05).
6. Turn interrupt semantics (Item 10).
7. Turn steering (Item 12).
8. Compaction (Items 09/13).

## Current Code Diagnosis

### Manual Notification Construction Is The Core Problem

`start_turn` (`openrouter_beta_threads.rs:771-1044`) contains approximately 12 manual `send_notification` calls per turn:

**Before spawn (lines 809-845):** 3 calls

- `TurnStarted`
- `ItemStarted` for user message
- `ItemCompleted` for user message

**Success path in spawned task (lines 895-977):** 4-8 calls

- `ItemStarted` + `ItemCompleted` per tool item (variable count)
- `ItemStarted` + `ItemCompleted` for assistant message
- `TurnCompleted`

**Error path in spawned task (lines 1005-1031):** 2 calls

- `Error`
- `TurnCompleted(Failed)`

**Review path (lines 1180-1769):** The same manual pattern repeats with different payloads — approximately 15+ additional `send_notification` calls.

Every new notification type requires updating every path. There is no compile-time guarantee that all paths emit the correct sequence.

### Concurrency Guard Is Fragile

```rust
// openrouter_beta_threads.rs:790-795
let mut state = thread.state.lock().await;
if state.active_turn_id.is_some() {
    return Err(invalid_request(...));
}
state.active_turn_id = Some(turn_id.clone());
```

Problems:

1. Check and spawn are not atomic — race condition between two `start_turn` requests arriving concurrently.
2. If spawned task panics, `active_turn_id` is never cleared — thread is permanently stuck in "active" state.
3. No cancel/interrupt surface — once spawned, the turn cannot be stopped.

### Stream Processing Is Mixed With Lifecycle

The spawned task (`tokio::spawn` at line 850) does everything:

- Runs `runtime.run_turn()` (stream processing)
- Updates thread state (lifecycle management)
- Persists snapshot (persistence)
- Emits notifications (notification dispatch)
- Handles errors (error lifecycle)

These are five distinct responsibilities in one closure. Adding new behavior requires modifying this monolith.

### `send_notification` Helper Is Stateless

```rust
// openrouter_beta_threads.rs:1973-1988
async fn send_notification(
    outgoing: &OutgoingMessageSender,
    thread_state_manager: &ThreadStateManager,
    thread_id: ThreadId,
    notification: ServerNotification,
) {
    let connection_ids = thread_state_manager
        .subscribed_connection_ids(thread_id).await;
    if connection_ids.is_empty() { return; }
    outgoing.send_server_notification_to_connections(connection_ids.as_slice(), notification).await;
}
```

This helper only delivers. It does not know about lifecycle state, ordering, or completeness. Every call site must know what notification to send and when.

### Codex Already Has A Better Model

`EventMsg` enum (`protocol.rs:1223-1302`) — 30+ typed event variants including:

- `TurnStarted`, `TurnComplete`
- `AgentMessage`, `AgentMessageDelta`
- `McpToolCallBegin`, `McpToolCallEnd`
- `ContextCompacted`, `ThreadRolledBack`

`AgentStatus` enum (`protocol.rs:1606-1622`):

- `PendingInit`, `Running`, `Interrupted`, `Completed`, `Errored`, `Shutdown`, `NotFound`

Codex's approach: `CodexThread` emits `EventMsg` through a typed channel → app-server listener maps `EventMsg` → `ServerNotification`. The mapper is centralized.

This is the direction Item 04 adopts: typed internal events → centralized mapper → notifications.

## Draft Target Model

### 1. Turn Lifecycle State Machine

```
         ┌──────────────────────────────────────────┐
         │                                          │
  [Idle] ──start_turn──▶ [Running] ──complete──▶ [Completed] ──reset──▶ [Idle]
         │                  │                            │
         │                  │──error──▶ [Failed] ──reset──▶ [Idle]
         │                  │
         │                  │──interrupt──▶ [Interrupted] ──resume──▶ [Running]
         │                  │                             ──reset──▶ [Idle]
         │
         └──start_turn (already busy)──▶ Error: Busy
```

State transitions:

- `Idle → Running`: `start_turn` validates, acquires lock, transitions
- `Running → Completed`: stream ends successfully
- `Running → Failed`: stream ends with error
- `Running → Interrupted`: cancel received (Item 10)
- `Completed/Failed/Interrupted → Idle`: cleanup, release lock
- `Idle → Error: Busy`: second `start_turn` while busy

This state machine is owned by LOGOS session-state layer, not by provider lanes.

### 2. Derived Notifications

Each state transition produces zero or more notifications automatically:

| Transition                   | Derived Notifications                                                 |
| ---------------------------- | --------------------------------------------------------------------- |
| `Idle → Running`             | `TurnStarted`, `ItemStarted(user)`, `ItemCompleted(user)`             |
| `Running → Completed`        | `ItemStarted(assistant)`, `ItemCompleted(assistant)`, `TurnCompleted` |
| `Running → Failed`           | `Error`, `TurnCompleted(Failed)`                                      |
| Item completed (tool result) | `ItemStarted(tool)`, `ItemCompleted(tool)`                            |

The mapping from transitions to notifications lives in one place. Call sites trigger transitions; the state machine derives notifications.

### 3. Concurrency Guard

Adopt the Runner pattern from OpenCode:

```rust
struct TurnRunner {
    state: Mutex<TurnRunnerState>,
}

enum TurnRunnerState {
    Idle,
    Running { cancel: CancellationToken, turn_id: String },
}
```

Properties:

- `try_start()` atomically checks state and transitions to `Running`
- `cancel()` transitions to `Interrupted` via `CancellationToken`
- On panic: `Drop` guard ensures state resets to `Idle` or `Failed`
- No race condition: single Mutex protects state check + transition

This replaces the current `active_turn_id` check with a proper state machine.

### 4. Stream Processor Boundary

Extract streaming concerns from the spawned task into a dedicated component:

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  TurnRunner      │     │  StreamProcessor  │     │  NotificationBus │
│  (lifecycle)     │────▶│  (stream events)  │────▶│  (delivery)      │
│                  │     │                    │     │                  │
│  Idle/Running/   │     │  consume provider  │     │  resolve subs    │
│  Failed/Done     │     │  stream, emit      │     │  deliver to      │
│                  │     │  typed events      │     │  connections     │
└─────────────────┘     └──────────────────┘     └──────────────────┘
```

`TurnRunner` owns lifecycle state transitions.
`StreamProcessor` owns streaming event handling (tool calls, deltas, messages).
`NotificationBus` owns delivery to subscribers.

The current monolithic spawned task splits into these three components.

### 5. Doom Loop Detection

Adopt Goose's `RepetitionInspector` pattern:

```rust
trait TurnInspector {
    fn inspect(&self, context: &TurnContext) -> Vec<InspectionResult>;
}

struct RepetitionInspector {
    max_repetitions: u32,
    history: Vec<ToolCall>,
}
```

Properties:

- Configurable threshold (default: 3)
- Tracks consecutive identical tool calls (same name + same parameters)
- Returns `Deny` with finding ID when threshold exceeded
- Pluggable: `ToolInspector` trait allows future inspectors (cost, time, etc.)

This is a safety net that currently does not exist in the OR path.

### 6. What About SyncEvent?

OpenCode uses `SyncEvent` for persistent event sourcing — every message mutation is a transactional event stored in SQLite. This provides:

- Audit trail of all mutations
- Transactional ordering
- Replay capability

LOGOS does not need `SyncEvent` because:

- Phase-4 journal already provides canonical event persistence (`MessageAppended`, `TurnStarted`, `TurnFinished`, etc.)
- Runtime lifecycle events (busy/idle transitions, streaming deltas) are transient — they do not need persistence
- Adding SyncEvent would duplicate journal's role

Instead, LOGOS uses:

- **Journal** for canonical persistent events (phase-4)
- **Typed internal events** for transient runtime lifecycle (Item 04)
- **NotificationBus** for delivery to subscribers (Item 04)

The SyncEvent concept (derive from state changes, not manual construction) is adopted. The SyncEvent implementation (persistent SQLite-backed event sourcing) is not.

## Practical Draft Shape

### Stage A: Turn Lifecycle State Machine

1. Define `TurnLifecycleState` enum (`Idle`, `Running`, `Completed`, `Failed`, `Interrupted`).
2. Define transition methods that validate and produce notifications.
3. Replace `active_turn_id` check with `TurnRunner::try_start()`.
4. Add `Drop` guard for panic safety.

This stage makes turn lifecycle authoritative and replaces the fragile concurrency check.

### Stage B: Derived Notifications

1. Define `TurnEvent` enum (typed internal events: `TurnStarted`, `ItemCompleted`, etc.).
2. Implement centralized `TurnEvent → ServerNotification` mapper.
3. Refactor `start_turn` to trigger transitions instead of calling `send_notification` directly.
4. Refactor error path to trigger `Running → Failed` transition.

This stage eliminates manual notification construction.

### Stage C: Stream Processor Extraction

1. Extract `StreamProcessor` component from the spawned task.
2. `StreamProcessor` consumes provider stream events, produces `TurnEvent` instances.
3. `TurnRunner` receives events, updates lifecycle state, triggers notification derivation.
4. `NotificationBus` handles delivery (reuses existing `send_notification` helper internally).

This stage separates concerns and makes the spawned task a thin orchestrator.

### Stage D: Doom Loop Detection

1. Define `TurnInspector` trait.
2. Implement `RepetitionInspector` with configurable threshold.
3. Hook into `StreamProcessor` — inspect before executing tool calls.
4. `Deny` result triggers `Running → Failed` transition with appropriate error.

This stage adds a safety net. Can ship independently of Stages B and C.

## Why This Still Supports The ACCEPT Verdict

1. No conflict with phase-4 journal — journal records canonical events, lifecycle state machine tracks runtime state. Complementary, not competing.
2. No conflict with OpenAI canonical path — Codex already uses `EventMsg` + `AgentStatus`. Item 04 brings the same pattern to stateless-provider lanes.
3. No provider-sensitive semantics — lifecycle state is provider-agnostic. Provider differences are in stream content, not in turn lifecycle.
4. Ownership Layer 2 already owns "thread and turn lifecycle" — Item 04 implements this correctly.
5. `SyncEvent` is explicitly not imported because journal covers the same need.

## Revisit Triggers

The ACCEPT verdict should be re-opened only if:

1. Provider-specific turn semantics prove that a provider-agnostic state machine cannot correctly capture all turn outcomes.
2. Codex upstream introduces a shared turn lifecycle abstraction that LOGOS should reuse directly instead of building its own.
3. Performance profiling shows that the notification derivation layer adds unacceptable latency to streaming paths.

## Closeout Expectation

The later strict spec derived from this draft should be considered successful only if:

1. `start_turn` no longer contains direct `send_notification` calls.
2. Turn concurrency is enforced through a state machine with panic safety.
3. All notifications are derived from lifecycle state transitions through a centralized mapper.
4. Stream processing is separated from lifecycle management.
5. Doom loop detection catches repeated identical tool calls.
6. OpenAI canonical path is not affected.
7. `SyncEvent`-style persistent event sourcing is not introduced — journal handles canonical persistence.

## Evidence

- path: `../research/2026-04-16-session-layer-research-transfer-ledger.md`
  symbol: `Item 04`
  lines: `221-236`
- path: `../research/session layer/RA-1.4-turn-lifecycle-notifications.md`
  symbol: `LOGOS Current Implementation`
  lines: `1-37`
- path: `../research/session layer/RA-1.4-turn-lifecycle-notifications.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `227-239`
- path: `../research/session layer/RA-1.4-turn-lifecycle-notifications.md`
  symbol: `Recommendations for LOGOS`
  lines: `241-267`
- path: `2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md`
  symbol: `Ownership Layers`
  lines: `79-93`
- path: `2026-04-14-logos-session-state-layer-extraction-discussion.md`
  symbol: `Over-owned By The OpenRouter Lane Today`
  lines: `55-64`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `start_turn`
  lines: `771-1044`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `send_notification`
  lines: `1973-1988`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `EventMsg`
  lines: `1223-1302`
- path: `../../../codex-rs/protocol/src/protocol.rs`
  symbol: `AgentStatus`
  lines: `1606-1622`
- path: `2026-04-11-phase-4-journal-record-taxonomy-discussion.md`
  symbol: `Journal Record Families`
  lines: `41-63`

## Links

- [[README.md]]
- [[2026-04-14-logos-session-state-ownership-and-provider-native-backend-model-discussion.md]]
- [[2026-04-14-logos-session-state-layer-extraction-discussion.md]]
- [[2026-04-16-item-01-thread-manager-draft-implementation-discussion.md]]
- [[../research/2026-04-16-session-layer-research-transfer-ledger.md]]
- [[../research/session layer/RA-1.4-turn-lifecycle-notifications.md]]
