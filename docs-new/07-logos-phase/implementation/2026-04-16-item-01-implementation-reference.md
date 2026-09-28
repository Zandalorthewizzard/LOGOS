---
id: logos-item-01-implementation-reference
title: Item 01 Thread Manager Implementation Reference
type: implementation-reference
status: active
tags:
  [logos, architecture, phase-6, session-state, thread-manager, implementation]
related:
  [
    [
      ../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md,
    ],
    [
      ../architecture/2026-04-14-logos-session-state-layer-extraction-discussion.md,
    ],
    [../research/session layer/RA-1.1-thread-manager.md],
    [../research/session layer/RA-1.2-session-persistence.md],
    [../research/session layer/SUMMARY.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-16-item-01-thread-manager-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.1-thread-manager.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 01 Thread Manager Implementation Reference

## Purpose

Concrete code patterns extracted from Goose (Rust) and OpenCode (TypeScript) for each stage of the Item 01 draft implementation. This document maps production patterns to LOGOS equivalents, identifying what can be refactored, adapted, or must be written from scratch.

## Source Projects

| Project  | Language   | Key Files                                                                                      |
| -------- | ---------- | ---------------------------------------------------------------------------------------------- |
| Goose    | Rust       | `crates/goose/src/session/session_manager.rs` (2032 lines)                                     |
| OpenCode | TypeScript | `src/storage/db.ts`, `src/session/session.sql.ts`, `src/sync/index.ts`, `src/session/index.ts` |

---

## Stage A: Durable Non-canonical Thread Registry

### A.1 SQLite Connection Pool Setup

**Goose** (`session_manager.rs:510-523`) — Directly refactorable:

```rust
fn create_pool(path: &Path) -> Pool<Sqlite> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create session database directory");
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(30))
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    SqlitePoolOptions::new().connect_lazy_with(options)
}
```

**OpenCode** (`db.ts:90-95`) — Equivalent PRAGMA set:

```
PRAGMA journal_mode = WAL
PRAGMA synchronous = NORMAL
PRAGMA busy_timeout = 5000
PRAGMA cache_size = -64000      // 64 MiB
PRAGMA foreign_keys = ON
```

**LOGOS action:** Refactor Goose's `create_pool` directly. Use `sqlx` crate (already Rust ecosystem). Add OpenCode's `synchronous = NORMAL` and `cache_size = -64000` PRAGMAs. Goose's 30-second busy timeout is generous — consider OpenCode's 5-second timeout with retry at the caller level.

### A.2 Schema Creation

**Goose** (`session_manager.rs:565-646`) — Refactorable with adaptation:

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    session_type TEXT NOT NULL DEFAULT 'user',
    working_dir TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    extension_data TEXT DEFAULT '{}',
    provider_name TEXT,
    model_config_json TEXT,
    goose_mode TEXT NOT NULL DEFAULT 'auto'
    -- ... token count columns ...
)
```

**OpenCode** (`session.sql.ts:14-44`) — Not refactorable (TypeScript/Drizzle), but schema design borrowable:

```typescript
time_archived: integer(),    // soft delete: NULL = active
parent_id: text(),           // self-reference for fork lineage
```

**LOGOS action:** Start from Goose's SQL DDL (Rust-native). Adapt for LOGOS needs:

```sql
CREATE TABLE thread_records (
    thread_id TEXT PRIMARY KEY,
    provider_lane TEXT NOT NULL,          -- 'openrouter_beta', future lanes
    provider_id TEXT NOT NULL,            -- provider-specific identifier
    name TEXT NOT NULL DEFAULT '',
    archived_at INTEGER,                  -- NULL = active (soft delete from OpenCode)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    session_state_path TEXT,              -- path to history JSON (temporary, until Item 02)
    config_snapshot_json TEXT NOT NULL
);

CREATE INDEX idx_thread_records_updated ON thread_records(updated_at DESC);
CREATE INDEX idx_thread_records_archived ON thread_records(archived_at);
CREATE INDEX idx_thread_records_provider ON thread_records(provider_lane);
```

Key difference from Goose: `provider_lane` column for lane-aware dispatch (Stage C). Key difference from OpenCode: `session_state_path` for backward compatibility with current JSON history files.

### A.3 Schema Migration Framework

**Goose** (`session_manager.rs:753-942`) — Directly refactorable:

```rust
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let current_version = Self::get_schema_version(&mut tx).await?;
    if current_version < CURRENT_SCHEMA_VERSION {
        for version in (current_version + 1)..=CURRENT_SCHEMA_VERSION {
            Self::apply_migration(&mut tx, version).await?;
            Self::update_schema_version(&mut tx, version).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
```

Migration version tracking:

```sql
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
```

Each migration is a `match` arm (`session_manager.rs:814-938`):

```rust
match version {
    1 => { /* CREATE TABLE schema_version */ }
    2 => { /* ALTER TABLE sessions ADD COLUMN ... */ }
    // ...
    _ => { anyhow::bail!("Unknown migration version: {}", version); }
}
```

**LOGOS action:** Refactor Goose's migration framework directly. Start at version 1 with the `thread_records` schema above. The `BEGIN IMMEDIATE` + sequential migration pattern is production-tested.

### A.4 Builder-Pattern Updates

**Goose** (`session_manager.rs:86-239, 1024-1137`) — Directly refactorable:

```rust
pub struct SessionUpdateBuilder<'a> {
    session_manager: &'a SessionManager,
    session_id: String,
    name: Option<String>,
    archived_at: Option<Option<i64>>,
    // ... other Option fields ...
}

impl<'a> SessionUpdateBuilder<'a> {
    pub async fn apply(self) -> Result<()> {
        self.session_manager.apply_update_inner(self).await
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
```

The `apply_update_inner` method dynamically builds SQL:

```rust
let mut query = String::from("UPDATE sessions SET ");
// macro generates: if field.is_some() { query.push_str("name = ?"); }
// only changed fields appear in the SQL
query.push_str("updated_at = datetime('now') WHERE id = ?");
```

**LOGOS action:** Refactor directly. LOGOS builder fields:

```rust
pub struct ThreadUpdateBuilder<'a> {
    registry: &'a ThreadRegistry,
    thread_id: ThreadId,
    name: Option<String>,
    archived_at: Option<Option<i64>>,
    config_snapshot: Option<String>,
}
```

### A.5 Legacy Import

**Goose** (`session_manager.rs:649-749`) — Directly refactorable pattern:

```rust
async fn import_legacy(pool: &Pool<Sqlite>, session_dir: &PathBuf) -> Result<()> {
    let sessions = match legacy::list_sessions(session_dir) {
        Ok(sessions) => sessions,
        Err(_) => { warn!("No legacy sessions found"); return Ok(()); }
    };
    for (session_name, session_path) in sessions {
        match legacy::load_session(&session_name, &session_path) {
            Ok(session) => match Self::import_legacy_session(pool, &session).await {
                Ok(_) => { imported_count += 1; }
                Err(e) => { failed_count += 1; }
            },
            Err(e) => { failed_count += 1; }
        }
    }
    Ok(())
}
```

**LOGOS action:** Refactor for our legacy format. Current OR sessions are JSON files in `logos-openrouter-beta/active-threads/`. The import reads each `PersistedOpenRouterThread` JSON and inserts into `thread_records` table. History JSON files remain in place (only metadata migrates in Stage A).

### A.6 Lazy Initialization

**Goose** (`session_manager.rs:52-53`):

```rust
static SESSION_STORAGE: LazyLock<Arc<SessionStorage>> =
    LazyLock::new(|| Arc::new(SessionStorage::new(Paths::data_dir())));
```

Plus `OnceCell` for migration-on-first-use (`session_manager.rs:536-557`):

```rust
async fn pool(&self) -> Result<&Pool<Sqlite>> {
    self.initialized.get_or_try_init(|| async {
        let schema_exists = /* check if schema_version table exists */;
        if schema_exists {
            Self::run_migrations(&self.pool).await?;
        } else {
            Self::create_schema(&self.pool).await?;
            Self::import_legacy(&self.pool, &self.session_dir).await?;
        }
        Ok(())
    }).await?;
    Ok(&self.pool)
}
```

**LOGOS action:** Use `std::sync::LazyLock` for global singleton + `tokio::sync::OnceCell` for lazy migration. Goose's pattern is correct.

### A.7 Listing with Filtering

**Goose** (`session_manager.rs:1261-1298`):

```rust
async fn list_sessions_by_types(&self, types: Option<&[SessionType]>) -> Result<Vec<Session>> {
    let (where_clause, binds) = match types {
        Some(t) if !t.is_empty() => {
            let placeholders = t.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            (format!("WHERE s.session_type IN ({})", placeholders), ...)
        }
        _ => (String::new(), Vec::new()),
    };
    let query = format!(
        "SELECT s.*, COUNT(m.id) as message_count
         FROM sessions s INNER JOIN messages m ON s.id = m.session_id
         {} GROUP BY s.id ORDER BY s.updated_at DESC",
        where_clause
    );
}
```

**OpenCode** (`session/index.ts:739-780`) — Dynamic WHERE builder:

```typescript
const conditions = [eq(SessionTable.project_id, project.id)];
if (input?.workspaceID)
  conditions.push(eq(SessionTable.workspace_id, input.workspaceID));
if (input?.directory)
  conditions.push(eq(SessionTable.directory, input.directory));
if (!input?.archived) conditions.push(isNull(SessionTable.time_archived));
```

**LOGOS action:** Use Goose's Rust-native dynamic query building. Add OpenCode's soft-delete filter (`WHERE archived_at IS NULL`). Our listing filter:

```rust
async fn list_threads(&self, filter: ThreadFilter) -> Result<Vec<ThreadRecord>> {
    let mut conditions = vec!["1=1".to_string()];
    if !filter.include_archived {
        conditions.push("archived_at IS NULL".to_string());
    }
    if let Some(lane) = &filter.provider_lane {
        conditions.push(format!("provider_lane = '{}'", lane));
    }
    // ...
}
```

### A.8 What LOGOS Must Write From Scratch

| Item                               | Why not refactorable                                         |
| ---------------------------------- | ------------------------------------------------------------ |
| `provider_lane` dispatch column    | Neither competitor has multi-lane concept                    |
| `session_state_path` bridge field  | Temporary bridge until Item 02                               |
| `ThreadRegistry` trait abstraction | LOGOS-specific: shared interface for stateless + Codex paths |

---

## Stage B: Ephemeral Runtime During Active Turn

### B.1 Turn Execution Flow: Load → Run → Persist → Drop

**Goose** (`scheduler.rs:872-911`) — Refactorable skeleton:

```rust
// 1. Load session from SQLite
let session = session_manager.get_session(&session.id, /*include_messages=*/ false).await?;

// 2. Create fresh agent (equivalent to SessionEngine::new)
//    Agent::reply() is called with full session config

// 3. Stream consumption
let mut stream = std::pin::pin!(agent.reply(user_message, session_config, Some(cancel_token)).await?);
while let Some(message_result) = stream.next().await {
    tokio::task::yield_now().await;
    match message_result {
        Ok(AgentEvent::Message(msg)) => { conversation.push(msg); }
        Ok(AgentEvent::HistoryReplaced(updated)) => { conversation = updated; }
        Err(e) => { tracing::error!("Error: {}", e); break; }
        _ => {}
    }
}

// 4. Persist results
session_manager.update(&session.id).schedule_id(...).apply().await?;
```

**OpenCode** (`prompt.ts:1337-1565`) — More detailed lifecycle:

```typescript
while (true) {
    // 1. LOAD history from store
    let msgs = yield* MessageV2.filterCompactedEffect(sessionID)

    // 2. PERSIST empty assistant message BEFORE streaming
    const msg = { id: MessageID.ascending(), ... }
    yield* sessions.updateMessage(msg)

    // 3. RUN the LLM stream
    const result = yield* handle.process(streamInput)

    // 4. DECIDE: continue | compact | stop
    if (result === "stop") break
    if (result === "compact") { yield* compaction.create(...); continue }
}
```

**LOGOS mapping to SessionEngine** (`session.rs:290-506`):

Our `run_turn_with_tools_inner` already implements this loop. The key insight: **SessionEngine is already ephemeral-capable**. We just need to:

1. Create it at turn-start (via `SessionEngine::new` + `seed_history`)
2. Run the turn
3. Extract updated state (history, continuation, tool_call_records)
4. Persist
5. Drop

```rust
// LOGOS Stage B turn flow:
async fn execute_turn(&self, thread_id: ThreadId, input: Vec<InternalMessage>) -> Result<TurnOutcome> {
    // 1. Load from registry
    let record = self.registry.get_thread(thread_id).await?;
    let history = self.load_history(&record).await?;  // from JSON file (Stage A)

    // 2. Create engine
    let adapter = self.adapter_factory.create(&record.provider_id)?;
    let mut engine = SessionEngine::new(adapter, record.model_ref(), record.system_prompt()).await?;
    engine.seed_history(history, record.continuation());

    // 3. Run
    let outcome = engine.run_turn_with_tools(input, tools).await?;

    // 4. Persist
    self.persist_history(&record, engine.history(), engine.continuation()).await?;
    self.registry.touch(thread_id).await?;

    // 5. Drop (engine goes out of scope)
    Ok(outcome)
}
```

### B.2 Concurrency Guard

**OpenCode** (`runner.ts:32-36, 111-139`) — Refactor concept, implement from scratch:

```typescript
type State<A, E> =
  | { readonly _tag: "Idle" }
  | { readonly _tag: "Running"; readonly run: RunHandle<A, E> }
  | { readonly _tag: "Shell"; readonly shell: ShellHandle<A, E> }
  | { readonly _tag: "ShellThenRun"; ... }
```

Key behavior:

- If `Running`: caller **waits** for existing run (serializes, not rejects)
- If `Shell`: queues pending run, transitions to `ShellThenRun`
- If `Idle`: starts the run

**LOGOS action:** Implement from scratch in Rust:

```rust
enum TurnGuardState {
    Idle,
    Running { done: tokio::sync::oneshot::Sender<Result<TurnOutcome>> },
}

struct TurnGuard {
    state: tokio::sync::Mutex<TurnGuardState>,
}

impl TurnGuard {
    async fn acquire(&self) -> Result<TurnGuardHandle> {
        let mut state = self.state.lock().await;
        match &*state {
            TurnGuardState::Running { .. } => {
                Err(BusyError)  // or wait, depending on design choice
            }
            TurnGuardState::Idle => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                *state = TurnGuardState::Running { done: tx };
                Ok(TurnGuardHandle { guard: self, rx })
            }
        }
    }
}
```

Decision: OpenCode waits, LOGOS should reject with `BusyError`. Reason: LOGOS app-server has explicit turn lifecycle in the protocol — client should know a turn is active, not silently queue.

### B.3 Status Tracking

**OpenCode** (`status.ts:60-82`) — Refactor concept:

```typescript
// In-memory Map, not persisted
const state = new Map<SessionID, Info>();

// Idle entries are REMOVED from map
function set(sessionID, status) {
  bus.publish(Event.Status, { sessionID, status });
  if (status.type === "idle") {
    state.delete(sessionID);
    return;
  }
  state.set(sessionID, status);
}
```

**LOGOS action:** Use `DashMap<ThreadId, SessionStatus>` with bus publish on change. Our `ThreadStateManager` already tracks subscriptions — status tracking is orthogonal and lightweight.

### B.4 Stream Consumption with Cleanup

**OpenCode** (`processor.ts:416-490`) — The error/cleanup stack:

```
onInterrupt → mark aborted
retry(policy) → retry on transient errors
catch(halt) → attach error to message
ensuring(cleanup) → always: flush text, mark tools aborted, persist
```

**Goose** (`scheduler.rs:883-902, 633-656`) — Simpler: cleanup always runs regardless of result.

**LOGOS action:** Our `SessionEngine.run_turn_with_tools_inner` already has error handling and journal writes. What we add:

1. **Cleanup guard** (from OpenCode pattern): wrap the turn in a struct that implements `Drop` to persist partial state
2. **Abort marking** (from OpenCode pattern): mark in-progress tool calls as "aborted" on cancellation

### B.5 Message Persistence Patterns

**Goose** (`session_manager.rs:1177-1210`) — Incremental insert:

```rust
async fn add_message(&self, session_id: &str, message: &Message) -> Result<()> {
    let pool = self.pool().await?;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query("INSERT INTO messages (...) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(message_id).bind(session_id).bind(role).bind(content_json)
        .bind(created).bind(metadata_json)
        .execute(&mut *tx).await?;
    sqlx::query("UPDATE sessions SET updated_at = datetime('now') WHERE id = ?")
        .bind(session_id).execute(&mut *tx).await?;
    tx.commit().await?;
}
```

**Goose** (`session_manager.rs:1212-1259`) — Full conversation swap (for compaction):

```rust
async fn replace_conversation_inner(pool, session_id, conversation) -> Result<()> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(session_id).execute(&mut *tx).await?;
    for message in conversation.messages() {
        sqlx::query("INSERT INTO messages (...) VALUES (...)").execute(&mut *tx).await?;
    }
    tx.commit().await?;
}
```

**LOGOS action:** In Stage A, history stays as JSON files. `add_message` is not needed yet — we persist the full history snapshot after turn completion (current OR pattern). When Item 02 introduces message-level persistence, refactor Goose's `add_message` pattern directly.

### B.6 What LOGOS Must Write From Scratch

| Item                                | Why not refactorable                                                  |
| ----------------------------------- | --------------------------------------------------------------------- |
| SessionEngine → SQLite bridge       | Neither competitor has SessionEngine                                  |
| Partial state persistence on cancel | Goose doesn't do this; OpenCode uses Effect which doesn't map to Rust |
| Provider-agnostic tool registration | Our `ToolRuntime` is unique                                           |
| Journal integration                 | Our journal system is unique                                          |

---

## Stage C: Dispatch Simplification

### C.1 Current LOGOS Problem

`codex_message_processor.rs` has 4 `contains()` checks + 3 listing calls:

```
Line 2751: if self.openrouter_beta_threads.contains(thread_id).await { ... }
Line 2880: if self.openrouter_beta_threads.contains(thread_uuid).await { ... }
Line 3648: self.openrouter_beta_threads.list_archived_threads().await
Line 3656: self.openrouter_beta_threads.list_threads().await
Line 3797: self.openrouter_beta_threads.loaded_thread_ids().await
Line 5878: if self.openrouter_beta_threads.contains(thread_id).await { ... }
Line 7014: if self.openrouter_beta_threads.contains(thread_id).await { ... }
```

Each `contains()` is: `lock HashMap → check key → return bool`. Then the operation locks again.

### C.2 How Goose Dispatches

Goose has no dispatch problem — there is only one session type and one SessionManager. Every operation goes directly to `SessionStorage`. No `if/else` branching.

```rust
// Goose: single entry point
let session = session_manager.get_session(id, include_messages).await?;
session_manager.update(&id).name("new name").apply().await?;
```

### C.3 How OpenCode Dispatches

OpenCode similarly has a single session model. No provider-lane branching at the session level.

```typescript
// OpenCode: single entry point through SyncEvent
SyncEvent.run(Event.Updated, { sessionID, info: { title: "new name" } });
```

### C.4 LOGOS Target

Replace `if OR.contains() → OR path, else → Codex path` with lane-aware registry lookup:

```rust
// Current (repeated 4+ times):
if self.openrouter_beta_threads.contains(thread_id).await {
    self.openrouter_beta_threads.set_thread_name(thread_id, name).await
} else {
    let thread = self.thread_manager.get_thread(thread_id).await?;
    self.submit_core_op(thread.as_ref(), Op::SetThreadName { name }).await
}

// Target (single dispatch):
let lane = self.registry.get_lane(thread_id).await?;  // "openrouter_beta" or "codex"
match lane {
    ProviderLane::OpenRouterBeta => self.or_handler.set_thread_name(thread_id, name).await,
    ProviderLane::Codex => {
        let thread = self.thread_manager.get_thread(thread_id).await?;
        self.submit_core_op(thread.as_ref(), Op::SetThreadName { name }).await
    }
}
```

The `registry.get_lane()` is a single SQLite query: `SELECT provider_lane FROM thread_records WHERE thread_id = ?`. No in-memory HashMap check.

For listing:

```rust
// Current: merge OR list + Codex list separately
let mut data = self.thread_manager.list_threads().await;
data.extend(self.openrouter_beta_threads.loaded_thread_ids().await);

// Target: single SQL query
let data = self.registry.list_threads(ThreadFilter::all()).await?;
```

### C.5 Migration Strategy

**Phase 1:** Add registry as authoritative source for OR threads. Codex threads still managed by ThreadManager. Dispatch checks registry first, falls back to ThreadManager.

**Phase 2:** Add Codex threads to registry (as read-only records). ThreadManager remains the runtime owner, but registry provides unified listing.

**Phase 3:** Full unification (post-MVP).

---

## Cross-Cutting: What Goose Gets Right That We Should Copy

| Pattern                          | Goose Source                   | Value for LOGOS                                |
| -------------------------------- | ------------------------------ | ---------------------------------------------- |
| `BEGIN IMMEDIATE` on every write | `session_manager.rs:516-523`   | Prevents SQLITE_BUSY without retry loops       |
| `LazyLock` global singleton      | `session_manager.rs:52-53`     | One pool per process, no arc churn             |
| `OnceCell` lazy migration        | `session_manager.rs:536-557`   | Schema creation on first use, not startup      |
| Builder pattern for updates      | `session_manager.rs:86-239`    | Type-safe partial updates, only changed fields |
| Date-prefixed sequential IDs     | `session_manager.rs:954-981`   | Natural ordering, no UUID sorting              |
| Legacy import on first run       | `session_manager.rs:649-691`   | Smooth migration from JSON files               |
| `datetime('now')` for updated_at | `session_manager.rs:1066-1067` | Consistent timestamps from SQLite              |

## Cross-Cutting: What OpenCode Gets Right That We Should Copy

| Pattern                                   | OpenCode Source        | Value for LOGOS                            |
| ----------------------------------------- | ---------------------- | ------------------------------------------ |
| Soft delete via nullable timestamp        | `session.sql.ts:38`    | Reversible archive without file moves      |
| Post-commit effects queue                 | `db.ts:144-151`        | Notifications fire only after commit       |
| Runner concurrency state machine          | `runner.ts:32-36`      | Clean 4-state model for turn serialization |
| Doom loop detection                       | `processor.ts:186-212` | Simple, effective safety guard             |
| Cleanup via `ensuring` (Rust: Drop guard) | `processor.ts:367-414` | Always flush partial state                 |
| Status map with idle removal              | `status.ts:73-82`      | Lightweight, no persistent overhead        |

## Cross-Cutting: What Both Get Wrong That LOGOS Should Improve

| Pattern                              | Problem                                       | LOGOS Better Approach                   |
| ------------------------------------ | --------------------------------------------- | --------------------------------------- |
| Goose: no concurrency guard          | `BEGIN IMMEDIATE` is not a logical lock       | Add TurnGuard per thread                |
| Goose: swallow stream errors         | `execute_job` always returns Ok               | Propagate errors, persist partial state |
| Goose: no `updated_at` on compaction | `replace_conversation` doesn't bump timestamp | Always bump `updated_at`                |
| OpenCode: per-delta event sourcing   | Stores every text delta                       | Only persist final turn state           |
| OpenCode: no sandbox                 | Shell execution unsandboxed                   | Keep Codex sandbox option               |
| Both: no eviction                    | Not needed (no in-memory map)                 | N/A for LOGOS (no in-memory map either) |

## Implementation Priority

| Step      | Action                                                    | Source                    | Estimated Effort |
| --------- | --------------------------------------------------------- | ------------------------- | ---------------- |
| 1         | Create `logos-session-state` crate                        | New                       | 0.5 day          |
| 2         | Port Goose's SQLite pool + PRAGMA setup                   | Goose refactor            | 0.5 day          |
| 3         | Port Goose's migration framework                          | Goose refactor            | 1 day            |
| 4         | Define `thread_records` schema                            | Adapted from both         | 0.5 day          |
| 5         | Implement `ThreadRegistry` trait + `SqliteThreadRegistry` | New                       | 2 days           |
| 6         | Port builder-pattern updates                              | Goose refactor            | 1 day            |
| 7         | Implement legacy JSON import                              | Goose pattern, LOGOS data | 1 day            |
| 8         | Wire registry into app-server (Stage A)                   | New                       | 2 days           |
| 9         | Implement ephemeral runtime (Stage B)                     | Adapt SessionEngine       | 2 days           |
| 10        | Simplify dispatch (Stage C)                               | New                       | 1.5 days         |
| **Total** |                                                           |                           | **~12 days**     |

## Evidence

- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::create_pool`
  lines: `510-523`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::create_schema`
  lines: `565-646`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `run_migrations`
  lines: `753-776`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `apply_migration`
  lines: `813-942`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionUpdateBuilder`
  lines: `86-239`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `apply_update`
  lines: `1024-1137`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `import_legacy`
  lines: `649-691`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `add_message`
  lines: `1177-1210`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `replace_conversation`
  lines: `1212-1259`
- path: `../../../input-sources-temp/goose-upstream/crates/goose/src/scheduler.rs`
  symbol: `execute_job` (stream consumption)
  lines: `872-911`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/storage/db.ts`
  symbol: `PRAGMA configuration`
  lines: `90-95`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/session/session.sql.ts`
  symbol: `SessionTable`
  lines: `14-44`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/sync/index.ts`
  symbol: `SyncEvent.run`
  lines: `195-227`
- path: `../../../input-sources-temp/opencode-upstream/packages/opencode/src/session/index.ts`
  symbol: `list`
  lines: `739-780`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `OpenRouter lane presence checks`
  lines: `2751, 2880, 5878, 7014`
- path: `../../../codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `OpenRouter lane listing calls`
  lines: `3648, 3656, 3797`
- path: `../../../codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `OpenRouterBetaThreadManager`
  lines: `72-108`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine::run_turn_with_tools_inner`
  lines: `290-506`
- path: `../../../codex-rs/logos-session-engine/src/session.rs`
  symbol: `SessionEngine::seed_history`
  lines: `135-142`
