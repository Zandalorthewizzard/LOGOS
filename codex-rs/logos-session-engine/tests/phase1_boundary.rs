#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use codex_core::ModelClient;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_otel::SessionTelemetry;
use codex_otel::TelemetryAuthMode;
use core_test_support::load_default_config_for_test;
use core_test_support::responses::WebSocketTestServer;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_output_text_delta;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::start_websocket_server;
use logos_provider_openai::OpenAiAdapter;
use logos_provider_openai::OpenAiAdapterConfig;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::ModelRef;
use logos_session_engine::SessionEngine;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;

const MODEL: &str = "gpt-5.2-codex";

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
async fn continuation_handle_is_preserved_for_follow_up_turn() {
    let server = start_websocket_server(vec![vec![
        vec![
            ev_response_created("resp-1"),
            ev_output_text_delta("First answer."),
            ev_assistant_message("msg-1", "First answer."),
            ev_completed("resp-1"),
        ],
        vec![
            ev_response_created("resp-2"),
            ev_output_text_delta("Second answer."),
            ev_assistant_message("msg-2", "Second answer."),
            ev_completed("resp-2"),
        ],
    ]])
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

    let connection = server.single_connection();
    assert_eq!(connection.len(), 2);
    let second = connection
        .get(1)
        .expect("missing second request")
        .body_json();
    assert_eq!(second["previous_response_id"].as_str(), Some("resp-1"));
    assert_eq!(
        second["input"],
        json!([{
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": "follow up"}]
        }])
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

async fn build_session(server: &WebSocketTestServer) -> SessionEngine {
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
    let provider = ModelProviderInfo {
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
    };
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
