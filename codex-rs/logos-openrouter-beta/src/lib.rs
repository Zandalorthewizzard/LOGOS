use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use codex_core::config::Config;
use codex_login::default_client::build_reqwest_client;
use codex_login::default_client::default_headers as codex_default_headers;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OPENROUTER_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_VALIDATED_MODEL_ID;
use codex_model_provider_info::is_openrouter_provider_id;
use logos_provider_openrouter::OpenRouterAdapter;
use logos_provider_openrouter::OpenRouterAdapterConfig;
use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::ToolExecutionKind;
use logos_session_engine::SessionEngine;
use logos_tool_runtime::PendingToolCall;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolExecutionOutput;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use serde_json::Value;
use serde_json::json;

#[cfg(test)]
pub const OPENROUTER_CUSTOM_PROVIDER_PREFIX: &str = "openrouter-";
const DEFAULT_READ_FILE_MAX_BYTES: usize = 8 * 1024;
const DEFAULT_LIST_DIR_MAX_ENTRIES: usize = 200;

pub fn uses_openrouter_beta_lane(config: &Config) -> bool {
    is_openrouter_provider_id(&config.model_provider_id)
}

pub fn openrouter_beta_tuning_note() -> &'static str {
    "Built-in `openrouter` stays immutable. To tune the OpenRouter beta path, define and select a custom provider id such as `openrouter-tuned`. The current shared non-websocket lane honors `request_max_retries` and `stream_idle_timeout_ms`; it still ignores `stream_max_retries` and `websocket_connect_timeout_ms` until a streamed transport exists."
}

pub fn unsupported_operation_message(surface: &str, operation: &str) -> String {
    format!(
        "OpenRouter beta {surface} path does not support {operation} yet. The shared lane remains replay-first and does not provide fake `previous_response_id` continuation or compaction semantics."
    )
}

pub struct OpenRouterBetaSession {
    session: SessionEngine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRouterTransportPolicy {
    pub default_headers: HeaderMap,
    pub query_params: Vec<(String, String)>,
    pub request_max_retries: u64,
    pub request_timeout: Duration,
    pub ignored_knobs: Vec<&'static str>,
}

#[derive(Debug)]
pub struct OpenRouterBetaTurnResult {
    pub new_history: Vec<InternalMessage>,
    pub tool_call_records: Vec<ToolCallRecord>,
    pub final_message: Option<String>,
}

impl OpenRouterBetaSession {
    pub async fn from_config(config: &Config) -> anyhow::Result<Self> {
        let api_key = config
            .model_provider
            .api_key()?
            .or_else(|| config.model_provider.experimental_bearer_token.clone())
            .ok_or_else(|| anyhow::anyhow!("OpenRouter beta path requires an API token"))?;
        let base_url =
            config.model_provider.base_url.clone().ok_or_else(|| {
                anyhow::anyhow!("OpenRouter beta path requires a provider base_url")
            })?;
        let model_id = config
            .model
            .clone()
            .unwrap_or_else(|| OPENROUTER_VALIDATED_MODEL_ID.to_string());
        let workspace_root = config
            .cwd
            .canonicalize()
            .with_context(|| format!("failed to canonicalize cwd {}", config.cwd.display()))?;
        let transport_policy = transport_policy_from_provider(&config.model_provider)?;

        Self::new(OpenRouterBetaSettings {
            api_key,
            base_url,
            model_id,
            workspace_root,
            system_prompt: system_prompt_from_config(config),
            transport_policy,
        })
        .await
    }

    pub async fn new(settings: OpenRouterBetaSettings) -> anyhow::Result<Self> {
        let adapter = Arc::new(OpenRouterAdapter::new(OpenRouterAdapterConfig {
            client: build_reqwest_client(),
            base_url: settings.base_url,
            api_key: settings.api_key,
            model_ref: ModelRef {
                provider_id: OPENROUTER_PROVIDER_ID.to_string(),
                model_id: settings.model_id.clone(),
            },
            default_headers: settings.transport_policy.default_headers.clone(),
            query_params: settings.transport_policy.query_params.clone(),
            request_max_retries: settings.transport_policy.request_max_retries,
            request_timeout: settings.transport_policy.request_timeout,
        }));
        let mut session = SessionEngine::new(
            adapter,
            ModelRef {
                provider_id: OPENROUTER_PROVIDER_ID.to_string(),
                model_id: settings.model_id,
            },
            settings.system_prompt,
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;

        session.register_runtime_tool(
            workspace_read_file_tool_def(),
            Arc::new(WorkspaceReadFileTool {
                workspace_root: settings.workspace_root.clone(),
            }),
        );
        session.register_runtime_tool(
            workspace_list_dir_tool_def(),
            Arc::new(WorkspaceListDirTool {
                workspace_root: settings.workspace_root,
            }),
        );

        Ok(Self { session })
    }

    pub async fn run_turn(
        &mut self,
        new_messages: Vec<InternalMessage>,
    ) -> anyhow::Result<OpenRouterBetaTurnResult> {
        let history_len = self.session.history().len();
        let tool_call_record_len = self.session.tool_call_records().len();
        self.session
            .run_turn(new_messages)
            .await
            .map_err(|error| anyhow::anyhow!(error.message))?;

        let new_history = self.session.history()[history_len..].to_vec();
        let tool_call_records = self.session.tool_call_records()[tool_call_record_len..].to_vec();
        let final_message = last_assistant_message(&new_history)
            .or_else(|| last_assistant_message(self.session.history()));

        Ok(OpenRouterBetaTurnResult {
            new_history,
            tool_call_records,
            final_message,
        })
    }

    pub fn history(&self) -> &[InternalMessage] {
        self.session.history()
    }

    pub fn continuation(&self) -> Option<&ContinuationHandle> {
        self.session.continuation()
    }

    pub async fn forked_from(
        source_history: Vec<InternalMessage>,
        source_continuation: Option<ContinuationHandle>,
        settings: OpenRouterBetaSettings,
    ) -> anyhow::Result<Self> {
        let mut new_session = Self::new(settings).await?;
        new_session
            .session
            .seed_history(source_history, source_continuation);
        Ok(new_session)
    }
}

pub struct OpenRouterBetaSettings {
    pub api_key: String,
    pub base_url: String,
    pub model_id: String,
    pub workspace_root: PathBuf,
    pub system_prompt: Option<String>,
    pub transport_policy: OpenRouterTransportPolicy,
}

pub fn transport_policy_from_provider(
    provider: &ModelProviderInfo,
) -> anyhow::Result<OpenRouterTransportPolicy> {
    Ok(OpenRouterTransportPolicy {
        default_headers: provider_default_headers(provider)?,
        query_params: provider
            .query_params
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect(),
        request_max_retries: provider.request_max_retries(),
        request_timeout: provider.stream_idle_timeout(),
        ignored_knobs: ignored_transport_knobs(provider),
    })
}

fn provider_default_headers(provider: &ModelProviderInfo) -> anyhow::Result<HeaderMap> {
    let mut headers = codex_default_headers();
    if let Some(extra) = provider.http_headers.as_ref() {
        for (name, value) in extra {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(name, value);
            }
        }
    }
    if let Some(env_headers) = provider.env_http_headers.as_ref() {
        for (name, env_var) in env_headers {
            if let Ok(value) = std::env::var(env_var)
                && !value.trim().is_empty()
                && let (Ok(name), Ok(value)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(&value),
                )
            {
                headers.insert(name, value);
            }
        }
    }
    Ok(headers)
}

fn ignored_transport_knobs(provider: &ModelProviderInfo) -> Vec<&'static str> {
    let mut ignored = Vec::new();
    if provider.stream_max_retries.is_some() {
        ignored.push("stream_max_retries");
    }
    if provider.websocket_connect_timeout_ms.is_some() {
        ignored.push("websocket_connect_timeout_ms");
    }
    ignored
}

pub fn system_prompt_from_config(config: &Config) -> Option<String> {
    let mut sections = Vec::new();
    if let Some(base) = config.base_instructions.as_ref().map(|value| value.trim())
        && !base.is_empty()
    {
        sections.push(base.to_string());
    }
    if let Some(developer) = config
        .developer_instructions
        .as_ref()
        .map(|value| value.trim())
        && !developer.is_empty()
    {
        sections.push(developer.to_string());
    }
    if let Some(user) = config.user_instructions.as_ref().map(|value| value.trim())
        && !user.is_empty()
    {
        sections.push(user.to_string());
    }
    (!sections.is_empty()).then(|| sections.join("\n\n"))
}

pub fn last_assistant_message(history: &[InternalMessage]) -> Option<String> {
    history.iter().rev().find_map(|message| match message {
        InternalMessage::Assistant { parts, .. } => Some(
            parts
                .iter()
                .filter_map(|part| match part {
                    ContentPart::Text { text } => Some(text.as_str()),
                    ContentPart::ImageRef { .. } | ContentPart::FileRef { .. } => None,
                })
                .collect::<String>(),
        ),
        _ => None,
    })
}

pub fn workspace_read_file_tool_def() -> InternalToolDef {
    InternalToolDef {
        name: "workspace_read_file".to_string(),
        description: "Read a text file from the current workspace root.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "max_bytes": {"type": "integer", "minimum": 1}
            },
            "required": ["path"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"},
                "truncated": {"type": "boolean"}
            },
            "required": ["path", "content", "truncated"]
        })),
        execution: ToolExecutionKind::RuntimeOwned,
        metadata: Default::default(),
    }
}

pub fn workspace_list_dir_tool_def() -> InternalToolDef {
    InternalToolDef {
        name: "workspace_list_dir".to_string(),
        description: "List entries from a directory under the current workspace root.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            }
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "entries": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "kind": {"type": "string"}
                        },
                        "required": ["name", "kind"]
                    }
                }
            },
            "required": ["path", "entries"]
        })),
        execution: ToolExecutionKind::RuntimeOwned,
        metadata: Default::default(),
    }
}

struct WorkspaceReadFileTool {
    workspace_root: PathBuf,
}

#[async_trait::async_trait]
impl RuntimeToolExecutor for WorkspaceReadFileTool {
    async fn execute(&self, call: &PendingToolCall) -> Result<ToolExecutionOutput, String> {
        let arguments = call.arguments();
        let path = arguments
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "workspace_read_file requires `path`".to_string())?;
        let max_bytes = arguments
            .get("max_bytes")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_READ_FILE_MAX_BYTES);
        let target = resolve_workspace_path(&self.workspace_root, path)?;
        let bytes = std::fs::read(&target)
            .map_err(|error| format!("failed to read {}: {error}", target.display()))?;
        let truncated = bytes.len() > max_bytes;
        let content = String::from_utf8_lossy(&bytes[..bytes.len().min(max_bytes)]).to_string();

        Ok(ToolExecutionOutput::success(json!({
            "path": relative_or_absolute(&self.workspace_root, &target),
            "content": content,
            "truncated": truncated
        })))
    }
}

struct WorkspaceListDirTool {
    workspace_root: PathBuf,
}

#[async_trait::async_trait]
impl RuntimeToolExecutor for WorkspaceListDirTool {
    async fn execute(&self, call: &PendingToolCall) -> Result<ToolExecutionOutput, String> {
        let arguments = call.arguments();
        let path = arguments.get("path").and_then(Value::as_str).unwrap_or(".");
        let target = resolve_workspace_path(&self.workspace_root, path)?;
        let entries = std::fs::read_dir(&target)
            .map_err(|error| format!("failed to list {}: {error}", target.display()))?
            .flatten()
            .take(DEFAULT_LIST_DIR_MAX_ENTRIES)
            .map(|entry| {
                let kind = match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => "dir",
                    Ok(file_type) if file_type.is_file() => "file",
                    Ok(_) => "other",
                    Err(_) => "unknown",
                };
                json!({
                    "name": entry.file_name().to_string_lossy().to_string(),
                    "kind": kind,
                })
            })
            .collect::<Vec<_>>();

        Ok(ToolExecutionOutput::success(json!({
            "path": relative_or_absolute(&self.workspace_root, &target),
            "entries": entries
        })))
    }
}

fn resolve_workspace_path(workspace_root: &Path, requested_path: &str) -> Result<PathBuf, String> {
    let candidate = if requested_path.is_empty() {
        workspace_root.to_path_buf()
    } else {
        workspace_root.join(requested_path)
    };
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", candidate.display()))?;
    if canonical.starts_with(workspace_root) {
        Ok(canonical)
    } else {
        Err(format!(
            "path {} escapes workspace root {}",
            canonical.display(),
            workspace_root.display()
        ))
    }
}

fn relative_or_absolute(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .map(|value| value.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::OPENROUTER_CUSTOM_PROVIDER_PREFIX;
    use super::OpenRouterBetaSession;
    use super::openrouter_beta_tuning_note;
    use super::transport_policy_from_provider;
    use super::uses_openrouter_beta_lane;
    use codex_core::config::Config;
    use codex_core::config::ConfigBuilder;
    use codex_model_provider_info::ModelProviderInfo;
    use codex_model_provider_info::WireApi;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use logos_runtime_types::InternalMessage;
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use serde_json::json;
    use serial_test::serial;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use tempfile::TempDir;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::Request;
    use wiremock::Respond;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[tokio::test]
    async fn custom_openrouter_provider_id_uses_beta_lane_and_completes_tool_roundtrip() {
        let server = MockServer::start().await;
        let workspace = TempDir::new().expect("workspace tempdir");
        std::fs::write(workspace.path().join("note.txt"), "hello from workspace")
            .expect("write workspace file");

        Mock::given(method("POST"))
            .and(path("/api/v1/responses"))
            .respond_with(SequencedOpenRouterResponder::default())
            .expect(2)
            .mount(&server)
            .await;

        let config = test_openrouter_config(
            workspace.path().to_path_buf(),
            format!("{}/api/v1", server.uri()),
        )
        .await;
        assert!(uses_openrouter_beta_lane(&config));

        let mut session = OpenRouterBetaSession::from_config(&config)
            .await
            .expect("session should build");
        let result = session
            .run_turn(vec![InternalMessage::user_text("inspect note.txt")])
            .await
            .expect("openrouter turn should succeed");

        assert_eq!(
            result.final_message.as_deref(),
            Some("The file says hello from workspace.")
        );
        assert_eq!(result.tool_call_records.len(), 1);
        assert_eq!(result.tool_call_records[0].tool_name, "workspace_read_file");

        let requests = server.received_requests().await.expect("received requests");
        assert_eq!(requests.len(), 2);
        let first: Value = serde_json::from_slice(&requests[0].body).expect("first body json");
        let second: Value = serde_json::from_slice(&requests[1].body).expect("second body json");
        assert_eq!(
            requests[0]
                .headers
                .get("originator")
                .and_then(|value| value.to_str().ok()),
            Some("codex_cli_rs")
        );
        assert!(requests[0].headers.contains_key("user-agent"));
        assert_eq!(
            first,
            json!({
                "model": "openai/o4-mini",
                "input": [{
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "inspect note.txt"
                    }]
                }],
                "tools": [
                    tool_json("workspace_list_dir"),
                    tool_json("workspace_read_file")
                ]
            })
        );
        assert_eq!(
            second,
            json!({
                "model": "openai/o4-mini",
                "input": [
                    {
                        "type": "message",
                        "role": "user",
                        "content": [{
                            "type": "input_text",
                            "text": "inspect note.txt"
                        }]
                    },
                    {
                        "type": "function_call",
                        "call_id": "call-1",
                        "name": "workspace_read_file",
                        "arguments": "{\"path\":\"note.txt\"}"
                    },
                    {
                        "type": "function_call_output",
                        "call_id": "call-1",
                        "output": json_string(json!({
                            "path": "note.txt",
                            "content": "hello from workspace",
                            "truncated": false
                        }))
                    }
                ],
                "tools": [
                    tool_json("workspace_list_dir"),
                    tool_json("workspace_read_file")
                ]
            })
        );
    }

    #[test]
    fn tuning_note_points_to_custom_openrouter_provider_ids() {
        assert!(openrouter_beta_tuning_note().contains(OPENROUTER_CUSTOM_PROVIDER_PREFIX));
    }

    #[test]
    #[serial]
    fn transport_policy_merges_default_and_provider_headers() {
        codex_login::default_client::set_default_client_residency_requirement(Some(
            codex_login::default_client::ResidencyRequirement::Us,
        ));
        let provider = test_provider("https://example.com/api/v1".to_string());

        let policy = transport_policy_from_provider(&provider).expect("transport policy");

        assert_eq!(
            policy
                .default_headers
                .get("originator")
                .and_then(|value| value.to_str().ok()),
            Some("codex_cli_rs")
        );
        assert_eq!(
            policy
                .default_headers
                .get("x-openai-internal-codex-residency")
                .and_then(|value| value.to_str().ok()),
            Some("us")
        );
        codex_login::default_client::set_default_client_residency_requirement(None);
    }

    #[test]
    fn transport_policy_marks_non_websocket_knobs_as_ignored() {
        let mut provider = test_provider("https://example.com/api/v1".to_string());
        provider.stream_max_retries = Some(9);
        provider.websocket_connect_timeout_ms = Some(1234);

        let policy = transport_policy_from_provider(&provider).expect("transport policy");

        assert_eq!(policy.request_max_retries, 1);
        assert_eq!(policy.request_timeout, Duration::from_millis(1000));
        assert_eq!(
            policy.ignored_knobs,
            vec!["stream_max_retries", "websocket_connect_timeout_ms"]
        );
    }

    #[tokio::test]
    async fn request_retry_knob_reaches_shared_runtime_lane() {
        let server = MockServer::start().await;
        let workspace = TempDir::new().expect("workspace tempdir");
        std::fs::write(workspace.path().join("note.txt"), "hello from workspace")
            .expect("write workspace file");

        Mock::given(method("POST"))
            .and(path("/api/v1/responses"))
            .respond_with(RetryThenSuccessResponder::default())
            .expect(3)
            .mount(&server)
            .await;

        let config = test_openrouter_config(
            workspace.path().to_path_buf(),
            format!("{}/api/v1", server.uri()),
        )
        .await;

        let mut session = OpenRouterBetaSession::from_config(&config)
            .await
            .expect("session should build");
        let result = session
            .run_turn(vec![InternalMessage::user_text("inspect note.txt")])
            .await
            .expect("request retry should recover");

        assert_eq!(
            result.final_message.as_deref(),
            Some("The file says hello from workspace.")
        );
        let requests = server.received_requests().await.expect("received requests");
        assert_eq!(requests.len(), 3);
    }

    async fn test_openrouter_config(cwd: PathBuf, base_url: String) -> Config {
        let codex_home = TempDir::new().expect("home tempdir").keep();
        let mut config = ConfigBuilder::default()
            .codex_home(codex_home)
            .fallback_cwd(Some(cwd.clone()))
            .build()
            .await
            .expect("build config");
        config.cwd = AbsolutePathBuf::from_absolute_path(cwd).expect("absolute cwd");
        config.model_provider_id = "openrouter-test".to_string();
        config.model = Some("openai/o4-mini".to_string());
        config.model_provider = test_provider(base_url);
        config
    }

    fn test_provider(base_url: String) -> ModelProviderInfo {
        ModelProviderInfo {
            name: "OpenRouter test".to_string(),
            base_url: Some(base_url),
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: Some("test-key".to_string()),
            auth: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            request_max_retries: Some(1),
            stream_max_retries: Some(0),
            stream_idle_timeout_ms: Some(1000),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
        }
    }

    fn tool_json(name: &str) -> Value {
        match name {
            "workspace_read_file" => json!({
                "type": "function",
                "name": "workspace_read_file",
                "description": "Read a text file from the current workspace root.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "max_bytes": {"type": "integer", "minimum": 1}
                    },
                    "required": ["path"]
                }
            }),
            "workspace_list_dir" => json!({
                "type": "function",
                "name": "workspace_list_dir",
                "description": "List entries from a directory under the current workspace root.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    }
                }
            }),
            other => panic!("unexpected tool json request: {other}"),
        }
    }

    fn json_string(value: Value) -> String {
        serde_json::to_string(&value).expect("serialize json string")
    }

    #[derive(Default)]
    struct SequencedOpenRouterResponder {
        next: Arc<Mutex<usize>>,
    }

    impl Respond for SequencedOpenRouterResponder {
        fn respond(&self, _request: &Request) -> ResponseTemplate {
            let mut next = self.next.lock().expect("response lock");
            let response = match *next {
                0 => ResponseTemplate::new(200).set_body_json(json!({
                    "id": "resp-1",
                    "output": [{
                        "type": "function_call",
                        "call_id": "call-1",
                        "name": "workspace_read_file",
                        "arguments": "{\"path\":\"note.txt\"}"
                    }],
                    "usage": {"input_tokens": 10, "output_tokens": 3, "total_tokens": 13},
                    "status": "completed"
                })),
                1 => ResponseTemplate::new(200).set_body_json(json!({
                    "id": "resp-2",
                    "output": [{
                        "type": "message",
                        "id": "msg-2",
                        "role": "assistant",
                        "status": "completed",
                        "content": [{
                            "type": "output_text",
                            "text": "The file says hello from workspace.",
                            "annotations": []
                        }]
                    }],
                    "usage": {"input_tokens": 16, "output_tokens": 6, "total_tokens": 22},
                    "status": "completed"
                })),
                _ => ResponseTemplate::new(500),
            };
            *next += 1;
            response
        }
    }

    #[derive(Default)]
    struct RetryThenSuccessResponder {
        next: Arc<Mutex<usize>>,
    }

    impl Respond for RetryThenSuccessResponder {
        fn respond(&self, _request: &Request) -> ResponseTemplate {
            let mut next = match self.next.lock() {
                Ok(next) => next,
                Err(error) => panic!("response lock: {error}"),
            };
            let response = match *next {
                0 => ResponseTemplate::new(503).set_body_json(json!({
                    "error": {"code": "server_error", "message": "temporary failure"}
                })),
                1 => ResponseTemplate::new(200).set_body_json(json!({
                    "id": "resp-1",
                    "output": [{
                        "type": "function_call",
                        "call_id": "call-1",
                        "name": "workspace_read_file",
                        "arguments": "{\"path\":\"note.txt\"}"
                    }],
                    "usage": {"input_tokens": 10, "output_tokens": 3, "total_tokens": 13},
                    "status": "completed"
                })),
                2 => ResponseTemplate::new(200).set_body_json(json!({
                    "id": "resp-2",
                    "output": [{
                        "type": "message",
                        "id": "msg-2",
                        "role": "assistant",
                        "status": "completed",
                        "content": [{
                            "type": "output_text",
                            "text": "The file says hello from workspace.",
                            "annotations": []
                        }]
                    }],
                    "usage": {"input_tokens": 16, "output_tokens": 6, "total_tokens": 22},
                    "status": "completed"
                })),
                _ => ResponseTemplate::new(500),
            };
            *next += 1;
            response
        }
    }
}
