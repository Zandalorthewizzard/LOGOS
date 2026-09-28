# RA-1.3 — Fork / Resume Orchestration

## LOGOS Current Implementation

### Fork

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:1075-1178`

Flow:

1. Load source thread (in-memory or from persistence)
2. Create new `SessionEngine` with `forked_from()` — seeds history from source
3. Copy `OpenRouterThreadState` with new ID
4. Persist new thread
5. Explicitly rejects: `path` mode, config overrides, dynamic tools

### Resume

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:413-479`

Two paths:

- **In-memory resume**: Thread already loaded → reattach listener
- **Persisted resume**: Load JSON snapshot → rebuild `SessionEngine` via `seed_history()` → reattach listener

Explicitly rejects: `history` mode, `path` mode, config overrides.

### Codex Equivalent

- **Fork**: `ThreadManager::fork_thread()` reads rollout JSONL → truncates at Nth user message → creates new CodexThread with forked `InitialHistory`
- **Resume**: `resume_thread_from_rollout()` reads JSONL → `InitialHistory::Resumed` → creates new CodexThread

**Known weaknesses:**

- Fork copies entire history array (no COW)
- No validation of config/model availability on resume
- Resume from persisted state can fail silently if JSON is corrupted
- No partial resume capability

## OpenCode Analysis

### Fork

**File:** `src/session/index.ts:511-546`

```typescript
static fork(from: SessionID, opts?: { messageID?: MessageID }) {
  // 1. Create new session with same project/directory
  // 2. Copy all messages (and parts) up to optional messageID cutoff
  // 3. Map old message IDs to new ones
  // 4. Preserve parent/child message relationships
}
```

Key differences from LOGOS:

- Fork is a **SQL INSERT...SELECT** — copies message rows with new IDs
- Cutoff at specific message (not turn boundary)
- No SessionEngine to rebuild — messages are just data
- Parent-child relationship via `parentID` field on the new session

### Resume (Reload)

OpenCode has no explicit "resume" operation. Sessions are stateless:

- Any session can be "resumed" by simply loading its messages from SQLite
- No in-memory state to restore (no continuation handle, no active streaming)
- The `runLoop` in `prompt.ts` loads history fresh on each turn via `MessageV2.filterCompactedEffect()`

### Revert (Related — Undo)

**File:** `src/session/revert.ts:42-91`

1. Walk messages to find revert target
2. Capture current snapshot if needed
3. Apply patch reversion via `snapshot.revert(patches)`
4. Store diff summary
5. On next prompt: remove all messages >= revert target

### Comparison

| Aspect               | LOGOS (OR)                                    | LOGOS (Codex)                              | OpenCode                         |
| -------------------- | --------------------------------------------- | ------------------------------------------ | -------------------------------- |
| Fork mechanism       | Copy history array → new SessionEngine        | Copy rollout JSONL → new CodexThread       | SQL INSERT...SELECT with new IDs |
| Fork granularity     | Full history only                             | Up to Nth user message                     | Up to specific message           |
| Resume concept       | Explicit: rebuild SessionEngine from snapshot | Explicit: rebuild CodexThread from rollout | Implicit: just load from DB      |
| Continuation handle  | ClientManagedHistory                          | previous_response_id                       | N/A                              |
| Config restore       | Snapshot in JSON                              | From rollout SessionMeta                   | From session record              |
| COW support          | No                                            | No                                         | Not needed (SQL)                 |
| Validation on resume | None                                          | None                                       | None                             |

### Reusable Patterns

1. **SQL-based fork**: INSERT...SELECT with ID remapping is cleaner than array copying. If LOGOS adopts SQLite for persistence, fork becomes a SQL operation.

2. **Message-level fork granularity**: OpenCode forks at message level, not turn level. This is more flexible — user can fork to any point in conversation.

3. **No resume concept**: OpenCode doesn't need resume because it has no in-memory session state. This is the ideal for stateless providers — if LOGOS can make sessions loadable on demand from persistent storage, resume becomes trivial.

### What to Avoid

1. **Don't make resume more complex than needed**: OpenCode proves that for stateless providers, "resume" should be "just load from DB". The current LOGOS OR resume is complex because it rebuilds SessionEngine from a JSON snapshot. If persistence is robust enough, resume = load + attach listener.

## Goose Analysis

### Copy Session (Fork)

**File:** `crates/goose/src/session/session_manager.rs:1415-1458`

`copy_session(session_id, new_name)`:

1. Loads original session with messages
2. Creates new session with new auto-generated ID
3. Copies metadata: `extension_data`, `schedule_id`, `recipe`, `user_recipe_values`
4. Copies `provider_name`, `model_config`, `goose_mode` (provider preserved across fork)
5. Replaces conversation in new session (full message INSERT)
6. Returns new session

### Truncate Conversation (Fork-to-point)

**File:** `crates/goose/src/session/session_manager.rs:1460-1469`

```rust
async fn truncate_conversation(&self, session_id: &str, timestamp: i64) -> Result<()> {
    sqlx::query("DELETE FROM messages WHERE session_id = ? AND created_timestamp >= ?")
        .bind(session_id).bind(timestamp)
        .execute(pool).await?;
    Ok(())
}
```

Hard delete of messages at or after a timestamp. Not a fork — modifies in-place.

### Resume

No explicit resume concept. Sessions are always loaded from SQLite on demand via `get_session(id, include_messages)`. The agent loop (`Agent::reply()`) receives the full conversation history and session config, but the actual agent loop code is in the missing `agents/` module.

### Scheduler (Background Sessions)

**File:** `crates/goose/src/scheduler.rs` (1128 lines)

- Cron-based job scheduler for automated sessions
- Each execution creates a new `Session` with `SessionType::Scheduled`
- `CancellationToken` per running job for cancellation
- Running jobs tracked in `HashMap<String, CancellationToken>`
- `kill_running_job()` cancels the token

### Key Patterns

1. **Copy = full duplicate**: Goose copies the entire conversation, not up to a point. No partial fork. Simple but loses the ability to fork from a specific message.

2. **Truncation by timestamp**: In-place deletion by timestamp, not by message ID or turn number. Crude — if multiple messages share the same timestamp (same-second tool request + response), they're all deleted or all kept.

3. **No parent tracking**: Copied sessions have no `parent_session_id` field. No lineage tracking. Forked session is fully independent.

4. **Provider preservation**: Fork copies `provider_name` and `model_config` from original. The forked session continues with the same provider.

### Comparison

| Aspect              | LOGOS (OR)                               | Goose                                             |
| ------------------- | ---------------------------------------- | ------------------------------------------------- |
| Fork mechanism      | Copy history array → new SessionEngine   | copy_session: SQL INSERT new + DELETE+INSERT msgs |
| Fork granularity    | Full history only                        | Full history only (no partial fork)               |
| Truncate            | Not implemented                          | By timestamp (in-place DELETE)                    |
| Resume concept      | Rebuild SessionEngine from JSON snapshot | Load from SQLite (implicit)                       |
| Continuation handle | ClientManagedHistory                     | None (stateless)                                  |
| Config restore      | Snapshot in JSON                         | From session record (provider_name, model_config) |
| Parent tracking     | No                                       | No                                                |
| Scheduler sessions  | No                                       | Yes (cron-based, SessionType::Scheduled)          |

## Cross-Project Comparison Matrix

| Aspect            | LOGOS (OR)                      | LOGOS (Codex)                    | OpenCode            | Goose                               |
| ----------------- | ------------------------------- | -------------------------------- | ------------------- | ----------------------------------- |
| Fork data copy    | Array clone                     | JSONL read + copy                | SQL INSERT...SELECT | SQL copy_session (full duplicate)   |
| Fork granularity  | Full history                    | Turn boundary                    | Message boundary    | Full history only                   |
| Resume mechanism  | Rebuild SessionEngine from JSON | Rebuild CodexThread from rollout | Load from DB        | Load from SQLite (implicit)         |
| COW support       | No                              | No                               | N/A (SQL)           | N/A (SQL)                           |
| Config validation | None                            | None                             | None                | None                                |
| Parent tracking   | No                              | thread_spawn_edges table         | parentID field      | No                                  |
| Truncation        | Not implemented                 | Rollout trim                     | DELETE messages     | DELETE messages by timestamp        |
| Scheduler         | No                              | No                               | No                  | Cron-based (SessionType::Scheduled) |

## Recommendations for LOGOS

### What to Borrow

1. **Message-level fork granularity**: Allow fork at any message, not just turn boundaries. OpenCode's pattern is more flexible.

2. **Parent tracking via field**: Add `parent_thread_id` to thread metadata (like OpenCode's `parentID`). This enables lineage tracking without a separate edge table.

3. **Trivialize resume**: If persistence is robust enough (SQLite per RA-1.2), resume should be: load history from DB → create SessionEngine with `seed_history()` → attach listener. No special "resume from snapshot" logic needed.

### What to Avoid

1. **Don't keep JSON snapshot resume**: It's fragile and duplicates logic. Move to SQL-backed loading on demand.

### What LOGOS Can Do Better

1. **COW for fork with long histories**: Neither OpenCode nor LOGOS does this. For sessions with thousands of messages, array cloning is expensive. Consider shared immutable history prefix + divergence marker.

2. **Validation on fork/resume**: Check that the model specified in config is still available. Check that tools referenced in history still exist. None of the projects do this.

### Risks and Open Questions

1. **SessionEngine rebuild cost**: Even with SQL-backed persistence, rebuilding a SessionEngine with thousands of messages takes time. Need benchmarks.

2. **Active streaming connections**: OpenCode doesn't have persistent streaming connections. LOGOS does. Resume must re-establish SSE/WebSocket listeners, which OpenCode doesn't need to worry about.
