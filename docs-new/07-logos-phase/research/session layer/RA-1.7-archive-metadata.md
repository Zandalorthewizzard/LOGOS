# RA-1.7 — Archive/Unarchive + Metadata

## LOGOS Current Implementation

**Where:** `codex-rs/app-server/src/openrouter_beta_threads.rs`

| Operation                         | Location | What it does                                                                                       |
| --------------------------------- | -------- | -------------------------------------------------------------------------------------------------- |
| `archive_thread`                  | :481-545 | Move JSON from `active-threads/` to `archived-threads/`, update in-memory state, emit notification |
| `unarchive_thread`                | :546-624 | Reverse of archive                                                                                 |
| `set_thread_name`                 | :314-333 | Mutate `OpenRouterThreadState.name`, persist to JSON                                               |
| `update_git_info`                 | :335-354 | Mutate git metadata, persist                                                                       |
| `increment/decrement_elicitation` | :356-397 | Local counter in thread state                                                                      |

**Codex equivalent:** Same operations through `Op::SetThreadName`, state DB mutations, and file moves via `archive_thread_common()`.

**Known weaknesses:**

- All operations require OR-specific code paths despite being identical in logic to Codex path
- No shared interface between OR and Codex metadata operations
- Archive is file-system-only, no search/query capability

## OpenCode Analysis

### Archive

**File:** `src/session/index.ts:559`

```typescript
setArchived(at: number) {
  return this.patch({ time: { archived: at } })
}
```

Archive is a **timestamp field** on the session record. No file moves. The session remains in the same SQLite table. Archived sessions are filtered out of default listing queries.

### Listing with Archive Awareness

**File:** `src/session/index.ts:739-849`

`list()` supports filtering:

- `start` — cursor for pagination by `time_updated`
- `search` — text search on session data
- Archived sessions excluded by default

`listGlobal()` (lines 782-849): Cross-project listing with same filters.

### Metadata Updates

**File:** `src/session/index.ts:548-596`

All mutations go through `patch()` which calls `SyncEvent.run(Event.Updated, ...)`. Methods:

- `touch()` — update `time.updated`
- `setTitle(title)` — change title
- `setArchived(at)` — archive/unarchive
- `setPermission(ruleset)` — change permissions
- `setRevert(state)` — set revert state
- `setSummary(summary)` — update diff summary

### Key Patterns

1. **Archive as soft delete**: No file moves, just a timestamp field. Simpler, reversible, queryable.
2. **Single mutation path**: All metadata changes go through `patch()` → `SyncEvent.run()` → transactional write.
3. **SQL-based filtering**: Archive status is a query filter, not a physical location change.

### Comparison

| Aspect             | LOGOS (OR)                                     | LOGOS (Codex)                  | OpenCode                           |
| ------------------ | ---------------------------------------------- | ------------------------------ | ---------------------------------- |
| Archive mechanism  | File move between directories                  | File move + state DB flag      | Timestamp field (soft delete)      |
| Unarchive          | Reverse file move                              | Reverse file move              | Clear timestamp                    |
| Metadata mutations | Direct state mutation + JSON persist           | `Op::SetThreadName` + state DB | `patch()` → SyncEvent → SQL update |
| Listing archived   | Scan archived directory                        | State DB query with filter     | SQL query with filter              |
| Atomicity          | Non-atomic (file move + state update separate) | Partially atomic               | Transactional (SQLite)             |
| Search             | None                                           | State DB text search           | SQL text search                    |

## Goose Analysis

### Archive

**File:** `crates/goose/src/session/session_manager.rs:1305-1331`

Goose has **hard delete only** — no archive/unarchive mechanism.

```rust
async fn delete_session(&self, session_id: &str) -> Result<()> {
    // BEGIN IMMEDIATE transaction
    // DELETE FROM messages WHERE session_id = ?
    // DELETE FROM sessions WHERE id = ?
    // commit
}
```

- Checks existence first, returns error if not found
- Uses `BEGIN IMMEDIATE` transaction for consistency
- Cascade: messages deleted before session row
- **No soft delete, no archive, no trash**

### Export/Import

- `export_session(id)` — serializes full session (with conversation) to JSON string
- `import_session(json, session_type_override)` — deserializes, creates new session, copies metadata + conversation
- Used for data portability, not archiving

### Metadata Updates

All metadata mutations go through `SessionUpdateBuilder`:

```rust
session_manager.update(&session.id)
    .user_provided_name("new name")
    .provider_name("anthropic")
    .model_config(model_config)
    .goose_mode(GooseMode::Auto)
    .apply()
    .await?;
```

- Builder collects changes, constructs dynamic `UPDATE sessions SET ... WHERE id = ?`
- Only changed fields are included in the SQL
- `updated_at` is always set to `datetime('now')`
- Transactional via `BEGIN IMMEDIATE`

### Session Name Auto-Generation

`maybe_update_name()` (lines 340-363):

- Checks if `user_set_name == false` and user message count <= 3
- Calls `provider.generate_session_name()` which uses the LLM itself to generate a short description
- System prompt from `session_name.md` template
- Uses fast model (with fallback to regular model) for cost efficiency

### Key Patterns

1. **Hard delete is simple but final**: No undo, no recovery. Users must rely on export for backup.

2. **LLM-generated session names**: Uses the configured provider to generate a session description. Smart but adds latency and cost on first few messages.

3. **Builder pattern for mutations**: Clean, extensible, type-safe partial updates. Better than OpenCode's generic `patch()` because it's compile-time checked.

### Comparison

| Aspect             | LOGOS (OR)                                     | Goose                                |
| ------------------ | ---------------------------------------------- | ------------------------------------ |
| Archive mechanism  | File move between directories                  | Not implemented (hard delete only)   |
| Unarchive          | Reverse file move                              | Not implemented                      |
| Metadata mutations | Direct state mutation + JSON persist           | SessionUpdateBuilder + SQL UPDATE    |
| Listing archived   | Scan archived directory                        | N/A                                  |
| Atomicity          | Non-atomic (file move + state update separate) | Transactional (SQLite)               |
| Search             | None                                           | chat_history_search (SQL LIKE query) |
| Session naming     | Manual or first message preview                | LLM-generated (provider call)        |

## Cross-Project Comparison Matrix

| Aspect           | LOGOS (OR)         | LOGOS (Codex)      | OpenCode                | Goose                          |
| ---------------- | ------------------ | ------------------ | ----------------------- | ------------------------------ |
| Archive model    | Physical file move | Physical file move | Soft delete (timestamp) | Hard delete only               |
| Mutation pattern | Direct + persist   | Op dispatch        | Event-sourced patch     | Builder + SQL UPDATE           |
| Transactional    | No                 | Partially          | Yes                     | Yes (BEGIN IMMEDIATE)          |
| Queryable        | No (file scan)     | Yes (state DB)     | Yes (SQL)               | Yes (SQL + chat history search |
| Search           | No                 | Yes                | Yes                     | Yes (keyword-based SQL LIKE)   |
| Session naming   | Manual             | Manual             | Manual                  | LLM-generated (auto)           |

## Recommendations for LOGOS

### What to Borrow

1. **Soft-delete archive**: Don't move files. Use an `archived_at` timestamp field (in SQLite metadata store). Simpler, reversible, queryable. If LOGOS adopts SQLite for metadata (per RA-1.2), this is natural.

2. **Single mutation interface**: Define a `ThreadMetadata` trait with methods like `set_name`, `set_archived`, `update_git_info`. Both OR and Codex paths implement it against their respective stores.

3. **Event-based mutations**: Consider SyncEvent-like pattern for metadata changes. Even if not full event sourcing, a `patch()` function that transactionally updates metadata and emits a notification is cleaner than direct mutation.

### What to Avoid

1. **Don't use file moves for archive**: OpenCode's soft-delete pattern is strictly better. File moves are non-atomic, require directory management, and don't support search.

### What LOGOS Can Do Better

1. **Unified metadata interface**: This is the easiest mechanic to migrate. Define a trait:

   ```rust
   trait ThreadMetadataStore {
       fn set_name(&self, thread_id: &ThreadId, name: &str) -> Result<()>;
       fn set_archived(&self, thread_id: &ThreadId, at: Option<i64>) -> Result<()>;
       fn update_git_info(&self, thread_id: &ThreadId, info: &GitInfo) -> Result<()>;
       fn elicitation_count(&self, thread_id: &ThreadId) -> Result<u64>;
       fn increment_elicitation(&self, thread_id: &ThreadId) -> Result<u64>;
       fn decrement_elicitation(&self, thread_id: &ThreadId) -> Result<u64>;
   }
   ```

   Two implementations: one backed by SQLite (stateless), one backed by Codex state DB.

2. **Archive with filter, not move**: Store archived state in metadata, filter in listing queries. Keep the JSON/history files where they are.

### Risks and Open Questions

1. **Minimal risk**: This is the simplest migration item. No complex state management, no streaming, no tool execution. Good candidate for first implementation.

2. **Backward compatibility**: Existing OR sessions archived via file move need to be handled during migration.
