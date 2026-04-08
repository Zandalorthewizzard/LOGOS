#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use codex_core::ModelClient;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_otel::SessionTelemetry;
use codex_otel::TelemetryAuthMode;
use core_test_support::load_default_config_for_test;
use core_test_support::responses::WebSocketConnectionConfig;
use core_test_support::responses::WebSocketTestServer;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_output_text_delta;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_compact_user_history_with_summary_once;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::responses::start_websocket_server;
use core_test_support::responses::start_websocket_server_with_headers;
use logos_provider_openai::OpenAiAdapter;
use logos_provider_openai::OpenAiAdapterConfig;
use logos_runtime_types::CompactionBackendKind;
use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ToolApprovalMode;
use logos_runtime_types::ToolApprovalOutcome;
use logos_runtime_types::ToolApprovalRequest;
use logos_runtime_types::ToolExecutionKind;
use logos_runtime_types::ToolExecutionPolicy;
use logos_session_engine::SessionEngine;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolApprover;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use wiremock::MockServer;

const MODEL: &str = "gpt-5.2-codex";
const TURN_STATE_HEADER: &str = "x-codex-turn-state";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn text_only_openai_turn_runs_through_boundary() {
    let server = start_websocket_server(vec![vec![vec![
        ev_response_created("resp-1"),
        ev_output_text_delta("Hello from Logos."),
        ev_assistant_message("msg-1", "Hello from Logos."),
        ev_completed("resp-1"),
    ]]])
    .await;

    let mut session = build_session(&server).await;
    let outcome = session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("text turn should succeed");

    assert!(outcome.events.contains(&InternalEvent::TextDelta {
        text: "Hello from Logos.".to_string(),
    }));
    assert!(
        outcome
            .events
            .contains(&InternalEvent::ContinuationUpdated {
                handle: ContinuationHandle::OpenAiPreviousResponseId("resp-1".to_string()),
            })
    );
    assert!(outcome.events.contains(&InternalEvent::Completed));
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::OpenAiPreviousResponseId(
            "resp-1".to_string()
        ))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_session_exposes_honest_phase_two_capabilities() {
    let server = start_websocket_server(vec![]).await;
    let session = build_session(&server).await;

    assert_eq!(
        session.provider_capabilities().compaction_backend,
        CompactionBackendKind::NativeRemoteRewrite
    );
    assert!(!session.model_capabilities().supports_tool_arg_streaming);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn turn_state_persists_within_runtime_tool_turn_and_resets_after() {
    let server = start_websocket_server_with_headers(vec![
        WebSocketConnectionConfig {
            requests: vec![vec![
                ev_response_created("resp-1"),
                ev_function_call("call-1", "logos_echo", r#"{"text":"hi"}"#),
                ev_completed("resp-1"),
            ]],
            response_headers: vec![(TURN_STATE_HEADER.to_string(), "ts-1".to_string())],
            accept_delay: None,
            close_after_requests: true,
        },
        WebSocketConnectionConfig {
            requests: vec![vec![
                ev_response_created("resp-2"),
                ev_output_text_delta("Echoed hi."),
                ev_assistant_message("msg-2", "Echoed hi."),
                ev_completed("resp-2"),
            ]],
            response_headers: Vec::new(),
            accept_delay: None,
            close_after_requests: true,
        },
        WebSocketConnectionConfig {
            requests: vec![vec![
                ev_response_created("resp-3"),
                ev_output_text_delta("Next turn."),
                ev_assistant_message("msg-3", "Next turn."),
                ev_completed("resp-3"),
            ]],
            response_headers: Vec::new(),
            accept_delay: None,
            close_after_requests: true,
        },
    ])
    .await;

    let mut session = build_session(&server).await;
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));

    session
        .run_turn(vec![InternalMessage::user_text("use the echo tool")])
        .await
        .expect("first turn should succeed");
    session
        .run_turn(vec![InternalMessage::user_text("next turn")])
        .await
        .expect("second turn should succeed");

    let handshakes = server.handshakes();
    assert_eq!(handshakes.len(), 3);
    assert_eq!(handshakes[0].header(TURN_STATE_HEADER), None);
    assert_eq!(
        handshakes[1].header(TURN_STATE_HEADER),
        Some("ts-1".to_string())
    );
    assert_eq!(handshakes[2].header(TURN_STATE_HEADER), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compact_history_preserves_openai_native_fast_path_and_resets_continuation() {
    let server = start_mock_server().await;
    let _turn = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_output_text_delta("First answer."),
            ev_assistant_message("msg-1", "First answer."),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    let compact = mount_compact_user_history_with_summary_once(&server, "summary").await;

    let mut session = build_http_session(&server).await;
    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("http turn should succeed");
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::OpenAiPreviousResponseId(
            "resp-1".to_string()
        ))
    );

    session
        .compact_history()
        .await
        .expect("compaction should succeed");

    assert_eq!(session.continuation(), None);
    assert_eq!(
        session.history(),
        &[
            InternalMessage::User {
                parts: vec![ContentPart::Text {
                    text: "hello".to_string(),
                }],
            },
            InternalMessage::CompactionCheckpoint {
                summary: "summary".to_string(),
            },
        ]
    );

    let body = compact.single_request().body_json();
    assert_eq!(body["input"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["input"][0]["role"].as_str(), Some("user"));
    assert_eq!(body["input"][1]["role"].as_str(), Some("assistant"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn turn_state_resets_after_follow_up_error() {
    let server = start_websocket_server_with_headers(vec![
        WebSocketConnectionConfig {
            requests: vec![vec![
                ev_response_created("resp-1"),
                ev_function_call("call-1", "logos_echo", r#"{"text":"hi"}"#),
                ev_completed("resp-1"),
            ]],
            response_headers: vec![(TURN_STATE_HEADER.to_string(), "ts-1".to_string())],
            accept_delay: None,
            close_after_requests: true,
        },
        WebSocketConnectionConfig {
            requests: Vec::new(),
            response_headers: Vec::new(),
            accept_delay: None,
            close_after_requests: true,
        },
        WebSocketConnectionConfig {
            requests: vec![vec![
                ev_response_created("resp-2"),
                ev_output_text_delta("Recovered."),
                ev_assistant_message("msg-2", "Recovered."),
                ev_completed("resp-2"),
            ]],
            response_headers: Vec::new(),
            accept_delay: None,
            close_after_requests: true,
        },
    ])
    .await;

    let mut session = build_session(&server).await;
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));

    let error = session
        .run_turn(vec![InternalMessage::user_text("trigger tool")])
        .await
        .expect_err("follow-up request should fail");
    assert_eq!(
        error.kind,
        logos_runtime_types::ProviderErrorKind::Transport
    );

    session
        .run_turn(vec![InternalMessage::user_text("next turn")])
        .await
        .expect("next turn should succeed without stale turn-state");

    let handshakes = server.handshakes();
    assert_eq!(handshakes.len(), 3);
    assert_eq!(handshakes[0].header(TURN_STATE_HEADER), None);
    assert_eq!(
        handshakes[1].header(TURN_STATE_HEADER),
        Some("ts-1".to_string())
    );
    assert_eq!(handshakes[2].header(TURN_STATE_HEADER), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn continuation_handle_is_preserved_for_follow_up_turn() {
    let server = start_websocket_server(vec![
        vec![vec![
            ev_response_created("resp-1"),
            ev_output_text_delta("First answer."),
            ev_assistant_message("msg-1", "First answer."),
            ev_completed("resp-1"),
        ]],
        vec![vec![
            ev_response_created("resp-2"),
            ev_output_text_delta("Second answer."),
            ev_assistant_message("msg-2", "Second answer."),
            ev_completed("resp-2"),
        ]],
    ])
    .await;

    let mut session = build_session(&server).await;
    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("first turn should succeed");
    session
        .run_turn(vec![InternalMessage::user_text("follow up")])
        .await
        .expect("second turn should succeed");

    let connections = server.connections();
    assert_eq!(connections.len(), 2);
    let second = connections
        .get(1)
        .and_then(|connection| connection.first())
        .expect("missing second request")
        .body_json();
    assert_eq!(second["previous_response_id"].as_str(), Some("resp-1"));
    assert_eq!(second["input"].as_array().map(Vec::len), Some(3));
    assert_eq!(second["input"][0]["role"].as_str(), Some("user"));
    assert_eq!(
        second["input"][0]["content"][0]["text"].as_str(),
        Some("hello")
    );
    assert_eq!(second["input"][1]["role"].as_str(), Some("assistant"));
    assert_eq!(
        second["input"][1]["content"][0]["text"].as_str(),
        Some("First answer.")
    );
    assert_eq!(second["input"][2]["role"].as_str(), Some("user"));
    assert_eq!(
        second["input"][2]["content"][0]["text"].as_str(),
        Some("follow up")
    );
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::OpenAiPreviousResponseId(
            "resp-2".to_string()
        ))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stable_tool_call_id_survives_normalized_boundary() {
    let server = start_websocket_server(vec![vec![vec![
        ev_response_created("resp-1"),
        ev_function_call("call-1", "shell", r#"{"command":"pwd"}"#),
        ev_completed("resp-1"),
    ]]])
    .await;

    let mut session = build_session(&server).await;
    let outcome = session
        .run_turn(vec![InternalMessage::user_text("run pwd")])
        .await
        .expect("tool boundary turn should succeed");

    assert_eq!(
        outcome.events,
        vec![
            InternalEvent::ToolCallStarted {
                call_id: "call-1".to_string(),
                name: "shell".to_string(),
            },
            InternalEvent::ToolCallArgumentsDelta {
                call_id: "call-1".to_string(),
                delta: r#"{"command":"pwd"}"#.to_string(),
            },
            InternalEvent::ToolCallFinished {
                call_id: "call-1".to_string(),
            },
            InternalEvent::Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
            InternalEvent::ContinuationUpdated {
                handle: ContinuationHandle::OpenAiPreviousResponseId("resp-1".to_string()),
            },
            InternalEvent::Completed,
        ]
    );
    assert_eq!(
        session.history().last(),
        Some(&InternalMessage::AssistantToolCall {
            call_id: "call-1".to_string(),
            name: "shell".to_string(),
            arguments: json!({"command": "pwd"}),
        })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declared_runtime_tools_are_forwarded_to_openai_request() {
    let server = start_websocket_server(vec![vec![vec![
        ev_response_created("resp-1"),
        ev_function_call("call-1", "logos_echo", r#"{"text":"hi"}"#),
        ev_completed("resp-1"),
    ]]])
    .await;

    let mut session = build_session(&server).await;
    session
        .run_turn_with_tools(
            vec![InternalMessage::user_text("use the tool")],
            vec![InternalToolDef {
                name: "logos_echo".to_string(),
                description: "Echo back provided text".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"}
                    },
                    "required": ["text"]
                }),
                output_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"}
                    },
                    "required": ["text"]
                })),
                execution: ToolExecutionKind::RuntimeOwned,
                metadata: Default::default(),
            }],
        )
        .await
        .expect("tool declaration turn should succeed");

    let body = server
        .single_connection()
        .first()
        .expect("missing request")
        .body_json();
    assert_eq!(body["parallel_tool_calls"].as_bool(), Some(true));
    assert_eq!(body["tools"][0]["type"].as_str(), Some("function"));
    assert_eq!(body["tools"][0]["name"].as_str(), Some("logos_echo"));
    assert_eq!(
        body["tools"][0]["parameters"],
        json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            },
            "required": ["text"]
        })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn() {
    let server = start_websocket_server(vec![
        vec![vec![
            ev_response_created("resp-1"),
            ev_function_call("call-1", "logos_echo", r#"{"text":"hi"}"#),
            ev_completed("resp-1"),
        ]],
        vec![vec![
            ev_response_created("resp-2"),
            ev_output_text_delta("Echoed hi."),
            ev_assistant_message("msg-2", "Echoed hi."),
            ev_completed("resp-2"),
        ]],
    ])
    .await;

    let mut session = build_session(&server).await;
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));

    let outcome = session
        .run_turn(vec![InternalMessage::user_text("use the echo tool")])
        .await
        .expect("runtime tool roundtrip should succeed");

    assert!(outcome.events.contains(&InternalEvent::ApprovalRequested {
        request: ToolApprovalRequest {
            call_id: "call-1".to_string(),
            tool_name: "logos_echo".to_string(),
            execution: ToolExecutionKind::RuntimeOwned,
        },
    }));
    assert!(outcome.events.contains(&InternalEvent::ApprovalResolved {
        call_id: "call-1".to_string(),
        outcome: ToolApprovalOutcome::Approved,
    }));
    assert!(outcome.events.contains(&InternalEvent::ToolCallStarted {
        call_id: "call-1".to_string(),
        name: "logos_echo".to_string(),
    }));
    assert!(outcome.events.contains(&InternalEvent::TextDelta {
        text: "Echoed hi.".to_string(),
    }));
    let second = server
        .connections()
        .get(1)
        .and_then(|connection| connection.first())
        .expect("missing second request")
        .body_json();
    assert_eq!(second["previous_response_id"].as_str(), Some("resp-1"));
    assert_eq!(second["input"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        second["input"][2]["type"].as_str(),
        Some("function_call_output")
    );
    assert_eq!(second["input"][2]["call_id"].as_str(), Some("call-1"));
    assert_eq!(second["input"][2]["output"], json!("{\"text\":\"hi\"}"));
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::OpenAiPreviousResponseId(
            "resp-2".to_string()
        ))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_tool_error_is_reinjected_as_tool_result() {
    let server = start_websocket_server(vec![
        vec![vec![
            ev_response_created("resp-1"),
            ev_function_call("call-err", "logos_fail", r#"{"text":"boom"}"#),
            ev_completed("resp-1"),
        ]],
        vec![vec![
            ev_response_created("resp-2"),
            ev_output_text_delta("Handled tool failure."),
            ev_assistant_message("msg-2", "Handled tool failure."),
            ev_completed("resp-2"),
        ]],
    ])
    .await;

    let mut session = build_session(&server).await;
    session.register_runtime_tool(runtime_tool_def("logos_fail"), Arc::new(FailTool));

    session
        .run_turn(vec![InternalMessage::user_text("fail")])
        .await
        .expect("error roundtrip should still succeed");

    assert!(session.history().iter().any(|message| {
        message
            == &InternalMessage::ToolResult {
                call_id: "call-err".to_string(),
                output: json!({"error": "tool exploded"}),
                is_error: true,
            }
    }));

    let second = server
        .connections()
        .get(1)
        .and_then(|connection| connection.first())
        .expect("missing second request")
        .body_json();
    assert_eq!(second["previous_response_id"].as_str(), Some("resp-1"));
    assert_eq!(second["input"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        second["input"][2]["type"].as_str(),
        Some("function_call_output")
    );
    assert_eq!(second["input"][2]["call_id"].as_str(), Some("call-err"));
    assert_eq!(
        second["input"][2]["output"],
        json!("{\"error\":\"tool exploded\"}")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn denied_runtime_tool_is_reinjected_without_execution() {
    let server = start_websocket_server(vec![
        vec![vec![
            ev_response_created("resp-1"),
            ev_function_call("call-deny", "logos_echo", r#"{"text":"hi"}"#),
            ev_completed("resp-1"),
        ]],
        vec![vec![
            ev_response_created("resp-2"),
            ev_output_text_delta("Approval denied was observed."),
            ev_assistant_message("msg-2", "Approval denied was observed."),
            ev_completed("resp-2"),
        ]],
    ])
    .await;

    let mut session = build_session(&server).await;
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    session.set_tool_execution_policy(ToolExecutionPolicy {
        runtime_owned_default: ToolApprovalMode::RequireApproval,
        ..ToolExecutionPolicy::default()
    });
    session.set_tool_approver(Arc::new(DenyApprover));

    let outcome = session
        .run_turn(vec![InternalMessage::user_text("deny the tool")])
        .await
        .expect("denied tool flow should succeed");

    assert!(outcome.events.contains(&InternalEvent::ApprovalResolved {
        call_id: "call-deny".to_string(),
        outcome: ToolApprovalOutcome::Denied,
    }));
    assert!(session.history().iter().any(|message| {
        message
            == &InternalMessage::ToolResult {
                call_id: "call-deny".to_string(),
                output: json!({"error": "approval denied"}),
                is_error: true,
            }
    }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provider_native_tool_declaration_is_capability_gated() {
    let server = start_websocket_server(vec![]).await;
    let mut session = build_session(&server).await;
    session.set_tool_execution_policy(ToolExecutionPolicy {
        allow_provider_native: true,
        ..ToolExecutionPolicy::default()
    });

    let error = session
        .run_turn_with_tools(
            vec![InternalMessage::user_text("use provider-native tool")],
            vec![InternalToolDef {
                name: "provider_search".to_string(),
                description: "Provider-native search".to_string(),
                input_schema: json!({"type": "object", "properties": {}}),
                output_schema: None,
                execution: ToolExecutionKind::ProviderNative,
                metadata: Default::default(),
            }],
        )
        .await
        .expect_err("provider-native tool should be rejected by capabilities");

    assert_eq!(
        error.kind,
        logos_runtime_types::ProviderErrorKind::UnsupportedCapability
    );
    assert!(
        error
            .message
            .contains("provider-native tool 'provider_search'")
    );
}

fn runtime_tool_def(name: &str) -> InternalToolDef {
    InternalToolDef {
        name: name.to_string(),
        description: format!("Runtime tool {name}"),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            },
            "required": ["text"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            }
        })),
        execution: ToolExecutionKind::RuntimeOwned,
        metadata: Default::default(),
    }
}

struct EchoTool;

#[async_trait]
impl RuntimeToolExecutor for EchoTool {
    async fn execute(&self, arguments: Value) -> Result<Value, String> {
        Ok(json!({
            "text": arguments["text"].as_str().unwrap_or_default()
        }))
    }
}

struct FailTool;

#[async_trait]
impl RuntimeToolExecutor for FailTool {
    async fn execute(&self, _arguments: Value) -> Result<Value, String> {
        Err("tool exploded".to_string())
    }
}

struct DenyApprover;

#[async_trait]
impl ToolApprover for DenyApprover {
    async fn decide(
        &self,
        _request: &ToolApprovalRequest,
        _mode: ToolApprovalMode,
    ) -> ToolApprovalOutcome {
        ToolApprovalOutcome::Denied
    }
}

async fn build_session(server: &WebSocketTestServer) -> SessionEngine {
    build_session_with_provider(ModelProviderInfo {
        name: "mock-ws".into(),
        base_url: Some(format!("{}/v1", server.uri())),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(5_000),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: true,
    })
    .await
}

async fn build_http_session(server: &MockServer) -> SessionEngine {
    build_session_with_provider(ModelProviderInfo {
        name: "mock-http".into(),
        base_url: Some(format!("{}/v1", server.uri())),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(5_000),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    })
    .await
}

async fn build_session_with_provider(provider: ModelProviderInfo) -> SessionEngine {
    let codex_home = TempDir::new().unwrap();
    let config = load_default_config_for_test(&codex_home).await;
    let conversation_id = codex_protocol::ThreadId::new();
    let auth_manager =
        codex_core::test_support::auth_manager_from_auth(CodexAuth::from_api_key("Test API Key"));
    let model_info = codex_core::test_support::construct_model_info_offline(MODEL, &config);
    let session_telemetry = SessionTelemetry::new(
        conversation_id,
        MODEL,
        model_info.slug.as_str(),
        /*account_id*/ None,
        Some("test@test.com".to_string()),
        auth_manager.auth_mode().map(TelemetryAuthMode::from),
        "test_originator".to_string(),
        /*log_user_prompts*/ false,
        "test".to_string(),
        codex_protocol::protocol::SessionSource::Exec,
    );
    let adapter = Arc::new(OpenAiAdapter::new(OpenAiAdapterConfig {
        client: ModelClient::new(
            /*auth_manager*/ None,
            conversation_id,
            provider,
            codex_protocol::protocol::SessionSource::Exec,
            config.model_verbosity,
            /*enable_request_compression*/ false,
            /*include_timing_metrics*/ false,
            /*beta_features_header*/ None,
        ),
        model_info,
        model_ref: ModelRef {
            provider_id: "openai".to_string(),
            model_id: MODEL.to_string(),
        },
        session_telemetry,
        reasoning_effort: None,
        reasoning_summary: codex_protocol::config_types::ReasoningSummary::Auto,
        service_tier: None,
    }));

    SessionEngine::new(
        adapter,
        ModelRef {
            provider_id: "openai".to_string(),
            model_id: MODEL.to_string(),
        },
        None,
    )
    .await
    .expect("session should build")
}
