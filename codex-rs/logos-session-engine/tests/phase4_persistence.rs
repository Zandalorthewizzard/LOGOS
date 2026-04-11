#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use codex_core::ModelClient;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_otel::SessionTelemetry;
use codex_otel::TelemetryAuthMode;
use core_test_support::load_default_config_for_test;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_output_text_delta;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_compact_user_history_with_summary_once;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::responses::start_websocket_server;
use futures::stream;
use logos_journal::ApprovalJournalState;
use logos_journal::JournalConfig;
use logos_journal::JournalRecordKind;
use logos_journal::JournalStore;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderEventStream;
use logos_provider_api::ProviderSession;
use logos_provider_openai::OpenAiAdapter;
use logos_provider_openai::OpenAiAdapterConfig;
use logos_runtime_types::CompactionBackendKind;
use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationMode;
use logos_runtime_types::InternalCompactionRequest;
use logos_runtime_types::InternalCompactionResult;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelMeta;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ProviderCapabilitySet;
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
use logos_tool_runtime::ToolApprover;
use logos_tool_runtime::ToolExecutionOutput;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use wiremock::MockServer;

const MODEL: &str = "gpt-5.2-codex";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_restart_resume_uses_persisted_continuation_handle() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
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

    let adapter = build_openai_adapter(websocket_provider(&server)).await;
    let mut session = SessionEngine::new(adapter.clone(), openai_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("first turn should succeed");

    let journal = store.open_session(&session_id);
    let view = journal.view().expect("journal view should load");
    assert!(view.snapshots.is_empty());

    let mut resumed = SessionEngine::resume_from_journal(adapter, journal)
        .await
        .expect("resume should succeed");
    resumed
        .run_turn(vec![InternalMessage::user_text("follow up")])
        .await
        .expect("follow up should succeed");

    let second = server
        .connections()
        .get(1)
        .and_then(|connection| connection.first())
        .expect("missing second request")
        .body_json();
    assert_eq!(second["previous_response_id"].as_str(), Some("resp-1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stateless_replay_resume_reconstructs_history_from_same_journal_family() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(RecordingFakeAdapter::new(vec![
        vec![vec![
            Ok(InternalEvent::TextDelta {
                text: "First answer.".to_string(),
            }),
            Ok(InternalEvent::MessageCompleted {
                message_id: Some("msg-1".to_string()),
            }),
            Ok(InternalEvent::Completed),
        ]],
        vec![vec![
            Ok(InternalEvent::TextDelta {
                text: "Second answer.".to_string(),
            }),
            Ok(InternalEvent::MessageCompleted {
                message_id: Some("msg-2".to_string()),
            }),
            Ok(InternalEvent::Completed),
        ]],
    ]));
    let mut session = SessionEngine::new(adapter.clone(), fake_model_ref(), None)
        .await
        .expect("fake session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("first turn should succeed");

    let journal = store.open_session(&session_id);
    let mut resumed = SessionEngine::resume_from_journal(adapter.clone(), journal)
        .await
        .expect("resume should succeed");
    resumed
        .run_turn(vec![InternalMessage::user_text("follow up")])
        .await
        .expect("second turn should succeed");

    let requests = adapter.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[1].messages,
        vec![
            InternalMessage::user_text("hello"),
            InternalMessage::Assistant {
                message_id: Some("msg-1".to_string()),
                parts: vec![ContentPart::Text {
                    text: "First answer.".to_string(),
                }],
            },
            InternalMessage::user_text("follow up"),
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pending_approval_survives_restart() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(RecordingFakeAdapter::new(vec![
        vec![vec![
            Ok(InternalEvent::ToolCallStarted {
                call_id: "call-pending".to_string(),
                name: "logos_echo".to_string(),
            }),
            Ok(InternalEvent::ToolCallArgumentsDelta {
                call_id: "call-pending".to_string(),
                delta: r#"{"text":"hi"}"#.to_string(),
            }),
            Ok(InternalEvent::ToolCallFinished {
                call_id: "call-pending".to_string(),
            }),
            Ok(InternalEvent::Completed),
        ]],
        vec![],
    ]));
    let mut session = SessionEngine::new(adapter.clone(), fake_model_ref(), None)
        .await
        .expect("session should build");
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    session.set_tool_execution_policy(ToolExecutionPolicy {
        runtime_owned_default: ToolApprovalMode::RequireApproval,
        ..ToolExecutionPolicy::default()
    });
    let approver = Arc::new(BlockingApprover::new());
    let entered = approver
        .entered_rx
        .lock()
        .expect("lock entered")
        .take()
        .unwrap();
    session.set_tool_approver(approver.clone());
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    let handle = tokio::spawn(async move {
        session
            .run_turn(vec![InternalMessage::user_text("need approval")])
            .await
    });

    entered.await.expect("approval should enter");
    let journal = store.open_session(&session_id);
    let view = journal.view().expect("view should load");
    assert_eq!(view.pending_approvals().len(), 1);

    handle.abort();

    let resumed = SessionEngine::resume_from_journal(adapter, journal)
        .await
        .expect("resume should succeed");
    assert_eq!(resumed.approvals().len(), 1);
    assert_eq!(resumed.approvals()[0].state, ApprovalJournalState::Pending);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolved_approval_survives_restart() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(RecordingFakeAdapter::new(vec![
        vec![
            vec![
                Ok(InternalEvent::ToolCallStarted {
                    call_id: "call-approved".to_string(),
                    name: "logos_echo".to_string(),
                }),
                Ok(InternalEvent::ToolCallArgumentsDelta {
                    call_id: "call-approved".to_string(),
                    delta: r#"{"text":"hi"}"#.to_string(),
                }),
                Ok(InternalEvent::ToolCallFinished {
                    call_id: "call-approved".to_string(),
                }),
                Ok(InternalEvent::Completed),
            ],
            vec![
                Ok(InternalEvent::TextDelta {
                    text: "Approved.".to_string(),
                }),
                Ok(InternalEvent::MessageCompleted {
                    message_id: Some("msg-approved".to_string()),
                }),
                Ok(InternalEvent::Completed),
            ],
        ],
        vec![],
    ]));
    let mut session = SessionEngine::new(adapter.clone(), fake_model_ref(), None)
        .await
        .expect("session should build");
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("approve")])
        .await
        .expect("turn should succeed");

    let resumed = SessionEngine::resume_from_journal(adapter, store.open_session(&session_id))
        .await
        .expect("resume should succeed");
    assert_eq!(resumed.approvals().len(), 1);
    assert_eq!(resumed.approvals()[0].state, ApprovalJournalState::Approved);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tool_terminal_records_survive_restart_for_all_branch_shapes() {
    assert_terminal_after_restart(
        BranchScenario::Completed,
        ToolCallTerminalState::CompletedWithResult,
        ToolCallResultDisposition::Success,
    )
    .await;
    assert_terminal_after_restart(
        BranchScenario::Denied,
        ToolCallTerminalState::DeniedByApproval,
        ToolCallResultDisposition::Error,
    )
    .await;
    assert_terminal_after_restart(
        BranchScenario::Failed,
        ToolCallTerminalState::FailedBeforeResult,
        ToolCallResultDisposition::Error,
    )
    .await;
    assert_terminal_after_restart(
        BranchScenario::Interrupted,
        ToolCallTerminalState::Interrupted,
        ToolCallResultDisposition::None,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compaction_checkpoint_changes_replay_boundary_without_provider_snapshots() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let server = start_mock_server().await;
    let _turn1 = mount_sse_once(
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

    let adapter = build_openai_adapter(http_provider(&server)).await;
    let mut session = SessionEngine::new(adapter.clone(), openai_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("turn should succeed");
    session
        .compact_history()
        .await
        .expect("compaction should succeed");

    let view = store
        .open_session(&session_id)
        .view()
        .expect("journal view should load");
    assert!(view.snapshots.is_empty());
    assert_eq!(
        view.history,
        vec![
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
    let records = store
        .open_session(&session_id)
        .records()
        .expect("records should load");
    let checkpoint_sequence_id = records
        .iter()
        .find_map(|record| match &record.kind {
            JournalRecordKind::MessageAppended {
                message: InternalMessage::CompactionCheckpoint { .. },
                ..
            } => Some(record.sequence_id),
            _ => None,
        })
        .expect("checkpoint record should exist");
    let compaction = records
        .iter()
        .find_map(|record| match &record.kind {
            JournalRecordKind::CompactionApplied {
                first_retained_sequence_id,
                checkpoint_sequence_id,
                ..
            } => Some((*first_retained_sequence_id, *checkpoint_sequence_id)),
            _ => None,
        })
        .expect("compaction applied should exist");
    assert!(compaction.0 < compaction.1);
    assert_eq!(compaction.1, checkpoint_sequence_id);
    let body = compact.single_request().body_json();
    assert_eq!(body["input"][0]["role"].as_str(), Some("user"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_compaction_without_checkpoint_fails_without_mutation() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(CompactionFakeAdapter::new(vec![vec![vec![
        Ok(InternalEvent::TextDelta {
            text: "First answer.".to_string(),
        }),
        Ok(InternalEvent::MessageCompleted {
            message_id: Some("msg-1".to_string()),
        }),
        Ok(InternalEvent::ContinuationUpdated {
            handle: logos_runtime_types::ContinuationHandle::OpenAiPreviousResponseId(
                "resp-1".to_string(),
            ),
        }),
        Ok(InternalEvent::Completed),
    ]]]));
    let mut session = SessionEngine::new(adapter.clone(), openai_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    session
        .run_turn(vec![InternalMessage::user_text("hello")])
        .await
        .expect("turn should succeed");
    let history_before = session.history().to_vec();
    let continuation_before = session.continuation().cloned();
    let records_before = store
        .open_session(&session_id)
        .records()
        .expect("records should load")
        .len();

    let error = session
        .compact_history()
        .await
        .expect_err("missing checkpoint should fail compaction");

    assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
    assert_eq!(session.history(), history_before.as_slice());
    assert_eq!(session.continuation(), continuation_before.as_ref());

    let records = store
        .open_session(&session_id)
        .records()
        .expect("records should load");
    assert_eq!(records.len(), records_before);
    assert!(
        !records
            .iter()
            .any(|record| matches!(record.kind, JournalRecordKind::CompactionApplied { .. }))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn locally_rejected_turn_does_not_write_orphan_started_turn() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(RecordingFakeAdapter::new(vec![vec![]]));
    let mut session = SessionEngine::new(adapter, fake_model_ref(), None)
        .await
        .expect("session should build");
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

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
        .expect_err("provider-native tool should be rejected before turn start");

    assert_eq!(error.kind, ProviderErrorKind::UnsupportedCapability);
    let records = store
        .open_session(&session_id)
        .records()
        .expect("records should load");
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0].kind,
        JournalRecordKind::SessionOpened(_)
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn interrupted_turn_does_not_rewrite_older_pending_approval() {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let adapter = Arc::new(RecordingFakeAdapter::new(vec![
        vec![vec![
            Ok(InternalEvent::ToolCallStarted {
                call_id: "call-pending".to_string(),
                name: "logos_echo".to_string(),
            }),
            Ok(InternalEvent::ToolCallArgumentsDelta {
                call_id: "call-pending".to_string(),
                delta: r#"{"text":"hi"}"#.to_string(),
            }),
            Ok(InternalEvent::ToolCallFinished {
                call_id: "call-pending".to_string(),
            }),
            Ok(InternalEvent::Completed),
        ]],
        vec![vec![
            Ok(InternalEvent::ToolCallStarted {
                call_id: "call-interrupted".to_string(),
                name: "logos_echo".to_string(),
            }),
            Ok(InternalEvent::ToolCallArgumentsDelta {
                call_id: "call-interrupted".to_string(),
                delta: r#"{"text":"bye"}"#.to_string(),
            }),
            Err(ProviderError::new(
                "fake",
                ProviderErrorKind::Transport,
                "stream interrupted",
            )),
        ]],
        vec![],
    ]));
    let mut session = SessionEngine::new(adapter.clone(), fake_model_ref(), None)
        .await
        .expect("session should build");
    session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
    session.set_tool_execution_policy(ToolExecutionPolicy {
        runtime_owned_default: ToolApprovalMode::RequireApproval,
        ..ToolExecutionPolicy::default()
    });
    let approver = Arc::new(BlockingApprover::new());
    let entered = approver
        .entered_rx
        .lock()
        .expect("lock entered")
        .take()
        .unwrap();
    session.set_tool_approver(approver);
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    let handle = tokio::spawn(async move {
        session
            .run_turn(vec![InternalMessage::user_text("need approval")])
            .await
    });
    entered.await.expect("approval should enter");
    handle.abort();

    let mut resumed =
        SessionEngine::resume_from_journal(adapter.clone(), store.open_session(&session_id))
            .await
            .expect("resume should succeed");
    let error = resumed
        .run_turn(vec![InternalMessage::user_text("interrupt next turn")])
        .await
        .expect_err("turn should fail");
    assert_eq!(error.kind, ProviderErrorKind::Transport);

    let resumed_again =
        SessionEngine::resume_from_journal(adapter, store.open_session(&session_id))
            .await
            .expect("resume should succeed");
    assert_eq!(resumed_again.approvals().len(), 1);
    assert_eq!(resumed_again.approvals()[0].request.call_id, "call-pending");
    assert_eq!(
        resumed_again.approvals()[0].state,
        ApprovalJournalState::Pending
    );
}

async fn assert_terminal_after_restart(
    scenario: BranchScenario,
    expected_terminal: ToolCallTerminalState,
    expected_result: ToolCallResultDisposition,
) {
    let dir = TempDir::new().expect("tempdir should exist");
    let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("journal store");
    let (adapter, call_id, tool_name) = scenario.adapter();
    let mut session = SessionEngine::new(adapter.clone(), fake_model_ref(), None)
        .await
        .expect("session should build");
    scenario.configure_session(&mut session);
    session
        .create_and_attach_journal(&store)
        .expect("journal should attach");
    let session_id = session.session_id().to_string();

    let result = session
        .run_turn(vec![InternalMessage::user_text("run tool")])
        .await;
    match scenario {
        BranchScenario::Interrupted => {
            result.expect_err("interrupted branch should fail");
        }
        _ => {
            result.expect("branch should complete");
        }
    }

    let resumed = SessionEngine::resume_from_journal(adapter, store.open_session(&session_id))
        .await
        .expect("resume should succeed");
    assert_eq!(
        resumed.tool_call_records(),
        &[logos_runtime_types::ToolCallRecord {
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            execution: ToolExecutionKind::RuntimeOwned,
            terminal_state: expected_terminal,
            mcp_correlation: None,
            result_disposition: expected_result,
        }]
    );
}

#[derive(Clone, Copy)]
enum BranchScenario {
    Completed,
    Denied,
    Failed,
    Interrupted,
}

impl BranchScenario {
    fn adapter(self) -> (Arc<RecordingFakeAdapter>, &'static str, &'static str) {
        match self {
            Self::Completed => (
                Arc::new(RecordingFakeAdapter::new(vec![
                    vec![
                        vec![
                            Ok(InternalEvent::ToolCallStarted {
                                call_id: "call-completed".to_string(),
                                name: "logos_echo".to_string(),
                            }),
                            Ok(InternalEvent::ToolCallArgumentsDelta {
                                call_id: "call-completed".to_string(),
                                delta: r#"{"text":"hi"}"#.to_string(),
                            }),
                            Ok(InternalEvent::ToolCallFinished {
                                call_id: "call-completed".to_string(),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                        vec![
                            Ok(InternalEvent::TextDelta {
                                text: "Completed.".to_string(),
                            }),
                            Ok(InternalEvent::MessageCompleted {
                                message_id: Some("msg-completed".to_string()),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                    ],
                    vec![],
                ])),
                "call-completed",
                "logos_echo",
            ),
            Self::Denied => (
                Arc::new(RecordingFakeAdapter::new(vec![
                    vec![
                        vec![
                            Ok(InternalEvent::ToolCallStarted {
                                call_id: "call-denied".to_string(),
                                name: "logos_echo".to_string(),
                            }),
                            Ok(InternalEvent::ToolCallArgumentsDelta {
                                call_id: "call-denied".to_string(),
                                delta: r#"{"text":"hi"}"#.to_string(),
                            }),
                            Ok(InternalEvent::ToolCallFinished {
                                call_id: "call-denied".to_string(),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                        vec![
                            Ok(InternalEvent::TextDelta {
                                text: "Denied.".to_string(),
                            }),
                            Ok(InternalEvent::MessageCompleted {
                                message_id: Some("msg-denied".to_string()),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                    ],
                    vec![],
                ])),
                "call-denied",
                "logos_echo",
            ),
            Self::Failed => (
                Arc::new(RecordingFakeAdapter::new(vec![
                    vec![
                        vec![
                            Ok(InternalEvent::ToolCallStarted {
                                call_id: "call-failed".to_string(),
                                name: "logos_fail".to_string(),
                            }),
                            Ok(InternalEvent::ToolCallArgumentsDelta {
                                call_id: "call-failed".to_string(),
                                delta: r#"{"text":"boom"}"#.to_string(),
                            }),
                            Ok(InternalEvent::ToolCallFinished {
                                call_id: "call-failed".to_string(),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                        vec![
                            Ok(InternalEvent::TextDelta {
                                text: "Failed.".to_string(),
                            }),
                            Ok(InternalEvent::MessageCompleted {
                                message_id: Some("msg-failed".to_string()),
                            }),
                            Ok(InternalEvent::Completed),
                        ],
                    ],
                    vec![],
                ])),
                "call-failed",
                "logos_fail",
            ),
            Self::Interrupted => (
                Arc::new(RecordingFakeAdapter::new(vec![
                    vec![vec![
                        Ok(InternalEvent::ToolCallStarted {
                            call_id: "call-interrupted".to_string(),
                            name: "logos_echo".to_string(),
                        }),
                        Ok(InternalEvent::ToolCallArgumentsDelta {
                            call_id: "call-interrupted".to_string(),
                            delta: r#"{"text":"hi"}"#.to_string(),
                        }),
                        Err(ProviderError::new(
                            "fake",
                            ProviderErrorKind::Transport,
                            "interrupted",
                        )),
                    ]],
                    vec![],
                ])),
                "call-interrupted",
                "logos_echo",
            ),
        }
    }

    fn configure_session(self, session: &mut SessionEngine) {
        match self {
            Self::Completed => {
                session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
            }
            Self::Denied => {
                session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
                session.set_tool_execution_policy(ToolExecutionPolicy {
                    runtime_owned_default: ToolApprovalMode::RequireApproval,
                    ..ToolExecutionPolicy::default()
                });
                session.set_tool_approver(Arc::new(DenyApprover));
            }
            Self::Failed => {
                session.register_runtime_tool(runtime_tool_def("logos_fail"), Arc::new(FailTool));
            }
            Self::Interrupted => {
                session.register_runtime_tool(runtime_tool_def("logos_echo"), Arc::new(EchoTool));
            }
        }
    }
}

fn runtime_tool_def(name: &str) -> InternalToolDef {
    InternalToolDef {
        name: name.to_string(),
        description: format!("Runtime tool {name}"),
        input_schema: json!({
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {"text": {"type": "string"}}
        })),
        execution: ToolExecutionKind::RuntimeOwned,
        metadata: Default::default(),
    }
}

struct EchoTool;

#[async_trait]
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

struct FailTool;

#[async_trait]
impl RuntimeToolExecutor for FailTool {
    async fn execute(
        &self,
        _call: &logos_tool_runtime::PendingToolCall,
    ) -> Result<ToolExecutionOutput, String> {
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

struct BlockingApprover {
    entered_tx: Mutex<Option<oneshot::Sender<()>>>,
    entered_rx: Mutex<Option<oneshot::Receiver<()>>>,
    release: Notify,
}

impl BlockingApprover {
    fn new() -> Self {
        let (tx, rx) = oneshot::channel();
        Self {
            entered_tx: Mutex::new(Some(tx)),
            entered_rx: Mutex::new(Some(rx)),
            release: Notify::new(),
        }
    }
}

#[async_trait]
impl ToolApprover for BlockingApprover {
    async fn decide(
        &self,
        _request: &ToolApprovalRequest,
        _mode: ToolApprovalMode,
    ) -> ToolApprovalOutcome {
        if let Some(tx) = self.entered_tx.lock().expect("entered tx lock").take() {
            let _ = tx.send(());
        }
        self.release.notified().await;
        ToolApprovalOutcome::Approved
    }
}

type FakeTurn = Vec<Result<InternalEvent, ProviderError>>;
type FakeSessionScript = VecDeque<FakeTurn>;
type FakeSessionQueue = Arc<Mutex<VecDeque<FakeSessionScript>>>;

#[derive(Clone)]
struct RecordingFakeAdapter {
    sessions: FakeSessionQueue,
    requests: Arc<Mutex<Vec<logos_runtime_types::InternalTurnRequest>>>,
}

#[derive(Clone)]
struct CompactionFakeAdapter {
    sessions: FakeSessionQueue,
}

impl CompactionFakeAdapter {
    fn new(sessions: Vec<Vec<Vec<Result<InternalEvent, ProviderError>>>>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(
                sessions.into_iter().map(VecDeque::from).collect(),
            )),
        }
    }
}

#[async_trait]
impl ProviderAdapter for CompactionFakeAdapter {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn provider_capabilities(&self) -> ProviderCapabilitySet {
        ProviderCapabilitySet {
            supports_server_side_state: true,
            supports_client_side_history: true,
            supports_structured_output: false,
            supports_context_cache: false,
            supports_runtime_owned_tools: true,
            supports_provider_builtin_tools: false,
            supports_approval_control: true,
            compaction_backend: CompactionBackendKind::NativeRemoteRewrite,
            continuation_mode: ContinuationMode::ServerManaged,
        }
    }

    fn model_capabilities(&self, _model: &ModelRef) -> ModelCapabilitySet {
        ModelCapabilitySet {
            supports_tool_calling: true,
            supports_parallel_tools: false,
            supports_tool_arg_streaming: false,
            supports_provider_native_tool_calls: false,
            supports_runtime_tool_reinjection: true,
            supports_reasoning_deltas: false,
            supports_opaque_reasoning_carryover: false,
            supports_images: false,
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError> {
        Ok(Vec::new())
    }

    async fn compact_history(
        &self,
        _request: InternalCompactionRequest,
    ) -> Result<InternalCompactionResult, ProviderError> {
        Ok(InternalCompactionResult {
            replacement_messages: vec![
                InternalMessage::User {
                    parts: vec![ContentPart::Text {
                        text: "hello".to_string(),
                    }],
                },
                InternalMessage::Assistant {
                    message_id: Some("msg-compact".to_string()),
                    parts: vec![ContentPart::Text {
                        text: "compacted".to_string(),
                    }],
                },
            ],
        })
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        let turns = self
            .sessions
            .lock()
            .expect("sessions lock")
            .pop_front()
            .expect("missing session script");
        Ok(Box::new(RecordingFakeSession {
            turns,
            requests: Arc::new(Mutex::new(Vec::new())),
        }))
    }
}

impl RecordingFakeAdapter {
    fn new(sessions: Vec<Vec<Vec<Result<InternalEvent, ProviderError>>>>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(
                sessions.into_iter().map(VecDeque::from).collect(),
            )),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<logos_runtime_types::InternalTurnRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

#[async_trait]
impl ProviderAdapter for RecordingFakeAdapter {
    fn provider_id(&self) -> &'static str {
        "fake"
    }

    fn provider_capabilities(&self) -> ProviderCapabilitySet {
        ProviderCapabilitySet {
            supports_server_side_state: false,
            supports_client_side_history: true,
            supports_structured_output: false,
            supports_context_cache: false,
            supports_runtime_owned_tools: true,
            supports_provider_builtin_tools: false,
            supports_approval_control: true,
            compaction_backend: CompactionBackendKind::Disabled,
            continuation_mode: ContinuationMode::ClientManaged,
        }
    }

    fn model_capabilities(&self, _model: &ModelRef) -> ModelCapabilitySet {
        ModelCapabilitySet {
            supports_tool_calling: true,
            supports_parallel_tools: false,
            supports_tool_arg_streaming: false,
            supports_provider_native_tool_calls: false,
            supports_runtime_tool_reinjection: true,
            supports_reasoning_deltas: false,
            supports_opaque_reasoning_carryover: false,
            supports_images: false,
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError> {
        Ok(Vec::new())
    }

    async fn compact_history(
        &self,
        _request: InternalCompactionRequest,
    ) -> Result<InternalCompactionResult, ProviderError> {
        Err(ProviderError::new(
            "fake",
            ProviderErrorKind::UnsupportedCapability,
            "compaction unsupported",
        ))
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        let turns = self
            .sessions
            .lock()
            .expect("sessions lock")
            .pop_front()
            .expect("missing session script");
        Ok(Box::new(RecordingFakeSession {
            turns,
            requests: Arc::clone(&self.requests),
        }))
    }
}

struct RecordingFakeSession {
    turns: VecDeque<Vec<Result<InternalEvent, ProviderError>>>,
    requests: Arc<Mutex<Vec<logos_runtime_types::InternalTurnRequest>>>,
}

#[async_trait]
impl ProviderSession for RecordingFakeSession {
    async fn stream_turn(
        &mut self,
        request: logos_runtime_types::InternalTurnRequest,
    ) -> Result<ProviderEventStream, ProviderError> {
        self.requests.lock().expect("requests lock").push(request);
        let turn = self.turns.pop_front().expect("turn already consumed");
        Ok(Box::pin(stream::iter(turn)))
    }
}

fn fake_model_ref() -> ModelRef {
    ModelRef {
        provider_id: "fake".to_string(),
        model_id: MODEL.to_string(),
    }
}

fn openai_model_ref() -> ModelRef {
    ModelRef {
        provider_id: "openai".to_string(),
        model_id: MODEL.to_string(),
    }
}

fn websocket_provider(
    server: &core_test_support::responses::WebSocketTestServer,
) -> ModelProviderInfo {
    ModelProviderInfo {
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
    }
}

fn http_provider(server: &MockServer) -> ModelProviderInfo {
    ModelProviderInfo {
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
    }
}

async fn build_openai_adapter(provider: ModelProviderInfo) -> Arc<dyn ProviderAdapter> {
    let codex_home = TempDir::new().expect("tempdir should exist");
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

    Arc::new(OpenAiAdapter::new(OpenAiAdapterConfig {
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
        model_ref: openai_model_ref(),
        session_telemetry,
        reasoning_effort: None,
        reasoning_summary: codex_protocol::config_types::ReasoningSummary::Auto,
        service_tier: None,
    }))
}
