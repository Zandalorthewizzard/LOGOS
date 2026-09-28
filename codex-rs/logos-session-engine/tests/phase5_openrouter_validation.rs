#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::Mutex;

use logos_journal::JournalConfig;
use logos_journal::JournalStore;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderEventStream;
use logos_provider_api::ProviderSession;
use logos_provider_openrouter::OpenRouterAdapter;
use logos_provider_openrouter::OpenRouterAdapterConfig;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use logos_runtime_types::ToolApprovalMode;
use logos_runtime_types::ToolApprovalOutcome;
use logos_runtime_types::ToolApprovalRequest;
use logos_runtime_types::ToolCallResultDisposition;
use logos_runtime_types::ToolCallTerminalState;
use logos_runtime_types::ToolExecutionKind;
use logos_runtime_types::ToolExecutionPolicy;
use logos_session_engine::SessionEngine;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolExecutionOutput;
use pretty_assertions::assert_eq;
use reqwest::Client;
use serde_json::json;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::body_json;
use wiremock::matchers::method;
use wiremock::matchers::path;

const OPENROUTER_MODEL: &str = "openai/o4-mini";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openrouter_text_only_turn_runs_through_shared_boundary() {
    let server = MockServer::start().await;
    mount_openrouter_text_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": "hello"
                }]
            }]
        }),
        "resp-1",
        "msg-1",
        "Hello from OpenRouter.",
    )
    .await;

    let adapter = build_openrouter_adapter(&server);
    let mut session = SessionEngine::new(adapter, openrouter_model_ref(), None)
        .await
        .expect("session should build");

    let outcome = session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("turn should succeed");

    assert!(outcome.events.contains(&InternalEvent::TextDelta {
        text: "Hello from OpenRouter.".to_string(),
    }));
    assert!(
        outcome
            .events
            .contains(&InternalEvent::ContinuationUpdated {
                handle: ContinuationHandle::ClientManagedHistory,
            })
    );
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::ClientManagedHistory)
    );

    let requests = server.received_requests().await.unwrap_or_default();
    assert_eq!(requests.len(), 1);
    let body = requests[0]
        .body_json::<serde_json::Value>()
        .expect("body json");
    assert!(body.get("previous_response_id").is_none());
    assert_eq!(body["model"].as_str(), Some(OPENROUTER_MODEL));
    assert_eq!(body["input"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["input"][0]["role"].as_str(), Some("user"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openrouter_follow_up_replays_history_from_journal_without_fake_provider_state() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let server = MockServer::start().await;
    mount_openrouter_text_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": "What is the capital of France?"
                }]
            }]
        }),
        "resp-1",
        "msg-1",
        "Paris.",
    )
    .await;
    mount_openrouter_text_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "What is the capital of France?"
                    }]
                },
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": "Paris.",
                        "annotations": []
                    }],
                    "id": "msg-1",
                    "status": "completed"
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "What is the population of that city?"
                    }]
                }
            ]
        }),
        "resp-2",
        "msg-2",
        "About 2.1 million.",
    )
    .await;

    let adapter = build_openrouter_adapter(&server);
    let mut session = SessionEngine::new(adapter.clone(), openrouter_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text(
            "What is the capital of France?",
        )])
        .await
        .expect("first turn should succeed");

    let journal = store.open_session(&session_id);
    let mut resumed = SessionEngine::resume_from_journal(adapter, journal)
        .await
        .expect("resume should succeed");
    resumed
        .run_turn(vec![InternalMessage::user_text(
            "What is the population of that city?",
        )])
        .await
        .expect("second turn should succeed");

    let requests = server.received_requests().await.unwrap_or_default();
    assert_eq!(requests.len(), 2);
    let second = requests[1]
        .body_json::<serde_json::Value>()
        .expect("body json");
    assert!(second.get("previous_response_id").is_none());
    assert_eq!(second["input"].as_array().map(Vec::len), Some(3));
    assert_eq!(second["input"][0]["role"].as_str(), Some("user"));
    assert_eq!(second["input"][1]["role"].as_str(), Some("assistant"));
    assert_eq!(second["input"][1]["id"].as_str(), Some("msg-1"));
    assert_eq!(second["input"][1]["status"].as_str(), Some("completed"));
    assert_eq!(second["input"][2]["role"].as_str(), Some("user"));
    assert_eq!(
        second["input"][2]["content"][0]["text"].as_str(),
        Some("What is the population of that city?")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openrouter_unsupported_parameter_and_limit_behavior_is_explicit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(ResponseTemplate::new(429).set_body_string("too many requests"))
        .mount(&server)
        .await;

    let adapter = build_openrouter_adapter(&server);
    let mut session = adapter.new_session().await.expect("session should build");
    let error = session
        .stream_turn(logos_runtime_types::InternalTurnRequest {
            model_ref: openrouter_model_ref(),
            system: None,
            messages: vec![InternalMessage::user_text("hello")],
            tools: Vec::new(),
            output_mode: logos_runtime_types::OutputMode::Text,
            reasoning_policy: logos_runtime_types::ReasoningPolicy::Default,
            continuation: Some(ContinuationHandle::ClientManagedHistory),
            limits: logos_runtime_types::TurnLimits::default(),
            metadata: Default::default(),
        })
        .await;
    let error = match error {
        Ok(_) => panic!("429 should normalize into provider error"),
        Err(error) => error,
    };
    assert_eq!(error.kind, ProviderErrorKind::RateLimited);
    assert_eq!(error.metadata["http_status"], json!(429));
    assert_eq!(error.metadata["body_parse_failed"], json!(true));

    let mut unsupported = adapter.new_session().await.expect("session should build");
    let error = unsupported
        .stream_turn(logos_runtime_types::InternalTurnRequest {
            model_ref: openrouter_model_ref(),
            system: None,
            messages: vec![InternalMessage::user_text("hello")],
            tools: Vec::new(),
            output_mode: logos_runtime_types::OutputMode::Structured,
            reasoning_policy: logos_runtime_types::ReasoningPolicy::Default,
            continuation: None,
            limits: logos_runtime_types::TurnLimits::default(),
            metadata: Default::default(),
        })
        .await;
    let error = match error {
        Ok(_) => panic!("structured output should be rejected explicitly"),
        Err(error) => error,
    };
    assert_eq!(error.kind, ProviderErrorKind::UnsupportedCapability);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openrouter_runtime_tool_roundtrip_reinjects_tool_result_and_finishes_turn() {
    let server = MockServer::start().await;
    mount_openrouter_json_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": "use the echo tool"
                }]
            }],
            "tools": [runtime_tool_declaration_json("logos_echo")]
        }),
        json!({
            "id": "resp-1",
            "output": [{
                "type": "function_call",
                "call_id": "call-1",
                "name": "logos_echo",
                "arguments": "{\"text\":\"hi\"}"
            }],
            "usage": {
                "input_tokens": 14,
                "output_tokens": 5,
                "total_tokens": 19
            },
            "status": "completed"
        }),
    )
    .await;
    mount_openrouter_json_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "use the echo tool"
                    }]
                },
                {
                    "type": "function_call",
                    "call_id": "call-1",
                    "name": "logos_echo",
                    "arguments": "{\"text\":\"hi\"}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call-1",
                    "output": "{\"text\":\"hi\"}"
                }
            ],
            "tools": [runtime_tool_declaration_json("logos_echo")]
        }),
        json!({
            "id": "resp-2",
            "output": [{
                "type": "message",
                "id": "msg-2",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "Echoed hi.",
                    "annotations": []
                }]
            }],
            "usage": {
                "input_tokens": 19,
                "output_tokens": 4,
                "total_tokens": 23
            },
            "status": "completed"
        }),
    )
    .await;

    let adapter = build_openrouter_adapter(&server);
    let mut session = SessionEngine::new(adapter, openrouter_model_ref(), None)
        .await
        .expect("session should build");
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    session.set_tool_execution_policy(ToolExecutionPolicy {
        runtime_owned_default: ToolApprovalMode::AutoApprove,
        ..ToolExecutionPolicy::default()
    });

    let outcome = session
        .run_turn(vec![InternalMessage::user_text("use the echo tool")])
        .await
        .expect("runtime tool roundtrip should succeed");

    assert!(outcome.events.contains(&InternalEvent::ApprovalRequested {
        request: ToolApprovalRequest {
            call_id: "call-1".to_string(),
            tool_name: "logos_echo".to_string(),
            execution: ToolExecutionKind::RuntimeOwned,
            mcp_correlation: None,
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
    assert!(outcome.events.contains(&InternalEvent::ToolCallTerminal {
        record: logos_runtime_types::ToolCallRecord {
            call_id: "call-1".to_string(),
            tool_name: "logos_echo".to_string(),
            execution: ToolExecutionKind::RuntimeOwned,
            terminal_state: ToolCallTerminalState::CompletedWithResult,
            mcp_correlation: None,
            result_disposition: ToolCallResultDisposition::Success,
        },
    }));
    assert!(session.history().iter().any(|message| {
        message
            == &InternalMessage::AssistantToolCall {
                call_id: "call-1".to_string(),
                name: "logos_echo".to_string(),
                arguments: json!({"text": "hi"}),
            }
    }));
    assert!(session.history().iter().any(|message| {
        message
            == &InternalMessage::ToolResult {
                call_id: "call-1".to_string(),
                output: json!({"text": "hi"}),
                is_error: false,
            }
    }));
    assert_eq!(
        session.continuation(),
        Some(&ContinuationHandle::ClientManagedHistory)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openrouter_follow_up_after_tool_execution_replays_journal_history() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let server = MockServer::start().await;
    mount_openrouter_json_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": "use the echo tool"
                }]
            }],
            "tools": [runtime_tool_declaration_json("logos_echo")]
        }),
        json!({
            "id": "resp-1",
            "output": [{
                "type": "function_call",
                "call_id": "call-1",
                "name": "logos_echo",
                "arguments": "{\"text\":\"hi\"}"
            }],
            "usage": {
                "input_tokens": 14,
                "output_tokens": 5,
                "total_tokens": 19
            },
            "status": "completed"
        }),
    )
    .await;
    mount_openrouter_json_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "use the echo tool"
                    }]
                },
                {
                    "type": "function_call",
                    "call_id": "call-1",
                    "name": "logos_echo",
                    "arguments": "{\"text\":\"hi\"}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call-1",
                    "output": "{\"text\":\"hi\"}"
                }
            ],
            "tools": [runtime_tool_declaration_json("logos_echo")]
        }),
        json!({
            "id": "resp-2",
            "output": [{
                "type": "message",
                "id": "msg-2",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "Echoed hi.",
                    "annotations": []
                }]
            }],
            "usage": {
                "input_tokens": 19,
                "output_tokens": 4,
                "total_tokens": 23
            },
            "status": "completed"
        }),
    )
    .await;
    mount_openrouter_json_response(
        &server,
        json!({
            "model": OPENROUTER_MODEL,
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "use the echo tool"
                    }]
                },
                {
                    "type": "function_call",
                    "call_id": "call-1",
                    "name": "logos_echo",
                    "arguments": "{\"text\":\"hi\"}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call-1",
                    "output": "{\"text\":\"hi\"}"
                },
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": "Echoed hi.",
                        "annotations": []
                    }],
                    "id": "msg-2",
                    "status": "completed"
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": "What tool ran before?"
                    }]
                }
            ]
        }),
        json!({
            "id": "resp-3",
            "output": [{
                "type": "message",
                "id": "msg-3",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "The logos_echo tool ran before.",
                    "annotations": []
                }]
            }],
            "usage": {
                "input_tokens": 27,
                "output_tokens": 7,
                "total_tokens": 34
            },
            "status": "completed"
        }),
    )
    .await;

    let adapter = build_openrouter_adapter(&server);
    let mut session = SessionEngine::new(adapter.clone(), openrouter_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("use the echo tool")])
        .await
        .expect("tool turn should succeed");

    let journal = store.open_session(&session_id);
    let mut resumed = SessionEngine::resume_from_journal(adapter, journal)
        .await
        .expect("resume should succeed");
    resumed
        .run_turn(vec![InternalMessage::user_text("What tool ran before?")])
        .await
        .expect("follow-up should succeed");

    let requests = server.received_requests().await.unwrap_or_default();
    assert_eq!(requests.len(), 3);
    let third = requests[2]
        .body_json::<serde_json::Value>()
        .expect("body json");
    assert!(third.get("previous_response_id").is_none());
    assert_eq!(third["input"].as_array().map(Vec::len), Some(5));
    assert_eq!(third["input"][1]["type"].as_str(), Some("function_call"));
    assert_eq!(third["input"][1]["call_id"].as_str(), Some("call-1"));
    assert_eq!(
        third["input"][1]["arguments"].as_str(),
        Some("{\"text\":\"hi\"}")
    );
    assert_eq!(
        third["input"][2]["type"].as_str(),
        Some("function_call_output")
    );
    assert_eq!(third["input"][2]["call_id"].as_str(), Some("call-1"));
    assert_eq!(
        third["input"][2]["output"].as_str(),
        Some("{\"text\":\"hi\"}")
    );
    assert_eq!(third["input"][3]["role"].as_str(), Some("assistant"));
    assert_eq!(third["input"][3]["id"].as_str(), Some("msg-2"));
    assert_eq!(third["input"][4]["role"].as_str(), Some("user"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_no_regression_scope_still_uses_previous_response_id() {
    let adapter = Arc::new(RecordingOpenAiAdapter::default());
    let mut session = SessionEngine::new(adapter.clone(), openai_model_ref(), None)
        .await
        .expect("session should build");

    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("first turn should succeed");
    session
        .run_turn(vec![InternalMessage::user_text("follow up")])
        .await
        .expect("second turn should succeed");

    let requests = adapter.requests.lock().expect("requests lock").clone();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[1].continuation,
        Some(ContinuationHandle::OpenAiPreviousResponseId(
            "resp-1".to_string()
        ))
    );
}

fn build_openrouter_adapter(server: &MockServer) -> Arc<dyn ProviderAdapter> {
    Arc::new(OpenRouterAdapter::new(OpenRouterAdapterConfig {
        client: Client::new(),
        base_url: format!("{}/api/v1", server.uri()),
        api_key: "test-key".to_string(),
        model_ref: openrouter_model_ref(),
        default_headers: reqwest::header::HeaderMap::new(),
        query_params: Vec::new(),
        request_max_retries: 0,
        request_timeout: std::time::Duration::from_secs(30),
    }))
}

async fn mount_openrouter_text_response(
    server: &MockServer,
    expected_body: serde_json::Value,
    response_id: &str,
    message_id: &str,
    text: &str,
) {
    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": response_id,
            "output": [{
                "type": "message",
                "id": message_id,
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": text,
                    "annotations": []
                }]
            }],
            "usage": {
                "input_tokens": 12,
                "output_tokens": 4,
                "total_tokens": 16
            },
            "status": "completed"
        })))
        .expect(1)
        .mount(server)
        .await;
}

async fn mount_openrouter_json_response(
    server: &MockServer,
    expected_body: serde_json::Value,
    response_body: serde_json::Value,
) {
    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .expect(1)
        .mount(server)
        .await;
}

fn runtime_tool_declaration_json(name: &str) -> serde_json::Value {
    json!({
        "type": "function",
        "name": name,
        "description": format!("Runtime tool {name}"),
        "parameters": {
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            },
            "required": ["text"]
        }
    })
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

#[async_trait::async_trait]
impl RuntimeToolExecutor for EchoTool {
    async fn execute(
        &self,
        call: &logos_tool_runtime::PendingToolCall,
    ) -> Result<ToolExecutionOutput, String> {
        Ok(ToolExecutionOutput::success(json!({
            "text": call.arguments()["text"].as_str().unwrap_or_default()
        })))
    }
}

fn openrouter_model_ref() -> ModelRef {
    ModelRef {
        provider_id: "openrouter".to_string(),
        model_id: OPENROUTER_MODEL.to_string(),
    }
}

fn openai_model_ref() -> ModelRef {
    ModelRef {
        provider_id: "openai".to_string(),
        model_id: "gpt-5.2-codex".to_string(),
    }
}

#[derive(Default)]
struct RecordingOpenAiAdapter {
    requests: Arc<Mutex<Vec<logos_runtime_types::InternalTurnRequest>>>,
}

#[async_trait::async_trait]
impl ProviderAdapter for RecordingOpenAiAdapter {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn provider_capabilities(&self) -> logos_runtime_types::ProviderCapabilitySet {
        logos_runtime_types::ProviderCapabilitySet {
            supports_server_side_state: true,
            supports_client_side_history: true,
            supports_structured_output: true,
            supports_context_cache: false,
            supports_runtime_owned_tools: true,
            supports_provider_builtin_tools: false,
            supports_approval_control: true,
            compaction_backend: logos_runtime_types::CompactionBackendKind::NativeRemoteRewrite,
            continuation_mode: logos_runtime_types::ContinuationMode::ServerManaged,
        }
    }

    fn model_capabilities(&self, _model: &ModelRef) -> logos_runtime_types::ModelCapabilitySet {
        logos_runtime_types::ModelCapabilitySet {
            supports_tool_calling: false,
            supports_parallel_tools: false,
            supports_tool_arg_streaming: false,
            supports_provider_native_tool_calls: false,
            supports_runtime_tool_reinjection: false,
            supports_reasoning_deltas: false,
            supports_opaque_reasoning_carryover: false,
            supports_images: false,
        }
    }

    async fn list_models(&self) -> Result<Vec<logos_runtime_types::ModelMeta>, ProviderError> {
        Ok(Vec::new())
    }

    async fn compact_history(
        &self,
        _request: logos_runtime_types::InternalCompactionRequest,
    ) -> Result<logos_runtime_types::InternalCompactionResult, ProviderError> {
        Err(ProviderError::new(
            "openai",
            ProviderErrorKind::UnsupportedCapability,
            "unused in test",
        ))
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        Ok(Box::new(RecordingOpenAiSession {
            requests: Arc::clone(&self.requests),
            response_id: 0,
        }))
    }
}

struct RecordingOpenAiSession {
    requests: Arc<Mutex<Vec<logos_runtime_types::InternalTurnRequest>>>,
    response_id: u64,
}

#[async_trait::async_trait]
impl ProviderSession for RecordingOpenAiSession {
    async fn stream_turn(
        &mut self,
        request: logos_runtime_types::InternalTurnRequest,
    ) -> Result<ProviderEventStream, ProviderError> {
        self.requests.lock().expect("requests lock").push(request);
        self.response_id += 1;
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(InternalEvent::TextDelta {
                text: format!("answer-{}", self.response_id),
            }),
            Ok(InternalEvent::MessageCompleted {
                message_id: Some(format!("msg-{}", self.response_id)),
            }),
            Ok(InternalEvent::ContinuationUpdated {
                handle: ContinuationHandle::OpenAiPreviousResponseId(format!(
                    "resp-{}",
                    self.response_id
                )),
            }),
            Ok(InternalEvent::Completed),
        ])))
    }
}
