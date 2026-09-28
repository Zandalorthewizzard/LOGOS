use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use codex_app_server_protocol::Thread;
use codex_core::ThreadConfigSnapshot;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use serde::Deserialize;
use serde::Serialize;

/// Legacy OpenRouter snapshot root used only for compatibility payload recovery.
const OPENROUTER_BETA_THREADS_DIR: &str = "logos-openrouter-beta";
/// Legacy active-snapshot residue used only for compatibility payload recovery.
const ACTIVE_THREADS_DIR: &str = "active-threads";
/// Legacy archived-snapshot residue used only for compatibility payload recovery.
const ARCHIVED_THREADS_DIR: &str = "archived-threads";
const SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedOpenRouterThread {
    pub(crate) version: u32,
    pub(crate) config_snapshot: PersistedThreadConfigSnapshot,
    pub(crate) thread: Thread,
    pub(crate) history: Vec<InternalMessage>,
    pub(crate) continuation: Option<ContinuationHandle>,
    pub(crate) elicitation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedThreadConfigSnapshot {
    pub(crate) model: String,
    pub(crate) model_provider_id: String,
    pub(crate) service_tier: Option<ServiceTier>,
    pub(crate) approval_policy: AskForApproval,
    pub(crate) approvals_reviewer: ApprovalsReviewer,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) cwd: PathBuf,
    pub(crate) ephemeral: bool,
    pub(crate) reasoning_effort: Option<codex_protocol::openai_models::ReasoningEffort>,
    pub(crate) personality: Option<Personality>,
    pub(crate) session_source: SessionSource,
}

impl PersistedThreadConfigSnapshot {
    pub(crate) fn to_core(&self) -> ThreadConfigSnapshot {
        ThreadConfigSnapshot {
            model: self.model.clone(),
            model_provider_id: self.model_provider_id.clone(),
            service_tier: self.service_tier,
            approval_policy: self.approval_policy,
            approvals_reviewer: self.approvals_reviewer,
            sandbox_policy: self.sandbox_policy.clone(),
            cwd: self.cwd.clone(),
            ephemeral: self.ephemeral,
            reasoning_effort: self.reasoning_effort,
            personality: self.personality,
            session_source: self.session_source.clone(),
        }
    }
}

impl From<&ThreadConfigSnapshot> for PersistedThreadConfigSnapshot {
    fn from(value: &ThreadConfigSnapshot) -> Self {
        Self {
            model: value.model.clone(),
            model_provider_id: value.model_provider_id.clone(),
            service_tier: value.service_tier,
            approval_policy: value.approval_policy,
            approvals_reviewer: value.approvals_reviewer,
            sandbox_policy: value.sandbox_policy.clone(),
            cwd: value.cwd.clone(),
            ephemeral: value.ephemeral,
            reasoning_effort: value.reasoning_effort,
            personality: value.personality,
            session_source: value.session_source.clone(),
        }
    }
}

pub(crate) fn active_thread_path(codex_home: &Path, thread_id: &str) -> PathBuf {
    codex_home
        .join(OPENROUTER_BETA_THREADS_DIR)
        .join(ACTIVE_THREADS_DIR)
        .join(snapshot_file_name(thread_id))
}

#[allow(dead_code)]
pub(crate) fn archived_thread_path(codex_home: &Path, thread_id: &str) -> PathBuf {
    codex_home
        .join(OPENROUTER_BETA_THREADS_DIR)
        .join(ARCHIVED_THREADS_DIR)
        .join(snapshot_file_name(thread_id))
}

pub(crate) async fn load_thread_snapshot(
    path: PathBuf,
) -> anyhow::Result<Option<PersistedOpenRouterThread>> {
    load_snapshot(path).await
}

#[allow(dead_code)]
pub(crate) async fn load_active_thread(
    codex_home: &Path,
    thread_id: &str,
) -> anyhow::Result<Option<PersistedOpenRouterThread>> {
    load_thread_snapshot(active_thread_path(codex_home, thread_id)).await
}

#[allow(dead_code)]
pub(crate) async fn load_archived_thread(
    codex_home: &Path,
    thread_id: &str,
) -> anyhow::Result<Option<PersistedOpenRouterThread>> {
    load_thread_snapshot(archived_thread_path(codex_home, thread_id)).await
}

pub(crate) async fn list_active_threads(
    codex_home: &Path,
) -> anyhow::Result<Vec<PersistedOpenRouterThread>> {
    // Legacy import helper: this does not define current archive truth.
    list_snapshots(
        codex_home
            .join(OPENROUTER_BETA_THREADS_DIR)
            .join(ACTIVE_THREADS_DIR),
    )
    .await
}

pub(crate) async fn list_archived_threads(
    codex_home: &Path,
) -> anyhow::Result<Vec<PersistedOpenRouterThread>> {
    // Legacy import helper: this does not define current archive truth.
    list_snapshots(
        codex_home
            .join(OPENROUTER_BETA_THREADS_DIR)
            .join(ARCHIVED_THREADS_DIR),
    )
    .await
}

pub(crate) async fn save_active_thread(
    codex_home: &Path,
    persisted: &PersistedOpenRouterThread,
) -> anyhow::Result<PathBuf> {
    // Compatibility snapshot residue only; current archive truth lives in the registry.
    write_snapshot(
        active_thread_path(codex_home, &persisted.thread.id),
        persisted,
    )
    .await
}

#[allow(dead_code)]
pub(crate) async fn save_thread_snapshot(
    path: PathBuf,
    persisted: &PersistedOpenRouterThread,
) -> anyhow::Result<PathBuf> {
    write_snapshot(path, persisted).await
}

#[allow(dead_code)]
pub(crate) async fn save_archived_thread(
    codex_home: &Path,
    persisted: &PersistedOpenRouterThread,
) -> anyhow::Result<PathBuf> {
    // Compatibility snapshot residue only; current archive truth lives in the registry.
    write_snapshot(
        archived_thread_path(codex_home, &persisted.thread.id),
        persisted,
    )
    .await
}

#[allow(dead_code)]
pub(crate) async fn remove_active_thread(codex_home: &Path, thread_id: &str) -> anyhow::Result<()> {
    // Compatibility cleanup only.
    remove_if_exists(active_thread_path(codex_home, thread_id)).await
}

#[allow(dead_code)]
pub(crate) async fn remove_archived_thread(
    codex_home: &Path,
    thread_id: &str,
) -> anyhow::Result<()> {
    // Compatibility cleanup only.
    remove_if_exists(archived_thread_path(codex_home, thread_id)).await
}

#[allow(dead_code)]
pub(crate) async fn remove_thread_snapshot(path: PathBuf) -> anyhow::Result<()> {
    remove_if_exists(path).await
}

fn snapshot_file_name(thread_id: &str) -> String {
    format!("{thread_id}.json")
}

async fn load_snapshot(path: PathBuf) -> anyhow::Result<Option<PersistedOpenRouterThread>> {
    let data = match tokio::fs::read(&path).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    let persisted = serde_json::from_slice::<PersistedOpenRouterThread>(&data)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(persisted))
}

async fn list_snapshots(dir: PathBuf) -> anyhow::Result<Vec<PersistedOpenRouterThread>> {
    let mut entries = match tokio::fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", dir.display()));
        }
    };
    let mut snapshots = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .with_context(|| format!("failed to enumerate {}", dir.display()))?
    {
        let path = entry.path();
        if path.extension() != Some(OsStr::new("json")) {
            continue;
        }
        if let Some(snapshot) = load_snapshot(path).await? {
            snapshots.push(snapshot);
        }
    }
    Ok(snapshots)
}

async fn write_snapshot(
    path: PathBuf,
    persisted: &PersistedOpenRouterThread,
) -> anyhow::Result<PathBuf> {
    let parent = path
        .parent()
        .context("openrouter beta snapshot path missing parent")?;
    tokio::fs::create_dir_all(parent)
        .await
        .with_context(|| format!("failed to create {}", parent.display()))?;
    let json = serde_json::to_vec_pretty(persisted)
        .with_context(|| format!("failed to serialize {}", persisted.thread.id))?;
    let tmp_path = path.with_extension("json.tmp");
    tokio::fs::write(&tmp_path, json)
        .await
        .with_context(|| format!("failed to write {}", tmp_path.display()))?;
    tokio::fs::rename(&tmp_path, &path).await.with_context(|| {
        format!(
            "failed to move {} to {}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(path)
}

#[allow(dead_code)]
async fn remove_if_exists(path: PathBuf) -> anyhow::Result<()> {
    match tokio::fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to remove {}", path.display())),
    }
}

pub(crate) fn persisted_with_path(
    mut persisted: PersistedOpenRouterThread,
    path: PathBuf,
) -> PersistedOpenRouterThread {
    persisted.thread.path = Some(path);
    persisted
}

pub(crate) fn build_persisted_thread(
    thread: Thread,
    config_snapshot: &ThreadConfigSnapshot,
    history: Vec<InternalMessage>,
    continuation: Option<ContinuationHandle>,
    elicitation_count: u64,
) -> PersistedOpenRouterThread {
    PersistedOpenRouterThread {
        version: SNAPSHOT_VERSION,
        config_snapshot: PersistedThreadConfigSnapshot::from(config_snapshot),
        thread,
        history,
        continuation,
        elicitation_count,
    }
}
