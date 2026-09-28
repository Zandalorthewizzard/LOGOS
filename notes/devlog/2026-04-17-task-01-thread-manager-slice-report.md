---
id: 2026-04-17-task-01-thread-manager-slice-report
title: "Task 01 Thread Manager — Slice Implementation Report"
type: implementation-report
status: active
tags: [logos, phase-6, session-state, sprint, task-01]
related:
  - "[[2026-04-17-session-state-foundation-sprint-strict-spec.md]]"
  - "[[2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md]]"
  - "[[2026-04-16-item-01-implementation-reference.md]]"
  - "[[2026-04-17-session-state-foundation-slice-implementation-report-template.md]]"
depends_on:
  - "[[2026-04-17-session-state-foundation-task-01-thread-manager-strict-spec.md]]"
generated: false
source_of_truth: non-canonical
---

# Task 01: Thread Manager — Slice Implementation Report

## 1. Closed

- Created `codex-rs/logos-session-state` crate with `SqliteThreadRegistry` implementing the `ThreadRegistry` trait.
  - evidence: `codex-rs/logos-session-state/src/registry.rs`, `SqliteThreadRegistry`, lines 35-269
- `ThreadRecord`, `ThreadRecordInput`, `ThreadFilter`, `ThreadUpdateBuilder` types for durable thread identity.
  - evidence: `codex-rs/logos-session-state/src/registry.rs`, lines 37-279
  - evidence: `codex-rs/logos-session-state/src/lib.rs`, lines 1-80
- SQLite schema: `thread_records` table with `thread_id` (PK), `provider_lane`, `provider_id`, `name`, `archived_at`, `created_at`, `updated_at`, `session_state_path`, `config_snapshot_json`.
  - evidence: `codex-rs/logos-session-state/src/registry.rs`, lines 154-162
- Indexes on `archived_at` and updated-at ordering.
  - evidence: `codex-rs/logos-session-state/src/registry.rs`, lines 159-160
- Connected `OpenRouterBetaThreadManager` to the registry:
  - `ensure_registry_ready` — one-time legacy import from filesystem via `OnceCell`.
    - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, lines 122-148
  - `upsert_registry_thread` — INSERT OR REPLACE on thread start/persist.
    - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, lines 176-185
  - `update_registry_path_and_archive_state` — UPDATE on archive/unarchive.
    - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, lines 187-202
  - `thread_record` / `touch_registry_thread` / `update_registry_thread_name`.
    - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, lines 160-235
- All authoritative queries (list, read, has_persisted, resume, archive) now go through registry; in-memory `self.threads` map is only a loaded/live overlay.
  - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, `list_registry_threads`, lines 339-377
  - evidence: `codex-rs/app-server/src/openrouter_beta_threads.rs`, `read_persisted_thread`, lines 379-398
- Fixed critical bug: `openrouter_thread_matches_filters` unconditionally returned `false` when `filters.archived=true`.
  - evidence: `codex-rs/app-server/src/codex_message_processor.rs`, `openrouter_thread_matches_filters`, lines 9499-9535
- Removed all debug `eprintln!` tracing.

## 2. Borrowed From References

### 2a. SQLite Connection Pool Setup

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 510-523 (`SessionStorage::create_pool`)
4. Pattern: `SqliteConnectOptions::new().filename(path).create_if_missing(true).busy_timeout(...)` then `SqlitePoolOptions::new().connect_lazy_with(options)`

Also borrowed PRAGMA configuration concept from:

1. Source project: OpenCode
2. Local snapshot: `input-sources-temp/opencode-upstream/packages/opencode/src/storage/db.ts`
3. Lines: 90-95 (`PRAGMA journal_mode = WAL, synchronous = NORMAL, busy_timeout = 5000`)
4. Pattern: PRAGMA tuning for SQLite connection; adapted to use `SqliteConnectOptions` builder methods.

### 2b. Schema Design

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 565-646 (`SessionStorage::create_schema`)
4. Pattern: `CREATE TABLE` with identity PK (`TEXT`), metadata columns, timestamp columns, JSON blob column. `INSERT ON CONFLICT DO UPDATE` for upsert.

Plus soft-delete pattern from:

1. Source project: OpenCode
2. Local snapshot: `input-sources-temp/opencode-upstream/packages/opencode/src/session/session.sql.ts`
3. Lines: 14-44 (`SessionTable`)
4. Pattern: `time_archived: integer()` — NULL means active, non-NULL means archived (reversible soft delete).

### 2c. Builder-Pattern Updates

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 86-239 (`SessionUpdateBuilder`)
4. Pattern: struct with `Option<Option<T>>` fields for partial updates; each setter consumes and returns `Self`; `apply()` builds dynamic SQL touching only changed columns.

### 2d. Legacy Import

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 649-691 (`import_legacy`)
4. Pattern: scan filesystem for legacy-format session files, parse each, insert into SQLite. Runs once on first pool access.

### 2e. Lazy Initialization with OnceCell

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 536-557 (`pool()` method)
4. Pattern: `OnceCell::get_or_try_init` for lazy schema creation and migration on first database access.

### 2f. Dynamic Query Building for Listing

1. Source project: Goose
2. Local snapshot: `input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
3. Lines: 1261-1298 (`list_sessions_by_types`)
4. Pattern: dynamically build WHERE clause based on filter options. Adapted to use `sqlx::QueryBuilder<Sqlite>` instead of string concatenation.

Plus filter pattern from:

1. Source project: OpenCode
2. Local snapshot: `input-sources-temp/opencode-upstream/packages/opencode/src/session/index.ts`
3. Lines: 739-780 (`list`)
4. Pattern: conditional filter conditions including `isNull(archived_at)` for active-only queries.

## 3. Adapted / Renamed In LOGOS

### 3a. `SqliteThreadRegistry`

1. Destination: `codex-rs/logos-session-state/src/registry.rs`
2. New symbol: `SqliteThreadRegistry` (struct + `ThreadRegistry` trait impl)
3. What changed from Goose:
   - Goose's `SessionStorage` wraps `Pool<Sqlite>` in an `Arc` with `OnceCell<()>` for initialization. LOGOS keeps the same shape but names it `SqliteThreadRegistry`.
   - Goose stores sessions in `sessions` table. LOGOS uses `thread_records` with multi-lane columns (`provider_lane`, `provider_id`) that neither Goose nor OpenCode has.
   - Goose uses `datetime('now')` for timestamps. LOGOS uses integer Unix seconds (`i64`) for consistency with the existing OpenRouter beta codebase timestamps.
   - Goose uses string-concat SQL building. LOGOS uses `sqlx::QueryBuilder<Sqlite>` for parameterized dynamic queries.
4. Why required: LOGOS needs lane-aware dispatch and cross-provider identity that Goose's single-provider model does not support.

### 3b. `ThreadUpdateBuilder`

1. Destination: `codex-rs/logos-session-state/src/registry.rs`, lines 271-359
2. New symbol: `ThreadUpdateBuilder<'a>`
3. What changed from Goose:
   - Goose's `SessionUpdateBuilder` has many session-specific fields (schedule_id, goose_mode, etc.). LOGOS strips to registry-relevant fields: `name`, `archived_at`, `session_state_path`, `config_snapshot_json`, `updated_at`.
   - Goose uses `SessionManager::apply_update_inner`. LOGOS inlines the SQL building in `ThreadUpdateBuilder::apply()` using `sqlx::QueryBuilder`.
4. Why required: LOGOS thread records have different shape than Goose sessions; builder must match LOGOS schema.

### 3c. `ThreadFilter` / `ProviderLane`

1. Destination: `codex-rs/logos-session-state/src/lib.rs`
2. New symbols: `ThreadFilter`, `ProviderLane`
3. What changed: Neither Goose nor OpenCode has a multi-provider lane concept. `ProviderLane` enum is LOGOS-native. `ThreadFilter { provider_lane, include_archived }` combines Goose's type-based filtering with OpenCode's soft-delete filter.
4. Why required: LOGOS architecture requires lane-scoped queries for dispatch simplification (Stage C in implementation reference).

### 3d. `thread_record_input_from_persisted`

1. Destination: `codex-rs/app-server/src/openrouter_beta_threads.rs`, lines 2381-2397
2. New symbol: `thread_record_input_from_persisted`
3. What changed from Goose's `import_legacy_session`:
   - Goose parses YAML/JSON session files into a session row. LOGOS converts `PersistedOpenRouterThread` (existing LOGOS type) into `ThreadRecordInput`.
   - Goose maps session fields to column values. LOGOS maps OR-specific fields: `thread.id` → `thread_id`, `config_snapshot.model_provider_id` → `provider_id`, `thread.path` → `session_state_path`.
   - Archive state is derived from `archived` boolean parameter, mapping to `archived_at = persisted.thread.updated_at` or `None`.
4. Why required: LOGOS has a different persisted format (`PersistedOpenRouterThread`) than Goose's legacy sessions.

## 4. Rejected Or Not Taken

- **Goose's file-per-session identity** (Goose `session_manager.rs`, legacy format): Goose stores metadata inline in session JSON/YAML files. Rejected because LOGOS needs cross-provider identity queries (list all threads for a lane, list archived across providers) which file-scanning cannot serve efficiently.
- **OpenCode's Drizzle ORM** (`db.ts`): Not available in Rust. `sqlx` with `QueryBuilder` was chosen instead for equivalent dynamic query construction.
- **Goose's WAL journal mode** (`session_manager.rs:515`): Initially considered rejected, but subsequently adopted. The registry uses `journal_mode(SqliteJournalMode::Wal)` and `synchronous(SqliteSynchronous::Normal)` (matching Goose and OpenCode patterns) via `SqliteConnectOptions` builder at `registry.rs:374-380`.
- **Goose's migration framework** (`session_manager.rs:753-942`): Considered and adapted. The registry starts with a `schema_version` table and a migration gate (`run_migrations`) that checks the current version against `CURRENT_SCHEMA_VERSION`. No migration steps exist yet (version 1 only), but the gate is in place for future schema evolution.
- **Separate archived/active tables**: Considered but rejected. A single `thread_records` table with `archived_at` column (from OpenCode's soft-delete pattern) is simpler and avoids schema duplication. The index on `archived_at` makes the active/archived split efficient.
- **In-memory-only registry**: Rejected per task spec: the registry must survive process restarts and support legacy import from filesystem.
- **`LazyLock` global singleton** (Goose `session_manager.rs:52-53`): Not used. LOGOS creates `SqliteThreadRegistry` per `OpenRouterBetaThreadManager` instance rather than as a global. This allows test isolation and multiple codex_home paths.

## 5. Stale Code Removed

- `openrouter_thread_matches_filters` had an unconditional `return false` when `filters.archived=true` (`codex-rs/app-server/src/codex_message_processor.rs`, previously lines 9504-9506). This was a proving-lane artifact from before the registry existed — archived OpenRouter threads were not queryable at all. The early return was removed; archived OpenRouter threads now pass through the filter correctly and appear in `thread/list` responses.

## 6. Stale Code Still Remaining

- `openrouter_beta_persistence::list_archived_threads` (`codex-rs/app-server/src/openrouter_beta_persistence.rs`) — filesystem-based listing still called in `ensure_registry_ready` for legacy import. This is transitional: it runs exactly once per process lifetime (via `OnceCell`) and then the registry is authoritative. Not stale owner logic; marked transitional because the import path will be unnecessary once all existing deployments have populated the registry.
- `openrouter_beta_persistence::list_active_threads` — same as above, used only for legacy import.
- `remove_thread_snapshot` in `openrouter_beta_persistence.rs` is flagged as unused (`dead_code` warning). Can be removed in a cleanup pass but is not stale owner logic.
- `OpenRouterBetaThreadManager.threads` in-memory `HashMap<ThreadId, Arc<OpenRouterBetaThread>>` — this is NOT stale. It correctly serves as the loaded/live overlay only. All authoritative identity queries (list, read, has_persisted, resume) go through the registry. The in-memory map tracks only threads currently loaded in the runtime. This aligns with the task spec requirement: "the permanent proving-lane threads map must stop being the authoritative durable registry" — it has stopped. It remains as a legitimate runtime-residency overlay.

## 7. Tests / Checks Run

### Registry unit tests

```
cargo test -p logos-session-state
→ 1 passed; 0 failed
```

### OpenRouter beta tests

```
cargo test -p codex-app-server openrouter_beta::
→ 21 passed; 0 failed
```

### Related thread tests

```
cargo test -p codex-app-server thread_archive
→ 2 passed; 0 failed

cargo test -p codex-app-server thread_unarchive
→ 1 passed; 0 failed

cargo test -p codex-app-server thread_read
→ 7 passed; 0 failed

cargo test -p codex-app-server thread_resume
→ 18 passed; 0 failed

cargo test -p codex-app-server thread_loaded_list
→ 2 passed; 0 failed
```

### Import/restart regression

```
cargo test -p codex-app-server app_server_openrouter_registry_import_supports_name_and_metadata_after_restart
→ 1 passed; 0 failed
```

### Corrective-pass edge-case tests

```
cargo test -p codex-app-server app_server_openrouter_partial_git_info_update_preserves_existing_fields_after_restart
→ 1 passed; 0 failed

cargo test -p codex-app-server app_server_openrouter_elicitation_works_after_restart
→ 1 passed; 0 failed
```

### Full integration suite

```
cargo test -p codex-app-server --test all
→ 293 passed; 5 failed; 3 ignored
```

The 5 failures are pre-existing core-path issues unrelated to this slice:

- `thread_name_websocket::*` (2): WebSocket transport timeout
- `thread_shell_command_runs_as_standalone_turn_and_persists_history`: output delta assertion mismatch
- `thread_start_with_elevated_sandbox_*` (2): config loading in sandboxed core path

None of these touch the OpenRouter lane or registry code.

### Lint / format

```
just fix -p codex-app-server
→ 0 new warnings (4 pre-existing expect_used in openrouter_beta_threads.rs)
just fmt
→ clean
```

## 8. Residual Risks

1. **`remove_thread_snapshot` is unused** — dead code in `openrouter_beta_persistence.rs`. Low risk, but should be cleaned up before beta.
2. **Registry DB file placement** — currently at `{codex_home}/logos-session-state.db`. If codex_home changes between runs, a stale DB may exist. The legacy-import path handles this gracefully (checks `has_any_records` first, and `OnceCell` re-initializes per process).
3. **Migration framework is minimal** — The registry has a `schema_version` table and a version-check gate (`run_migrations` at `registry.rs:173-186`), but no actual migration steps yet (only version 1). When the schema evolves, migration steps must be added before the version is bumped.
4. **Thread-filter `archived_at` semantics** — `archived_at` stores the timestamp when archived. The `include_archived` filter uses `IS NULL` / no-filter. If a thread is archived and unarchived, `archived_at` is set back to `NULL`. This is correct but should be documented in a canonical doc.
5. **Codex threads not yet in registry** — The registry currently only contains OpenRouter beta threads. Core/Codex threads remain managed by `ThreadManager`. This is expected per Phase 1 of the dispatch migration strategy (implementation reference Section C.5). Task 02 or later sprint items should add Codex thread records as read-only entries.
6. **Partial metadata patch now covered for persisted threads** — Post-corrective-pass fix: `update_git_info` in `openrouter_beta_threads.rs` now applies patch semantics internally for both loaded and persisted-but-not-loaded threads. Test `app_server_openrouter_partial_git_info_update_preserves_existing_fields_after_restart` covers this.
7. **Elicitation counters now covered for persisted threads** — Post-corrective-pass fix: `increment_elicitation` and `decrement_elicitation` now fall through to a persisted snapshot load/save path for threads not in the live overlay. Test `app_server_openrouter_elicitation_works_after_restart` covers this.

## Evidence

- path: `codex-rs/logos-session-state/src/registry.rs`
  symbol: `SqliteThreadRegistry`
  lines: 35-269
- path: `codex-rs/logos-session-state/src/lib.rs`
  symbol: `ThreadRegistry` trait, `ThreadFilter`, `ProviderLane`
  lines: 1-80
- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `ensure_registry_ready`
  lines: 122-148
- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `list_registry_threads`
  lines: 339-377
- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `read_persisted_thread`
  lines: 379-398
- path: `codex-rs/app-server/src/openrouter_beta_threads.rs`
  symbol: `archive_persisted`
  lines: 843-901
- path: `codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `openrouter_thread_matches_filters`
  lines: 9499-9535
- path: `codex-rs/app-server/src/codex_message_processor.rs`
  symbol: `thread_list` archived branch
  lines: 3658-3674
