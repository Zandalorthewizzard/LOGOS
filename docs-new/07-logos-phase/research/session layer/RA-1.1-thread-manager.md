# RA-1.1 — Stateless Thread Manager

## LOGOS Current Implementation

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs:72-90`

`OpenRouterBetaThreadManager` owns:

- `HashMap<ThreadId, Arc<OpenRouterBetaThread>>` — in-memory loaded threads
- Each thread holds an `OpenRouterThreadState` with config snapshot, metadata, turn tracking, elicitation count
- Separate from Codex `ThreadManager` (`codex-rs/core/src/thread_manager.rs:192`) which owns `HashMap<ThreadId, Arc<CodexThread>>`

**Known weaknesses:**

- No TTL or eviction policy — threads stay loaded indefinitely
- No limit on concurrently loaded threads
- Duplicated pattern with Codex ThreadManager
- Dispatch in `codex_message_processor.rs` does `if OR → OR manager, else → ThreadManager` for every operation

## OpenCode Analysis

### Architecture

OpenCode has **no thread concept**. It uses flat **sessions** with optional parent-child relationships.

**Key files:**

- `src/session/index.ts` — Session CRUD and lifecycle
- `src/session/session.sql.ts` — SQL schema (SessionTable, MessageTable, PartTable)
- `src/session/status.ts` — In-memory status tracking
- `src/storage/db.ts` — SQLite via Drizzle ORM

### Session Model

Sessions are the top-level unit (equivalent to Codex threads):

- `Session.Info`: id, slug, projectID, directory, parentID, title, timestamps, metadata
- Parent-child via `parentID` (self-referencing FK in SQL)
- No in-memory session map — sessions loaded from SQLite on demand
- Status tracked separately in-memory: `Map<SessionID, {status: "idle"|"busy"|"retry"}>`

### Key Differences from LOGOS

| Aspect            | LOGOS (OR adapter)                               | OpenCode                                                       |
| ----------------- | ------------------------------------------------ | -------------------------------------------------------------- |
| In-memory store   | `HashMap<ThreadId, Arc<Thread>>` — always loaded | No persistent in-memory map; loaded from SQLite on demand      |
| Persistence       | JSON files on disk                               | SQLite (Drizzle ORM) with event sourcing                       |
| Listing           | In-memory + persisted file scan                  | SQL query with filters, pagination, search                     |
| Concurrency guard | None (manual)                                    | `Runner` per session with busy/idle tracking + `BusyError`     |
| Eviction          | None                                             | Not needed — no persistent in-memory store                     |
| ID scheme         | UUID                                             | Descending timestamp IDs (newer = smaller, natural sort order) |
| Parent-child      | Fork creates independent thread                  | `parentID` field enables explicit hierarchy                    |
| Provider coupling | Thread bound to provider at creation             | Session stores provider/model IDs but resolves at call time    |

### Reusable Patterns

1. **Runner-based concurrency control** (`session/prompt.ts:105-134`): Each session gets a `Runner` that tracks busy/idle state. Throws `BusyError` if session already processing. This prevents concurrent turn conflicts that LOGOS OR adapter doesn't guard against.

2. **Descending ID scheme** (`src/id/id.ts:32-33`): Newer sessions naturally sort first without explicit `ORDER BY time_created DESC`. Minor but elegant.

3. **SQL-based listing with filters** (`session/index.ts:739-849`): Project-scoped, workspace-scoped, cursor-paginated, text-searchable. LOGOS OR adapter does file scanning which doesn't scale.

4. **Status as separate concern** (`session/status.ts`): Status (`idle`/`busy`/`retry`) tracked in a lightweight in-memory map, not embedded in session state. Published to event bus. When idle, entry is removed from map.

5. **Event-sourced mutations** (`src/sync/index.ts`): All session mutations go through `SyncEvent.run()` — immediate SQLite transaction + projector + persist + bus publish. This gives audit trail and replay capability.

### What to Avoid

1. **No eviction problem**: OpenCode doesn't need eviction because it doesn't keep sessions in memory. LOGOS should consider whether in-memory thread map is necessary at all, or if SQLite-backed loading on demand is sufficient.

2. **Flat session model**: No thread abstraction means no resume/fork semantics. OpenCode's "fork" copies messages up to a cutoff, but there's no continuation handle concept. This is simpler but less powerful.

## Goose Analysis

### Architecture

Goose has **no thread concept**. It uses flat **sessions** managed by `SessionManager` backed by SQLite.

**Key files:**

- `crates/goose/src/session/session_manager.rs` — SessionManager (1538+ lines), SessionStorage (SQLite)
- `crates/goose/src/session/mod.rs` — Re-exports
- `crates/goose/src/session_context.rs` — Task-local session ID propagation

### Session Model

- **`Session`** struct: `id`, `working_dir`, `name`, `user_set_name`, `session_type`, `created_at`, `updated_at`, `extension_data`, token counts, `schedule_id`, `recipe`, `provider_name`, `model_config`, `goose_mode`, `conversation`, `message_count`
- `SessionType` enum: `User`, `Scheduled`, `SubAgent`, `Hidden`, `Terminal`, `Gateway`, `Acp`
- **SessionStorage**: `pool: Pool<Sqlite>` — SQLite with WAL mode, `session_dir: PathBuf`
- **SessionManager**: Thin wrapper around `Arc<SessionStorage>`, provides CRUD + `SessionUpdateBuilder`
- **Global singleton**: `SESSION_STORAGE: LazyLock<Arc<SessionStorage>>` — one SQLite pool per process

### ID Scheme

Date-prefixed sequential IDs: `YYYYMMDD_N` where N is auto-incremented per day via SQL:

```sql
INSERT INTO sessions (id, ...) VALUES (? || '_' || CAST(COALESCE(MAX(..)+1) AS TEXT), ...)
```

Newer sessions sort first by `updated_at DESC` index.

### Key Differences from LOGOS

| Aspect            | LOGOS (OR adapter)                             | Goose                                                      |
| ----------------- | ---------------------------------------------- | ---------------------------------------------------------- |
| In-memory store   | `HashMap<ThreadId, Arc<Thread>>` always loaded | No in-memory session map; loaded from SQLite on demand     |
| Persistence       | JSON files on disk                             | SQLite (sqlx) with WAL mode, schema migrations             |
| Listing           | In-memory + persisted file scan                | SQL query with type filters, JOIN for message counts       |
| Concurrency guard | None                                           | None (relies on SQLite `BEGIN IMMEDIATE` transactions)     |
| Eviction          | None                                           | Not needed — no persistent in-memory store                 |
| ID scheme         | UUID                                           | `YYYYMMDD_N` (date-prefixed sequential)                    |
| Provider coupling | Thread bound to provider at creation           | Session stores provider/model but resolved at call time    |
| Parent/child      | Fork creates independent thread                | `copy_session` duplicates full conversation + metadata     |
| Session types     | Single type                                    | 7 types (User, Scheduled, SubAgent, Hidden, Terminal, etc) |
| Update pattern    | Direct mutation + JSON persist                 | Builder pattern (`SessionUpdateBuilder`) + SQL UPDATE      |
| Schema evolution  | Manual version field                           | 9 sequential migrations with version tracking              |

### Reusable Patterns

1. **Builder-pattern updates** (`SessionUpdateBuilder`, lines 86-239): Fluent API for partial updates. Only changed fields are included in the SQL UPDATE. Clean, extensible, type-safe.

2. **SQLite WAL with `BEGIN IMMEDIATE`** (`session_manager.rs:516-523`): WAL mode for concurrent reads, `BEGIN IMMEDIATE` for exclusive write transactions. 30-second busy timeout. Well-tuned defaults.

3. **Schema migration with version tracking** (`apply_migration`, lines 813-942): 9 incremental migrations, each in a transaction. Current version is `CURRENT_SCHEMA_VERSION = 9`. Clean upgrade path.

4. **Date-prefixed sequential IDs** (line 954-981): Natural ordering by creation date without requiring timestamp parsing. Auto-incrementing per-day counter avoids collisions.

5. **Legacy import** (`import_legacy`, lines 649-691): On first run, detects old JSONL sessions and imports them into SQLite. Smooth migration path.

6. **Session type categorization** (`SessionType` enum): Multiple session types allow filtering listings. Types like `SubAgent`, `Hidden`, `Terminal`, `Acp` support internal use cases without polluting user-facing lists.

### What to Avoid

1. **No concurrency guard**: Goose has no per-session lock to prevent concurrent turns. SQLite's `BEGIN IMMEDIATE` provides transaction-level isolation but doesn't prevent logical conflicts (two simultaneous agent loops on the same session). LOGOS should add an explicit concurrency guard.

2. **Full conversation replacement**: `replace_conversation` does `DELETE FROM messages` + re-insert all. This is safe but expensive for large conversations. Consider incremental append.

## Cross-Project Comparison Matrix

| Aspect             | LOGOS (OR adapter)       | LOGOS (Codex)              | OpenCode                   | Goose                         |
| ------------------ | ------------------------ | -------------------------- | -------------------------- | ----------------------------- |
| In-memory store    | HashMap, always loaded   | HashMap, always loaded     | None (SQLite on demand)    | None (SQLite on demand)       |
| Persistence format | JSON snapshot            | Rollout JSONL + state DB   | SQLite + event sourcing    | SQLite (WAL, 9 migrations)    |
| Concurrency guard  | None                     | Single executor per thread | Runner with BusyError      | None (SQLite tx isolation)    |
| Eviction policy    | None                     | None                       | Not needed                 | Not needed                    |
| Listing mechanism  | File scan + memory merge | State DB query             | SQL query + pagination     | SQL query with type filters   |
| ID ordering        | UUID (random)            | UUID (random)              | Descending timestamp       | YYYYMMDD_N sequential         |
| Provider binding   | At thread creation       | At thread creation         | At call time (deferred)    | At call time (deferred)       |
| Parent/child       | Fork (copy, independent) | Fork (rollout copy)        | parentID field (hierarchy) | copy_session (full duplicate) |

## Recommendations for LOGOS

### What to Borrow

1. **Runner-based concurrency guard**: Add a `Runner` or equivalent to prevent concurrent turn submissions to the same thread. OpenCode's pattern is clean and simple.

2. **Consider SQLite on-demand loading**: Instead of keeping all threads in memory, load from persistence on demand. Only keep "active" threads in memory with a TTL-based eviction.

3. **Separate status tracking**: Don't embed status in thread state. Track it separately as a lightweight in-memory map, publish changes to event bus.

4. **Event-sourced mutations**: Consider event sourcing for thread state mutations — gives audit trail and replay capability that JSON snapshot doesn't.

### What to Avoid

1. **Don't eliminate in-memory map entirely**: LOGOS has long-running sessions with active streaming connections. OpenCode can load on demand because it doesn't have persistent SSE/WebSocket connections per session. LOGOS needs in-memory for active sessions but should add eviction for idle ones.

2. **Don't use flat session model**: LOGOS thread model (with continuation handles, server-side state references, etc.) is more powerful than OpenCode's flat sessions. Keep the richer model but extract the generic parts.

### What LOGOS Can Do Better

1. **Unified Thread trait**: Both OR ThreadManager and Codex ThreadManager should implement a common trait (`ThreadStore` or similar). OpenCode doesn't need this because it has one persistence model, but LOGOS has two (server-state vs client-state).

2. **Eviction policy with TTL**: Neither Codex nor OpenCode has this. LOGOS can add LRU eviction for idle threads, configurable TTL, and max-loaded limit.

3. **Provider-deferred resolution**: OpenCode resolves provider at call time, not at session creation. LOGOS currently binds provider at thread creation. Consider deferring provider resolution to turn-start time for more flexibility.

### Risks and Open Questions

1. **Migration risk**: Moving thread management out of OR adapter requires changing the dispatch logic in `codex_message_processor.rs` (10285 lines). Any change to this file is high-risk.

2. **Codex ThreadManager compatibility**: The shared `ThreadStore` trait must accommodate Codex's `ThreadManager` which has different ownership semantics (CodexThread owns its own Op dispatch loop).

3. **Performance**: Loading from SQLite on demand vs keeping in memory — what's the latency impact for resume/fork operations?
