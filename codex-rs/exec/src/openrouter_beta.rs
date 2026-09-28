use std::io::Write;
use std::path::PathBuf;

use anyhow::bail;
use codex_core::config::Config;
use codex_protocol::user_input::UserInput;
use logos_openrouter_beta::OpenRouterBetaSession;
use logos_openrouter_beta::unsupported_operation_message;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::ToolCallRecord;

use crate::event_processor::handle_last_message;

#[derive(Debug)]
pub(crate) struct OpenRouterExecTurnResult {
    pub(crate) final_message: Option<String>,
    #[allow(dead_code)]
    pub(crate) tool_call_records: Vec<ToolCallRecord>,
}

pub(crate) async fn run_openrouter_exec_turn(
    config: &Config,
    items: Vec<UserInput>,
) -> anyhow::Result<OpenRouterExecTurnResult> {
    let prompt = extract_single_text_prompt(items)?;
    let mut session = OpenRouterBetaSession::from_config(config).await?;
    let result = session
        .run_turn(vec![InternalMessage::user_text(prompt)])
        .await?;

    Ok(OpenRouterExecTurnResult {
        final_message: result.final_message,
        tool_call_records: result.tool_call_records,
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
        bail!(unsupported_operation_message("exec", "`--json`"));
    }
    if output_schema_requested {
        bail!(unsupported_operation_message("exec", "`--output-schema`"));
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

#[cfg(test)]
mod tests {
    use super::run_openrouter_exec_turn;
    use codex_core::config::Config;
    use codex_core::config::ConfigBuilder;
    use codex_model_provider_info::ModelProviderInfo;
    use codex_model_provider_info::WireApi;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use std::path::PathBuf;
    use tempfile::TempDir;

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
        assert!(logos_openrouter_beta::openrouter_beta_tuning_note().contains("openrouter-tuned"));
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
}
