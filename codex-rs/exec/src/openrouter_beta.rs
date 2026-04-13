use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::bail;
use codex_core::config::Config;
use codex_model_provider_info::OPENROUTER_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_VALIDATED_MODEL_ID;
use codex_model_provider_info::is_openrouter_provider_id;
use codex_protocol::user_input::UserInput;
use logos_provider_openrouter::OpenRouterAdapter;
use logos_provider_openrouter::OpenRouterAdapterConfig;
use logos_runtime_types::ContentPart;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::ToolExecutionKind;
use logos_session_engine::SessionEngine;
use logos_tool_runtime::PendingToolCall;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolExecutionOutput;
use reqwest::Client;
use serde_json::Value;
use serde_json::json;

use crate::event_processor::handle_last_message;

#[cfg(test)]
pub(crate) const OPENROUTER_CUSTOM_PROVIDER_PREFIX: &str = "openrouter-";
const DEFAULT_READ_FILE_MAX_BYTES: usize = 8 * 1024;
const DEFAULT_LIST_DIR_MAX_ENTRIES: usize = 200;

pub(crate) fn uses_openrouter_beta_lane(config: &Config) -> bool {
    is_openrouter_provider_id(&config.model_provider_id)
}

pub(crate) fn openrouter_beta_tuning_note() -> &'static str {
    "Built-in `openrouter` stays immutable. To tune retry or timeout knobs for the OpenRouter beta path, define and select a custom provider id such as `openrouter-tuned`."
}

#[derive(Debug)]
pub(crate) struct OpenRouterExecTurnResult {
    pub(crate) final_message: Option<String>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) tool_call_records: Vec<ToolCallRecord>,
}

pub(crate) async fn run_openrouter_exec_turn(
    config: &Config,
    items: Vec<UserInput>,
) -> anyhow::Result<OpenRouterExecTurnResult> {
    let prompt = extract_single_text_prompt(items)?;
    let api_key = config
        .model_provider
        .api_key()?
        .or_else(|| config.model_provider.experimental_bearer_token.clone())
        .ok_or_else(|| anyhow::anyhow!("OpenRouter beta path requires an API token"))?;
    let base_url = config
        .model_provider
        .base_url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("OpenRouter beta path requires a provider base_url"))?;
    let model_id = config
        .model
        .clone()
        .unwrap_or_else(|| OPENROUTER_VALIDATED_MODEL_ID.to_string());
    let workspace_root = config
        .cwd
        .canonicalize()
        .with_context(|| format!("failed to canonicalize cwd {}", config.cwd.display()))?;

    let adapter = Arc::new(OpenRouterAdapter::new(OpenRouterAdapterConfig {
        client: Client::new(),
        base_url,
        api_key,
        model_ref: ModelRef {
            provider_id: OPENROUTER_PROVIDER_ID.to_string(),
            model_id: model_id.clone(),
        },
    }));
    let mut session = SessionEngine::new(
        adapter,
        ModelRef {
            provider_id: OPENROUTER_PROVIDER_ID.to_string(),
            model_id,
        },
        build_system_prompt(config),
    )
    .await
    .map_err(|error| anyhow::anyhow!(error.message))?;

    session.register_runtime_tool(
        workspace_read_file_tool_def(),
        Arc::new(WorkspaceReadFileTool {
            workspace_root: workspace_root.clone(),
        }),
    );
    session.register_runtime_tool(
        workspace_list_dir_tool_def(),
        Arc::new(WorkspaceListDirTool { workspace_root }),
    );

    session
        .run_turn(vec![InternalMessage::user_text(prompt)])
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;

    Ok(OpenRouterExecTurnResult {
        final_message: last_assistant_message(session.history()),
        tool_call_records: session.tool_call_records().to_vec(),
    })
}

pub(crate) async fn run_openrouter_exec_path(
    config: &Config,
    items: Vec<UserInput>,
    json_mode: bool,
    output_schema_requested: bool,
    last_message_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    if json_mode {
        bail!("OpenRouter beta exec path does not support `--json` yet");
    }
    if output_schema_requested {
        bail!("OpenRouter beta exec path does not support `--output-schema` yet");
    }

    for warning in &config.startup_warnings {
        eprintln!("Warning: {warning}");
    }

    let result = run_openrouter_exec_turn(config, items).await?;
    if let Some(final_message) = result.final_message.as_deref() {
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{final_message}")?;
    }
    if let Some(path) = last_message_file.as_deref() {
        handle_last_message(result.final_message.as_deref(), path);
    }
    Ok(())
}

fn build_system_prompt(config: &Config) -> Option<String> {
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

fn extract_single_text_prompt(items: Vec<UserInput>) -> anyhow::Result<String> {
    let mut prompt = None;
    for item in items {
        match item {
            UserInput::Text { text, .. } => prompt = Some(text),
            UserInput::LocalImage { .. } => {
                bail!("OpenRouter beta exec path does not support image inputs yet")
            }
            UserInput::Image { .. } | UserInput::Skill { .. } | UserInput::Mention { .. } => {
                bail!("OpenRouter beta exec path does not support this input type yet")
            }
            _ => bail!("OpenRouter beta exec path does not support this input type yet"),
        }
    }
    let prompt = prompt
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("OpenRouter beta exec path requires a text prompt"))?;
    Ok(prompt)
}

fn last_assistant_message(history: &[InternalMessage]) -> Option<String> {
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

fn workspace_read_file_tool_def() -> InternalToolDef {
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

fn workspace_list_dir_tool_def() -> InternalToolDef {
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
    use super::openrouter_beta_tuning_note;
    use super::run_openrouter_exec_turn;
    use super::uses_openrouter_beta_lane;
    use codex_core::config::Config;
    use codex_core::config::ConfigBuilder;
    use codex_model_provider_info::ModelProviderInfo;
    use codex_model_provider_info::WireApi;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
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

        let result = run_openrouter_exec_turn(
            &config,
            vec![codex_protocol::user_input::UserInput::Text {
                text: "inspect note.txt".to_string(),
                text_elements: Vec::new(),
            }],
        )
        .await
        .expect("openrouter exec turn should succeed");

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

    #[tokio::test]
    async fn image_inputs_are_rejected_for_openrouter_exec_beta() {
        let workspace = TempDir::new().expect("workspace tempdir");
        let config = test_openrouter_config(
            workspace.path().to_path_buf(),
            "https://example.com".to_string(),
        )
        .await;

        let error = run_openrouter_exec_turn(
            &config,
            vec![codex_protocol::user_input::UserInput::LocalImage {
                path: workspace.path().join("image.png"),
            }],
        )
        .await
        .expect_err("image input should fail");

        assert!(error.to_string().contains("does not support image inputs"));
    }

    #[test]
    fn tuning_note_points_to_custom_openrouter_provider_ids() {
        assert!(openrouter_beta_tuning_note().contains(OPENROUTER_CUSTOM_PROVIDER_PREFIX));
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
        config.model_provider = ModelProviderInfo {
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
            request_max_retries: Some(0),
            stream_max_retries: Some(0),
            stream_idle_timeout_ms: Some(1000),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
        };
        config
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
}
