# RA-1.2 — Session Persistence Layer

## LOGOS Current Implementation

**Where:** `codex-rs/app-server/src/openrouter_beta_persistence.rs` (267 lines)

`PersistedOpenRouterThread` stores:

- `version` — snapshot format version
- `config_snapshot` — model, provider, policies, cwd, ephemeral settings, reasoning_effort, personality, session_source
- `thread` — Thread metadata (id, turns, name, status, preview, path, git_info)
- `history: Vec<InternalMessage>` — full message history for replay
- `continuation: Option<ContinuationHandle>` — resume context
- `elicitation_count: u64` — approval gating state

**Storage:** JSON files in `logos-openrouter-beta/active-threads/` and `logos-openrouter-beta/archived-threads/`

**Codex equivalent:** Rollout JSONL files (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`) + SQLite state DB

**Known weaknesses:**

- Monolithic JSON — corruption = total loss of entire session
- No incremental writes — full rewrite on every persist
- Snapshot grows linearly with history (no compression)
- No transactional guarantee during write
- Different format from Codex rollout (no shared tooling)

## OpenCode Analysis

### Architecture

OpenCode uses a **dual persistence model**: SQLite for structured data + JSON file store for large blobs.

**Key files:**

- `src/storage/db.ts` — SQLite via Drizzle ORM (WAL mode, Bun native driver)
- `src/session/session.sql.ts` — Table schemas
- `src/storage/storage.ts` — JSON file store for diffs and large data
- `src/sync/index.ts` — Event sourcing framework

### SQLite Schema

**SessionTable** (`session.sql.ts:14-44`):

- `id` (PK), `project_id` (FK cascade), `workspace_id`, `parent_id` (self-ref FK)
- `slug`, `directory`, `title`, `version`, `share_url`
- `summary_additions`, `summary_deletions`, `summary_files`, `summary_diffs` (JSON)
- `revert` (JSON), `permission` (JSON)
- `time_created`, `time_updated`, `time_compacting`, `time_archived`

**MessageTable** (`session.sql.ts:46-58`):

- `id` (PK), `session_id` (FK cascade)
- `data` (JSON) — stores full message object minus id/sessionID
- `time_created`, `time_updated`

**PartTable** (`session.sql.ts:60-76`):

- `id` (PK), `message_id` (FK cascade), `session_id`
- `data` (JSON) — stores full part object

### Event Sourcing

**`SyncEvent.run()`** (`src/sync/index.ts:195-227`):

1. Opens **immediate SQLite transaction**
2. Allocates EventID, reads/increments sequence number for aggregate
3. Calls projector function (writes to read-model tables)
4. Persists event to EventTable + updates EventSequenceTable
5. Commits transaction
6. Emits to in-process bus

This gives:

- **Transactional integrity** — mutations are atomic
- **Audit trail** — every mutation recorded as event with sequence number
- **Replay capability** — `SyncEvent.replay()` can rebuild state from events
- **Idempotency** — replay checks sequence numbers

### JSON File Store

**`src/storage/storage.ts`:**

- Path: `Global.Path.data/storage/`
- Used for session diffs (`["session_diff", sessionID]`) and large data
- Effect's `TxReentrantLock` for file-level locking
- Migration logic from older formats

### Key Differences

| Aspect                | LOGOS (OR)               | LOGOS (Codex)                | OpenCode                              |
| --------------------- | ------------------------ | ---------------------------- | ------------------------------------- |
| Storage format        | Monolithic JSON          | JSONL (append-only) + SQLite | SQLite + event sourcing + JSON files  |
| Write pattern         | Full rewrite             | Append line                  | Event-sourced transactional           |
| Corruption resilience | Total loss on corruption | Lose last line at worst      | SQLite WAL + transactions = resilient |
| Incremental updates   | None                     | Append-only                  | Event append + projector              |
| Query capability      | None (load entire file)  | Scan JSONL lines             | Full SQL queries                      |
| Schema migration      | Manual version field     | Not formalized               | Drizzle migrations                    |
| Concurrency           | File locking             | Internal locking             | SQLite transactions + Effect Context  |
| Large data handling   | Everything in one file   | Everything in one file       | Separate JSON store for blobs         |

### Reusable Patterns

1. **Event sourcing with projectors** (`src/sync/index.ts`): Every mutation is an event → projector → read model. This is significantly more robust than LOGOS OR's monolithic JSON snapshot. Consider this for the shared stateless layer.

2. **SQLite + WAL mode** (`src/storage/db.ts:90-95`): WAL mode enables concurrent reads during writes, NORMAL sync for performance, foreign keys enforced. Well-tuned defaults (64MB cache, 5s busy timeout).

3. **Context-based transaction propagation** (`src/storage/db.ts:125-173`): Uses Effect's `Context.create` to thread transactions through call chains. `Database.use()` joins existing transaction or creates new one. `Database.effect()` defers side effects (bus publishes) until after commit.

4. **Separate blob storage** (`src/storage/storage.ts`): Large data (diffs, images) stored in separate JSON files, not in SQLite columns. Keeps DB lean.

5. **Cascade deletes** (`session.sql.ts`): `ON DELETE CASCADE` on foreign keys — deleting a session automatically cleans up messages and parts.

### What to Avoid

1. **Don't store everything in SQLite**: OpenCode stores message content as JSON blobs in SQLite columns. This works for their scale but limits queryability. For LOGOS, consider whether structured columns or JSON blobs are better for the access patterns needed.

2. **Don't use SQLite for streaming data**: OpenCode's approach of storing every streaming delta (text delta, reasoning delta) through event sourcing is expensive. LOGOS should only persist final state, not intermediate streaming events.

## Goose Analysis

### Architecture

Goose uses **SQLite as its sole persistence backend**, with schema migrations and a legacy import path.

**Key files:**

- `crates/goose/src/session/session_manager.rs` — SessionStorage + SessionManager (1538+ lines)
- `crates/goose/src/session/legacy.rs` — Legacy JSONL session import (141 lines)
- `crates/goose/src/token_counter.rs` — Token counting with tiktoken (305 lines)

### SQLite Schema

**sessions table** (line 582-608):

- `id TEXT PRIMARY KEY` — `YYYYMMDD_N` format
- `name`, `description`, `user_set_name`, `session_type`, `working_dir`
- `created_at`, `updated_at` — timestamps
- `extension_data TEXT` — JSON blob
- `total_tokens`, `input_tokens`, `output_tokens` — per-session token counts
- `accumulated_total_tokens`, etc. — cumulative across compaction cycles
- `schedule_id`, `recipe_json`, `user_recipe_values_json` — scheduling metadata
- `provider_name TEXT`, `model_config_json TEXT` — per-session provider/model
- `goose_mode TEXT` — auto/manual/etc

**messages table** (line 612-628):

- `id INTEGER PK AUTOINCREMENT`
- `message_id TEXT` — stable ID for updates (`msg_{session_id}_{uuid}`)
- `session_id TEXT FK` — with index
- `role TEXT`, `content_json TEXT`, `created_timestamp INTEGER`
- `tokens INTEGER`, `metadata_json TEXT`

### Write Pattern

- **Add message**: `INSERT INTO messages` + `UPDATE sessions SET updated_at` in `BEGIN IMMEDIATE` transaction
- **Replace conversation**: `DELETE FROM messages WHERE session_id = ?` + re-INSERT all in `BEGIN IMMEDIATE`
- **Partial update**: `SessionUpdateBuilder` constructs dynamic SQL `UPDATE sessions SET ... WHERE id = ?` with only changed fields
- **Schema migration**: 9 sequential migrations, each in a transaction, with version tracking

### Token Counting

`TokenCounter` (token_counter.rs):

- Uses `tiktoken_rs::o200k_base()` (GPT-4o tokenizer) as singleton
- `DashMap` cache (10K entries) with hash-based lookup
- `count_chat_tokens(system, messages, tools)` — counts full context
- `count_tokens_for_tools(tools)` — estimates tool schema cost with fixed constants
- Respects `message.metadata.agent_visible` — skips invisible messages
- Single tokenizer for all providers (approximation for non-OpenAI models)

### Key Differences

| Aspect                | LOGOS (OR)           | Goose                                          |
| --------------------- | -------------------- | ---------------------------------------------- |
| Storage format        | Monolithic JSON file | SQLite (sqlx, WAL mode)                        |
| Write pattern         | Full JSON rewrite    | SQL INSERT/UPDATE in transactions              |
| Corruption resilience | Low (total loss)     | High (WAL + transactions)                      |
| Schema evolution      | Manual version field | 9 incremental migrations with version tracking |
| Message IDs           | None visible         | Stable `msg_{session_id}_{uuid}`               |
| Token tracking        | None                 | Per-session + accumulated token counts         |
| Provider/model store  | In config snapshot   | Per-session `provider_name` + `model_config`   |
| Legacy migration      | None                 | JSONL → SQLite auto-import on first run        |
| Conversation replace  | Full JSON rewrite    | DELETE + re-INSERT in transaction              |

### Reusable Patterns

1. **Schema migration framework** (`apply_migration`): Clean version-tracked migration system. Each migration is a match arm on version number. `BEGIN IMMEDIATE` ensures exclusive access during migration. LOGOS should adopt this pattern.

2. **Accumulated token counts**: Goose tracks both per-session and accumulated token counts. Accumulated counts persist across compaction/summarization cycles where per-session counts may reset. Useful for billing/analytics.

3. **Stable message IDs**: `msg_{session_id}_{uuid}` format allows updating message metadata without message content changes. Useful for per-message provider tracking, approval states, etc.

4. **Token counting with cache**: `TokenCounter` with `DashMap` concurrent cache is a good pattern for context window management. 10K entry limit with simple eviction prevents unbounded memory growth.

### What to Avoid

1. **Single tokenizer for all providers**: Goose uses `o200k_base` (OpenAI's tokenizer) for all providers. This is an approximation for Anthropic, Google, etc. LOGOS should use provider-specific tokenizers where available.

2. **Full conversation replacement**: `replace_conversation` does `DELETE + INSERT all`. For large conversations this is expensive. Incremental append + selective delete is more efficient.

## Cross-Project Comparison Matrix

| Aspect                | LOGOS (OR)        | LOGOS (Codex)           | OpenCode                | Goose                          |
| --------------------- | ----------------- | ----------------------- | ----------------------- | ------------------------------ |
| Primary store         | JSON file         | JSONL + SQLite          | SQLite + event log      | SQLite (WAL, sqlx)             |
| Write atomicity       | None              | Line append             | SQLite transaction      | SQLite `BEGIN IMMEDIATE`       |
| Corruption resilience | Low (total loss)  | Medium (lose last line) | High (WAL + tx)         | High (WAL + tx)                |
| Incremental update    | No (full rewrite) | Yes (append)            | Yes (event append)      | Partial (INSERT msg / builder) |
| Query support         | None              | Scan                    | Full SQL                | Full SQL                       |
| Schema evolution      | Version field     | None                    | Drizzle migrations      | 9 versioned migrations         |
| Blob storage          | In-file           | In-file                 | Separate JSON store     | In-DB (content_json column)    |
| Event sourcing        | No                | No                      | Yes                     | No                             |
| Concurrency control   | File lock         | Internal                | SQLite + Effect Context | SQLite tx + busy timeout       |
| Legacy migration      | None              | None                    | N/A                     | JSONL → SQLite auto-import     |
| Token counting        | None              | None                    | External (provider)     | tiktoken_rs with DashMap cache |

## Recommendations for LOGOS

### What to Borrow

1. **SQLite as primary store for stateless sessions**: Replace JSON files with SQLite. Benefits: transactional writes, query capability, WAL mode for concurrent reads, schema migrations via Drizzle or equivalent.

2. **Event-sourced write path**: Consider SyncEvent-like pattern for stateless session mutations. Every mutation → event → projector → read model. This gives audit trail and replay capability.

3. **Cascade deletes**: Foreign key relationships with cascade delete simplify cleanup.

4. **Separate large data**: Keep history messages and tool outputs in a separate store (could be JSON files) while metadata goes to SQLite. Don't bloat the DB with large blobs.

### What to Avoid

1. **Don't replicate OpenCode's per-delta event sourcing**: OpenCode persists every streaming text delta. For LOGOS, only persist final turn state — streaming deltas are transient.

2. **Don't abandon append-only for history**: JSONL-style append-only for message history is a good pattern that Codex uses. Consider hybrid: SQLite for metadata + JSONL for history.

### What LOGOS Can Do Better

1. **Hybrid persistence model**: SQLite for metadata (thread info, listings, status) + append-only JSONL for history (like Codex rollout). Best of both worlds: queryability for metadata, resilience for history.

2. **Compaction-aware storage**: Design persistence with compaction in mind from the start. OpenCode's tool output pruning (marking outputs as compacted) is a good pattern — old tool outputs replaced with `[Old tool result content cleared]`.

3. **Shared format between Codex and stateless paths**: If possible, design the stateless persistence format to be compatible with or derived from Codex rollout format. This reduces the gap between the two paths.

### Risks and Open Questions

1. **SQLite availability in LOGOS**: LOGOS is Rust-based. SQLite is available via `rusqlite` but adds a dependency. Is this acceptable? Codex already uses SQLite for state DB, so the precedent exists.

2. **Performance under load**: OpenCode uses SQLite for everything. At scale (many sessions, long histories), will SQLite remain performant? Need benchmarks.

3. **Migration path**: Current OR sessions are stored as JSON files. Migration to new format needs to be designed.
