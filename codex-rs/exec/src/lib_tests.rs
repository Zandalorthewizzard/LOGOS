use super::*;
use codex_app_server_client::DEFAULT_IN_PROCESS_CHANNEL_CAPACITY;
use codex_core::config::ConfigOverrides;
use codex_core::config_loader::CloudRequirementsLoader;
use codex_core::config_loader::LoaderOverrides;
use codex_feedback::CodexFeedback;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_otel::set_parent_from_w3c_trace_context;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::protocol::SessionSource;
use codex_utils_absolute_path::AbsolutePathBuf;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::trace::TraceId;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::tempdir;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

fn test_tracing_subscriber() -> impl tracing::Subscriber + Send + Sync {
    let provider = SdkTracerProvider::builder().build();
    let tracer = provider.tracer("codex-exec-tests");
    tracing_subscriber::registry().with(tracing_opentelemetry::layer().with_tracer(tracer))
}

#[test]
fn exec_defaults_analytics_to_enabled() {
    assert_eq!(DEFAULT_ANALYTICS_ENABLED, true);
}

#[test]
fn exec_root_span_can_be_parented_from_trace_context() {
    let subscriber = test_tracing_subscriber();
    let _guard = tracing::subscriber::set_default(subscriber);

    let parent = codex_protocol::protocol::W3cTraceContext {
        traceparent: Some("00-00000000000000000000000000000077-0000000000000088-01".into()),
        tracestate: Some("vendor=value".into()),
    };
    let exec_span = exec_root_span();
    assert!(set_parent_from_w3c_trace_context(&exec_span, &parent));

    let trace_id = exec_span.context().span().span_context().trace_id();
    assert_eq!(
        trace_id,
        TraceId::from_hex("00000000000000000000000000000077").expect("trace id")
    );
}

#[test]
fn builds_uncommitted_review_request() {
    let args = ReviewArgs {
        uncommitted: true,
        base: None,
        commit: None,
        commit_title: None,
        prompt: None,
    };
    let request = build_review_request(&args).expect("builds uncommitted review request");

    let expected = ReviewRequest {
        target: ReviewTarget::UncommittedChanges,
        user_facing_hint: None,
    };

    assert_eq!(request, expected);
}

#[test]
fn builds_commit_review_request_with_title() {
    let args = ReviewArgs {
        uncommitted: false,
        base: None,
        commit: Some("123456789".to_string()),
        commit_title: Some("Add review command".to_string()),
        prompt: None,
    };
    let request = build_review_request(&args).expect("builds commit review request");

    let expected = ReviewRequest {
        target: ReviewTarget::Commit {
            sha: "123456789".to_string(),
            title: Some("Add review command".to_string()),
        },
        user_facing_hint: None,
    };

    assert_eq!(request, expected);
}

#[test]
fn builds_custom_review_request_trims_prompt() {
    let args = ReviewArgs {
        uncommitted: false,
        base: None,
        commit: None,
        commit_title: None,
        prompt: Some("  custom review instructions  ".to_string()),
    };
    let request = build_review_request(&args).expect("builds custom review request");

    let expected = ReviewRequest {
        target: ReviewTarget::Custom {
            instructions: "custom review instructions".to_string(),
        },
        user_facing_hint: None,
    };

    assert_eq!(request, expected);
}

#[test]
fn decode_prompt_bytes_strips_utf8_bom() {
    let input = [0xEF, 0xBB, 0xBF, b'h', b'i', b'\n'];

    let out = decode_prompt_bytes(&input).expect("decode utf-8 with BOM");

    assert_eq!(out, "hi\n");
}

#[test]
fn decode_prompt_bytes_decodes_utf16le_bom() {
    // UTF-16LE BOM + "hi\n"
    let input = [0xFF, 0xFE, b'h', 0x00, b'i', 0x00, b'\n', 0x00];

    let out = decode_prompt_bytes(&input).expect("decode utf-16le with BOM");

    assert_eq!(out, "hi\n");
}

#[test]
fn decode_prompt_bytes_decodes_utf16be_bom() {
    // UTF-16BE BOM + "hi\n"
    let input = [0xFE, 0xFF, 0x00, b'h', 0x00, b'i', 0x00, b'\n'];

    let out = decode_prompt_bytes(&input).expect("decode utf-16be with BOM");

    assert_eq!(out, "hi\n");
}

#[test]
fn decode_prompt_bytes_rejects_utf32le_bom() {
    // UTF-32LE BOM + "hi\n"
    let input = [
        0xFF, 0xFE, 0x00, 0x00, b'h', 0x00, 0x00, 0x00, b'i', 0x00, 0x00, 0x00, b'\n', 0x00, 0x00,
        0x00,
    ];

    let err = decode_prompt_bytes(&input).expect_err("utf-32le should be rejected");

    assert_eq!(
        err,
        PromptDecodeError::UnsupportedBom {
            encoding: "UTF-32LE"
        }
    );
}

#[test]
fn decode_prompt_bytes_rejects_utf32be_bom() {
    // UTF-32BE BOM + "hi\n"
    let input = [
        0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, b'h', 0x00, 0x00, 0x00, b'i', 0x00, 0x00, 0x00,
        b'\n',
    ];

    let err = decode_prompt_bytes(&input).expect_err("utf-32be should be rejected");

    assert_eq!(
        err,
        PromptDecodeError::UnsupportedBom {
            encoding: "UTF-32BE"
        }
    );
}

#[test]
fn decode_prompt_bytes_rejects_invalid_utf8() {
    // Invalid UTF-8 sequence: 0xC3 0x28
    let input = [0xC3, 0x28];

    let err = decode_prompt_bytes(&input).expect_err("invalid utf-8 should fail");

    assert_eq!(err, PromptDecodeError::InvalidUtf8 { valid_up_to: 0 });
}

#[test]
fn prompt_with_stdin_context_wraps_stdin_block() {
    let combined = prompt_with_stdin_context("Summarize this concisely", "my output");

    assert_eq!(
        combined,
        "Summarize this concisely\n\n<stdin>\nmy output\n</stdin>"
    );
}

#[test]
fn prompt_with_stdin_context_preserves_trailing_newline() {
    let combined = prompt_with_stdin_context("Summarize this concisely", "my output\n");

    assert_eq!(
        combined,
        "Summarize this concisely\n\n<stdin>\nmy output\n</stdin>"
    );
}

#[test]
fn lagged_event_warning_message_is_explicit() {
    assert_eq!(
        lagged_event_warning_message(/*skipped*/ 7),
        "in-process app-server event stream lagged; dropped 7 events".to_string()
    );
}

#[tokio::test]
async fn resume_lookup_model_providers_filters_only_last_lookup() {
    let codex_home = tempdir().expect("create temp codex home");
    let cwd = tempdir().expect("create temp cwd");
    let mut config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .fallback_cwd(Some(cwd.path().to_path_buf()))
        .build()
        .await
        .expect("build default config");
    config.model_provider_id = "test-provider".to_string();

    let last_args = crate::cli::ResumeArgs {
        session_id: None,
        last: true,
        all: false,
        images: vec![],
        prompt: None,
    };
    let named_args = crate::cli::ResumeArgs {
        session_id: Some("named-session".to_string()),
        last: false,
        all: false,
        images: vec![],
        prompt: None,
    };

    assert_eq!(
        resume_lookup_model_providers(&config, &last_args),
        Some(vec!["test-provider".to_string()])
    );
    assert_eq!(resume_lookup_model_providers(&config, &named_args), None);
}

#[tokio::test]
async fn openrouter_beta_exec_session_dispatches_into_real_exec_lane() {
    let server = MockServer::start().await;
    let workspace = tempdir().expect("workspace tempdir");
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")
        .expect("write workspace file");
    let last_message_path = workspace.path().join("last-message.txt");

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(SequencedOpenRouterResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    let config = test_openrouter_exec_config(
        workspace.path().to_path_buf(),
        format!("{}/api/v1", server.uri()),
    )
    .await;

    run_exec_session(ExecRunArgs {
        in_process_start_args: test_in_process_start_args(config.clone()),
        command: None,
        config,
        dangerously_bypass_approvals_and_sandbox: false,
        exec_span: exec_root_span(),
        images: Vec::new(),
        json_mode: false,
        last_message_file: Some(last_message_path.clone()),
        model_provider: None,
        oss: false,
        output_schema_path: None,
        prompt: Some("inspect note.txt".to_string()),
        skip_git_repo_check: true,
        stderr_with_ansi: false,
    })
    .await
    .expect("openrouter exec dispatch should succeed");

    let last_message = std::fs::read_to_string(&last_message_path)
        .expect("last message should be written through exec lane");
    assert_eq!(last_message, "The file says hello from workspace.");
}

#[tokio::test]
async fn openrouter_beta_exec_session_rejects_resume_before_dispatch() {
    let workspace = tempdir().expect("workspace tempdir");
    let config = test_openrouter_exec_config(
        workspace.path().to_path_buf(),
        "https://example.com/api/v1".to_string(),
    )
    .await;

    let error = run_exec_session(ExecRunArgs {
        in_process_start_args: test_in_process_start_args(config.clone()),
        command: Some(ExecCommand::Resume(crate::cli::ResumeArgs {
            session_id: Some("thread-1".to_string()),
            last: false,
            all: false,
            images: Vec::new(),
            prompt: Some("continue".to_string()),
        })),
        config,
        dangerously_bypass_approvals_and_sandbox: false,
        exec_span: exec_root_span(),
        images: Vec::new(),
        json_mode: false,
        last_message_file: None,
        model_provider: None,
        oss: false,
        output_schema_path: None,
        prompt: None,
        skip_git_repo_check: true,
        stderr_with_ansi: false,
    })
    .await
    .expect_err("resume should be rejected on openrouter beta lane");

    assert!(
        error
            .to_string()
            .contains("supports only new user turns; resume and review remain unavailable")
    );
}

#[test]
fn turn_items_for_thread_returns_matching_turn_items() {
    let thread = AppServerThread {
        id: "thread-1".to_string(),
        forked_from_id: None,
        preview: String::new(),
        ephemeral: false,
        model_provider: "openai".to_string(),
        created_at: 0,
        updated_at: 0,
        status: codex_app_server_protocol::ThreadStatus::Idle,
        path: None,
        cwd: PathBuf::from("/tmp/project"),
        cli_version: "0.0.0-test".to_string(),
        source: codex_app_server_protocol::SessionSource::Exec,
        agent_nickname: None,
        agent_role: None,
        git_info: None,
        name: None,
        turns: vec![
            codex_app_server_protocol::Turn {
                id: "turn-1".to_string(),
                items: vec![AppServerThreadItem::AgentMessage {
                    id: "msg-1".to_string(),
                    text: "hello".to_string(),
                    phase: None,
                    memory_citation: None,
                }],
                status: codex_app_server_protocol::TurnStatus::Completed,
                error: None,
            },
            codex_app_server_protocol::Turn {
                id: "turn-2".to_string(),
                items: vec![AppServerThreadItem::Plan {
                    id: "plan-1".to_string(),
                    text: "ship it".to_string(),
                }],
                status: codex_app_server_protocol::TurnStatus::Completed,
                error: None,
            },
        ],
    };

    assert_eq!(
        turn_items_for_thread(&thread, "turn-1"),
        Some(vec![AppServerThreadItem::AgentMessage {
            id: "msg-1".to_string(),
            text: "hello".to_string(),
            phase: None,
            memory_citation: None,
        }])
    );
    assert_eq!(turn_items_for_thread(&thread, "missing-turn"), None);
}

async fn test_openrouter_exec_config(cwd: PathBuf, base_url: String) -> Config {
    let codex_home = tempdir().expect("home tempdir");
    let mut config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .harness_overrides(ConfigOverrides {
            cwd: Some(cwd.clone()),
            ..Default::default()
        })
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

fn test_in_process_start_args(config: Config) -> InProcessClientStartArgs {
    InProcessClientStartArgs {
        arg0_paths: codex_arg0::Arg0DispatchPaths::default(),
        config: Arc::new(config),
        cli_overrides: Vec::new(),
        loader_overrides: LoaderOverrides::default(),
        cloud_requirements: CloudRequirementsLoader::default(),
        feedback: CodexFeedback::new(),
        config_warnings: Vec::new(),
        session_source: SessionSource::Exec,
        enable_codex_api_key_env: false,
        client_name: "codex-exec-test".to_string(),
        client_version: "0.0.0-test".to_string(),
        experimental_api: true,
        opt_out_notification_methods: Vec::new(),
        channel_capacity: DEFAULT_IN_PROCESS_CHANNEL_CAPACITY,
    }
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

#[test]
fn canceled_mcp_server_elicitation_response_uses_cancel_action() {
    let value = canceled_mcp_server_elicitation_response()
        .expect("mcp elicitation cancel response should serialize");
    let response: McpServerElicitationRequestResponse =
        serde_json::from_value(value).expect("cancel response should deserialize");

    assert_eq!(
        response,
        McpServerElicitationRequestResponse {
            action: McpServerElicitationAction::Cancel,
            content: None,
            meta: None,
        }
    );
}

#[tokio::test]
async fn thread_start_params_include_review_policy_when_review_policy_is_manual_only() {
    let codex_home = tempdir().expect("create temp codex home");
    let cwd = tempdir().expect("create temp cwd");
    let config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .harness_overrides(ConfigOverrides {
            approvals_reviewer: Some(ApprovalsReviewer::User),
            ..Default::default()
        })
        .fallback_cwd(Some(cwd.path().to_path_buf()))
        .build()
        .await
        .expect("build config with manual-only review policy");

    let params = thread_start_params_from_config(&config);

    assert_eq!(
        params.approvals_reviewer,
        Some(codex_app_server_protocol::ApprovalsReviewer::User)
    );
}

#[tokio::test]
async fn thread_start_params_include_review_policy_when_auto_review_is_enabled() {
    let codex_home = tempdir().expect("create temp codex home");
    let cwd = tempdir().expect("create temp cwd");
    let config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .harness_overrides(ConfigOverrides {
            approvals_reviewer: Some(ApprovalsReviewer::GuardianSubagent),
            ..Default::default()
        })
        .fallback_cwd(Some(cwd.path().to_path_buf()))
        .build()
        .await
        .expect("build config with guardian review policy");

    let params = thread_start_params_from_config(&config);

    assert_eq!(
        params.approvals_reviewer,
        Some(codex_app_server_protocol::ApprovalsReviewer::GuardianSubagent)
    );
}

#[test]
fn session_configured_from_thread_response_uses_review_policy_from_response() {
    let response = ThreadStartResponse {
        thread: codex_app_server_protocol::Thread {
            id: "67e55044-10b1-426f-9247-bb680e5fe0c8".to_string(),
            forked_from_id: None,
            preview: String::new(),
            ephemeral: false,
            model_provider: "openai".to_string(),
            created_at: 0,
            updated_at: 0,
            status: codex_app_server_protocol::ThreadStatus::Idle,
            path: Some(PathBuf::from("/tmp/rollout.jsonl")),
            cwd: PathBuf::from("/tmp"),
            cli_version: "0.0.0".to_string(),
            source: codex_app_server_protocol::SessionSource::Cli,
            agent_nickname: None,
            agent_role: None,
            git_info: None,
            name: Some("thread".to_string()),
            turns: vec![],
        },
        model: "gpt-5.4".to_string(),
        model_provider: "openai".to_string(),
        service_tier: None,
        cwd: PathBuf::from("/tmp"),
        approval_policy: codex_app_server_protocol::AskForApproval::OnRequest,
        approvals_reviewer: codex_app_server_protocol::ApprovalsReviewer::GuardianSubagent,
        sandbox: codex_app_server_protocol::SandboxPolicy::WorkspaceWrite {
            writable_roots: vec![],
            read_only_access: codex_app_server_protocol::ReadOnlyAccess::FullAccess,
            network_access: false,
            exclude_tmpdir_env_var: false,
            exclude_slash_tmp: false,
        },
        reasoning_effort: None,
    };

    let event = session_configured_from_thread_start_response(&response)
        .expect("build bootstrap session configured event");

    assert_eq!(
        event.approvals_reviewer,
        ApprovalsReviewer::GuardianSubagent
    );
}
