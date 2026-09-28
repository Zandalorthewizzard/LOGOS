use anyhow::Result;
use app_test_support::McpProcess;
use app_test_support::to_response;
use base64::Engine;
use codex_app_server_protocol::CommandExecutionOutputDeltaNotification;
use codex_app_server_protocol::CommandExecutionSource;
use codex_app_server_protocol::CommandExecutionStatus;
use codex_app_server_protocol::DynamicToolCallStatus;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::JSONRPCError;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ReviewDelivery;
use codex_app_server_protocol::ReviewStartParams;
use codex_app_server_protocol::ReviewTarget;
use codex_app_server_protocol::ThreadArchiveParams;
use codex_app_server_protocol::ThreadBackgroundTerminalsCleanParams;
use codex_app_server_protocol::ThreadBackgroundTerminalsCleanResponse;
use codex_app_server_protocol::ThreadCompactStartParams;
use codex_app_server_protocol::ThreadDecrementElicitationParams;
use codex_app_server_protocol::ThreadDecrementElicitationResponse;
use codex_app_server_protocol::ThreadForkParams;
use codex_app_server_protocol::ThreadIncrementElicitationParams;
use codex_app_server_protocol::ThreadIncrementElicitationResponse;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadListParams;
use codex_app_server_protocol::ThreadListResponse;
use codex_app_server_protocol::ThreadMetadataGitInfoUpdateParams;
use codex_app_server_protocol::ThreadMetadataUpdateParams;
use codex_app_server_protocol::ThreadMetadataUpdateResponse;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadSetNameParams;
use codex_app_server_protocol::ThreadSetNameResponse;
use codex_app_server_protocol::ThreadShellCommandParams;
use codex_app_server_protocol::ThreadShellCommandResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadStatus;
use codex_app_server_protocol::ThreadUnarchiveParams;
use codex_app_server_protocol::ThreadUnarchiveResponse;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_core::shell::default_user_shell;
use codex_protocol::ThreadId;
use logos_session_state::SqliteThreadRegistry;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::TempDir;
use tokio::time::timeout;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

#[cfg(windows)]
const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(25);
#[cfg(not(windows))]
const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[tokio::test]
async fn app_server_openrouter_turn_start_uses_shared_lane_and_reaches_tool_roundtrip() -> Result<()>
{
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(SequencedOpenRouterResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            model: Some("openai/o4-mini".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let turn_req = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            input: vec![V2UserInput::Text {
                text: "inspect note.txt".to_string(),
                text_elements: Vec::new(),
            }],
            cwd: Some(workspace.path().to_path_buf()),
            model: Some("openai/o4-mini".to_string()),
            approval_policy: Some(codex_app_server_protocol::AskForApproval::Never),
            approvals_reviewer: Some(codex_app_server_protocol::ApprovalsReviewer::User),
            sandbox_policy: Some(codex_app_server_protocol::SandboxPolicy::ReadOnly {
                access: Default::default(),
                network_access: false,
            }),
            ..Default::default()
        })
        .await?;
    let turn_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(turn_req)),
    )
    .await??;
    let TurnStartResponse { turn } = to_response::<TurnStartResponse>(turn_resp)?;

    let mut saw_tool = None;
    let mut saw_assistant = None;
    let completed: TurnCompletedNotification = timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            match mcp.read_next_message().await? {
                JSONRPCMessage::Notification(notification)
                    if notification.method == "item/completed" =>
                {
                    let item: ItemCompletedNotification = serde_json::from_value(
                        notification.params.expect("item/completed params"),
                    )?;
                    match item.item {
                        ThreadItem::DynamicToolCall { tool, status, .. } => {
                            saw_tool = Some((tool, status));
                        }
                        ThreadItem::AgentMessage { text, .. } => {
                            saw_assistant = Some(text);
                        }
                        _ => {}
                    }
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" =>
                {
                    let completed: TurnCompletedNotification = serde_json::from_value(
                        notification.params.expect("turn/completed params"),
                    )?;
                    return Ok::<TurnCompletedNotification, anyhow::Error>(completed);
                }
                _ => {}
            }
        }
    })
    .await??;

    assert_eq!(turn.id, completed.turn.id);
    assert_eq!(
        completed.turn.status,
        codex_app_server_protocol::TurnStatus::Completed
    );
    assert_eq!(
        saw_tool,
        Some((
            "workspace_read_file".to_string(),
            DynamicToolCallStatus::Completed,
        ))
    );
    assert_eq!(
        saw_assistant,
        Some("The file says hello from workspace.".to_string())
    );

    let requests = server.received_requests().await.expect("received requests");
    assert_eq!(requests.len(), 2);
    let first: Value = serde_json::from_slice(&requests[0].body)?;
    let second: Value = serde_json::from_slice(&requests[1].body)?;
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

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_resume_reconnects_to_existing_thread() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let resume_req = mcp
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: thread.id.clone(),
            ..Default::default()
        })
        .await?;
    let resume_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(resume_req)),
    )
    .await??;
    let ThreadResumeResponse {
        thread: resumed_thread,
        ..
    } = to_response::<ThreadResumeResponse>(resume_resp)?;
    assert_eq!(resumed_thread.id, thread.id);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_compact_is_explicitly_unsupported() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let compact_req = mcp
        .send_thread_compact_start_request(ThreadCompactStartParams {
            thread_id: thread.id,
        })
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(compact_req)),
    )
    .await??;
    assert!(error.message.contains("does not support compact yet"));
    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_thread_management_support_is_consistent() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: None,
            source_kinds: None,
            archived: None,
            cwd: None,
            search_term: None,
        })
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let ThreadListResponse { data, .. } = to_response::<ThreadListResponse>(list_resp)?;
    assert!(data.iter().any(|candidate| candidate.id == thread.id));

    let set_name_req = mcp
        .send_thread_set_name_request(ThreadSetNameParams {
            thread_id: thread.id.clone(),
            name: "OpenRouter Beta Thread".to_string(),
        })
        .await?;
    let set_name_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(set_name_req)),
    )
    .await??;
    to_response::<ThreadSetNameResponse>(set_name_resp)?;

    let metadata_req = mcp
        .send_thread_metadata_update_request(ThreadMetadataUpdateParams {
            thread_id: thread.id.clone(),
            git_info: Some(ThreadMetadataGitInfoUpdateParams {
                sha: Some(Some("abc123".to_string())),
                branch: Some(Some("feature/openrouter".to_string())),
                origin_url: Some(Some("https://example.com/repo.git".to_string())),
            }),
        })
        .await?;
    let metadata_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(metadata_req)),
    )
    .await??;
    let ThreadMetadataUpdateResponse {
        thread: updated_thread,
    } = to_response::<ThreadMetadataUpdateResponse>(metadata_resp)?;
    assert_eq!(
        updated_thread.name.as_deref(),
        Some("OpenRouter Beta Thread")
    );
    assert_eq!(
        updated_thread
            .git_info
            .as_ref()
            .and_then(|info| info.sha.as_deref()),
        Some("abc123")
    );

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread.id.clone(),
            include_turns: false,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse {
        thread: read_thread,
    } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(read_thread.name.as_deref(), Some("OpenRouter Beta Thread"));
    assert_eq!(
        read_thread
            .git_info
            .as_ref()
            .and_then(|info| info.branch.as_deref()),
        Some("feature/openrouter")
    );

    let inc_req = mcp
        .send_thread_increment_elicitation_request(ThreadIncrementElicitationParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let inc_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(inc_req)),
    )
    .await??;
    let ThreadIncrementElicitationResponse { count, paused } =
        to_response::<ThreadIncrementElicitationResponse>(inc_resp)?;
    assert_eq!(count, 1);
    assert!(paused);

    let dec_req = mcp
        .send_thread_decrement_elicitation_request(ThreadDecrementElicitationParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let dec_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(dec_req)),
    )
    .await??;
    let ThreadDecrementElicitationResponse { count, paused } =
        to_response::<ThreadDecrementElicitationResponse>(dec_resp)?;
    assert_eq!(count, 0);
    assert!(!paused);

    let archive_req = mcp
        .send_thread_archive_request(ThreadArchiveParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let archive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archive_req)),
    )
    .await??;
    let _: JSONRPCResponse = archive_resp;
    let archived_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(true),
            cwd: None,
            search_term: None,
        })
        .await?;
    let archived_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archived_list_req)),
    )
    .await??;
    let ThreadListResponse {
        data: archived_data,
        ..
    } = to_response::<ThreadListResponse>(archived_list_resp)?;
    assert!(
        archived_data
            .iter()
            .any(|candidate| candidate.id == thread.id)
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_registry_import_supports_name_and_metadata_after_restart()
-> Result<()> {
    let workspace = TempDir::new()?;
    let server = MockServer::start().await;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;
        thread.id
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let set_name_req = mcp
        .send_thread_set_name_request(ThreadSetNameParams {
            thread_id: thread_id.clone(),
            name: "Recovered OpenRouter Thread".to_string(),
        })
        .await?;
    let set_name_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(set_name_req)),
    )
    .await??;
    to_response::<ThreadSetNameResponse>(set_name_resp)?;

    let metadata_req = mcp
        .send_thread_metadata_update_request(ThreadMetadataUpdateParams {
            thread_id: thread_id.clone(),
            git_info: Some(ThreadMetadataGitInfoUpdateParams {
                sha: Some(Some("def456".to_string())),
                branch: Some(Some("feature/restarted-openrouter".to_string())),
                origin_url: Some(Some("https://example.com/restarted.git".to_string())),
            }),
        })
        .await?;
    let metadata_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(metadata_req)),
    )
    .await??;
    let ThreadMetadataUpdateResponse { thread } =
        to_response::<ThreadMetadataUpdateResponse>(metadata_resp)?;
    assert_eq!(thread.name.as_deref(), Some("Recovered OpenRouter Thread"));
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|info| info.branch.as_deref()),
        Some("feature/restarted-openrouter")
    );

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.clone(),
            include_turns: false,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.id, thread_id);
    assert_eq!(thread.name.as_deref(), Some("Recovered OpenRouter Thread"));
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|info| info.sha.as_deref()),
        Some("def456")
    );

    let list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: None,
            cwd: None,
            search_term: None,
        })
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let ThreadListResponse { data, .. } = to_response::<ThreadListResponse>(list_resp)?;
    assert!(data.iter().any(|candidate| {
        candidate.id == thread_id
            && candidate.name.as_deref() == Some("Recovered OpenRouter Thread")
    }));

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_turn_metadata_stays_fresh_after_restart() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(ToolRoundTripResponder)
        .expect(2)
        .mount(&server)
        .await;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

        let turn_req = mcp
            .send_turn_start_request(TurnStartParams {
                thread_id: thread.id.clone(),
                input: vec![V2UserInput::Text {
                    text: "inspect note.txt".to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            })
            .await?;
        let _ = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(turn_req)),
        )
        .await??;

        let _: TurnCompletedNotification = serde_json::from_value(
            timeout(
                DEFAULT_READ_TIMEOUT,
                mcp.read_stream_until_notification_message("turn/completed"),
            )
            .await??
            .params
            .expect("turn/completed params"),
        )?;

        thread.id
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.clone(),
            include_turns: false,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.id, thread_id);
    assert_eq!(thread.preview, "The file says hello from workspace.");
    assert_eq!(thread.status, ThreadStatus::NotLoaded);
    assert!(thread.updated_at >= thread.created_at);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_thread_list_uses_registry_metadata_after_restart() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(ToolRoundTripResponder)
        .expect(4)
        .mount(&server)
        .await;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let (first_thread_id, second_thread_id) = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let first_thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let first_thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(first_thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } =
            to_response::<ThreadStartResponse>(first_thread_resp)?;

        let first_turn_req = mcp
            .send_turn_start_request(TurnStartParams {
                thread_id: thread.id.clone(),
                input: vec![V2UserInput::Text {
                    text: "inspect note.txt".to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            })
            .await?;
        let _ = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(first_turn_req)),
        )
        .await??;
        let _: TurnCompletedNotification = serde_json::from_value(
            timeout(
                DEFAULT_READ_TIMEOUT,
                mcp.read_stream_until_notification_message("turn/completed"),
            )
            .await??
            .params
            .expect("turn/completed params"),
        )?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let second_thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let second_thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(second_thread_req)),
        )
        .await??;
        let ThreadStartResponse {
            thread: second_thread,
            ..
        } = to_response::<ThreadStartResponse>(second_thread_resp)?;

        let second_turn_req = mcp
            .send_turn_start_request(TurnStartParams {
                thread_id: second_thread.id.clone(),
                input: vec![V2UserInput::Text {
                    text: "inspect note.txt".to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            })
            .await?;
        let _ = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(second_turn_req)),
        )
        .await??;
        let _: TurnCompletedNotification = serde_json::from_value(
            timeout(
                DEFAULT_READ_TIMEOUT,
                mcp.read_stream_until_notification_message("turn/completed"),
            )
            .await??
            .params
            .expect("turn/completed params"),
        )?;

        assert_ne!(thread.id, second_thread.id);
        (thread.id, second_thread.id)
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: None,
            cwd: None,
            search_term: None,
        })
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let ThreadListResponse { data, .. } = to_response::<ThreadListResponse>(list_resp)?;
    assert!(data.len() >= 2);
    assert_eq!(data[0].id, second_thread_id);
    assert_eq!(data[1].id, first_thread_id);
    assert_eq!(data[0].preview, "The file says hello from workspace.");
    assert!(data[0].updated_at >= data[1].updated_at);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_shell_command_runs_as_standalone_turn() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let (command, expected_output) = current_shell_output_command("hello from openrouter")?;
    let shell_req = mcp
        .send_thread_shell_command_request(ThreadShellCommandParams {
            thread_id: thread.id.clone(),
            command,
        })
        .await?;
    let shell_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(shell_req)),
    )
    .await??;
    let _: ThreadShellCommandResponse = to_response::<ThreadShellCommandResponse>(shell_resp)?;

    let started = wait_for_command_execution_started(&mut mcp).await?;
    assert_eq!(started.thread_id, thread.id);
    let command_id = match &started.item {
        ThreadItem::CommandExecution {
            id, source, status, ..
        } => {
            assert_eq!(source, &CommandExecutionSource::UserShell);
            assert_eq!(status, &CommandExecutionStatus::InProgress);
            id.clone()
        }
        _ => unreachable!("expected command execution"),
    };

    let output = wait_for_command_execution_output(&mut mcp, &command_id).await?;
    assert_eq!(output, expected_output);

    let completed = wait_for_command_execution_completed(&mut mcp, &command_id).await?;
    match completed.item {
        ThreadItem::CommandExecution {
            source,
            status,
            aggregated_output,
            exit_code,
            ..
        } => {
            assert_eq!(source, CommandExecutionSource::UserShell);
            assert!(matches!(
                status,
                CommandExecutionStatus::Completed | CommandExecutionStatus::Failed
            ));
            assert_eq!(aggregated_output.as_deref(), Some(expected_output.as_str()));
            assert!(exit_code.is_some());
        }
        _ => unreachable!("expected command execution"),
    }

    let turn_completed: TurnCompletedNotification = serde_json::from_value(
        timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_notification_message("turn/completed"),
        )
        .await??
        .params
        .expect("turn/completed params"),
    )?;
    assert_eq!(
        turn_completed.turn.status,
        codex_app_server_protocol::TurnStatus::Completed
    );

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread.id,
            include_turns: true,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.turns.len(), 1);
    assert!(
        thread.turns[0].items.iter().any(|item| {
            matches!(
                item,
                ThreadItem::CommandExecution {
                    source: CommandExecutionSource::UserShell,
                    aggregated_output,
                    ..
                } if aggregated_output.as_deref() == Some(expected_output.as_str())
            )
        }),
        "expected standalone OpenRouter shell command to persist into thread history"
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_shell_metadata_stays_fresh_after_restart() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let (first_thread_id, second_thread_id) = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let first_thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let first_thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(first_thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } =
            to_response::<ThreadStartResponse>(first_thread_resp)?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let second_thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let second_thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(second_thread_req)),
        )
        .await??;
        let ThreadStartResponse {
            thread: second_thread,
            ..
        } = to_response::<ThreadStartResponse>(second_thread_resp)?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let (command, _) = current_shell_output_command("registry freshness")?;
        let shell_req = mcp
            .send_thread_shell_command_request(ThreadShellCommandParams {
                thread_id: thread.id.clone(),
                command,
            })
            .await?;
        let shell_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(shell_req)),
        )
        .await??;
        let _: ThreadShellCommandResponse = to_response::<ThreadShellCommandResponse>(shell_resp)?;

        let started = wait_for_command_execution_started(&mut mcp).await?;
        let command_id = match started.item {
            ThreadItem::CommandExecution { id, .. } => id,
            _ => unreachable!("expected command execution"),
        };
        let _ = wait_for_command_execution_completed(&mut mcp, &command_id).await?;
        let _: TurnCompletedNotification = serde_json::from_value(
            timeout(
                DEFAULT_READ_TIMEOUT,
                mcp.read_stream_until_notification_message("turn/completed"),
            )
            .await??
            .params
            .expect("turn/completed params"),
        )?;

        (thread.id, second_thread.id)
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: None,
            cwd: None,
            search_term: None,
        })
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let ThreadListResponse { data, .. } = to_response::<ThreadListResponse>(list_resp)?;
    assert!(data.len() >= 2);
    let first_thread = data
        .iter()
        .find(|thread| thread.id == first_thread_id)
        .expect("first thread in list");
    let second_thread = data
        .iter()
        .find(|thread| thread.id == second_thread_id)
        .expect("second thread in list");
    assert!(first_thread.updated_at > second_thread.updated_at);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_shell_command_uses_existing_active_turn() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(DelayedAssistantResponder)
        .expect(1)
        .mount(&server)
        .await;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let turn_req = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            input: vec![V2UserInput::Text {
                text: "wait briefly".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        })
        .await?;
    let turn_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(turn_req)),
    )
    .await??;
    let TurnStartResponse { turn } = to_response::<TurnStartResponse>(turn_resp)?;

    let (command, expected_output) = current_shell_output_command("attached shell")?;
    let shell_req = mcp
        .send_thread_shell_command_request(ThreadShellCommandParams {
            thread_id: thread.id.clone(),
            command,
        })
        .await?;
    let shell_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(shell_req)),
    )
    .await??;
    let _: ThreadShellCommandResponse = to_response::<ThreadShellCommandResponse>(shell_resp)?;

    let started = wait_for_command_execution_started(&mut mcp).await?;
    assert_eq!(started.turn_id, turn.id);
    let command_id = match &started.item {
        ThreadItem::CommandExecution { id, source, .. } => {
            assert_eq!(source, &CommandExecutionSource::UserShell);
            id.clone()
        }
        _ => unreachable!("expected command execution"),
    };
    let completed = wait_for_command_execution_completed(&mut mcp, &command_id).await?;
    assert_eq!(completed.turn_id, turn.id);
    match completed.item {
        ThreadItem::CommandExecution {
            aggregated_output, ..
        } => {
            assert_eq!(aggregated_output.as_deref(), Some(expected_output.as_str()));
        }
        _ => unreachable!("expected command execution"),
    }

    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread.id,
            include_turns: true,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.turns.len(), 1);
    assert!(
        thread.turns[0].items.iter().any(|item| {
            matches!(
                item,
                ThreadItem::CommandExecution {
                    source: CommandExecutionSource::UserShell,
                    aggregated_output,
                    ..
                } if aggregated_output.as_deref() == Some(expected_output.as_str())
            )
        }),
        "expected active-turn OpenRouter shell command to persist on the existing turn"
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_background_terminal_cleanup_stops_running_shell() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let shell_req = mcp
        .send_thread_shell_command_request(ThreadShellCommandParams {
            thread_id: thread.id.clone(),
            command: long_running_shell_command(),
        })
        .await?;
    let shell_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(shell_req)),
    )
    .await??;
    let _: ThreadShellCommandResponse = to_response::<ThreadShellCommandResponse>(shell_resp)?;

    let started = wait_for_command_execution_started(&mut mcp).await?;
    let command_id = match &started.item {
        ThreadItem::CommandExecution { id, .. } => id.clone(),
        _ => unreachable!("expected command execution"),
    };

    let clean_req = mcp
        .send_thread_background_terminals_clean_request(ThreadBackgroundTerminalsCleanParams {
            thread_id: thread.id,
        })
        .await?;
    let clean_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(clean_req)),
    )
    .await??;
    let _: ThreadBackgroundTerminalsCleanResponse =
        to_response::<ThreadBackgroundTerminalsCleanResponse>(clean_resp)?;

    let completed = wait_for_command_execution_completed(&mut mcp, &command_id).await?;
    match completed.item {
        ThreadItem::CommandExecution { status, .. } => {
            assert!(
                matches!(
                    status,
                    CommandExecutionStatus::Completed | CommandExecutionStatus::Failed
                ),
                "expected clean to finish the shell command"
            );
        }
        _ => unreachable!("expected command execution"),
    }

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_archive_unarchive_visibility_is_coherent() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let archive_req = mcp
        .send_thread_archive_request(ThreadArchiveParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let archive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archive_req)),
    )
    .await??;
    let _: codex_app_server_protocol::ThreadArchiveResponse =
        to_response::<codex_app_server_protocol::ThreadArchiveResponse>(archive_resp)?;
    let active_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(false),
            cwd: None,
            search_term: None,
        })
        .await?;
    let active_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(active_list_req)),
    )
    .await??;
    let ThreadListResponse {
        data: active_data, ..
    } = to_response::<ThreadListResponse>(active_list_resp)?;
    assert!(
        !active_data
            .iter()
            .any(|candidate| candidate.id == thread.id)
    );

    let archived_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(true),
            cwd: None,
            search_term: None,
        })
        .await?;
    let archived_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archived_list_req)),
    )
    .await??;
    let ThreadListResponse {
        data: archived_data,
        ..
    } = to_response::<ThreadListResponse>(archived_list_resp)?;
    assert!(
        archived_data
            .iter()
            .any(|candidate| candidate.id == thread.id)
    );

    let unarchive_req = mcp
        .send_thread_unarchive_request(ThreadUnarchiveParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let unarchive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(unarchive_req)),
    )
    .await??;
    let ThreadUnarchiveResponse {
        thread: restored_thread,
    } = to_response::<ThreadUnarchiveResponse>(unarchive_resp)?;
    assert_eq!(restored_thread.status, ThreadStatus::NotLoaded);
    let resume_req = mcp
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: restored_thread.id.clone(),
            ..Default::default()
        })
        .await?;
    let resume_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(resume_req)),
    )
    .await??;
    let ThreadResumeResponse {
        thread: resumed_thread,
        ..
    } = to_response::<ThreadResumeResponse>(resume_resp)?;
    assert_eq!(resumed_thread.id, restored_thread.id);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_archive_unarchive_restart_uses_registry_state_not_snapshot_location()
-> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

        let set_name_req = mcp
            .send_thread_set_name_request(ThreadSetNameParams {
                thread_id: thread.id.clone(),
                name: "Registry Truth Thread".to_string(),
            })
            .await?;
        let set_name_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(set_name_req)),
        )
        .await??;
        to_response::<ThreadSetNameResponse>(set_name_resp)?;

        let metadata_req = mcp
            .send_thread_metadata_update_request(ThreadMetadataUpdateParams {
                thread_id: thread.id.clone(),
                git_info: Some(ThreadMetadataGitInfoUpdateParams {
                    sha: Some(Some("abc123".to_string())),
                    branch: Some(Some("feature/registry".to_string())),
                    origin_url: Some(Some("https://example.com/repo.git".to_string())),
                }),
            })
            .await?;
        let metadata_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(metadata_req)),
        )
        .await??;
        to_response::<ThreadMetadataUpdateResponse>(metadata_resp)?;

        let archive_req = mcp
            .send_thread_archive_request(ThreadArchiveParams {
                thread_id: thread.id.clone(),
            })
            .await?;
        let archive_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(archive_req)),
        )
        .await??;
        to_response::<codex_app_server_protocol::ThreadArchiveResponse>(archive_resp)?;

        thread.id
    };

    let active_snapshot_path = workspace
        .path()
        .join("logos-openrouter-beta")
        .join("active-threads")
        .join(format!("{thread_id}.json"));
    let archived_snapshot_path = workspace
        .path()
        .join("logos-openrouter-beta")
        .join("archived-threads")
        .join(format!("{thread_id}.json"));
    std::fs::create_dir_all(
        archived_snapshot_path
            .parent()
            .expect("archived snapshot dir"),
    )?;
    std::fs::copy(&active_snapshot_path, &archived_snapshot_path)?;
    std::fs::remove_file(&active_snapshot_path)?;
    std::fs::remove_file(
        workspace
            .path()
            .join("thread-history")
            .join(format!("{thread_id}.jsonl")),
    )?;

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.clone(),
            include_turns: true,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.id, thread_id);
    assert_eq!(thread.name.as_deref(), Some("Registry Truth Thread"));
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|info| info.sha.as_deref()),
        Some("abc123")
    );
    assert_eq!(thread.status, ThreadStatus::NotLoaded);

    let archived_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(true),
            cwd: None,
            search_term: None,
        })
        .await?;
    let archived_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archived_list_req)),
    )
    .await??;
    let ThreadListResponse {
        data: archived_data,
        ..
    } = to_response::<ThreadListResponse>(archived_list_resp)?;
    assert!(
        archived_data
            .iter()
            .any(|candidate| candidate.id == thread_id)
    );

    let unarchive_req = mcp
        .send_thread_unarchive_request(ThreadUnarchiveParams {
            thread_id: thread_id.clone(),
        })
        .await?;
    let unarchive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(unarchive_req)),
    )
    .await??;
    let ThreadUnarchiveResponse { thread } =
        to_response::<ThreadUnarchiveResponse>(unarchive_resp)?;
    assert_eq!(thread.id, thread_id);

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.clone(),
            include_turns: true,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(thread.id, thread_id);
    assert_eq!(thread.name.as_deref(), Some("Registry Truth Thread"));
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|info| info.branch.as_deref()),
        Some("feature/registry")
    );
    assert_eq!(thread.status, ThreadStatus::NotLoaded);

    let active_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(false),
            cwd: None,
            search_term: None,
        })
        .await?;
    let active_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(active_list_req)),
    )
    .await??;
    let ThreadListResponse { data, .. } = to_response::<ThreadListResponse>(active_list_resp)?;
    assert!(data.iter().any(|candidate| candidate.id == thread_id));

    let archived_list_req = mcp
        .send_thread_list_request(ThreadListParams {
            cursor: None,
            limit: None,
            sort_key: None,
            model_providers: Some(vec!["openrouter-test".to_string()]),
            source_kinds: None,
            archived: Some(true),
            cwd: None,
            search_term: None,
        })
        .await?;
    let archived_list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archived_list_req)),
    )
    .await??;
    let ThreadListResponse {
        data: archived_data,
        ..
    } = to_response::<ThreadListResponse>(archived_list_resp)?;
    assert!(
        !archived_data
            .iter()
            .any(|candidate| candidate.id == thread_id)
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_fork_creates_new_thread_with_history() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(SequencedOpenRouterResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let turn_req = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            input: vec![V2UserInput::Text {
                text: "inspect note.txt".to_string(),
                text_elements: Vec::new(),
            }],
            cwd: Some(workspace.path().to_path_buf()),
            model: Some("openai/o4-mini".to_string()),
            approval_policy: Some(codex_app_server_protocol::AskForApproval::Never),
            ..Default::default()
        })
        .await?;
    let turn_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(turn_req)),
    )
    .await??;
    to_response::<TurnStartResponse>(turn_resp)?;
    timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            match mcp.read_next_message().await? {
                JSONRPCMessage::Notification(n) if n.method == "turn/completed" => {
                    return Ok::<(), anyhow::Error>(());
                }
                _ => {}
            }
        }
    })
    .await??;

    let fork_req = mcp
        .send_thread_fork_request(ThreadForkParams {
            thread_id: thread.id.clone(),
            ..Default::default()
        })
        .await?;
    let fork_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(fork_req)),
    )
    .await??;
    let fork_response = to_response::<codex_app_server_protocol::ThreadForkResponse>(fork_resp)?;
    assert_ne!(fork_response.thread.id, thread.id);
    assert_eq!(fork_response.thread.forked_from_id, Some(thread.id));
    assert_eq!(fork_response.model_provider, "openrouter-test");

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_fork_succeeds_from_not_loaded_thread() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let archive_req = mcp
        .send_thread_archive_request(ThreadArchiveParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let archive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(archive_req)),
    )
    .await??;
    let _: codex_app_server_protocol::ThreadArchiveResponse =
        to_response::<codex_app_server_protocol::ThreadArchiveResponse>(archive_resp)?;

    let unarchive_req = mcp
        .send_thread_unarchive_request(ThreadUnarchiveParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let unarchive_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(unarchive_req)),
    )
    .await??;
    let ThreadUnarchiveResponse {
        thread: restored_thread,
    } = to_response::<ThreadUnarchiveResponse>(unarchive_resp)?;
    assert_eq!(restored_thread.status, ThreadStatus::NotLoaded);

    let fork_req = mcp
        .send_thread_fork_request(ThreadForkParams {
            thread_id: restored_thread.id.clone(),
            ..Default::default()
        })
        .await?;
    let fork_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(fork_req)),
    )
    .await??;
    let fork_response = to_response::<codex_app_server_protocol::ThreadForkResponse>(fork_resp)?;
    assert_ne!(fork_response.thread.id, restored_thread.id);
    assert_eq!(
        fork_response.thread.forked_from_id,
        Some(restored_thread.id)
    );
    assert_eq!(fork_response.thread.status, ThreadStatus::Idle);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_review_start_inline_succeeds() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(SequencedOpenRouterResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let review_req = mcp
        .send_review_start_request(ReviewStartParams {
            thread_id: thread.id.clone(),
            target: ReviewTarget::Custom {
                instructions: "Review the code".to_string(),
            },
            delivery: Some(ReviewDelivery::Inline),
        })
        .await?;
    let review_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(review_req)),
    )
    .await??;
    let review_response =
        to_response::<codex_app_server_protocol::ReviewStartResponse>(review_resp)?;
    assert_eq!(review_response.review_thread_id, thread.id);

    let turn_completed: TurnCompletedNotification = timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            let JSONRPCMessage::Notification(notification) = mcp.read_next_message().await? else {
                continue;
            };
            if notification.method == "turn/completed" {
                return serde_json::from_value(notification.params.expect("turn/completed params"))
                    .map_err(anyhow::Error::from);
            }
        }
    })
    .await??;
    assert_eq!(turn_completed.thread_id, thread.id);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_resume_with_history_is_explicitly_rejected() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let resume_req = mcp
        .send_raw_request(
            "thread/resume",
            Some(json!({
                "threadId": thread.id,
                "history": [{"type": "message", "role": "user", "content": []}]
            })),
        )
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(resume_req)),
    )
    .await??;
    assert!(
        error.message.contains("history"),
        "expected rejection mentioning 'history', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_resume_with_path_is_explicitly_rejected() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let resume_req = mcp
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: thread.id,
            path: Some(std::path::PathBuf::from("/some/rollout/path")),
            ..Default::default()
        })
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(resume_req)),
    )
    .await??;
    assert!(
        error.message.contains("path"),
        "expected rejection mentioning 'path', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_fork_with_path_is_explicitly_rejected() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let fork_req = mcp
        .send_thread_fork_request(ThreadForkParams {
            thread_id: thread.id,
            path: Some(std::path::PathBuf::from("/some/rollout/path")),
            ..Default::default()
        })
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(fork_req)),
    )
    .await??;
    assert!(
        error.message.contains("path"),
        "expected rejection mentioning 'path', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_review_start_detached_creates_thread_and_emits_notifications()
-> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    std::fs::write(workspace.path().join("note.txt"), "hello from workspace")?;

    Mock::given(method("POST"))
        .and(path("/api/v1/responses"))
        .respond_with(SequencedOpenRouterResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let review_req = mcp
        .send_review_start_request(ReviewStartParams {
            thread_id: thread.id.clone(),
            target: ReviewTarget::Custom {
                instructions: "Review the code".to_string(),
            },
            delivery: Some(ReviewDelivery::Detached),
        })
        .await?;
    let review_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(review_req)),
    )
    .await??;
    let review_response =
        to_response::<codex_app_server_protocol::ReviewStartResponse>(review_resp)?;
    assert_ne!(
        review_response.review_thread_id, thread.id,
        "detached review must return a different thread id"
    );

    let mut saw_thread_started = false;
    let mut saw_turn_completed = false;
    timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            if let JSONRPCMessage::Notification(notification) = mcp.read_next_message().await? {
                match notification.method.as_str() {
                    "thread/started" => {
                        saw_thread_started = true;
                    }
                    "turn/completed" => {
                        saw_turn_completed = true;
                        return Ok::<(), anyhow::Error>(());
                    }
                    _ => {}
                }
            }
        }
    })
    .await??;
    assert!(
        saw_thread_started,
        "detached review must emit thread/started notification"
    );
    assert!(
        saw_turn_completed,
        "detached review must emit turn/completed notification"
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_resume_with_config_override_is_explicitly_rejected() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let resume_req = mcp
        .send_raw_request(
            "thread/resume",
            Some(json!({
                "threadId": thread.id,
                "model": "gpt-4"
            })),
        )
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(resume_req)),
    )
    .await??;
    assert!(
        error.message.contains("config overrides"),
        "expected rejection mentioning 'config overrides', got: {}",
        error.message
    );
    assert!(
        error.message.contains("model"),
        "expected rejection naming 'model', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_fork_with_config_override_is_explicitly_rejected() -> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let fork_req = mcp
        .send_raw_request(
            "thread/fork",
            Some(json!({
                "threadId": thread.id,
                "model": "gpt-4"
            })),
        )
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(fork_req)),
    )
    .await??;
    assert!(
        error.message.contains("config overrides"),
        "expected rejection mentioning 'config overrides', got: {}",
        error.message
    );
    assert!(
        error.message.contains("model"),
        "expected rejection naming 'model', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_resume_with_history_on_unknown_thread_via_model_provider_is_explicitly_rejected()
-> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let resume_req = mcp
        .send_raw_request(
            "thread/resume",
            Some(json!({
                "threadId": "nonexistent-thread-id",
                "modelProvider": "openrouter-test",
                "history": [{"type": "message", "role": "user", "content": []}]
            })),
        )
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(resume_req)),
    )
    .await??;
    assert!(
        error.message.contains("history"),
        "expected rejection mentioning 'history', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_fork_with_path_on_unknown_thread_via_model_provider_is_explicitly_rejected()
-> Result<()> {
    let server = MockServer::start().await;
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let fork_req = mcp
        .send_raw_request(
            "thread/fork",
            Some(json!({
                "threadId": "nonexistent-thread-id",
                "modelProvider": "openrouter-test",
                "path": "/some/rollout/path"
            })),
        )
        .await?;
    let JSONRPCError { error, .. } = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(fork_req)),
    )
    .await??;
    assert!(
        error.message.contains("path"),
        "expected rejection mentioning 'path', got: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_partial_git_info_update_preserves_existing_fields_after_restart()
-> Result<()> {
    let workspace = TempDir::new()?;
    let server = MockServer::start().await;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

        let full_meta_req = mcp
            .send_thread_metadata_update_request(ThreadMetadataUpdateParams {
                thread_id: thread.id.clone(),
                git_info: Some(ThreadMetadataGitInfoUpdateParams {
                    sha: Some(Some("initial-sha".to_string())),
                    branch: Some(Some("initial-branch".to_string())),
                    origin_url: Some(Some("https://example.com/initial.git".to_string())),
                }),
            })
            .await?;
        let _ = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(full_meta_req)),
        )
        .await??;

        thread.id
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let partial_meta_req = mcp
        .send_thread_metadata_update_request(ThreadMetadataUpdateParams {
            thread_id: thread_id.clone(),
            git_info: Some(ThreadMetadataGitInfoUpdateParams {
                sha: None,
                branch: Some(Some("updated-branch".to_string())),
                origin_url: None,
            }),
        })
        .await?;
    let partial_meta_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(partial_meta_req)),
    )
    .await??;
    let ThreadMetadataUpdateResponse { thread } =
        to_response::<ThreadMetadataUpdateResponse>(partial_meta_resp)?;

    assert_eq!(
        thread.git_info.as_ref().and_then(|g| g.sha.as_deref()),
        Some("initial-sha"),
        "sha should be preserved when omitted from partial update"
    );
    assert_eq!(
        thread.git_info.as_ref().and_then(|g| g.branch.as_deref()),
        Some("updated-branch"),
        "branch should be updated"
    );
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|g| g.origin_url.as_deref()),
        Some("https://example.com/initial.git"),
        "origin_url should be preserved when omitted from partial update"
    );

    let read_req = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.clone(),
            include_turns: false,
        })
        .await?;
    let read_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread } = to_response::<ThreadReadResponse>(read_resp)?;
    assert_eq!(
        thread.git_info.as_ref().and_then(|g| g.sha.as_deref()),
        Some("initial-sha")
    );
    assert_eq!(
        thread.git_info.as_ref().and_then(|g| g.branch.as_deref()),
        Some("updated-branch")
    );
    assert_eq!(
        thread
            .git_info
            .as_ref()
            .and_then(|g| g.origin_url.as_deref()),
        Some("https://example.com/initial.git")
    );

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_snapshot_fallback_keeps_registry_config_truth() -> Result<()> {
    let workspace = TempDir::new()?;
    let server = MockServer::start().await;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;
        thread.id
    };

    let thread_id_value = ThreadId::from_string(&thread_id)?;
    let registry = SqliteThreadRegistry::new(workspace.path().to_path_buf());
    let registry_cwd = workspace.path().join("registry-owned-cwd");
    let snapshot_cwd = workspace.path().join("snapshot-stale-cwd");
    std::fs::create_dir_all(&registry_cwd)?;
    std::fs::create_dir_all(&snapshot_cwd)?;

    let mut registry_config: Value = serde_json::from_str(
        &registry
            .get_config_snapshot(thread_id_value)
            .await?
            .expect("registry config snapshot"),
    )?;
    registry_config["cwd"] = Value::String(registry_cwd.to_string_lossy().into_owned());
    registry
        .upsert_config_snapshot(thread_id_value, serde_json::to_string(&registry_config)?)
        .await?;

    let snapshot_path = workspace
        .path()
        .join("logos-openrouter-beta")
        .join("active-threads")
        .join(format!("{thread_id}.json"));
    let mut snapshot_json: Value = serde_json::from_slice(&std::fs::read(&snapshot_path)?)?;
    snapshot_json["config_snapshot"]["cwd"] =
        Value::String(snapshot_cwd.to_string_lossy().into_owned());
    std::fs::write(&snapshot_path, serde_json::to_vec_pretty(&snapshot_json)?)?;

    let history_path = workspace
        .path()
        .join("thread-history")
        .join(format!("{thread_id}.jsonl"));
    std::fs::remove_file(history_path)?;

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let resume_req = mcp
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: thread_id.clone(),
            ..Default::default()
        })
        .await?;
    let resume_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(resume_req)),
    )
    .await??;
    let ThreadResumeResponse { cwd, .. } = to_response::<ThreadResumeResponse>(resume_resp)?;
    assert_eq!(cwd, registry_cwd);

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_elicitation_works_after_restart() -> Result<()> {
    let workspace = TempDir::new()?;
    let server = MockServer::start().await;
    write_openrouter_beta_config(workspace.path(), &server)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];

    let thread_id = {
        let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
        timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

        let thread_req = mcp
            .send_thread_start_request(ThreadStartParams {
                model_provider: Some("openrouter-test".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

        let inc_req = mcp
            .send_thread_increment_elicitation_request(ThreadIncrementElicitationParams {
                thread_id: thread.id.clone(),
            })
            .await?;
        let inc_resp = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_stream_until_response_message(RequestId::Integer(inc_req)),
        )
        .await??;
        let ThreadIncrementElicitationResponse { count, .. } =
            to_response::<ThreadIncrementElicitationResponse>(inc_resp)?;
        assert_eq!(count, 1);

        thread.id
    };

    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let inc_req = mcp
        .send_thread_increment_elicitation_request(ThreadIncrementElicitationParams {
            thread_id: thread_id.clone(),
        })
        .await?;
    let inc_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(inc_req)),
    )
    .await??;
    let ThreadIncrementElicitationResponse { count, paused } =
        to_response::<ThreadIncrementElicitationResponse>(inc_resp)?;
    assert_eq!(count, 2);
    assert!(paused);

    let dec_req = mcp
        .send_thread_decrement_elicitation_request(ThreadDecrementElicitationParams {
            thread_id: thread_id.clone(),
        })
        .await?;
    let dec_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(dec_req)),
    )
    .await??;
    let ThreadDecrementElicitationResponse { count, paused } =
        to_response::<ThreadDecrementElicitationResponse>(dec_resp)?;
    assert_eq!(count, 1);
    assert!(paused);

    Ok(())
}

fn write_openrouter_beta_config(codex_home: &Path, server: &MockServer) -> Result<()> {
    std::fs::write(
        codex_home.join("config.toml"),
        format!(
            r#"
model = "openai/o4-mini"
approval_policy = "never"
sandbox_mode = "read-only"
model_provider = "openrouter-test"

[features]
personality = true

[model_providers.openrouter-test]
name = "OpenRouter test"
base_url = "{}/api/v1"
env_key = "OPENROUTER_API_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
supports_websockets = false
"#,
            server.uri()
        ),
    )?;
    Ok(())
}

fn current_shell_output_command(text: &str) -> Result<(String, String)> {
    let command_and_output = match default_user_shell().name() {
        "powershell" => {
            let escaped_text = text.replace('\'', "''");
            (
                format!("Write-Output '{escaped_text}'"),
                format!("{text}\r\n"),
            )
        }
        "cmd" => (format!("echo {text}"), format!("{text}\r\n")),
        _ => {
            let quoted_text = shlex::try_quote(text)?;
            (format!("printf '%s\\n' {quoted_text}"), format!("{text}\n"))
        }
    };
    Ok(command_and_output)
}

fn long_running_shell_command() -> String {
    match default_user_shell().name() {
        "powershell" => "Start-Sleep -Seconds 30".to_string(),
        "cmd" => "timeout /t 30 >NUL".to_string(),
        _ => "sleep 30".to_string(),
    }
}

async fn wait_for_command_execution_started(
    mcp: &mut McpProcess,
) -> Result<ItemStartedNotification> {
    loop {
        let notif = mcp
            .read_stream_until_notification_message("item/started")
            .await?;
        let started: ItemStartedNotification = serde_json::from_value(
            notif
                .params
                .ok_or_else(|| anyhow::anyhow!("missing item/started params"))?,
        )?;
        if matches!(started.item, ThreadItem::CommandExecution { .. }) {
            return Ok(started);
        }
    }
}

async fn wait_for_command_execution_output(mcp: &mut McpProcess, item_id: &str) -> Result<String> {
    let mut output = String::new();
    loop {
        let notif = mcp
            .read_stream_until_notification_message("item/commandExecution/outputDelta")
            .await?;
        let delta: CommandExecutionOutputDeltaNotification = serde_json::from_value(
            notif
                .params
                .ok_or_else(|| anyhow::anyhow!("missing output delta params"))?,
        )?;
        if delta.item_id == item_id {
            output.push_str(&delta.delta);
            if output.ends_with('\n') {
                return Ok(output);
            }
        }
    }
}

async fn wait_for_command_execution_completed(
    mcp: &mut McpProcess,
    item_id: &str,
) -> Result<ItemCompletedNotification> {
    loop {
        let notif = mcp
            .read_stream_until_notification_message("item/completed")
            .await?;
        let completed: ItemCompletedNotification = serde_json::from_value(
            notif
                .params
                .ok_or_else(|| anyhow::anyhow!("missing item/completed params"))?,
        )?;
        match &completed.item {
            ThreadItem::CommandExecution { id, .. } if id == item_id => return Ok(completed),
            _ => {}
        }
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
    match serde_json::to_string(&value) {
        Ok(value) => value,
        Err(error) => panic!("serialize json string: {error}"),
    }
}

#[derive(Default)]
struct SequencedOpenRouterResponder {
    next: Arc<Mutex<usize>>,
}

#[derive(Default)]
struct ToolRoundTripResponder;

#[derive(Default)]
struct DelayedAssistantResponder;

impl Respond for ToolRoundTripResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = match serde_json::from_slice(&request.body) {
            Ok(body) => body,
            Err(error) => panic!("parse request body: {error}"),
        };
        let has_call_output = body
            .get("input")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("type") == Some(&Value::String("function_call_output".to_string()))
                })
            });
        if has_call_output {
            ResponseTemplate::new(200).set_body_json(json!({
                "id": "resp-tool-message",
                "output": [{
                    "type": "message",
                    "id": "msg-tool-message",
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
            }))
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
                "id": "resp-tool-call",
                "output": [{
                    "type": "function_call",
                    "call_id": "call-1",
                    "name": "workspace_read_file",
                    "arguments": "{\"path\":\"note.txt\"}"
                }],
                "usage": {"input_tokens": 10, "output_tokens": 3, "total_tokens": 13},
                "status": "completed"
            }))
        }
    }
}

impl Respond for DelayedAssistantResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        std::thread::sleep(std::time::Duration::from_millis(750));
        ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp-delayed",
            "output": [{
                "type": "message",
                "id": "msg-delayed",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "done",
                    "annotations": []
                }]
            }],
            "usage": {"input_tokens": 4, "output_tokens": 1, "total_tokens": 5},
            "status": "completed"
        }))
    }
}

impl Respond for SequencedOpenRouterResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let mut next = match self.next.lock() {
            Ok(next) => next,
            Err(error) => panic!("response lock: {error}"),
        };
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

#[tokio::test]
async fn app_server_openrouter_background_terminal_start_registers_in_shared_owner() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let start_req = mcp
        .send_thread_background_terminal_start_request(
            codex_app_server_protocol::ThreadBackgroundTerminalStartParams {
                thread_id: thread.id.clone(),
                command: long_running_shell_command(),
            },
        )
        .await?;
    let start_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(start_req)),
    )
    .await??;
    let start_result: codex_app_server_protocol::ThreadBackgroundTerminalStartResponse =
        to_response(start_resp)?;

    assert!(
        !start_result.call_id.is_empty(),
        "call_id must be non-empty"
    );
    assert!(
        !start_result.process_id.is_empty(),
        "process_id must be non-empty"
    );

    let list_req = mcp
        .send_thread_background_terminals_list_request(
            codex_app_server_protocol::ThreadBackgroundTerminalsListParams {
                thread_id: thread.id.clone(),
            },
        )
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let list_result: codex_app_server_protocol::ThreadBackgroundTerminalsListResponse =
        to_response(list_resp)?;

    assert_eq!(
        list_result.processes.len(),
        1,
        "expected exactly one registered process"
    );
    let proc = &list_result.processes[0];
    assert_eq!(proc.call_id, start_result.call_id);
    assert!(proc.interactive, "expected interactive=true");

    let clean_req = mcp
        .send_thread_background_terminals_clean_request(ThreadBackgroundTerminalsCleanParams {
            thread_id: thread.id,
        })
        .await?;
    let clean_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(clean_req)),
    )
    .await??;
    let _: ThreadBackgroundTerminalsCleanResponse =
        to_response::<ThreadBackgroundTerminalsCleanResponse>(clean_resp)?;

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_background_terminal_write_stdin_and_poll() -> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let start_cmd = match default_user_shell().name() {
        "powershell" => "cmd /c more".to_string(),
        "cmd" => "more".to_string(),
        _ => "cat".to_string(),
    };
    let start_req = mcp
        .send_thread_background_terminal_start_request(
            codex_app_server_protocol::ThreadBackgroundTerminalStartParams {
                thread_id: thread.id.clone(),
                command: start_cmd,
            },
        )
        .await?;
    let start_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(start_req)),
    )
    .await??;
    let start_result: codex_app_server_protocol::ThreadBackgroundTerminalStartResponse =
        to_response(start_resp)?;

    let stdin_data = base64::engine::general_purpose::STANDARD.encode("hello\n");
    let write_req = mcp
        .send_thread_background_terminal_write_stdin_request(
            codex_app_server_protocol::ThreadBackgroundTerminalWriteStdinParams {
                thread_id: thread.id.clone(),
                call_id: start_result.call_id.clone(),
                data_base64: Some(stdin_data),
                close_stdin: false,
            },
        )
        .await?;
    let write_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(write_req)),
    )
    .await??;
    let _: codex_app_server_protocol::ThreadBackgroundTerminalWriteStdinResponse =
        to_response(write_resp)?;

    let poll_req = mcp
        .send_thread_background_terminal_write_stdin_request(
            codex_app_server_protocol::ThreadBackgroundTerminalWriteStdinParams {
                thread_id: thread.id.clone(),
                call_id: start_result.call_id.clone(),
                data_base64: None,
                close_stdin: false,
            },
        )
        .await?;
    let poll_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(poll_req)),
    )
    .await??;
    let _: codex_app_server_protocol::ThreadBackgroundTerminalWriteStdinResponse =
        to_response(poll_resp)?;

    let clean_req = mcp
        .send_thread_background_terminals_clean_request(ThreadBackgroundTerminalsCleanParams {
            thread_id: thread.id,
        })
        .await?;
    let clean_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(clean_req)),
    )
    .await??;
    let _: ThreadBackgroundTerminalsCleanResponse =
        to_response::<ThreadBackgroundTerminalsCleanResponse>(clean_resp)?;

    Ok(())
}

#[tokio::test]
async fn app_server_openrouter_background_terminals_list_returns_empty_for_cleaned_thread()
-> Result<()> {
    let workspace = TempDir::new()?;
    write_openrouter_beta_config(workspace.path(), &MockServer::start().await)?;

    let logos_home = workspace.path().to_string_lossy().to_string();
    let env_overrides = [
        ("LOGOS_HOME", Some(logos_home.as_str())),
        ("OPENROUTER_API_KEY", Some("test-key")),
    ];
    let mut mcp = McpProcess::new_with_env(workspace.path(), &env_overrides).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams {
            model_provider: Some("openrouter-test".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;

    let list_req = mcp
        .send_thread_background_terminals_list_request(
            codex_app_server_protocol::ThreadBackgroundTerminalsListParams {
                thread_id: thread.id.clone(),
            },
        )
        .await?;
    let list_resp = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(list_req)),
    )
    .await??;
    let list_result: codex_app_server_protocol::ThreadBackgroundTerminalsListResponse =
        to_response(list_resp)?;

    assert!(
        list_result.processes.is_empty(),
        "expected empty list for new thread"
    );

    Ok(())
}
