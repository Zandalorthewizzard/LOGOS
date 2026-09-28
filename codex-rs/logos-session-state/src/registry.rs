use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use codex_protocol::ThreadId;
use sqlx::ConnectOptions;
use sqlx::Executor;
use sqlx::QueryBuilder;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::sqlite::SqliteSynchronous;
use tokio::sync::OnceCell;

pub const THREAD_REGISTRY_DIR: &str = "logos-session-state";
pub const THREAD_REGISTRY_DB_FILENAME: &str = "thread-registry.sqlite";
const CURRENT_SCHEMA_VERSION: i64 = 2;
const THREAD_RECORD_COLUMNS: &str = "thread_id, provider_lane, provider_id, forked_from_id, name, archived_at, status, config_snapshot_ref, session_state_ref, created_at, updated_at, session_state_path, config_snapshot_json, cwd, ephemeral, cli_version, source_json, agent_nickname, agent_role, git_branch, git_commit, git_origin_url, preview";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderLane {
    OpenRouterBeta,
}

impl ProviderLane {
    fn as_str(self) -> &'static str {
        match self {
            Self::OpenRouterBeta => "openrouter_beta",
        }
    }
}

impl FromStr for ProviderLane {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openrouter_beta" => Ok(Self::OpenRouterBeta),
            _ => anyhow::bail!("unknown provider lane: {value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadRecord {
    pub thread_id: ThreadId,
    pub provider_lane: ProviderLane,
    pub provider_id: String,
    pub forked_from_id: Option<String>,
    pub name: Option<String>,
    pub archived_at: Option<i64>,
    pub status: String,
    pub config_snapshot_ref: Option<String>,
    pub session_state_ref: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub session_state_path: Option<PathBuf>,
    pub config_snapshot_json: String,
    pub cwd: PathBuf,
    pub ephemeral: bool,
    pub cli_version: String,
    pub source_json: String,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub git_origin_url: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ThreadRecordInput {
    pub thread_id: ThreadId,
    pub provider_lane: ProviderLane,
    pub provider_id: String,
    pub forked_from_id: Option<String>,
    pub name: Option<String>,
    pub archived_at: Option<i64>,
    pub status: String,
    pub config_snapshot_ref: Option<String>,
    pub session_state_ref: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub session_state_path: Option<PathBuf>,
    pub config_snapshot_json: String,
    pub cwd: PathBuf,
    pub ephemeral: bool,
    pub cli_version: String,
    pub source_json: String,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub git_origin_url: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ThreadFilter {
    pub provider_lane: Option<ProviderLane>,
    pub include_archived: bool,
}

impl ThreadFilter {
    pub fn for_lane(provider_lane: ProviderLane) -> Self {
        Self {
            provider_lane: Some(provider_lane),
            include_archived: false,
        }
    }

    pub fn including_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }
}

/// Partial Git metadata patch for a thread record.
///
/// `None` means "leave unchanged", while `Some(None)` clears the stored value.
#[derive(Debug, Clone, Default)]
pub struct ThreadGitInfoPatch {
    pub sha: Option<Option<String>>,
    pub branch: Option<Option<String>>,
    pub origin_url: Option<Option<String>>,
}

/// Shared LOGOS-owned thread registry abstraction for non-canonical lanes.
///
/// Implementations own durable thread identity and coarse metadata, while
/// provider-specific runtime state remains outside the registry.
#[async_trait]
pub trait ThreadRegistry: Send + Sync {
    async fn upsert_thread(&self, input: ThreadRecordInput) -> Result<()>;

    async fn get_thread(&self, thread_id: ThreadId) -> Result<Option<ThreadRecord>>;

    async fn get_provider_lane(&self, thread_id: ThreadId) -> Result<Option<ProviderLane>>;

    async fn list_threads(&self, filter: ThreadFilter) -> Result<Vec<ThreadRecord>>;

    async fn has_any_records(&self) -> Result<bool>;

    fn update(&self, thread_id: ThreadId) -> ThreadUpdateBuilder<'_>;
}

#[derive(Debug)]
pub struct SqliteThreadRegistry {
    pool: SqlitePool,
    initialized: OnceCell<()>,
}

impl SqliteThreadRegistry {
    pub fn new(codex_home: PathBuf) -> Self {
        let db_path = thread_registry_db_path(codex_home.as_path());
        Self {
            pool: create_pool(db_path.as_path()),
            initialized: OnceCell::new(),
        }
    }

    async fn pool(&self) -> Result<&SqlitePool> {
        self.initialized
            .get_or_try_init(|| async {
                let schema_exists = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_version'",
                )
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0)
                    > 0;

                if schema_exists {
                    self.run_migrations().await?;
                } else {
                    self.create_schema().await?;
                }

                Ok::<(), anyhow::Error>(())
            })
            .await?;
        Ok(&self.pool)
    }

    async fn create_schema(&self) -> Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        tx.execute(
            "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .await?;
        tx.execute(
            "CREATE TABLE thread_records (thread_id TEXT PRIMARY KEY, provider_lane TEXT NOT NULL, provider_id TEXT NOT NULL, forked_from_id TEXT, name TEXT, archived_at INTEGER, status TEXT NOT NULL DEFAULT 'active', config_snapshot_ref TEXT, session_state_ref TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, session_state_path TEXT, config_snapshot_json TEXT NOT NULL, cwd TEXT NOT NULL, ephemeral INTEGER NOT NULL DEFAULT 0, cli_version TEXT NOT NULL DEFAULT '', source_json TEXT NOT NULL DEFAULT 'null', agent_nickname TEXT, agent_role TEXT, git_branch TEXT, git_commit TEXT, git_origin_url TEXT, preview TEXT)",
        )
        .await?;
        tx.execute("CREATE INDEX idx_thread_records_updated ON thread_records(updated_at DESC)")
            .await?;
        tx.execute("CREATE INDEX idx_thread_records_archived ON thread_records(archived_at)")
            .await?;
        tx.execute("CREATE INDEX idx_thread_records_provider ON thread_records(provider_lane)")
            .await?;
        tx.execute(
            "CREATE TABLE config_snapshots (thread_id TEXT PRIMARY KEY NOT NULL, data TEXT NOT NULL, FOREIGN KEY (thread_id) REFERENCES thread_records(thread_id) ON DELETE CASCADE)",
        )
        .await?;
        sqlx::query(
            "INSERT INTO schema_version (version, applied_at) VALUES (?, strftime('%s','now'))",
        )
        .bind(CURRENT_SCHEMA_VERSION)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn run_migrations(&self) -> Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let current_version =
            sqlx::query_scalar::<_, i64>("SELECT COALESCE(MAX(version), 0) FROM schema_version")
                .fetch_one(&mut *tx)
                .await?;
        if current_version > CURRENT_SCHEMA_VERSION {
            anyhow::bail!(
                "thread registry schema version {current_version} is newer than supported {CURRENT_SCHEMA_VERSION}"
            );
        }
        for version in (current_version + 1)..=CURRENT_SCHEMA_VERSION {
            Self::apply_migration(&mut tx, version).await?;
            sqlx::query(
                "INSERT INTO schema_version (version, applied_at) VALUES (?, strftime('%s','now'))",
            )
            .bind(version)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn apply_migration(tx: &mut sqlx::Transaction<'_, Sqlite>, version: i64) -> Result<()> {
        match version {
            2 => {
                sqlx::query(
                    "ALTER TABLE thread_records ADD COLUMN status TEXT NOT NULL DEFAULT 'active'",
                )
                .execute(&mut **tx)
                .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN forked_from_id TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN config_snapshot_ref TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN session_state_ref TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN cwd TEXT NOT NULL DEFAULT ''")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query(
                    "ALTER TABLE thread_records ADD COLUMN ephemeral INTEGER NOT NULL DEFAULT 0",
                )
                .execute(&mut **tx)
                .await?;
                sqlx::query(
                    "ALTER TABLE thread_records ADD COLUMN cli_version TEXT NOT NULL DEFAULT ''",
                )
                .execute(&mut **tx)
                .await?;
                sqlx::query(
                    "ALTER TABLE thread_records ADD COLUMN source_json TEXT NOT NULL DEFAULT 'null'",
                )
                .execute(&mut **tx)
                .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN agent_nickname TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN agent_role TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN git_branch TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN git_commit TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN git_origin_url TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("ALTER TABLE thread_records ADD COLUMN preview TEXT")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query(
                    "CREATE TABLE config_snapshots (thread_id TEXT PRIMARY KEY NOT NULL, data TEXT NOT NULL, FOREIGN KEY (thread_id) REFERENCES thread_records(thread_id) ON DELETE CASCADE)",
                )
                .execute(&mut **tx)
                .await?;
            }
            _ => {
                anyhow::bail!("unknown thread registry migration version: {version}");
            }
        }
        Ok(())
    }

    pub async fn upsert_config_snapshot(&self, thread_id: ThreadId, data: String) -> Result<()> {
        let pool = self.pool().await?;
        sqlx::query(
            "INSERT INTO config_snapshots (thread_id, data) VALUES (?, ?) ON CONFLICT(thread_id) DO UPDATE SET data = excluded.data",
        )
        .bind(thread_id.to_string())
        .bind(data)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_config_snapshot(&self, thread_id: ThreadId) -> Result<Option<String>> {
        let pool = self.pool().await?;
        let row = sqlx::query_scalar::<_, String>(
            "SELECT data FROM config_snapshots WHERE thread_id = ?",
        )
        .bind(thread_id.to_string())
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_config_snapshot(&self, thread_id: ThreadId) -> Result<()> {
        let pool = self.pool().await?;
        sqlx::query("DELETE FROM config_snapshots WHERE thread_id = ?")
            .bind(thread_id.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Mark a thread as archived in the shared non-canonical registry.
    pub async fn archive_thread(&self, thread_id: ThreadId, archived_at: i64) -> Result<()> {
        self.update(thread_id)
            .archived_at(Some(archived_at))
            .updated_at(archived_at)
            .apply()
            .await
    }

    /// Clear the archived marker for a thread in the shared non-canonical registry.
    pub async fn unarchive_thread(&self, thread_id: ThreadId, updated_at: i64) -> Result<()> {
        self.update(thread_id)
            .archived_at(None)
            .updated_at(updated_at)
            .apply()
            .await
    }

    /// Update the registry-backed thread name and freshness timestamp.
    pub async fn set_thread_name(
        &self,
        thread_id: ThreadId,
        name: String,
        updated_at: i64,
    ) -> Result<()> {
        self.update(thread_id)
            .name(Some(name))
            .updated_at(updated_at)
            .apply()
            .await
    }

    /// Update the registry-backed Git metadata and freshness timestamp.
    pub async fn update_thread_git_info(
        &self,
        thread_id: ThreadId,
        patch: ThreadGitInfoPatch,
        updated_at: i64,
    ) -> Result<()> {
        let mut update = self.update(thread_id);
        let trim = |value: Option<Option<String>>| {
            value.map(|value| value.map(|value| value.trim().to_string()))
        };

        if let Some(sha) = trim(patch.sha) {
            update = update.git_commit(sha);
        }
        if let Some(branch) = trim(patch.branch) {
            update = update.git_branch(branch);
        }
        if let Some(origin_url) = trim(patch.origin_url) {
            update = update.git_origin_url(origin_url);
        }

        update.updated_at(updated_at).apply().await
    }

    /// Refresh only the `updated_at` timestamp for a thread.
    pub async fn touch_thread_updated_at(
        &self,
        thread_id: ThreadId,
        updated_at: i64,
    ) -> Result<()> {
        self.update(thread_id).updated_at(updated_at).apply().await
    }
}

#[async_trait]
impl ThreadRegistry for SqliteThreadRegistry {
    async fn upsert_thread(&self, input: ThreadRecordInput) -> Result<()> {
        let pool = self.pool().await?;
        sqlx::query(
            "INSERT INTO thread_records (thread_id, provider_lane, provider_id, forked_from_id, name, archived_at, status, config_snapshot_ref, session_state_ref, created_at, updated_at, session_state_path, config_snapshot_json, cwd, ephemeral, cli_version, source_json, agent_nickname, agent_role, git_branch, git_commit, git_origin_url, preview) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(thread_id) DO UPDATE SET provider_lane = excluded.provider_lane, provider_id = excluded.provider_id, forked_from_id = excluded.forked_from_id, name = excluded.name, archived_at = excluded.archived_at, status = excluded.status, config_snapshot_ref = excluded.config_snapshot_ref, session_state_ref = excluded.session_state_ref, created_at = excluded.created_at, updated_at = excluded.updated_at, session_state_path = excluded.session_state_path, config_snapshot_json = excluded.config_snapshot_json, cwd = excluded.cwd, ephemeral = excluded.ephemeral, cli_version = excluded.cli_version, source_json = excluded.source_json, agent_nickname = excluded.agent_nickname, agent_role = excluded.agent_role, git_branch = excluded.git_branch, git_commit = excluded.git_commit, git_origin_url = excluded.git_origin_url, preview = excluded.preview",
        )
        .bind(input.thread_id.to_string())
        .bind(input.provider_lane.as_str())
        .bind(input.provider_id)
        .bind(input.forked_from_id)
        .bind(input.name)
        .bind(input.archived_at)
        .bind(input.status)
        .bind(input.config_snapshot_ref)
        .bind(input.session_state_ref)
        .bind(input.created_at)
        .bind(input.updated_at)
        .bind(path_to_string(input.session_state_path))
        .bind(input.config_snapshot_json)
        .bind(input.cwd.to_string_lossy().into_owned())
        .bind(input.ephemeral)
        .bind(input.cli_version)
        .bind(input.source_json)
        .bind(input.agent_nickname)
        .bind(input.agent_role)
        .bind(input.git_branch)
        .bind(input.git_commit)
        .bind(input.git_origin_url)
        .bind(input.preview)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_thread(&self, thread_id: ThreadId) -> Result<Option<ThreadRecord>> {
        let pool = self.pool().await?;
        let row = sqlx::query(&format!(
            "SELECT {THREAD_RECORD_COLUMNS} FROM thread_records WHERE thread_id = ?"
        ))
        .bind(thread_id.to_string())
        .fetch_optional(pool)
        .await?;
        row.map(thread_record_from_row).transpose()
    }

    async fn get_provider_lane(&self, thread_id: ThreadId) -> Result<Option<ProviderLane>> {
        let pool = self.pool().await?;
        let lane = sqlx::query_scalar::<_, String>(
            "SELECT provider_lane FROM thread_records WHERE thread_id = ?",
        )
        .bind(thread_id.to_string())
        .fetch_optional(pool)
        .await?;
        lane.map(|value| ProviderLane::from_str(&value)).transpose()
    }

    async fn list_threads(&self, filter: ThreadFilter) -> Result<Vec<ThreadRecord>> {
        let pool = self.pool().await?;
        let mut query = QueryBuilder::<Sqlite>::new(format!(
            "SELECT {THREAD_RECORD_COLUMNS} FROM thread_records WHERE 1 = 1"
        ));
        if !filter.include_archived {
            query.push(" AND archived_at IS NULL");
        }
        if let Some(provider_lane) = filter.provider_lane {
            query
                .push(" AND provider_lane = ")
                .push_bind(provider_lane.as_str());
        }
        query.push(" ORDER BY updated_at DESC");
        let rows = query.build().fetch_all(pool).await?;
        rows.into_iter().map(thread_record_from_row).collect()
    }

    async fn has_any_records(&self) -> Result<bool> {
        let pool = self.pool().await?;
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM thread_records")
            .fetch_one(pool)
            .await?;
        Ok(count > 0)
    }

    fn update(&self, thread_id: ThreadId) -> ThreadUpdateBuilder<'_> {
        ThreadUpdateBuilder {
            registry: self,
            thread_id,
            name: None,
            archived_at: None,
            status: None,
            config_snapshot_ref: None,
            session_state_ref: None,
            session_state_path: None,
            config_snapshot_json: None,
            forked_from_id: None,
            cwd: None,
            ephemeral: None,
            cli_version: None,
            source_json: None,
            agent_nickname: None,
            agent_role: None,
            git_branch: None,
            git_commit: None,
            git_origin_url: None,
            preview: None,
            updated_at: None,
        }
    }
}

pub struct ThreadUpdateBuilder<'a> {
    registry: &'a SqliteThreadRegistry,
    thread_id: ThreadId,
    name: Option<Option<String>>,
    archived_at: Option<Option<i64>>,
    status: Option<String>,
    config_snapshot_ref: Option<Option<String>>,
    session_state_ref: Option<Option<String>>,
    session_state_path: Option<Option<PathBuf>>,
    config_snapshot_json: Option<String>,
    forked_from_id: Option<Option<String>>,
    cwd: Option<PathBuf>,
    ephemeral: Option<bool>,
    cli_version: Option<String>,
    source_json: Option<String>,
    agent_nickname: Option<Option<String>>,
    agent_role: Option<Option<String>>,
    git_branch: Option<Option<String>>,
    git_commit: Option<Option<String>>,
    git_origin_url: Option<Option<String>>,
    preview: Option<Option<String>>,
    updated_at: Option<i64>,
}

impl<'a> ThreadUpdateBuilder<'a> {
    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = Some(name);
        self
    }

    pub fn archived_at(mut self, archived_at: Option<i64>) -> Self {
        self.archived_at = Some(archived_at);
        self
    }

    pub fn status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn config_snapshot_ref(mut self, config_snapshot_ref: Option<String>) -> Self {
        self.config_snapshot_ref = Some(config_snapshot_ref);
        self
    }

    pub fn session_state_ref(mut self, session_state_ref: Option<String>) -> Self {
        self.session_state_ref = Some(session_state_ref);
        self
    }

    pub fn session_state_path(mut self, session_state_path: Option<PathBuf>) -> Self {
        self.session_state_path = Some(session_state_path);
        self
    }

    pub fn config_snapshot_json(mut self, config_snapshot_json: String) -> Self {
        self.config_snapshot_json = Some(config_snapshot_json);
        self
    }

    pub fn forked_from_id(mut self, forked_from_id: Option<String>) -> Self {
        self.forked_from_id = Some(forked_from_id);
        self
    }

    pub fn cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = Some(ephemeral);
        self
    }

    pub fn cli_version(mut self, cli_version: String) -> Self {
        self.cli_version = Some(cli_version);
        self
    }

    pub fn source_json(mut self, source_json: String) -> Self {
        self.source_json = Some(source_json);
        self
    }

    pub fn agent_nickname(mut self, agent_nickname: Option<String>) -> Self {
        self.agent_nickname = Some(agent_nickname);
        self
    }

    pub fn agent_role(mut self, agent_role: Option<String>) -> Self {
        self.agent_role = Some(agent_role);
        self
    }

    pub fn git_branch(mut self, git_branch: Option<String>) -> Self {
        self.git_branch = Some(git_branch);
        self
    }

    pub fn git_commit(mut self, git_commit: Option<String>) -> Self {
        self.git_commit = Some(git_commit);
        self
    }

    pub fn git_origin_url(mut self, git_origin_url: Option<String>) -> Self {
        self.git_origin_url = Some(git_origin_url);
        self
    }

    pub fn preview(mut self, preview: Option<String>) -> Self {
        self.preview = Some(preview);
        self
    }

    pub fn updated_at(mut self, updated_at: i64) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    pub async fn apply(self) -> Result<()> {
        let pool = self.registry.pool().await?;
        let mut query = QueryBuilder::<Sqlite>::new("UPDATE thread_records SET ");
        let mut wrote_any = false;

        macro_rules! push_opt {
            ($field:expr, $col:expr) => {
                if let Some(value) = $field {
                    if wrote_any {
                        query.push(", ");
                    }
                    query.push($col).push(" = ").push_bind(value);
                    wrote_any = true;
                }
            };
        }

        push_opt!(self.name, "name");
        push_opt!(self.archived_at, "archived_at");
        push_opt!(self.status, "status");
        push_opt!(self.config_snapshot_ref, "config_snapshot_ref");
        push_opt!(self.session_state_ref, "session_state_ref");
        push_opt!(self.forked_from_id, "forked_from_id");
        push_opt!(self.agent_nickname, "agent_nickname");
        push_opt!(self.agent_role, "agent_role");
        push_opt!(self.git_branch, "git_branch");
        push_opt!(self.git_commit, "git_commit");
        push_opt!(self.git_origin_url, "git_origin_url");
        push_opt!(self.preview, "preview");

        if let Some(cwd) = self.cwd {
            if wrote_any {
                query.push(", ");
            }
            query
                .push("cwd = ")
                .push_bind(cwd.to_string_lossy().into_owned());
            wrote_any = true;
        }
        if let Some(ephemeral) = self.ephemeral {
            if wrote_any {
                query.push(", ");
            }
            query.push("ephemeral = ").push_bind(ephemeral);
            wrote_any = true;
        }
        if let Some(cli_version) = self.cli_version {
            if wrote_any {
                query.push(", ");
            }
            query.push("cli_version = ").push_bind(cli_version);
            wrote_any = true;
        }
        if let Some(source_json) = self.source_json {
            if wrote_any {
                query.push(", ");
            }
            query.push("source_json = ").push_bind(source_json);
            wrote_any = true;
        }

        if let Some(session_state_path) = self.session_state_path {
            if wrote_any {
                query.push(", ");
            }
            query
                .push("session_state_path = ")
                .push_bind(path_to_string(session_state_path));
            wrote_any = true;
        }
        if let Some(config_snapshot_json) = self.config_snapshot_json {
            if wrote_any {
                query.push(", ");
            }
            query
                .push("config_snapshot_json = ")
                .push_bind(config_snapshot_json);
            wrote_any = true;
        }

        if wrote_any {
            query.push(", ");
        }
        query
            .push("updated_at = ")
            .push_bind(self.updated_at.unwrap_or_else(now_ts));
        query
            .push(" WHERE thread_id = ")
            .push_bind(self.thread_id.to_string());

        let result = query.build().execute(pool).await?;
        if result.rows_affected() == 0 {
            anyhow::bail!("thread record not found: {}", self.thread_id);
        }
        Ok(())
    }
}

pub fn thread_registry_db_path(codex_home: &Path) -> PathBuf {
    codex_home
        .join(THREAD_REGISTRY_DIR)
        .join(THREAD_REGISTRY_DB_FILENAME)
}

fn create_pool(path: &Path) -> SqlitePool {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create thread registry directory");
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .log_statements(log::LevelFilter::Off);

    SqlitePoolOptions::new().connect_lazy_with(options)
}

fn thread_record_from_row(row: sqlx::sqlite::SqliteRow) -> Result<ThreadRecord> {
    let thread_id = ThreadId::from_string(&row.try_get::<String, _>("thread_id")?)?;
    let provider_lane = ProviderLane::from_str(&row.try_get::<String, _>("provider_lane")?)?;
    let session_state_path = row
        .try_get::<Option<String>, _>("session_state_path")?
        .map(PathBuf::from);
    Ok(ThreadRecord {
        thread_id,
        provider_lane,
        provider_id: row.try_get("provider_id")?,
        forked_from_id: row.try_get("forked_from_id")?,
        name: row.try_get("name")?,
        archived_at: row.try_get("archived_at")?,
        status: row.try_get("status")?,
        config_snapshot_ref: row.try_get("config_snapshot_ref")?,
        session_state_ref: row.try_get("session_state_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        session_state_path,
        config_snapshot_json: row.try_get("config_snapshot_json")?,
        cwd: PathBuf::from(row.try_get::<String, _>("cwd")?),
        ephemeral: row.try_get("ephemeral")?,
        cli_version: row.try_get("cli_version")?,
        source_json: row.try_get("source_json")?,
        agent_nickname: row.try_get("agent_nickname")?,
        agent_role: row.try_get("agent_role")?,
        git_branch: row.try_get("git_branch")?,
        git_commit: row.try_get("git_commit")?,
        git_origin_url: row.try_get("git_origin_url")?,
        preview: row.try_get("preview")?,
    })
}

fn path_to_string(path: Option<PathBuf>) -> Option<String> {
    path.map(|value| value.to_string_lossy().into_owned())
}

fn now_ts() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

#[cfg(test)]
mod tests {
    use super::ProviderLane;
    use super::SqliteThreadRegistry;
    use super::ThreadFilter;
    use super::ThreadGitInfoPatch;
    use super::ThreadRecordInput;
    use super::ThreadRegistry;
    use anyhow::Result;
    use codex_protocol::ThreadId;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[tokio::test]
    async fn sqlite_registry_round_trips_and_filters_archive_state() -> Result<()> {
        let home = TempDir::new()?;
        let registry = SqliteThreadRegistry::new(home.path().to_path_buf());
        let thread_id = ThreadId::new();

        registry
            .upsert_thread(ThreadRecordInput {
                thread_id,
                provider_lane: ProviderLane::OpenRouterBeta,
                provider_id: "openrouter-test".to_string(),
                forked_from_id: Some("parent-thread".to_string()),
                name: Some("Thread A".to_string()),
                archived_at: None,
                status: "active".to_string(),
                config_snapshot_ref: None,
                session_state_ref: None,
                created_at: 10,
                updated_at: 10,
                session_state_path: Some(home.path().join("thread-a.json")),
                config_snapshot_json: "{\"model\":\"openrouter-test\"}".to_string(),
                cwd: home.path().to_path_buf(),
                ephemeral: false,
                cli_version: "1.2.3".to_string(),
                source_json: "\"cli\"".to_string(),
                agent_nickname: Some("agent-a".to_string()),
                agent_role: Some("reviewer".to_string()),
                git_branch: Some("main".to_string()),
                git_commit: Some("abc123".to_string()),
                git_origin_url: Some("https://example.com/repo.git".to_string()),
                preview: None,
            })
            .await?;

        let record = registry
            .get_thread(thread_id)
            .await?
            .expect("thread record");
        assert_eq!(record.provider_lane, ProviderLane::OpenRouterBeta);
        assert_eq!(record.name.as_deref(), Some("Thread A"));
        assert_eq!(record.forked_from_id.as_deref(), Some("parent-thread"));
        assert_eq!(record.git_branch.as_deref(), Some("main"));
        assert_eq!(record.git_commit.as_deref(), Some("abc123"));
        assert_eq!(
            record.git_origin_url.as_deref(),
            Some("https://example.com/repo.git")
        );

        let active = registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta))
            .await?;
        assert_eq!(active.len(), 1);

        registry
            .set_thread_name(thread_id, "Thread B".to_string(), 15)
            .await?;
        registry
            .update_thread_git_info(
                thread_id,
                ThreadGitInfoPatch {
                    sha: Some(Some("def456".to_string())),
                    branch: Some(Some("feature/openrouter".to_string())),
                    origin_url: Some(Some("https://example.com/updated.git".to_string())),
                },
                18,
            )
            .await?;
        registry.archive_thread(thread_id, 20).await?;

        let archived_record = registry
            .get_thread(thread_id)
            .await?
            .expect("archived thread record");
        assert_eq!(archived_record.name.as_deref(), Some("Thread B"));
        assert_eq!(archived_record.git_commit.as_deref(), Some("def456"));
        assert_eq!(
            archived_record.git_origin_url.as_deref(),
            Some("https://example.com/updated.git")
        );
        assert_eq!(archived_record.archived_at, Some(20));

        let active = registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta))
            .await?;
        assert!(active.is_empty());

        let archived = registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta).including_archived())
            .await?;
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].name.as_deref(), Some("Thread B"));

        registry.unarchive_thread(thread_id, 25).await?;
        registry.touch_thread_updated_at(thread_id, 27).await?;

        let active = registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta))
            .await?;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name.as_deref(), Some("Thread B"));
        assert_eq!(active[0].updated_at, 27);

        let archived = registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta).including_archived())
            .await?;
        assert_eq!(archived.len(), 1);
        assert!(archived[0].archived_at.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn sqlite_registry_v1_migrates_to_v2_with_new_columns() -> Result<()> {
        use sqlx::Executor as _;

        let home = TempDir::new()?;
        let db_path = home
            .path()
            .join("logos-session-state")
            .join("thread-registry.sqlite");
        std::fs::create_dir_all(db_path.parent().unwrap())?;

        let options = super::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(super::SqliteJournalMode::Wal);
        let pool = super::SqlitePoolOptions::new().connect_lazy_with(options);

        pool.execute(
            "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .await?;
        pool.execute(
            "CREATE TABLE thread_records (thread_id TEXT PRIMARY KEY, provider_lane TEXT NOT NULL, provider_id TEXT NOT NULL, name TEXT, archived_at INTEGER, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, session_state_path TEXT, config_snapshot_json TEXT NOT NULL)",
        )
        .await?;
        pool.execute("CREATE INDEX idx_thread_records_updated ON thread_records(updated_at DESC)")
            .await?;
        pool.execute("CREATE INDEX idx_thread_records_archived ON thread_records(archived_at)")
            .await?;
        pool.execute("CREATE INDEX idx_thread_records_provider ON thread_records(provider_lane)")
            .await?;
        pool.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (1, strftime('%s','now'))",
        )
        .await?;
        let thread_id = ThreadId::new().to_string();
        pool.execute(
            &*format!("INSERT INTO thread_records (thread_id, provider_lane, provider_id, name, archived_at, created_at, updated_at, config_snapshot_json) VALUES ('{thread_id}', 'openrouter_beta', 'test-provider', 'V1 Thread', NULL, 100, 100, '{{}}')"),
        )
        .await?;
        pool.close().await;

        let registry = SqliteThreadRegistry::new(home.path().to_path_buf());
        let tid = ThreadId::from_string(&thread_id)?;
        let record = registry.get_thread(tid).await?.expect("migrated record");
        assert_eq!(record.name.as_deref(), Some("V1 Thread"));
        assert_eq!(record.status, "active");
        assert!(record.git_branch.is_none());
        assert!(record.config_snapshot_ref.is_none());

        let config_data = r#"{"model":"test-model"}"#.to_string();
        registry
            .upsert_config_snapshot(tid, config_data.clone())
            .await?;
        let loaded = registry.get_config_snapshot(tid).await?;
        assert_eq!(loaded, Some(config_data));

        Ok(())
    }
}
