use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use codex_app_server_protocol::CommandExecutionOutputDeltaNotification;
use codex_app_server_protocol::CommandExecutionSource;
use codex_app_server_protocol::CommandExecutionStatus;
use codex_app_server_protocol::DynamicToolCallStatus;
use codex_app_server_protocol::ErrorNotification;
use codex_app_server_protocol::GitInfo;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ReviewDelivery as ApiReviewDelivery;
use codex_app_server_protocol::ReviewStartResponse;
use codex_app_server_protocol::ReviewTarget as ApiReviewTarget;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::SessionSource as ApiSessionSource;
use codex_app_server_protocol::TerminalInteractionNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadBackgroundTerminalStartResponse;
use codex_app_server_protocol::ThreadBackgroundTerminalsCleanResponse;
use codex_app_server_protocol::ThreadForkParams;
use codex_app_server_protocol::ThreadForkResponse;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadShellCommandResponse;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadStartedNotification;
use codex_app_server_protocol::ThreadStatus;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnError;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStartedNotification;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_config::types::ShellEnvironmentPolicy;
use codex_core::ThreadConfigSnapshot;
use codex_core::config::Config;
use codex_core::exec_env::create_env;
use codex_core::host_exec::HostExecApprovalPolicy;
use codex_core::host_exec::HostExecBegin;
use codex_core::host_exec::HostExecContext;
use codex_core::host_exec::HostExecEnd;
use codex_core::host_exec::HostExecNotificationSink;
use codex_core::host_exec::HostExecOutcome;
use codex_core::host_exec::HostExecRegistration;
use codex_core::host_exec::HostExecRegistry;
use codex_core::host_exec::HostExecSandboxPolicy;
use codex_core::host_exec::run_one_shot_shell_command;
use codex_core::shell::default_user_shell;
use codex_protocol::ThreadId;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecCommandSource as CoreExecCommandSource;
use codex_protocol::protocol::ExecCommandStatus as CoreExecCommandStatus;
use codex_protocol::protocol::ReviewDelivery as CoreReviewDelivery;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ReviewTarget as CoreReviewTarget;
use codex_protocol::protocol::SessionSource;
use codex_shell_command::parse_command::shlex_join;
use logos_openrouter_beta::OpenRouterBetaSession;
use logos_openrouter_beta::OpenRouterBetaSettings;
use logos_openrouter_beta::unsupported_operation_message;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::ToolCallResultDisposition;
use logos_session_state::HistoryEntry;
use logos_session_state::HistoryFile;
use logos_session_state::ProviderLane;
use logos_session_state::SqliteThreadRegistry;
use logos_session_state::ThreadFilter;
use logos_session_state::ThreadGitInfoPatch;
use logos_session_state::ThreadRecord;
use logos_session_state::ThreadRecordInput;
use logos_session_state::ThreadRegistry;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::error_code::INTERNAL_ERROR_CODE;
use crate::error_code::INVALID_REQUEST_ERROR_CODE;
use crate::openrouter_beta_persistence;
use crate::openrouter_beta_persistence::PersistedOpenRouterThread;
use crate::openrouter_beta_persistence::PersistedThreadConfigSnapshot;
use crate::outgoing_message::ConnectionId;
use crate::outgoing_message::OutgoingMessageSender;
use crate::thread_state::ThreadStateManager;

#[derive(Clone)]
pub(crate) struct OpenRouterBetaThreadManager {
    codex_home: Arc<PathBuf>,
    threads: Arc<Mutex<HashMap<ThreadId, Arc<OpenRouterBetaThread>>>>,
    registry: Arc<SqliteThreadRegistry>,
    registry_ready: Arc<OnceCell<()>>,
    host_exec_registry: Arc<HostExecRegistry>,
}
struct OpenRouterBetaThread {
    runtime: Mutex<OpenRouterBetaSession>,
    state: Mutex<OpenRouterThreadState>,
}

struct OpenRouterThreadState {
    config_snapshot: ThreadConfigSnapshot,
    shell_environment_policy: ShellEnvironmentPolicy,
    thread: Thread,
    lifecycle: TurnLifecycleState,
    elicitation_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenRouterResumeBackend {
    ClientManagedReplay,
}

#[derive(Clone, Debug, Default)]
enum TurnLifecycleState {
    #[default]
    Idle,
    Running {
        turn_id: String,
    },
}

impl TurnLifecycleState {
    fn active_turn_id(&self) -> Option<&str> {
        match self {
            Self::Idle => None,
            Self::Running { turn_id } => Some(turn_id.as_str()),
        }
    }

    fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }
}

enum TurnEvent {
    TurnStarted {
        thread_id: ThreadId,
        turn: Turn,
    },
    ItemStarted {
        thread_id: ThreadId,
        turn_id: String,
        item: ThreadItem,
    },
    ItemCompleted {
        thread_id: ThreadId,
        turn_id: String,
        item: ThreadItem,
    },
    Error {
        thread_id: ThreadId,
        turn_id: String,
        error: TurnError,
    },
    TurnCompleted {
        thread_id: ThreadId,
        turn: Turn,
    },
}

pub(crate) struct StartedOpenRouterThread {
    pub response: ThreadStartResponse,
    pub notification: ThreadStartedNotification,
}

impl Default for OpenRouterBetaThreadManager {
    fn default() -> Self {
        Self::new(PathBuf::new(), Arc::new(HostExecRegistry::default()))
    }
}

impl OpenRouterBetaThreadManager {
    pub(crate) fn new(codex_home: PathBuf, host_exec_registry: Arc<HostExecRegistry>) -> Self {
        Self {
            registry: Arc::new(SqliteThreadRegistry::new(codex_home.clone())),
            registry_ready: Arc::new(OnceCell::new()),
            codex_home: Arc::new(codex_home),
            host_exec_registry,
            threads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn ensure_registry_ready(&self) -> anyhow::Result<()> {
        self.registry_ready
            .get_or_try_init(|| async {
                if self.registry.has_any_records().await? {
                    return Ok::<(), anyhow::Error>(());
                }

                for persisted in
                    openrouter_beta_persistence::list_active_threads(self.codex_home.as_path())
                        .await?
                {
                    self.import_legacy_thread(persisted, /*archived*/ false)
                        .await?;
                }
                for persisted in
                    openrouter_beta_persistence::list_archived_threads(self.codex_home.as_path())
                        .await?
                {
                    self.import_legacy_thread(persisted, /*archived*/ true)
                        .await?;
                }

                Ok::<(), anyhow::Error>(())
            })
            .await?;
        Ok(())
    }

    async fn import_legacy_thread(
        &self,
        persisted: PersistedOpenRouterThread,
        archived: bool,
    ) -> anyhow::Result<()> {
        let thread_id = ThreadId::from_string(&persisted.thread.id)?;
        let history_file = HistoryFile::new(self.codex_home.as_path(), &thread_id.to_string());
        let entries = build_history_entries(
            &persisted.history,
            &persisted.continuation,
            persisted.elicitation_count,
        );
        history_file.write_all(&entries).await?;
        let mut persisted = persisted;
        persisted.thread.path = Some(history_file.path().to_path_buf());
        self.upsert_registry_thread(persisted, archived).await
    }

    async fn thread_record(
        &self,
        thread_id: ThreadId,
    ) -> Result<Option<ThreadRecord>, JSONRPCErrorError> {
        self.ensure_registry_ready().await.map_err(|error| {
            internal_error(format!(
                "failed to initialize OpenRouter beta thread registry: {error}"
            ))
        })?;
        self.registry.get_thread(thread_id).await.map_err(|error| {
            internal_error(format!(
                "failed to read OpenRouter beta thread registry: {error}"
            ))
        })
    }

    async fn config_snapshot_from_record(
        &self,
        record: &ThreadRecord,
    ) -> Result<PersistedThreadConfigSnapshot, JSONRPCErrorError> {
        let config_snapshot_id = record
            .config_snapshot_ref
            .as_deref()
            .map(ThreadId::from_string)
            .transpose()
            .map_err(|error| {
                internal_error(format!(
                    "failed to resolve OpenRouter beta config snapshot reference {}: {error}",
                    record.thread_id
                ))
            })?
            .unwrap_or(record.thread_id);
        let config_json = if record.config_snapshot_ref.is_some() {
            self.registry
                .get_config_snapshot(config_snapshot_id)
                .await
                .map_err(|error| {
                    internal_error(format!(
                        "failed to load OpenRouter beta config snapshot {} from registry: {error}",
                        record.thread_id
                    ))
                })?
                .unwrap_or_else(|| record.config_snapshot_json.clone())
        } else {
            record.config_snapshot_json.clone()
        };

        serde_json::from_str::<PersistedThreadConfigSnapshot>(&config_json).map_err(|error| {
            internal_error(format!(
                "failed to deserialize OpenRouter beta config snapshot {}: {error}",
                record.thread_id
            ))
        })
    }

    fn history_file_from_record(&self, record: &ThreadRecord) -> HistoryFile {
        if let Some(path) = record.session_state_path.clone() {
            return HistoryFile::from_path(path);
        }
        if let Some(reference) = record.session_state_ref.clone() {
            return HistoryFile::from_path(PathBuf::from(reference));
        }
        HistoryFile::new(self.codex_home.as_path(), &record.thread_id.to_string())
    }

    async fn upsert_registry_thread(
        &self,
        persisted: PersistedOpenRouterThread,
        archived: bool,
    ) -> anyhow::Result<()> {
        let config_json = serde_json::to_string(&persisted.config_snapshot)?;
        let thread_id = ThreadId::from_string(&persisted.thread.id)?;
        self.registry
            .upsert_thread(thread_record_input_from_persisted(persisted, archived)?)
            .await?;
        self.registry
            .upsert_config_snapshot(thread_id, config_json)
            .await?;
        Ok(())
    }

    pub(crate) async fn has_known_thread(
        &self,
        thread_id: ThreadId,
    ) -> Result<bool, JSONRPCErrorError> {
        Ok(self.thread_record(thread_id).await?.is_some())
    }

    pub(crate) async fn is_loaded(&self, thread_id: ThreadId) -> bool {
        self.threads.lock().await.contains_key(&thread_id)
    }

    pub(crate) async fn start_thread(
        &self,
        config: Config,
        connection_id: ConnectionId,
        thread_state_manager: &ThreadStateManager,
    ) -> Result<StartedOpenRouterThread, JSONRPCErrorError> {
        let thread_id = ThreadId::new();
        let config_snapshot = thread_config_snapshot(&config);
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let thread = Thread {
            id: thread_id.to_string(),
            forked_from_id: None,
            preview: String::new(),
            ephemeral: config_snapshot.ephemeral,
            model_provider: config_snapshot.model_provider_id.clone(),
            created_at: now,
            updated_at: now,
            status: ThreadStatus::Idle,
            path: (!config_snapshot.ephemeral).then(|| {
                HistoryFile::new(self.codex_home.as_path(), &thread_id.to_string())
                    .path()
                    .to_path_buf()
            }),
            cwd: config_snapshot.cwd.clone(),
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            agent_nickname: config_snapshot.session_source.get_nickname(),
            agent_role: config_snapshot.session_source.get_agent_role(),
            source: config_snapshot.session_source.clone().into(),
            git_info: None,
            name: None,
            turns: Vec::new(),
        };
        let runtime = OpenRouterBetaSession::from_config(&config)
            .await
            .map_err(|error| {
                internal_error(format!("failed to create OpenRouter beta thread: {error}"))
            })?;
        let openrouter_thread = Arc::new(OpenRouterBetaThread {
            runtime: Mutex::new(runtime),
            state: Mutex::new(OpenRouterThreadState {
                config_snapshot: config_snapshot.clone(),
                shell_environment_policy: config.permissions.shell_environment_policy.clone(),
                thread: thread.clone(),
                lifecycle: TurnLifecycleState::Idle,
                elicitation_count: 0,
            }),
        });
        self.persist_thread(openrouter_thread.clone())
            .await
            .map_err(|error| {
                internal_error(format!("failed to persist OpenRouter beta thread: {error}"))
            })?;
        self.threads
            .lock()
            .await
            .insert(thread_id, openrouter_thread);
        let _ = thread_state_manager
            .try_add_connection_to_thread(thread_id, connection_id)
            .await;

        Ok(StartedOpenRouterThread {
            response: ThreadStartResponse {
                thread: thread.clone(),
                model: config_snapshot.model.clone(),
                model_provider: config_snapshot.model_provider_id.clone(),
                service_tier: config_snapshot.service_tier,
                cwd: config_snapshot.cwd.clone(),
                approval_policy: config_snapshot.approval_policy.into(),
                approvals_reviewer: config_snapshot.approvals_reviewer.into(),
                sandbox: config_snapshot.sandbox_policy.clone().into(),
                reasoning_effort: config_snapshot.reasoning_effort,
            },
            notification: ThreadStartedNotification { thread },
        })
    }

    pub(crate) async fn contains(&self, thread_id: ThreadId) -> bool {
        self.has_known_thread(thread_id).await.unwrap_or(false)
    }

    pub(crate) async fn read_thread(
        &self,
        thread_id: ThreadId,
        include_turns: bool,
    ) -> Option<Thread> {
        let thread = self.threads.lock().await.get(&thread_id).cloned()?;
        let state = thread.state.lock().await;
        let mut snapshot = state.thread.clone();
        if !include_turns {
            snapshot.turns.clear();
        }
        Some(snapshot)
    }

    async fn list_registry_threads(
        &self,
        filter: ThreadFilter,
    ) -> Result<Vec<Thread>, JSONRPCErrorError> {
        self.ensure_registry_ready().await.map_err(|error| {
            internal_error(format!(
                "failed to initialize OpenRouter beta thread registry: {error}"
            ))
        })?;
        let records = self.registry.list_threads(filter).await.map_err(|error| {
            internal_error(format!(
                "failed to list OpenRouter beta threads from registry: {error}"
            ))
        })?;
        let loaded = self.threads.lock().await.clone();
        let mut threads = Vec::with_capacity(records.len());
        for record in records {
            if let Some(thread) = loaded.get(&record.thread_id) {
                threads.push(thread.state.lock().await.thread.clone());
                continue;
            }
            threads.push(thread_from_record(record)?);
        }
        Ok(threads)
    }

    pub(crate) async fn read_persisted_thread(
        &self,
        thread_id: ThreadId,
        include_turns: bool,
    ) -> Result<Option<Thread>, JSONRPCErrorError> {
        let Some(record) = self.thread_record(thread_id).await? else {
            return Ok(None);
        };
        if !include_turns {
            return Ok(Some(thread_from_record(record)?));
        }
        let Some(persisted) = self
            .load_persisted_thread_from_record(thread_id, &record)
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(persisted_thread_to_thread(persisted, true)))
    }

    pub(crate) async fn has_persisted_active_thread(
        &self,
        thread_id: ThreadId,
    ) -> Result<bool, JSONRPCErrorError> {
        Ok(self.thread_record(thread_id).await?.is_some_and(|record| {
            record.archived_at.is_none() && record.session_state_path.is_some()
        }))
    }

    pub(crate) async fn loaded_thread_ids(&self) -> Vec<String> {
        self.threads
            .lock()
            .await
            .keys()
            .map(ToString::to_string)
            .collect()
    }

    pub(crate) async fn list_threads(&self) -> Result<Vec<Thread>, JSONRPCErrorError> {
        self.list_registry_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta))
            .await
    }

    pub(crate) async fn list_archived_threads(&self) -> Result<Vec<Thread>, JSONRPCErrorError> {
        self.ensure_registry_ready().await.map_err(|error| {
            internal_error(format!(
                "failed to initialize OpenRouter beta thread registry: {error}"
            ))
        })?;
        let records = self
            .registry
            .list_threads(ThreadFilter::for_lane(ProviderLane::OpenRouterBeta).including_archived())
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to list archived OpenRouter beta threads from registry: {error}"
                ))
            })?;
        let mut threads = Vec::new();
        for record in records {
            if record.archived_at.is_none() {
                continue;
            }
            threads.push(thread_from_record(record)?);
        }
        Ok(threads)
    }

    pub(crate) async fn set_thread_name(
        &self,
        thread_id: ThreadId,
        name: String,
    ) -> Result<Thread, JSONRPCErrorError> {
        let loaded_thread = {
            let threads = self.threads.lock().await;
            threads.get(&thread_id).cloned()
        };

        if let Some(thread) = loaded_thread {
            let mut state = thread.state.lock().await;
            state.thread.updated_at = now_ts();
            state.thread.name = Some(name);
            let snapshot = state.thread.clone();
            drop(state);
            self.persist_thread(thread).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta thread name: {error}"
                ))
            })?;
            return Ok(snapshot);
        }

        let Some(record) = self.thread_record(thread_id).await? else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };
        let now = now_ts();
        self.registry
            .set_thread_name(thread_id, name.clone(), now)
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to update OpenRouter beta thread registry name: {error}"
                ))
            })?;
        let mut updated = thread_from_record(record)?;
        updated.name = Some(name);
        updated.updated_at = now;
        Ok(updated)
    }

    pub(crate) async fn update_git_info(
        &self,
        thread_id: ThreadId,
        sha_patch: Option<Option<String>>,
        branch_patch: Option<Option<String>>,
        origin_url_patch: Option<Option<String>>,
    ) -> Result<Thread, JSONRPCErrorError> {
        if let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() {
            let mut state = thread.state.lock().await;
            let current = state.thread.git_info.clone().unwrap_or(GitInfo {
                sha: None,
                branch: None,
                origin_url: None,
            });
            state.thread.updated_at = now_ts();
            state.thread.git_info = Some(apply_git_info_patch(
                current,
                sha_patch,
                branch_patch,
                origin_url_patch,
            ));
            let snapshot = state.thread.clone();
            drop(state);
            self.persist_thread(thread).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta git metadata: {error}"
                ))
            })?;
            return Ok(snapshot);
        }

        let Some(record) = self.thread_record(thread_id).await? else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };
        let now = now_ts();
        let current = GitInfo {
            sha: record.git_commit.clone(),
            branch: record.git_branch.clone(),
            origin_url: record.git_origin_url.clone(),
        };
        let patched = apply_git_info_patch(
            current,
            sha_patch.clone(),
            branch_patch.clone(),
            origin_url_patch.clone(),
        );
        let patch = ThreadGitInfoPatch {
            sha: sha_patch,
            branch: branch_patch,
            origin_url: origin_url_patch,
        };
        self.registry
            .update_thread_git_info(thread_id, patch, now)
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to update OpenRouter beta thread registry git metadata: {error}"
                ))
            })?;
        let mut updated = thread_from_record(record)?;
        updated.git_info = Some(patched);
        updated.updated_at = now;
        Ok(updated)
    }

    pub(crate) async fn increment_elicitation(
        &self,
        thread_id: ThreadId,
    ) -> Result<u64, JSONRPCErrorError> {
        if let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() {
            let mut state = thread.state.lock().await;
            state.elicitation_count = state.elicitation_count.saturating_add(1);
            let count = state.elicitation_count;
            drop(state);
            self.persist_thread(thread).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta elicitation count: {error}"
                ))
            })?;
            return Ok(count);
        }

        let count = self
            .persisted_elicitation_update(thread_id, |c| Ok(c.saturating_add(1)))
            .await?;
        Ok(count)
    }

    pub(crate) async fn decrement_elicitation(
        &self,
        thread_id: ThreadId,
    ) -> Result<u64, JSONRPCErrorError> {
        if let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() {
            let mut state = thread.state.lock().await;
            if state.elicitation_count == 0 {
                return Err(invalid_request(
                    "out-of-band elicitation count is already zero".to_string(),
                ));
            }
            state.elicitation_count -= 1;
            let count = state.elicitation_count;
            drop(state);
            self.persist_thread(thread).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta elicitation count: {error}"
                ))
            })?;
            return Ok(count);
        }

        self.persisted_elicitation_update(thread_id, |c| {
            if c == 0 {
                return Err(invalid_request(
                    "out-of-band elicitation count is already zero".to_string(),
                ));
            }
            Ok(c - 1)
        })
        .await
    }

    async fn persisted_elicitation_update<F>(
        &self,
        thread_id: ThreadId,
        update: F,
    ) -> Result<u64, JSONRPCErrorError>
    where
        F: FnOnce(u64) -> Result<u64, JSONRPCErrorError>,
    {
        let Some(record) = self.thread_record(thread_id).await? else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };

        let history_file = self.history_file_from_record(&record);
        let entries = history_file.read_all().await.map_err(|error| {
            internal_error(format!(
                "failed to read history for OpenRouter beta thread {thread_id} elicitation: {error}"
            ))
        })?;

        let (messages, continuation, mut elicitation_count) =
            HistoryFile::into_history_parts(entries).await;
        let new_count = update(elicitation_count)?;
        elicitation_count = new_count;

        let new_entries = build_history_entries(&messages, &continuation, elicitation_count);
        history_file.write_all(&new_entries).await.map_err(|error| {
            internal_error(format!(
                "failed to write history for OpenRouter beta thread {thread_id} elicitation: {error}"
            ))
        })?;

        let now = now_ts();
        self.registry
            .touch_thread_updated_at(thread_id, now)
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to update OpenRouter beta thread registry timestamp: {error}"
                ))
            })?;
        Ok(new_count)
    }

    async fn persist_thread(&self, thread: Arc<OpenRouterBetaThread>) -> anyhow::Result<()> {
        let (config_snapshot, thread_snapshot, elicitation_count) = {
            let state = thread.state.lock().await;
            if state.config_snapshot.ephemeral {
                return Ok(());
            }
            (
                state.config_snapshot.clone(),
                state.thread.clone(),
                state.elicitation_count,
            )
        };
        let (history, continuation) = {
            let runtime = thread.runtime.lock().await;
            (runtime.history().to_vec(), runtime.continuation().cloned())
        };
        let thread_id = ThreadId::from_string(&thread_snapshot.id)?;

        let history_file = HistoryFile::new(self.codex_home.as_path(), &thread_id.to_string());
        let entries = build_history_entries(&history, &continuation, elicitation_count);
        history_file.write_all(&entries).await?;

        let persisted = openrouter_beta_persistence::build_persisted_thread(
            thread_snapshot,
            &config_snapshot,
            history,
            continuation,
            elicitation_count,
        );
        let persisted = openrouter_beta_persistence::persisted_with_path(
            persisted,
            history_file.path().to_path_buf(),
        );
        self.upsert_registry_thread(persisted.clone(), false)
            .await?;
        openrouter_beta_persistence::save_active_thread(self.codex_home.as_path(), &persisted)
            .await?;
        Ok(())
    }

    async fn load_persisted_thread_from_record(
        &self,
        thread_id: ThreadId,
        record: &ThreadRecord,
    ) -> Result<Option<PersistedOpenRouterThread>, JSONRPCErrorError> {
        let config_snapshot = self.config_snapshot_from_record(record).await?;

        let history_file = self.history_file_from_record(record);
        let entries = history_file.read_all().await.map_err(|error| {
            internal_error(format!(
                "failed to read history for OpenRouter beta thread {thread_id}: {error}"
            ))
        })?;

        if entries.is_empty() {
            // Legacy snapshot fallback only recovers payload state.
            // Archive truth still comes from the registry record.
            let snapshot = openrouter_beta_persistence::load_thread_snapshot(
                openrouter_beta_persistence::active_thread_path(
                    self.codex_home.as_path(),
                    &thread_id.to_string(),
                ),
            )
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to load persisted OpenRouter beta thread {thread_id}: {error}"
                ))
            })?
            .or(openrouter_beta_persistence::load_thread_snapshot(
                openrouter_beta_persistence::archived_thread_path(
                    self.codex_home.as_path(),
                    &thread_id.to_string(),
                ),
            )
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to load archived persisted OpenRouter beta thread {thread_id}: {error}"
                ))
            })?);
            if let Some(snapshot) = snapshot {
                let mut thread_meta = thread_from_record(record.clone())?;
                thread_meta.turns = snapshot.thread.turns;
                return Ok(Some(PersistedOpenRouterThread {
                    version: snapshot.version,
                    config_snapshot,
                    thread: thread_meta,
                    history: snapshot.history,
                    continuation: snapshot.continuation,
                    elicitation_count: snapshot.elicitation_count,
                }));
            }
            return Ok(None);
        }

        let (messages, continuation, elicitation_count) =
            HistoryFile::into_history_parts(entries).await;

        let thread_meta = thread_from_record(record.clone())?;
        Ok(Some(PersistedOpenRouterThread {
            version: 1,
            config_snapshot,
            thread: thread_meta,
            history: messages,
            continuation,
            elicitation_count,
        }))
    }

    fn validate_resume_snapshot(
        &self,
        config: &Config,
        snapshot: &ThreadConfigSnapshot,
    ) -> Result<OpenRouterResumeBackend, JSONRPCErrorError> {
        let provider_known = config
            .model_providers
            .contains_key(&snapshot.model_provider_id)
            || config
                .model_provider
                .name
                .to_ascii_lowercase()
                .contains("openrouter");
        if !provider_known {
            return Err(invalid_request(format!(
                "OpenRouter beta thread expects provider {} but current config does not expose it",
                snapshot.model_provider_id
            )));
        }
        let provider = config
            .model_providers
            .get(&snapshot.model_provider_id)
            .cloned()
            .unwrap_or_else(|| config.model_provider.clone());
        if !provider.name.to_ascii_lowercase().contains("openrouter") {
            return Err(invalid_request(format!(
                "OpenRouter beta thread expects an OpenRouter provider for {} but current config resolved {}",
                snapshot.model_provider_id, provider.name
            )));
        }
        logos_openrouter_beta::transport_policy_from_provider(&provider).map_err(|error| {
            invalid_request(format!(
                "OpenRouter beta thread provider {} is not resumable with current config: {error}",
                snapshot.model_provider_id
            ))
        })?;
        if snapshot.model.trim().is_empty() {
            return Err(invalid_request(
                "OpenRouter beta thread snapshot is missing a model id".to_string(),
            ));
        }
        Ok(OpenRouterResumeBackend::ClientManagedReplay)
    }

    async fn build_runtime_from_persisted(
        &self,
        thread_id: ThreadId,
        config: &Config,
        snapshot: &ThreadConfigSnapshot,
        persisted: &PersistedOpenRouterThread,
    ) -> Result<OpenRouterBetaSession, JSONRPCErrorError> {
        match self.validate_resume_snapshot(config, snapshot)? {
            OpenRouterResumeBackend::ClientManagedReplay => OpenRouterBetaSession::forked_from(
                persisted.history.clone(),
                persisted.continuation.clone(),
                session_settings_from_persisted(config, snapshot),
            )
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to restore OpenRouter beta thread {thread_id}: {error}"
                ))
            }),
        }
    }

    pub(crate) async fn resume_persisted_thread(
        &self,
        thread_id: ThreadId,
        config: Config,
        connection_id: ConnectionId,
        thread_state_manager: &ThreadStateManager,
    ) -> Result<Option<ThreadResumeResponse>, JSONRPCErrorError> {
        let Some(record) = self.thread_record(thread_id).await? else {
            return Ok(None);
        };
        if record.archived_at.is_some() {
            return Ok(None);
        }
        let config_snapshot = self.config_snapshot_from_record(&record).await?.to_core();
        let Some(persisted) = self
            .load_persisted_thread_from_record(thread_id, &record)
            .await?
        else {
            return Ok(None);
        };
        let runtime = self
            .build_runtime_from_persisted(thread_id, &config, &config_snapshot, &persisted)
            .await?;

        let thread = Arc::new(OpenRouterBetaThread {
            runtime: Mutex::new(runtime),
            state: Mutex::new(OpenRouterThreadState {
                config_snapshot: config_snapshot.clone(),
                shell_environment_policy: config.permissions.shell_environment_policy.clone(),
                thread: persisted.thread.clone(),
                lifecycle: TurnLifecycleState::Idle,
                elicitation_count: persisted.elicitation_count,
            }),
        });
        self.threads.lock().await.insert(thread_id, thread.clone());
        let _ = thread_state_manager
            .try_add_connection_to_thread(thread_id, connection_id)
            .await;
        self.persist_thread(thread).await.map_err(|error| {
            internal_error(format!(
                "failed to persist restored OpenRouter beta thread {thread_id}: {error}"
            ))
        })?;

        Ok(Some(ThreadResumeResponse {
            thread: persisted.thread,
            model: config_snapshot.model.clone(),
            model_provider: config_snapshot.model_provider_id.clone(),
            service_tier: config_snapshot.service_tier,
            cwd: config_snapshot.cwd.clone(),
            approval_policy: config_snapshot.approval_policy.into(),
            approvals_reviewer: config_snapshot.approvals_reviewer.into(),
            sandbox: config_snapshot.sandbox_policy.clone().into(),
            reasoning_effort: config_snapshot.reasoning_effort,
        }))
    }

    pub(crate) async fn archive_thread(
        &self,
        thread_id: ThreadId,
        thread_state_manager: &ThreadStateManager,
    ) -> Result<bool, JSONRPCErrorError> {
        let loaded_thread = {
            let threads = self.threads.lock().await;
            threads.get(&thread_id).cloned()
        };

        if let Some(thread) = loaded_thread {
            {
                let state = thread.state.lock().await;
                if state.config_snapshot.ephemeral {
                    return Err(invalid_request(format!(
                        "OpenRouter beta thread {thread_id} is ephemeral and cannot be archived"
                    )));
                }
                if state.lifecycle.is_running() {
                    return Err(invalid_request(format!(
                        "OpenRouter beta thread {thread_id} has an active turn and cannot be archived"
                    )));
                }
            }
            if self.host_exec_registry.has_thread_commands(thread_id).await {
                return Err(invalid_request(format!(
                    "OpenRouter beta thread {thread_id} has running shell commands and cannot be archived"
                )));
            }
            self.persist_thread(thread.clone()).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta thread before archive: {error}"
                ))
            })?;
            let now = now_ts();
            self.registry
                .archive_thread(thread_id, now)
                .await
                .map_err(|error| {
                    internal_error(format!(
                        "failed to update OpenRouter beta thread registry during archive: {error}"
                    ))
                })?;
            self.threads.lock().await.remove(&thread_id);
            let thread_state_manager = thread_state_manager.clone();
            tokio::spawn(async move {
                thread_state_manager.remove_thread_state(thread_id).await;
            });
            return Ok(true);
        }

        let Some(record) = self.thread_record(thread_id).await? else {
            return Ok(false);
        };
        if record.archived_at.is_some() {
            return Ok(false);
        }
        let now = now_ts();
        self.registry
            .archive_thread(thread_id, now)
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to update OpenRouter beta thread registry during archive: {error}"
                ))
            })?;
        Ok(true)
    }

    pub(crate) async fn unarchive_thread(
        &self,
        thread_id: ThreadId,
    ) -> Result<Option<Thread>, JSONRPCErrorError> {
        let Some(record) = self.thread_record(thread_id).await? else {
            return Ok(None);
        };
        if record.archived_at.is_none() {
            return Ok(None);
        }
        let now = now_ts();
        self.registry
            .unarchive_thread(thread_id, now)
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to update OpenRouter beta thread registry during unarchive: {error}"
                ))
            })?;
        let updated_record = self
            .thread_record(thread_id)
            .await?
            .ok_or_else(|| invalid_request(format!("thread not found: {thread_id}")))?;
        Ok(Some(thread_from_record(updated_record)?))
    }

    pub(crate) async fn background_terminals_clean(
        &self,
        thread_id: ThreadId,
    ) -> Result<ThreadBackgroundTerminalsCleanResponse, JSONRPCErrorError> {
        if !self.threads.lock().await.contains_key(&thread_id) {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        }
        self.host_exec_registry.cancel_thread(thread_id).await;
        Ok(ThreadBackgroundTerminalsCleanResponse {})
    }

    pub(crate) async fn shell_command(
        &self,
        thread_id: ThreadId,
        command: String,
        outgoing: Arc<OutgoingMessageSender>,
        thread_state_manager: ThreadStateManager,
    ) -> Result<ThreadShellCommandResponse, JSONRPCErrorError> {
        let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };
        let (turn_id, standalone_turn, cwd) = {
            let mut state = thread.state.lock().await;
            let cwd = state.config_snapshot.cwd.clone();
            if let Some(active_turn_id) = state.lifecycle.active_turn_id().map(str::to_owned) {
                (active_turn_id, false, cwd)
            } else {
                let turn_id = Uuid::new_v4().to_string();
                state.lifecycle = TurnLifecycleState::Running {
                    turn_id: turn_id.clone(),
                };
                state.thread.updated_at = now_ts();
                state.thread.status = ThreadStatus::Active {
                    active_flags: Vec::new(),
                };
                state.thread.turns.push(Turn {
                    id: turn_id.clone(),
                    items: Vec::new(),
                    status: TurnStatus::InProgress,
                    error: None,
                });
                (turn_id, true, cwd)
            }
        };

        if standalone_turn {
            send_notification(
                &outgoing,
                &thread_state_manager,
                thread_id,
                ServerNotification::TurnStarted(TurnStartedNotification {
                    thread_id: thread_id.to_string(),
                    turn: Turn {
                        id: turn_id.clone(),
                        items: Vec::new(),
                        status: TurnStatus::InProgress,
                        error: None,
                    },
                }),
            )
            .await;
        }

        let item_id = Uuid::new_v4().to_string();
        let process_id = format!("openrouter-shell-{item_id}");
        let shell_environment_policy = {
            let state = thread.state.lock().await;
            state.shell_environment_policy.clone()
        };
        let manager = self.clone();
        let host_exec_registry = self.host_exec_registry.clone();
        tokio::spawn(async move {
            let sink = Arc::new(OpenRouterHostExecSink {
                outgoing: outgoing.clone(),
                thread_state_manager: thread_state_manager.clone(),
                thread_id,
                turn_id: turn_id.clone(),
            });
            let outcome = run_one_shot_shell_command(HostExecContext {
                turn_id: turn_id.clone(),
                call_id: item_id,
                process_id: Some(process_id),
                raw_command: command,
                cwd,
                env: create_env(&shell_environment_policy, Some(thread_id)),
                shell: Arc::new(default_user_shell()),
                use_login_shell: !cfg!(windows),
                shell_environment_overrides: shell_environment_policy.r#set.clone(),
                source: CoreExecCommandSource::UserShell,
                sandbox_policy: HostExecSandboxPolicy::NoSandbox,
                approval_policy: HostExecApprovalPolicy::SkipApproval,
                notification_sink: sink,
                cancellation_token: tokio_util::sync::CancellationToken::new(),
                registration: Some(HostExecRegistration {
                    registry: host_exec_registry,
                    thread_id,
                }),
                network: None,
                windows_sandbox_level: WindowsSandboxLevel::Disabled,
                windows_sandbox_private_desktop: false,
            })
            .await;
            manager
                .finalize_openrouter_shell_command(
                    thread,
                    thread_id,
                    turn_id,
                    standalone_turn,
                    outgoing,
                    thread_state_manager,
                    outcome,
                )
                .await;
        });

        Ok(ThreadShellCommandResponse {})
    }

    #[allow(clippy::too_many_arguments)]
    async fn finalize_openrouter_shell_command(
        &self,
        thread: Arc<OpenRouterBetaThread>,
        thread_id: ThreadId,
        turn_id: String,
        standalone_turn: bool,
        outgoing: Arc<OutgoingMessageSender>,
        thread_state_manager: ThreadStateManager,
        outcome: anyhow::Result<HostExecOutcome>,
    ) {
        let completed_item = match outcome {
            Ok(outcome) => openrouter_command_execution_item(outcome),
            Err(_) => return,
        };

        {
            let mut state = thread.state.lock().await;
            if let Some(turn) = state
                .thread
                .turns
                .iter_mut()
                .find(|turn| turn.id == turn_id)
            {
                turn.items.push(completed_item.clone());
                if standalone_turn {
                    turn.status = TurnStatus::Completed;
                    turn.error = None;
                }
            }
            state.thread.updated_at = now_ts();
            if standalone_turn {
                state.lifecycle = TurnLifecycleState::Idle;
                state.thread.status = ThreadStatus::Idle;
            }
        }
        let _ = self.persist_thread(thread.clone()).await;

        if standalone_turn {
            let completed_turn = {
                let state = thread.state.lock().await;
                state
                    .thread
                    .turns
                    .iter()
                    .find(|turn| turn.id == turn_id)
                    .cloned()
                    .unwrap_or(Turn {
                        id: turn_id.clone(),
                        items: Vec::new(),
                        status: TurnStatus::Completed,
                        error: None,
                    })
            };
            send_notification(
                &outgoing,
                &thread_state_manager,
                thread_id,
                ServerNotification::TurnCompleted(TurnCompletedNotification {
                    thread_id: thread_id.to_string(),
                    turn: completed_turn,
                }),
            )
            .await;
        }
    }

    pub(crate) async fn background_terminal_start(
        &self,
        thread_id: ThreadId,
        command: String,
        outgoing: Arc<OutgoingMessageSender>,
        thread_state_manager: ThreadStateManager,
    ) -> Result<ThreadBackgroundTerminalStartResponse, JSONRPCErrorError> {
        let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };

        let item_id = Uuid::new_v4().to_string();
        let process_id = format!("openrouter-bgterm-{item_id}");
        let cwd = {
            let state = thread.state.lock().await;
            state.config_snapshot.cwd.clone()
        };
        let shell_environment_policy = {
            let state = thread.state.lock().await;
            state.shell_environment_policy.clone()
        };

        let sink = Arc::new(OpenRouterHostExecSink {
            outgoing: outgoing.clone(),
            thread_state_manager: thread_state_manager.clone(),
            thread_id,
            turn_id: String::new(),
        });

        let host_exec_registry = self.host_exec_registry.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();

        let ctx = codex_core::host_exec::InteractiveHostExecContext {
            turn_id: String::new(),
            call_id: item_id.clone(),
            process_id: Some(process_id.clone()),
            raw_command: command,
            cwd: cwd.clone(),
            env: create_env(&shell_environment_policy, Some(thread_id)),
            shell: Arc::new(default_user_shell()),
            use_login_shell: !cfg!(windows),
            shell_environment_overrides: shell_environment_policy.r#set.clone(),
            source: CoreExecCommandSource::UnifiedExecStartup,
            sandbox_policy: HostExecSandboxPolicy::NoSandbox,
            approval_policy: HostExecApprovalPolicy::SkipApproval,
            notification_sink: sink,
            cancellation_token,
            registration: Some(HostExecRegistration {
                registry: host_exec_registry,
                thread_id,
            }),
            windows_sandbox_level: WindowsSandboxLevel::Disabled,
            windows_sandbox_private_desktop: false,
        };

        codex_core::host_exec::spawn_interactive_process(ctx)
            .await
            .map_err(|err| internal_error(format!("failed to start background terminal: {err}")))?;

        Ok(ThreadBackgroundTerminalStartResponse {
            call_id: item_id,
            process_id,
        })
    }

    pub(crate) async fn start_turn(
        &self,
        thread_id: ThreadId,
        params: TurnStartParams,
        outgoing: Arc<OutgoingMessageSender>,
        thread_state_manager: ThreadStateManager,
    ) -> Result<TurnStartResponse, JSONRPCErrorError> {
        let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() else {
            return Err(invalid_request(format!("thread not found: {thread_id}")));
        };
        validate_turn_params(&thread, &params).await?;
        let runtime_messages = runtime_messages_from_input(&params.input)?;
        let turn_id = Uuid::new_v4().to_string();
        let user_item = ThreadItem::UserMessage {
            id: format!("user-{turn_id}"),
            content: params.input.clone(),
        };

        let start_events = {
            let mut state = thread.state.lock().await;
            begin_turn(&mut state, thread_id, turn_id.clone(), user_item.clone())?
        };
        dispatch_turn_events(&outgoing, &thread_state_manager, thread_id, start_events).await;

        let thread_for_task = thread.clone();
        let turn_id_for_task = turn_id.clone();
        let manager = self.clone();
        tokio::spawn(async move {
            let result = {
                let mut runtime = thread_for_task.runtime.lock().await;
                runtime.run_turn(runtime_messages).await
            };
            match result {
                Ok(result) => {
                    let tool_items =
                        dynamic_tool_items(&result.new_history, &result.tool_call_records);
                    let assistant_item =
                        result
                            .final_message
                            .clone()
                            .map(|text| ThreadItem::AgentMessage {
                                id: format!("assistant-{turn_id_for_task}"),
                                text,
                                phase: None,
                                memory_citation: None,
                            });
                    {
                        let mut state = thread_for_task.state.lock().await;
                        update_completed_turn(
                            &mut state,
                            &turn_id_for_task,
                            tool_items.clone(),
                            assistant_item.clone(),
                            result.final_message.clone(),
                        );
                    }
                    let _ = manager.persist_thread(thread_for_task.clone()).await;
                    let completed_turn = {
                        let state = thread_for_task.state.lock().await;
                        state
                            .thread
                            .turns
                            .iter()
                            .find(|turn| turn.id == turn_id_for_task)
                            .cloned()
                            .unwrap_or_else(|| empty_completed_turn(turn_id_for_task.clone()))
                    };
                    let events = complete_turn_events(
                        thread_id,
                        &turn_id_for_task,
                        tool_items,
                        assistant_item,
                        completed_turn,
                    );
                    dispatch_turn_events(&outgoing, &thread_state_manager, thread_id, events).await;
                }
                Err(error) => {
                    let turn_error = TurnError {
                        message: error.to_string(),
                        codex_error_info: None,
                        additional_details: None,
                    };
                    {
                        let mut state = thread_for_task.state.lock().await;
                        update_failed_turn(&mut state, &turn_id_for_task, turn_error.clone());
                    }
                    let _ = manager.persist_thread(thread_for_task.clone()).await;
                    let events =
                        failed_turn_events(thread_id, turn_id_for_task.clone(), turn_error);
                    dispatch_turn_events(&outgoing, &thread_state_manager, thread_id, events).await;
                }
            }
        });

        Ok(TurnStartResponse {
            turn: Turn {
                id: turn_id,
                items: Vec::new(),
                status: TurnStatus::InProgress,
                error: None,
            },
        })
    }

    pub(crate) async fn resume_thread(
        &self,
        thread_id: ThreadId,
        connection_id: ConnectionId,
        thread_state_manager: &ThreadStateManager,
    ) -> Result<ThreadResumeResponse, JSONRPCErrorError> {
        let Some(thread) = self.threads.lock().await.get(&thread_id).cloned() else {
            return Err(invalid_request(format!(
                "OpenRouter beta thread not found: {thread_id}"
            )));
        };
        let _ = thread_state_manager
            .try_add_connection_to_thread(thread_id, connection_id)
            .await;
        let state = thread.state.lock().await;
        let snapshot = &state.config_snapshot;
        Ok(ThreadResumeResponse {
            thread: state.thread.clone(),
            model: snapshot.model.clone(),
            model_provider: snapshot.model_provider_id.clone(),
            service_tier: snapshot.service_tier,
            cwd: snapshot.cwd.clone(),
            approval_policy: snapshot.approval_policy.into(),
            approvals_reviewer: snapshot.approvals_reviewer.into(),
            sandbox: snapshot.sandbox_policy.clone().into(),
            reasoning_effort: snapshot.reasoning_effort,
        })
    }

    pub(crate) async fn fork_thread(
        &self,
        source_thread_id: ThreadId,
        config: Config,
        connection_id: ConnectionId,
        thread_state_manager: &ThreadStateManager,
    ) -> Result<(ThreadForkResponse, ThreadStartedNotification), JSONRPCErrorError> {
        if let Some(source_thread) = self.threads.lock().await.get(&source_thread_id).cloned() {
            {
                let state = source_thread.state.lock().await;
                if state.lifecycle.is_running() {
                    return Err(invalid_request(format!(
                        "OpenRouter beta thread {source_thread_id} has an active turn and cannot be forked"
                    )));
                }
            }
            self.persist_thread(source_thread).await.map_err(|error| {
                internal_error(format!(
                    "failed to persist OpenRouter beta thread before fork: {error}"
                ))
            })?;
        }
        let Some(record) = self.thread_record(source_thread_id).await? else {
            return Err(invalid_request(format!(
                "OpenRouter beta thread not found: {source_thread_id}"
            )));
        };
        if record.archived_at.is_some() {
            return Err(invalid_request(format!(
                "OpenRouter beta thread {source_thread_id} is archived and cannot be forked"
            )));
        }
        let Some(source_persisted) = self
            .load_persisted_thread_from_record(source_thread_id, &record)
            .await?
        else {
            return Err(internal_error(format!(
                "failed to resolve durable state for OpenRouter beta thread {source_thread_id}"
            )));
        };

        let new_thread_id = ThreadId::new();
        let config_snapshot = thread_config_snapshot(&config);
        let now = now_ts();

        self.validate_resume_snapshot(&config, &config_snapshot)?;
        let forked_session = OpenRouterBetaSession::forked_from(
            source_persisted.history.clone(),
            source_persisted.continuation.clone(),
            session_settings_from_config(&config, &config_snapshot),
        )
        .await
        .map_err(|error| {
            internal_error(format!(
                "failed to create forked OpenRouter beta session: {error}"
            ))
        })?;

        let source_thread_meta = source_persisted.thread.clone();

        let new_thread = Thread {
            id: new_thread_id.to_string(),
            forked_from_id: Some(source_thread_id.to_string()),
            preview: source_thread_meta.preview.clone(),
            ephemeral: config_snapshot.ephemeral,
            model_provider: config_snapshot.model_provider_id.clone(),
            created_at: now,
            updated_at: now,
            status: ThreadStatus::Idle,
            path: (!config_snapshot.ephemeral).then(|| {
                HistoryFile::new(self.codex_home.as_path(), &new_thread_id.to_string())
                    .path()
                    .to_path_buf()
            }),
            cwd: config_snapshot.cwd.clone(),
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            agent_nickname: config_snapshot.session_source.get_nickname(),
            agent_role: config_snapshot.session_source.get_agent_role(),
            source: config_snapshot.session_source.clone().into(),
            git_info: None,
            name: None,
            turns: source_thread_meta.turns.clone(),
        };

        let new_or_thread = Arc::new(OpenRouterBetaThread {
            runtime: Mutex::new(forked_session),
            state: Mutex::new(OpenRouterThreadState {
                config_snapshot: config_snapshot.clone(),
                shell_environment_policy: config.permissions.shell_environment_policy.clone(),
                thread: new_thread.clone(),
                lifecycle: TurnLifecycleState::Idle,
                elicitation_count: 0,
            }),
        });
        self.persist_thread(new_or_thread.clone())
            .await
            .map_err(|error| {
                internal_error(format!(
                    "failed to persist forked OpenRouter beta thread: {error}"
                ))
            })?;
        self.threads
            .lock()
            .await
            .insert(new_thread_id, new_or_thread);
        let _ = thread_state_manager
            .try_add_connection_to_thread(new_thread_id, connection_id)
            .await;

        Ok((
            ThreadForkResponse {
                thread: new_thread.clone(),
                model: config_snapshot.model.clone(),
                model_provider: config_snapshot.model_provider_id.clone(),
                service_tier: config_snapshot.service_tier,
                cwd: config_snapshot.cwd.clone(),
                approval_policy: config_snapshot.approval_policy.into(),
                approvals_reviewer: config_snapshot.approvals_reviewer.into(),
                sandbox: config_snapshot.sandbox_policy.clone().into(),
                reasoning_effort: config_snapshot.reasoning_effort,
            },
            ThreadStartedNotification { thread: new_thread },
        ))
    }

    pub(crate) async fn start_review(
        &self,
        source_thread_id: ThreadId,
        config: Config,
        target: ApiReviewTarget,
        delivery: Option<ApiReviewDelivery>,
        connection_id: ConnectionId,
        outgoing: Arc<OutgoingMessageSender>,
        thread_state_manager: ThreadStateManager,
    ) -> Result<ReviewStartResponse, JSONRPCErrorError> {
        let Some(source_thread) = self.threads.lock().await.get(&source_thread_id).cloned() else {
            return Err(invalid_request(format!(
                "OpenRouter beta thread not found: {source_thread_id}"
            )));
        };

        let (review_request, _display_text) = review_request_from_target(target)?;
        let delivery = delivery.unwrap_or(ApiReviewDelivery::Inline);
        let delivery_core = delivery.to_core();

        match delivery_core {
            CoreReviewDelivery::Inline => {
                let review_prompt = review_prompt_from_request(&review_request);
                let turn_id = Uuid::new_v4().to_string();
                let user_item = ThreadItem::UserMessage {
                    id: format!("user-{turn_id}"),
                    content: vec![V2UserInput::Text {
                        text: review_prompt.clone(),
                        text_elements: Vec::new(),
                    }],
                };

                let start_events = {
                    let mut state = source_thread.state.lock().await;
                    begin_turn(
                        &mut state,
                        source_thread_id,
                        turn_id.clone(),
                        user_item.clone(),
                    )?
                };
                dispatch_turn_events(
                    &outgoing,
                    &thread_state_manager,
                    source_thread_id,
                    start_events,
                )
                .await;

                let thread_for_task = source_thread.clone();
                let turn_id_for_task = turn_id.clone();
                let runtime_messages = vec![InternalMessage::user_text(review_prompt)];
                tokio::spawn(async move {
                    let result = {
                        let mut runtime = thread_for_task.runtime.lock().await;
                        runtime.run_turn(runtime_messages).await
                    };
                    match result {
                        Ok(result) => {
                            let tool_items =
                                dynamic_tool_items(&result.new_history, &result.tool_call_records);
                            let assistant_item =
                                result
                                    .final_message
                                    .clone()
                                    .map(|text| ThreadItem::AgentMessage {
                                        id: format!("assistant-{turn_id_for_task}"),
                                        text,
                                        phase: None,
                                        memory_citation: None,
                                    });
                            {
                                let mut state = thread_for_task.state.lock().await;
                                update_completed_turn(
                                    &mut state,
                                    &turn_id_for_task,
                                    tool_items.clone(),
                                    assistant_item.clone(),
                                    result.final_message.clone(),
                                );
                            }
                            let completed_turn = {
                                let state = thread_for_task.state.lock().await;
                                state
                                    .thread
                                    .turns
                                    .iter()
                                    .find(|turn| turn.id == turn_id_for_task)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        empty_completed_turn(turn_id_for_task.clone())
                                    })
                            };
                            let events = complete_turn_events(
                                source_thread_id,
                                &turn_id_for_task,
                                tool_items,
                                assistant_item,
                                completed_turn,
                            );
                            dispatch_turn_events(
                                &outgoing,
                                &thread_state_manager,
                                source_thread_id,
                                events,
                            )
                            .await;
                        }
                        Err(error) => {
                            let turn_error = TurnError {
                                message: error.to_string(),
                                codex_error_info: None,
                                additional_details: None,
                            };
                            {
                                let mut state = thread_for_task.state.lock().await;
                                update_failed_turn(
                                    &mut state,
                                    &turn_id_for_task,
                                    turn_error.clone(),
                                );
                            }
                            let events = failed_turn_events(
                                source_thread_id,
                                turn_id_for_task.clone(),
                                turn_error,
                            );
                            dispatch_turn_events(
                                &outgoing,
                                &thread_state_manager,
                                source_thread_id,
                                events,
                            )
                            .await;
                        }
                    }
                });

                Ok(ReviewStartResponse {
                    turn: Turn {
                        id: turn_id,
                        items: Vec::new(),
                        status: TurnStatus::InProgress,
                        error: None,
                    },
                    review_thread_id: source_thread_id.to_string(),
                })
            }
            CoreReviewDelivery::Detached => {
                let config_snapshot = thread_config_snapshot(&config);
                let new_thread_id = ThreadId::new();
                let now = now_ts();

                let source_runtime = source_thread.runtime.lock().await;
                let source_history = source_runtime.history().to_vec();
                let source_continuation = source_runtime.continuation().cloned();
                drop(source_runtime);
                let forked_session = OpenRouterBetaSession::forked_from(
                    source_history,
                    source_continuation,
                    session_settings_from_config(&config, &config_snapshot),
                )
                .await
                .map_err(|error| {
                    internal_error(format!("failed to create detached review session: {error}"))
                })?;

                let source_thread_meta = {
                    let state = source_thread.state.lock().await;
                    state.thread.clone()
                };

                let review_thread = Thread {
                    id: new_thread_id.to_string(),
                    forked_from_id: Some(source_thread_id.to_string()),
                    preview: source_thread_meta.preview.clone(),
                    ephemeral: config_snapshot.ephemeral,
                    model_provider: config_snapshot.model_provider_id.clone(),
                    created_at: now,
                    updated_at: now,
                    status: ThreadStatus::Idle,
                    path: (!config_snapshot.ephemeral).then(|| {
                        HistoryFile::new(self.codex_home.as_path(), &new_thread_id.to_string())
                            .path()
                            .to_path_buf()
                    }),
                    cwd: config_snapshot.cwd.clone(),
                    cli_version: env!("CARGO_PKG_VERSION").to_string(),
                    agent_nickname: config_snapshot.session_source.get_nickname(),
                    agent_role: config_snapshot.session_source.get_agent_role(),
                    source: config_snapshot.session_source.clone().into(),
                    git_info: None,
                    name: None,
                    turns: source_thread_meta.turns.clone(),
                };

                let review_or_thread = Arc::new(OpenRouterBetaThread {
                    runtime: Mutex::new(forked_session),
                    state: Mutex::new(OpenRouterThreadState {
                        config_snapshot: config_snapshot.clone(),
                        shell_environment_policy: config
                            .permissions
                            .shell_environment_policy
                            .clone(),
                        thread: review_thread.clone(),
                        lifecycle: TurnLifecycleState::Idle,
                        elicitation_count: 0,
                    }),
                });
                self.persist_thread(review_or_thread.clone())
                    .await
                    .map_err(|error| {
                        internal_error(format!(
                            "failed to persist detached review OpenRouter beta thread: {error}"
                        ))
                    })?;
                self.threads
                    .lock()
                    .await
                    .insert(new_thread_id, review_or_thread.clone());

                let _ = thread_state_manager
                    .try_add_connection_to_thread(new_thread_id, connection_id)
                    .await;

                send_notification(
                    &outgoing,
                    &thread_state_manager,
                    new_thread_id,
                    ServerNotification::ThreadStarted(ThreadStartedNotification {
                        thread: review_thread.clone(),
                    }),
                )
                .await;

                let review_prompt = review_prompt_from_request(&review_request);
                let turn_id = Uuid::new_v4().to_string();
                let user_item = ThreadItem::UserMessage {
                    id: format!("user-{turn_id}"),
                    content: vec![V2UserInput::Text {
                        text: review_prompt.clone(),
                        text_elements: Vec::new(),
                    }],
                };

                let start_events = {
                    let mut state = review_or_thread.state.lock().await;
                    begin_turn(
                        &mut state,
                        new_thread_id,
                        turn_id.clone(),
                        user_item.clone(),
                    )?
                };
                dispatch_turn_events(
                    &outgoing,
                    &thread_state_manager,
                    new_thread_id,
                    start_events,
                )
                .await;

                let review_or_thread_clone = review_or_thread.clone();
                let turn_id_for_task = turn_id.clone();
                let runtime_messages = vec![InternalMessage::user_text(review_prompt)];
                tokio::spawn(async move {
                    let result = {
                        let mut runtime = review_or_thread_clone.runtime.lock().await;
                        runtime.run_turn(runtime_messages).await
                    };
                    match result {
                        Ok(result) => {
                            let tool_items =
                                dynamic_tool_items(&result.new_history, &result.tool_call_records);
                            let assistant_item =
                                result
                                    .final_message
                                    .clone()
                                    .map(|text| ThreadItem::AgentMessage {
                                        id: format!("assistant-{turn_id_for_task}"),
                                        text,
                                        phase: None,
                                        memory_citation: None,
                                    });
                            {
                                let mut state = review_or_thread_clone.state.lock().await;
                                update_completed_turn(
                                    &mut state,
                                    &turn_id_for_task,
                                    tool_items.clone(),
                                    assistant_item.clone(),
                                    result.final_message,
                                );
                            }
                            let completed_turn = {
                                let state = review_or_thread_clone.state.lock().await;
                                state
                                    .thread
                                    .turns
                                    .iter()
                                    .find(|turn| turn.id == turn_id_for_task)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        empty_completed_turn(turn_id_for_task.clone())
                                    })
                            };
                            let events = complete_turn_events(
                                new_thread_id,
                                &turn_id_for_task,
                                tool_items,
                                assistant_item,
                                completed_turn,
                            );
                            dispatch_turn_events(
                                &outgoing,
                                &thread_state_manager,
                                new_thread_id,
                                events,
                            )
                            .await;
                        }
                        Err(error) => {
                            let turn_error = TurnError {
                                message: error.to_string(),
                                codex_error_info: None,
                                additional_details: None,
                            };
                            {
                                let mut state = review_or_thread_clone.state.lock().await;
                                update_failed_turn(
                                    &mut state,
                                    &turn_id_for_task,
                                    turn_error.clone(),
                                );
                            }
                            let events = failed_turn_events(
                                new_thread_id,
                                turn_id_for_task.clone(),
                                turn_error,
                            );
                            dispatch_turn_events(
                                &outgoing,
                                &thread_state_manager,
                                new_thread_id,
                                events,
                            )
                            .await;
                        }
                    }
                });

                Ok(ReviewStartResponse {
                    turn: Turn {
                        id: turn_id,
                        items: Vec::new(),
                        status: TurnStatus::InProgress,
                        error: None,
                    },
                    review_thread_id: new_thread_id.to_string(),
                })
            }
        }
    }
}

fn thread_config_snapshot(config: &Config) -> ThreadConfigSnapshot {
    ThreadConfigSnapshot {
        model: config
            .model
            .clone()
            .unwrap_or_else(|| "openai/o4-mini".to_string()),
        model_provider_id: config.model_provider_id.clone(),
        service_tier: config.service_tier,
        approval_policy: config.permissions.approval_policy.value(),
        approvals_reviewer: config.approvals_reviewer,
        sandbox_policy: config.permissions.sandbox_policy.get().clone(),
        cwd: config.cwd.to_path_buf(),
        ephemeral: config.ephemeral,
        reasoning_effort: config.model_reasoning_effort,
        personality: config.personality,
        session_source: SessionSource::Custom("app_server".to_string()),
    }
}

async fn validate_turn_params(
    thread: &Arc<OpenRouterBetaThread>,
    params: &TurnStartParams,
) -> Result<(), JSONRPCErrorError> {
    let state = thread.state.lock().await;
    let snapshot = &state.config_snapshot;
    let mismatches = [
        params
            .cwd
            .as_ref()
            .filter(|cwd| cwd != &&snapshot.cwd)
            .map(|cwd| format!("cwd={}", cwd.display())),
        params
            .approval_policy
            .filter(|policy| policy.to_core() != snapshot.approval_policy)
            .map(|policy| format!("approvalPolicy={policy:?}")),
        params
            .approvals_reviewer
            .filter(|reviewer| reviewer.to_core() != snapshot.approvals_reviewer)
            .map(|reviewer| format!("approvalsReviewer={reviewer:?}")),
        params
            .sandbox_policy
            .as_ref()
            .filter(|policy| policy.to_core() != snapshot.sandbox_policy)
            .map(|policy| format!("sandboxPolicy={policy:?}")),
        params
            .model
            .as_ref()
            .filter(|model| *model != &snapshot.model)
            .map(|model| format!("model={model}")),
        params
            .service_tier
            .filter(|service_tier| *service_tier != snapshot.service_tier)
            .map(|service_tier| format!("serviceTier={service_tier:?}")),
        params
            .effort
            .filter(|effort| Some(*effort) != snapshot.reasoning_effort)
            .map(|effort| format!("effort={effort:?}")),
        params
            .summary
            .as_ref()
            .map(|summary| format!("summary={summary:?}")),
        params
            .personality
            .filter(|personality| Some(*personality) != snapshot.personality)
            .map(|personality| format!("personality={personality:?}")),
        params
            .collaboration_mode
            .as_ref()
            .map(|_| "collaborationMode".to_string()),
        params
            .output_schema
            .as_ref()
            .map(|_| "outputSchema".to_string()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(invalid_request(format!(
            "OpenRouter beta app-server path does not support turn-level overrides yet: {}",
            mismatches.join(", ")
        )))
    }
}

fn runtime_messages_from_input(
    input: &[V2UserInput],
) -> Result<Vec<InternalMessage>, JSONRPCErrorError> {
    input
        .iter()
        .map(|item| match item {
            V2UserInput::Text { text, .. } => Ok(InternalMessage::user_text(text.clone())),
            _ => Err(invalid_request(
                "OpenRouter beta app-server path currently supports only text input items"
                    .to_string(),
            )),
        })
        .collect()
}

fn dynamic_tool_items(
    new_history: &[InternalMessage],
    tool_call_records: &[ToolCallRecord],
) -> Vec<ThreadItem> {
    let arguments_by_call_id = new_history
        .iter()
        .filter_map(|message| match message {
            InternalMessage::AssistantToolCall {
                call_id, arguments, ..
            } => Some((call_id.clone(), arguments.clone())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    tool_call_records
        .iter()
        .map(|record| ThreadItem::DynamicToolCall {
            id: record.call_id.clone(),
            tool: record.tool_name.clone(),
            arguments: arguments_by_call_id
                .get(&record.call_id)
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            status: if record.result_disposition == ToolCallResultDisposition::Success {
                DynamicToolCallStatus::Completed
            } else {
                DynamicToolCallStatus::Failed
            },
            content_items: None,
            success: Some(record.result_disposition == ToolCallResultDisposition::Success),
            duration_ms: None,
        })
        .collect()
}

fn in_progress_tool_item(item: &ThreadItem) -> ThreadItem {
    match item {
        ThreadItem::DynamicToolCall {
            id,
            tool,
            arguments,
            ..
        } => ThreadItem::DynamicToolCall {
            id: id.clone(),
            tool: tool.clone(),
            arguments: arguments.clone(),
            status: DynamicToolCallStatus::InProgress,
            content_items: None,
            success: None,
            duration_ms: None,
        },
        other => other.clone(),
    }
}

fn update_completed_turn(
    state: &mut OpenRouterThreadState,
    turn_id: &str,
    tool_items: Vec<ThreadItem>,
    assistant_item: Option<ThreadItem>,
    final_message: Option<String>,
) {
    if let Some(turn) = state
        .thread
        .turns
        .iter_mut()
        .find(|turn| turn.id == turn_id)
    {
        turn.items.extend(tool_items);
        if let Some(item) = assistant_item {
            turn.items.push(item);
        }
        turn.status = TurnStatus::Completed;
        turn.error = None;
    }
    state.lifecycle = TurnLifecycleState::Idle;
    state.thread.updated_at = now_ts();
    state.thread.status = ThreadStatus::Idle;
    if let Some(final_message) = final_message {
        state.thread.preview = final_message;
    }
}

fn update_failed_turn(state: &mut OpenRouterThreadState, turn_id: &str, error: TurnError) {
    if let Some(turn) = state
        .thread
        .turns
        .iter_mut()
        .find(|turn| turn.id == turn_id)
    {
        turn.status = TurnStatus::Failed;
        turn.error = Some(error);
    }
    state.lifecycle = TurnLifecycleState::Idle;
    state.thread.updated_at = now_ts();
    state.thread.status = ThreadStatus::SystemError;
}

fn begin_turn(
    state: &mut OpenRouterThreadState,
    thread_id: ThreadId,
    turn_id: String,
    user_item: ThreadItem,
) -> Result<Vec<TurnEvent>, JSONRPCErrorError> {
    if state.lifecycle.is_running() {
        return Err(invalid_request(format!(
            "thread {thread_id} already has an active OpenRouter beta turn"
        )));
    }
    state.lifecycle = TurnLifecycleState::Running {
        turn_id: turn_id.clone(),
    };
    state.thread.updated_at = now_ts();
    state.thread.status = ThreadStatus::Active {
        active_flags: Vec::new(),
    };
    state.thread.turns.push(Turn {
        id: turn_id.clone(),
        items: vec![user_item.clone()],
        status: TurnStatus::InProgress,
        error: None,
    });

    Ok(vec![
        TurnEvent::TurnStarted {
            thread_id,
            turn: Turn {
                id: turn_id.clone(),
                items: Vec::new(),
                status: TurnStatus::InProgress,
                error: None,
            },
        },
        TurnEvent::ItemStarted {
            thread_id,
            turn_id: turn_id.clone(),
            item: user_item.clone(),
        },
        TurnEvent::ItemCompleted {
            thread_id,
            turn_id,
            item: user_item,
        },
    ])
}

fn empty_completed_turn(turn_id: String) -> Turn {
    Turn {
        id: turn_id,
        items: Vec::new(),
        status: TurnStatus::Completed,
        error: None,
    }
}

fn complete_turn_events(
    thread_id: ThreadId,
    turn_id: &str,
    tool_items: Vec<ThreadItem>,
    assistant_item: Option<ThreadItem>,
    completed_turn: Turn,
) -> Vec<TurnEvent> {
    let mut events = Vec::new();
    for tool_item in tool_items {
        events.push(TurnEvent::ItemStarted {
            thread_id,
            turn_id: turn_id.to_string(),
            item: in_progress_tool_item(&tool_item),
        });
        events.push(TurnEvent::ItemCompleted {
            thread_id,
            turn_id: turn_id.to_string(),
            item: tool_item,
        });
    }
    if let Some(item) = assistant_item {
        events.push(TurnEvent::ItemStarted {
            thread_id,
            turn_id: turn_id.to_string(),
            item: started_item(&item),
        });
        events.push(TurnEvent::ItemCompleted {
            thread_id,
            turn_id: turn_id.to_string(),
            item,
        });
    }
    events.push(TurnEvent::TurnCompleted {
        thread_id,
        turn: completed_turn,
    });
    events
}

fn failed_turn_events(
    thread_id: ThreadId,
    turn_id: String,
    turn_error: TurnError,
) -> Vec<TurnEvent> {
    vec![
        TurnEvent::Error {
            thread_id,
            turn_id: turn_id.clone(),
            error: turn_error.clone(),
        },
        TurnEvent::TurnCompleted {
            thread_id,
            turn: Turn {
                id: turn_id,
                items: Vec::new(),
                status: TurnStatus::Failed,
                error: Some(turn_error),
            },
        },
    ]
}

fn started_item(item: &ThreadItem) -> ThreadItem {
    match item {
        ThreadItem::AgentMessage { id, .. } => ThreadItem::AgentMessage {
            id: id.clone(),
            text: String::new(),
            phase: None,
            memory_citation: None,
        },
        _ => item.clone(),
    }
}

fn notification_for_turn_event(event: TurnEvent) -> ServerNotification {
    match event {
        TurnEvent::TurnStarted { thread_id, turn } => {
            ServerNotification::TurnStarted(TurnStartedNotification {
                thread_id: thread_id.to_string(),
                turn,
            })
        }
        TurnEvent::ItemStarted {
            thread_id,
            turn_id,
            item,
        } => ServerNotification::ItemStarted(ItemStartedNotification {
            item,
            thread_id: thread_id.to_string(),
            turn_id,
        }),
        TurnEvent::ItemCompleted {
            thread_id,
            turn_id,
            item,
        } => ServerNotification::ItemCompleted(ItemCompletedNotification {
            item,
            thread_id: thread_id.to_string(),
            turn_id,
        }),
        TurnEvent::Error {
            thread_id,
            turn_id,
            error,
        } => ServerNotification::Error(ErrorNotification {
            error,
            will_retry: false,
            thread_id: thread_id.to_string(),
            turn_id,
        }),
        TurnEvent::TurnCompleted { thread_id, turn } => {
            ServerNotification::TurnCompleted(TurnCompletedNotification {
                thread_id: thread_id.to_string(),
                turn,
            })
        }
    }
}

async fn dispatch_turn_events(
    outgoing: &OutgoingMessageSender,
    thread_state_manager: &ThreadStateManager,
    thread_id: ThreadId,
    events: Vec<TurnEvent>,
) {
    for event in events {
        send_notification(
            outgoing,
            thread_state_manager,
            thread_id,
            notification_for_turn_event(event),
        )
        .await;
    }
}

async fn send_notification(
    outgoing: &OutgoingMessageSender,
    thread_state_manager: &ThreadStateManager,
    thread_id: ThreadId,
    notification: ServerNotification,
) {
    let connection_ids = thread_state_manager
        .subscribed_connection_ids(thread_id)
        .await;
    if connection_ids.is_empty() {
        return;
    }
    outgoing
        .send_server_notification_to_connections(connection_ids.as_slice(), notification)
        .await;
}

fn invalid_request(message: String) -> JSONRPCErrorError {
    JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message,
        data: None,
    }
}

fn internal_error(message: String) -> JSONRPCErrorError {
    JSONRPCErrorError {
        code: INTERNAL_ERROR_CODE,
        message,
        data: None,
    }
}

fn now_ts() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

pub(crate) fn openrouter_thread_operation_unsupported(operation: &str) -> JSONRPCErrorError {
    invalid_request(unsupported_operation_message("app-server", operation))
}

fn session_settings_from_config(
    config: &Config,
    snapshot: &ThreadConfigSnapshot,
) -> OpenRouterBetaSettings {
    use logos_openrouter_beta::transport_policy_from_provider;

    let transport_policy = transport_policy_from_provider(&config.model_provider)
        .expect("transport policy should be valid for a known OpenRouter thread");
    OpenRouterBetaSettings {
        api_key: config
            .model_provider
            .api_key()
            .ok()
            .flatten()
            .or_else(|| config.model_provider.experimental_bearer_token.clone())
            .unwrap_or_default(),
        base_url: config.model_provider.base_url.clone().unwrap_or_default(),
        model_id: snapshot.model.clone(),
        workspace_root: snapshot.cwd.clone(),
        system_prompt: logos_openrouter_beta::system_prompt_from_config(config),
        transport_policy,
    }
}

fn session_settings_from_persisted(
    config: &Config,
    snapshot: &ThreadConfigSnapshot,
) -> OpenRouterBetaSettings {
    let provider = config
        .model_providers
        .get(&snapshot.model_provider_id)
        .cloned()
        .unwrap_or_else(|| config.model_provider.clone());
    let transport_policy = logos_openrouter_beta::transport_policy_from_provider(&provider)
        .expect("transport policy should be valid for a persisted OpenRouter thread");
    OpenRouterBetaSettings {
        api_key: provider
            .api_key()
            .ok()
            .flatten()
            .or_else(|| provider.experimental_bearer_token.clone())
            .unwrap_or_default(),
        base_url: provider.base_url.unwrap_or_default(),
        model_id: snapshot.model.clone(),
        workspace_root: snapshot.cwd.clone(),
        system_prompt: logos_openrouter_beta::system_prompt_from_config(config),
        transport_policy,
    }
}

fn persisted_thread_to_thread(
    mut persisted: PersistedOpenRouterThread,
    include_turns: bool,
) -> Thread {
    if !include_turns {
        persisted.thread.turns.clear();
    }
    persisted.thread.status = ThreadStatus::NotLoaded;
    persisted.thread
}

fn thread_from_record(record: ThreadRecord) -> Result<Thread, JSONRPCErrorError> {
    let source =
        serde_json::from_str::<ApiSessionSource>(&record.source_json).map_err(|error| {
            internal_error(format!(
                "failed to deserialize OpenRouter beta thread {} source metadata: {error}",
                record.thread_id
            ))
        })?;

    Ok(Thread {
        id: record.thread_id.to_string(),
        forked_from_id: record.forked_from_id,
        preview: record.preview.unwrap_or_default(),
        ephemeral: record.ephemeral,
        model_provider: record.provider_id,
        created_at: record.created_at,
        updated_at: record.updated_at,
        status: ThreadStatus::NotLoaded,
        path: record.session_state_path,
        cwd: record.cwd,
        cli_version: record.cli_version,
        source,
        agent_nickname: record.agent_nickname,
        agent_role: record.agent_role,
        git_info: Some(GitInfo {
            sha: record.git_commit,
            branch: record.git_branch,
            origin_url: record.git_origin_url,
        }),
        name: record.name,
        turns: Vec::new(),
    })
}

fn thread_record_input_from_persisted(
    persisted: PersistedOpenRouterThread,
    archived: bool,
) -> anyhow::Result<ThreadRecordInput> {
    let archived_at = archived.then_some(persisted.thread.updated_at);
    let thread_id = ThreadId::from_string(&persisted.thread.id)?;
    Ok(ThreadRecordInput {
        provider_lane: ProviderLane::OpenRouterBeta,
        provider_id: persisted.config_snapshot.model_provider_id.clone(),
        forked_from_id: persisted.thread.forked_from_id.clone(),
        name: persisted.thread.name.clone(),
        archived_at,
        status: "active".to_string(),
        config_snapshot_ref: Some(thread_id.to_string()),
        session_state_ref: persisted
            .thread
            .path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned()),
        created_at: persisted.thread.created_at,
        updated_at: persisted.thread.updated_at,
        session_state_path: persisted.thread.path.clone(),
        config_snapshot_json: serde_json::to_string(&persisted.config_snapshot)?,
        cwd: persisted.thread.cwd.clone(),
        ephemeral: persisted.thread.ephemeral,
        cli_version: persisted.thread.cli_version.clone(),
        source_json: serde_json::to_string(&persisted.thread.source)?,
        agent_nickname: persisted.thread.agent_nickname.clone(),
        agent_role: persisted.thread.agent_role.clone(),
        git_branch: persisted
            .thread
            .git_info
            .as_ref()
            .and_then(|g| g.branch.clone()),
        git_commit: persisted
            .thread
            .git_info
            .as_ref()
            .and_then(|g| g.sha.clone()),
        git_origin_url: persisted
            .thread
            .git_info
            .as_ref()
            .and_then(|g| g.origin_url.clone()),
        preview: Some(persisted.thread.preview.clone()),
        thread_id,
    })
}

fn apply_git_info_patch(
    current: GitInfo,
    sha_patch: Option<Option<String>>,
    branch_patch: Option<Option<String>>,
    origin_url_patch: Option<Option<String>>,
) -> GitInfo {
    fn patch_field(current: Option<String>, incoming: Option<Option<String>>) -> Option<String> {
        match incoming {
            Some(Some(v)) => Some(v.trim().to_string()),
            Some(None) => None,
            None => current,
        }
    }
    GitInfo {
        sha: patch_field(current.sha, sha_patch),
        branch: patch_field(current.branch, branch_patch),
        origin_url: patch_field(current.origin_url, origin_url_patch),
    }
}

fn build_history_entries(
    history: &[InternalMessage],
    continuation: &Option<ContinuationHandle>,
    elicitation_count: u64,
) -> Vec<HistoryEntry> {
    let mut entries = Vec::with_capacity(history.len() + 2);
    for (i, msg) in history.iter().enumerate() {
        entries.push(HistoryEntry::Message {
            seq: i as u64 + 1,
            data: msg.clone(),
        });
    }
    let mut seq = history.len() as u64 + 1;
    entries.push(HistoryEntry::Continuation {
        seq,
        data: continuation.clone(),
    });
    seq += 1;
    entries.push(HistoryEntry::Elicitation {
        seq,
        count: elicitation_count,
    });
    entries
}

struct OpenRouterHostExecSink {
    outgoing: Arc<OutgoingMessageSender>,
    thread_state_manager: ThreadStateManager,
    thread_id: ThreadId,
    turn_id: String,
}

#[async_trait]
impl HostExecNotificationSink for OpenRouterHostExecSink {
    async fn emit_begin(&self, event: HostExecBegin) -> anyhow::Result<()> {
        send_notification(
            &self.outgoing,
            &self.thread_state_manager,
            self.thread_id,
            ServerNotification::ItemStarted(ItemStartedNotification {
                thread_id: self.thread_id.to_string(),
                turn_id: event.turn_id,
                item: ThreadItem::CommandExecution {
                    id: event.call_id,
                    command: shlex_join(&event.command),
                    cwd: event.cwd,
                    process_id: event.process_id,
                    source: CommandExecutionSource::UserShell,
                    status: CommandExecutionStatus::InProgress,
                    command_actions: event.parsed_cmd.into_iter().map(Into::into).collect(),
                    aggregated_output: None,
                    exit_code: None,
                    duration_ms: None,
                },
            }),
        )
        .await;
        Ok(())
    }

    async fn emit_delta(&self, event: ExecCommandOutputDeltaEvent) -> anyhow::Result<()> {
        send_notification(
            &self.outgoing,
            &self.thread_state_manager,
            self.thread_id,
            ServerNotification::CommandExecutionOutputDelta(
                CommandExecutionOutputDeltaNotification {
                    thread_id: self.thread_id.to_string(),
                    turn_id: self.turn_id.clone(),
                    item_id: event.call_id,
                    delta: String::from_utf8_lossy(&event.chunk).to_string(),
                },
            ),
        )
        .await;
        Ok(())
    }

    async fn emit_end(&self, event: HostExecEnd) -> anyhow::Result<()> {
        send_notification(
            &self.outgoing,
            &self.thread_state_manager,
            self.thread_id,
            ServerNotification::ItemCompleted(ItemCompletedNotification {
                thread_id: self.thread_id.to_string(),
                turn_id: event.turn_id,
                item: openrouter_command_execution_item(HostExecOutcome {
                    call_id: event.call_id,
                    process_id: event.process_id,
                    turn_id: String::new(),
                    command: event.command,
                    cwd: event.cwd,
                    parsed_cmd: event.parsed_cmd,
                    source: event.source,
                    output: event.output,
                    status: event.status,
                }),
            }),
        )
        .await;
        Ok(())
    }

    async fn emit_terminal_interaction(
        &self,
        event: codex_core::host_exec::HostExecTerminalInteraction,
    ) -> anyhow::Result<()> {
        send_notification(
            &self.outgoing,
            &self.thread_state_manager,
            self.thread_id,
            ServerNotification::TerminalInteraction(TerminalInteractionNotification {
                thread_id: self.thread_id.to_string(),
                turn_id: self.turn_id.clone(),
                item_id: event.call_id,
                process_id: event.process_id.unwrap_or_default(),
                stdin: event.stdin,
            }),
        )
        .await;
        Ok(())
    }
}

fn openrouter_command_execution_item(outcome: HostExecOutcome) -> ThreadItem {
    ThreadItem::CommandExecution {
        id: outcome.call_id,
        command: shlex_join(&outcome.command),
        cwd: outcome.cwd,
        process_id: outcome.process_id,
        source: CommandExecutionSource::UserShell,
        status: match outcome.status {
            CoreExecCommandStatus::Completed => CommandExecutionStatus::Completed,
            CoreExecCommandStatus::Declined => CommandExecutionStatus::Failed,
            CoreExecCommandStatus::Failed => CommandExecutionStatus::Failed,
        },
        command_actions: outcome.parsed_cmd.into_iter().map(Into::into).collect(),
        aggregated_output: (!outcome.output.aggregated_output.text.is_empty())
            .then_some(outcome.output.aggregated_output.text),
        exit_code: Some(outcome.output.exit_code),
        duration_ms: Some(i64::try_from(outcome.output.duration.as_millis()).unwrap_or(i64::MAX)),
    }
}

fn review_request_from_target(
    target: ApiReviewTarget,
) -> Result<(ReviewRequest, String), JSONRPCErrorError> {
    let core_target = match target {
        ApiReviewTarget::UncommittedChanges => CoreReviewTarget::UncommittedChanges,
        ApiReviewTarget::BaseBranch { branch } => {
            let branch = branch.trim().to_string();
            if branch.is_empty() {
                return Err(invalid_request("branch must not be empty".to_string()));
            }
            CoreReviewTarget::BaseBranch { branch }
        }
        ApiReviewTarget::Commit { sha, title } => {
            let sha = sha.trim().to_string();
            if sha.is_empty() {
                return Err(invalid_request("sha must not be empty".to_string()));
            }
            let title = title
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty());
            CoreReviewTarget::Commit { sha, title }
        }
        ApiReviewTarget::Custom { instructions } => {
            let trimmed = instructions.trim().to_string();
            if trimmed.is_empty() {
                return Err(invalid_request(
                    "instructions must not be empty".to_string(),
                ));
            }
            CoreReviewTarget::Custom {
                instructions: trimmed,
            }
        }
    };
    let hint = codex_core::review_prompts::user_facing_hint(&core_target);
    let review_request = ReviewRequest {
        target: core_target,
        user_facing_hint: Some(hint.clone()),
    };
    Ok((review_request, hint))
}

fn review_prompt_from_request(review_request: &ReviewRequest) -> String {
    let hint = review_request
        .user_facing_hint
        .as_deref()
        .unwrap_or("Review the current state");
    format!("{hint}\n\nPlease review and provide your analysis.")
}

pub(crate) fn validate_openrouter_resume_params(
    params: &ThreadResumeParams,
) -> Result<(), JSONRPCErrorError> {
    if params.history.is_some() {
        return Err(invalid_request(
            "OpenRouter beta thread/resume does not support the `history` mode. \
             OpenRouter beta threads are in-memory only; use `thread_id` to reconnect to a live thread.".to_string(),
        ));
    }
    if params.path.is_some() {
        return Err(invalid_request(
            "OpenRouter beta thread/resume does not support the `path` mode. \
             OpenRouter beta threads are in-memory only; use `thread_id` to reconnect to a live thread.".to_string(),
        ));
    }
    let unsupported_overrides: Vec<&str> = [
        params.model.as_ref().map(|_| "model"),
        params.model_provider.as_ref().map(|_| "modelProvider"),
        params.service_tier.map(|_| "serviceTier"),
        params.cwd.as_ref().map(|_| "cwd"),
        params.approval_policy.map(|_| "approvalPolicy"),
        params.approvals_reviewer.map(|_| "approvalsReviewer"),
        params.sandbox.as_ref().map(|_| "sandbox"),
        params.config.as_ref().map(|_| "config"),
        params
            .base_instructions
            .as_ref()
            .map(|_| "baseInstructions"),
        params
            .developer_instructions
            .as_ref()
            .map(|_| "developerInstructions"),
        params.personality.map(|_| "personality"),
        params
            .persist_extended_history
            .then_some("persistExtendedHistory"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !unsupported_overrides.is_empty() {
        return Err(invalid_request(format!(
            "OpenRouter beta thread/resume does not support config overrides in this pass: {}",
            unsupported_overrides.join(", ")
        )));
    }
    Ok(())
}

pub(crate) fn validate_openrouter_fork_params(
    params: &ThreadForkParams,
) -> Result<(), JSONRPCErrorError> {
    if params.path.is_some() {
        return Err(invalid_request(
            "OpenRouter beta thread/fork does not support the `path` mode. \
             OpenRouter beta threads are in-memory only; fork by `thread_id` from a live thread."
                .to_string(),
        ));
    }
    let unsupported_overrides: Vec<&str> = [
        params.model.as_ref().map(|_| "model"),
        params.model_provider.as_ref().map(|_| "modelProvider"),
        params.service_tier.map(|_| "serviceTier"),
        params.cwd.as_ref().map(|_| "cwd"),
        params.approval_policy.map(|_| "approvalPolicy"),
        params.approvals_reviewer.map(|_| "approvalsReviewer"),
        params.sandbox.as_ref().map(|_| "sandbox"),
        params.config.as_ref().map(|_| "config"),
        params
            .base_instructions
            .as_ref()
            .map(|_| "baseInstructions"),
        params
            .developer_instructions
            .as_ref()
            .map(|_| "developerInstructions"),
        params.ephemeral.then_some("ephemeral"),
        params
            .persist_extended_history
            .then_some("persistExtendedHistory"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !unsupported_overrides.is_empty() {
        return Err(invalid_request(format!(
            "OpenRouter beta thread/fork does not support config overrides in this pass: {}",
            unsupported_overrides.join(", ")
        )));
    }
    Ok(())
}
