---
id: logos-item-02-session-persistence-implementation-reference
title: Item 02 Session Persistence Implementation Reference
type: implementation-reference
status: active
tags:
  [
    logos,
    architecture,
    phase-6,
    session-state,
    persistence,
    session-persistence,
    implementation,
  ]
related:
  [
    [
      ../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.2-session-persistence.md],
    [../research/2026-04-16-session-layer-research-transfer-ledger.md],
    [../implementation/2026-04-16-item-01-implementation-reference.md],
  ]
depends_on:
  [
    [
      ../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md,
    ],
    [../research/session layer/RA-1.2-session-persistence.md],
  ]
generated: false
source_of_truth: working-doc
outline: []
---

# Item 02: Session Persistence Implementation Reference

## Purpose

Concrete code patterns extracted from Goose (Rust) and OpenCode (TypeScript) for each stage of the Item 02 draft implementation. This document maps production patterns to LOGOS equivalents, identifying what can be refactored, adapted, or must be written from scratch.

## Source Projects

| Project  | Language   | Key Files                                                                                      | Lines |
| -------- | ---------- | ---------------------------------------------------------------------------------------------- | ----- |
| Goose    | Rust       | `crates/goose/src/session/session_manager.rs` — SQLite pool, migrations, builder, CRUD, legacy | 2032  |
| OpenCode | TypeScript | `src/storage/db.ts` — WAL config, tx propagation, migrations                                   | 174   |
| OpenCode | TypeScript | `src/session/session.sql.ts` — Schema with cascade deletes, indexes, soft-archive              | 103   |

---

## Stage A: Metadata Store Foundation

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

**LOGOS action:** Refactor with minimal changes. LOGOS uses `rusqlite` (synchronous, Codex precedent in `codex-rs/state/`) rather than `sqlx` (async). The pool setup pattern stays the same but uses `rusqlite::Connection` with `PRAGMA journal_mode=WAL` and `busy_timeout`. Codex `state` crate already uses this exact pattern.

**OpenCode** (`db.ts:90-95`) — Pattern-level reference:

```typescript
db.run("PRAGMA journal_mode = WAL");
db.run("PRAGMA synchronous = NORMAL");
db.run("PRAGMA busy_timeout = 5000");
db.run("PRAGMA cache_size = -64000");
db.run("PRAGMA foreign_keys = ON");
db.run("PRAGMA wal_checkpoint(PASSIVE)");
```

**LOGOS action:** Adopt the same PRAGMAs. `cache_size = -64000` (64MB) and `synchronous = NORMAL` are good defaults. `foreign_keys = ON` is essential for cascade deletes.

### A.2 Schema Creation

**Goose** (`session_manager.rs:565-646`) — Adapt:

```rust
async fn create_schema(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(r#"CREATE TABLE schema_version (...)"#).execute(pool).await?;
    sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
        .bind(CURRENT_SCHEMA_VERSION).execute(pool).await?;
    sqlx::query(r#"CREATE TABLE sessions (...)"#).execute(pool).await?;
    sqlx::query(r#"CREATE TABLE messages (...)"#).execute(pool).await?;
    // indexes...
    Ok(())
}
```

**LOGOS action:** Adapt schema for LOGOS `thread_metadata` table (see draft discussion). LOGOS does not need `messages` table — history is in JSONL/journal. LOGOS needs `thread_metadata` + `config_snapshots` + `thread_edges` (Item 03). The `schema_version` table pattern is taken directly.

**OpenCode** (`session.sql.ts:14-44`) — Pattern-level reference for schema design:

```typescript
export const SessionTable = sqliteTable("session", {
  id: text().primaryKey(),
  project_id: text()
    .notNull()
    .references(() => ProjectTable.id, { onDelete: "cascade" }),
  parent_id: text().$type<SessionID>(),
  time_archived: integer(), // soft-archive: NULL = active
  ...Timestamps,
});
```

**LOGOS action:** Borrow `time_archived` as `archived_at` field (soft-archive). Borrow `parent_id` for lineage (Item 03). Borrow cascade delete pattern via FK.

### A.3 Migration Framework

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

async fn apply_migration(tx: &mut Transaction, version: i32) -> Result<()> {
    match version {
        1 => { /* CREATE TABLE schema_version */ }
        2 => { /* ALTER TABLE sessions ADD COLUMN */ }
        // ... 9 migrations total
        _ => { anyhow::bail!("Unknown migration version: {}", version); }
    }
}
```

Key properties:

- `BEGIN IMMEDIATE` for exclusive access during migration
- Forward-only: no downgrade
- Each migration is a match arm
- Version tracked in `schema_version` table

**LOGOS action:** Refactor directly. LOGOS needs fewer migrations initially (Migration 1 = create tables + indexes). The pattern is clean and proven across 9 Goose migrations.

### A.4 Lazy Initialization With OnceCell

**Goose** (`session_manager.rs:535-557`) — Directly refactorable:

```rust
pub struct SessionStorage {
    pool: Pool<Sqlite>,
    initialized: tokio::sync::OnceCell<()>,
    session_dir: PathBuf,
}

async fn pool(&self) -> Result<&Pool<Sqlite>> {
    self.initialized.get_or_try_init(|| async {
        let schema_exists = /* check sqlite_master */;
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

**LOGOS action:** Refactor directly. The `OnceCell` pattern ensures migrations run exactly once. The `schema_exists` check separates fresh install from upgrade path. LOGOS needs `import_legacy` for current JSON snapshots.

### A.5 Legacy Import Path

**Goose** (`session_manager.rs:649-751`) — Directly refactorable:

```rust
async fn import_legacy(pool: &Pool<Sqlite>, session_dir: &PathBuf) -> Result<()> {
    let sessions = legacy::list_sessions(session_dir)?;
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

**LOGOS action:** Refactor directly. LOGOS legacy source is `logos-openrouter-beta/active-threads/*.json` and `archived-threads/*.json`. Each `PersistedOpenRouterThread` JSON becomes:

1. One `thread_metadata` row (extract metadata fields)
2. One `config_snapshots` row (extract config)
3. One JSONL history file (extract `Vec<InternalMessage>`)

Original JSON files are NOT deleted — left as backup until cleanup pass confirms success.

### A.6 Transaction Propagation

**OpenCode** (`db.ts:125-173`) — Pattern-level reference:

```typescript
const ctx = Context.create<{ tx: TxOrDb; effects: (() => void)[] }>("database");

export function use<T>(callback: (trx: TxOrDb) => T): T {
  try {
    return callback(ctx.use().tx); // join existing tx
  } catch (err) {
    if (err instanceof Context.NotFound) {
      const effects = [];
      const result = ctx.provide({ effects, tx: Client() }, () =>
        callback(Client()),
      );
      for (const effect of effects) effect(); // deferred side effects
      return result;
    }
  }
}

export function transaction<T>(callback: (tx: TxOrDb) => T): T {
  // same nesting pattern, wraps in real SQLite tx
}
```

**LOGOS action:** Adapt pattern to Rust. LOGOS can use a simpler approach: pass `&Connection` or `&mut Connection` through function calls. Rust's borrow checker provides the same safety as OpenCode's context propagation without needing an explicit context system. The key idea to borrow: deferred side effects (publish to bus after commit, not during).

---

## Stage B: History/Blob Separation

### B.1 Partial Update Builder

**Goose** (`session_manager.rs:86-239, 1024-1123`) — Directly refactorable:

```rust
pub struct SessionUpdateBuilder<'a> {
    session_manager: &'a SessionManager,
    session_id: String,
    name: Option<String>,
    // ... each field is Option<FieldType>
}

impl<'a> SessionUpdateBuilder<'a> {
    pub fn user_provided_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name);
        self
    }
    pub async fn apply(self) -> Result<()> {
        self.session_manager.apply_update_inner(self).await
    }
}
```

Dynamic SQL generation (`session_manager.rs:1024-1123`):

```rust
async fn apply_update(&self, builder: SessionUpdateBuilder<'_>) -> Result<()> {
    let mut updates = Vec::new();
    let mut query = String::from("UPDATE sessions SET ");
    macro_rules! add_update {
        ($field:expr, $name:expr) => {
            if $field.is_some() {
                if !updates.is_empty() { query.push_str(", "); }
                updates.push($name);
                query.push_str($name);
                query.push_str(" = ?");
            }
        };
    }
    add_update!(builder.name, "name");
    // ...
    query.push_str(", updated_at = datetime('now') WHERE id = ?");
    // bind values...
}
```

**LOGOS action:** Refactor directly. LOGOS `ThreadMetadataUpdateBuilder` will have LOGOS-specific fields:

- `name: Option<String>`
- `archived_at: Option<Option<i64>>` (Some(None) = unarchive)
- `status: Option<String>`
- `preview: Option<String>`
- `git_branch: Option<Option<String>>`
- `git_commit: Option<Option<String>>`

The macro-based dynamic SQL pattern is clean and avoids the current "rewrite entire JSON" problem. Only changed fields produce SQL SET clauses.

### B.2 History Separation: Append-Only JSONL

**No competitor equivalent.** Both Goose and OpenCode store messages in SQLite. LOGOS deliberately separates history into append-only JSONL files because:

1. Phase-4 journal is append-oriented. JSONL is a transitional bridge.
2. Monolithic JSON rewrite is the current weakness. JSONL append is resilient.
3. Large histories do not bloat the metadata SQLite DB.

**LOGOS action:** Write from scratch.

Transitional JSONL format:

```rust
struct HistoryEntry {
    seq: u64,
    kind: HistoryEntryKind,
}

enum HistoryEntryKind {
    Message(InternalMessage),
    Continuation(ContinuationHandle),
    Elicitation { count: u64 },
}
```

Write pattern: `tokio::fs::OpenOptions::new().append(true).open(path)` → serialize one line → write + flush.

Read pattern: `tokio::fs::read_to_string(path)` → split lines → deserialize each. Last corrupt line is discarded.

### B.3 Config Snapshot Storage

**Goose** stores `model_config_json TEXT` inline in sessions table. **OpenCode** stores message `data` as JSON blob inline.

**LOGOS action:** Write from scratch, but simpler than competitors. LOGOS config snapshots are small (~1KB). Store inline in `config_snapshots` SQLite table:

```sql
CREATE TABLE config_snapshots (
    thread_id TEXT PRIMARY KEY NOT NULL,
    data TEXT NOT NULL,  -- JSON blob of PersistedThreadConfigSnapshot
    FOREIGN KEY (thread_id) REFERENCES thread_metadata(thread_id) ON DELETE CASCADE
);
```

This avoids a separate file store while keeping config out of the metadata table.

---

## Stage C: Journal Integration

### C.1 Journal Boundary Definition

**No competitor equivalent.** Journal is LOGOS-specific. No competitor has a separate append-oriented canonical truth layer alongside a SQLite metadata store.

**LOGOS action:** Write from scratch. The boundary is:

- When journal ships: `session_state_ref` in `thread_metadata` changes from JSONL file path to journal session identifier.
- `MessageAppended` journal records become the canonical history source.
- `ContinuationUpdated` journal records become the canonical continuation source.
- JSONL files become import source for journal on first journal-backed startup.
- SQLite metadata store is unchanged — only the reference target changes.

This is a runtime behavior change, not a schema change. The `session_state_ref TEXT` field is opaque — it can point to a JSONL file or a journal session ID.

---

## Cross-Cutting: What Goose Gets Right

| Pattern                      | Source                       | Value for LOGOS                               |
| ---------------------------- | ---------------------------- | --------------------------------------------- |
| SQLite + WAL pool setup      | `session_manager.rs:510-523` | Production-proven defaults                    |
| Version-tracked migrations   | `session_manager.rs:753-942` | 9 migrations, forward-only, `BEGIN IMMEDIATE` |
| Lazy init with OnceCell      | `session_manager.rs:535-557` | Run migrations exactly once                   |
| Legacy import on first run   | `session_manager.rs:649-751` | Graceful JSONL → SQLite migration             |
| Partial update builder       | `session_manager.rs:86-239`  | Dynamic SQL, only changed fields              |
| `BEGIN IMMEDIATE` for writes | `session_manager.rs:694`     | Exclusive access during critical writes       |

## Cross-Cutting: What OpenCode Gets Right

| Pattern                          | Source                 | Value for LOGOS                  |
| -------------------------------- | ---------------------- | -------------------------------- |
| WAL + NORMAL sync + busy timeout | `db.ts:90-95`          | Well-tuned performance defaults  |
| Cascade deletes via FK           | `session.sql.ts:20-21` | Clean parent-child cleanup       |
| Soft-archive field               | `session.sql.ts:37`    | `time_archived` — no file moves  |
| Deferred side effects            | `db.ts:144-151`        | Publish after commit, not during |

## Cross-Cutting: What Both Get Wrong

| Pattern                      | Problem                                                 | LOGOS Better Approach                                |
| ---------------------------- | ------------------------------------------------------- | ---------------------------------------------------- |
| Everything in SQLite         | Both store messages as JSON blobs in SQLite             | Separate JSONL for history; SQLite for metadata only |
| No journal boundary          | No separation between canonical truth and derived index | Journal = canonical; SQLite = non-canonical index    |
| No config validation on load | Neither validates model/tool availability               | Validation step on thread load (Item 03)             |

## Implementation Priority

| Step      | Action                                               | Source                 | Estimated Effort |
| --------- | ---------------------------------------------------- | ---------------------- | ---------------- |
| 1         | SQLite connection pool + PRAGMAs                     | Goose refactor         | 0.5 day          |
| 2         | Schema creation (thread_metadata, config_snapshots)  | Goose adapt            | 0.5 day          |
| 3         | Migration framework (version 0 → 1)                  | Goose refactor         | 1 day            |
| 4         | CRUD operations for thread_metadata                  | Goose adapt            | 1 day            |
| 5         | Partial update builder                               | Goose refactor         | 1 day            |
| 6         | Legacy JSON → SQLite + JSONL import                  | Goose refactor + new   | 1.5 days         |
| 7         | JSONL append-only history writer/reader              | Write from scratch     | 1.5 days         |
| 8         | Config snapshot CRUD                                 | Write from scratch     | 0.5 day          |
| 9         | Query surfaces (list, filter, search)                | New                    | 1 day            |
| 10        | Integration with openrouter_beta_persistence callers | Adapt existing         | 1 day            |
| 11        | Deferred side effects pattern                        | OpenCode pattern-level | 0.5 day          |
| **Total** |                                                      |                        | **~10.5 days**   |

## Evidence

- path: `../architecture/2026-04-17-item-02-session-persistence-draft-implementation-discussion.md`
  symbol: `Draft Target Model`
  lines: `101-165`
- path: `../research/session layer/RA-1.2-session-persistence.md`
  symbol: `Cross-Project Comparison Matrix`
  lines: `202-216`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::create_pool`
  lines: `510-523`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::create_schema`
  lines: `565-646`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::run_migrations`
  lines: `753-776`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::apply_migration`
  lines: `812-942`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::import_legacy`
  lines: `649-751`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionUpdateBuilder`
  lines: `86-239`
- path: `../../input-sources-temp/goose-upstream/crates/goose/src/session/session_manager.rs`
  symbol: `SessionStorage::apply_update`
  lines: `1024-1123`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/storage/db.ts`
  symbol: `Database.Client`
  lines: `85-116`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/storage/db.ts`
  symbol: `Database.use / Database.effect`
  lines: `130-151`
- path: `../../input-sources-temp/opencode-upstream/packages/opencode/src/session/session.sql.ts`
  symbol: `SessionTable`
  lines: `14-44`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `PersistedOpenRouterThread`
  lines: `24-47`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `write_snapshot`
  lines: `210-234`
- path: `../../../codex-rs/app-server/src/openrouter_beta_persistence.rs`
  symbol: `list_snapshots`
  lines: `185-208`
- path: `../../../codex-rs/state/src/runtime/threads.rs`
  symbol: `upsert_thread_spawn_edge`
  lines: `84-108`
