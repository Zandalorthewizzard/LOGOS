use std::sync::Arc;

use futures::StreamExt;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderSession;
use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use logos_runtime_types::ReasoningPolicy;
use logos_runtime_types::ToolApprovalOutcome;
use logos_runtime_types::ToolApprovalRequest;
use logos_runtime_types::ToolExecutionKind;
use logos_runtime_types::ToolExecutionPolicy;
use logos_runtime_types::TurnLimits;
use logos_tool_runtime::PolicyApprover;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolApprover;
use logos_tool_runtime::ToolRuntime;

use crate::tool_loop::tool_calls_from_events;
use crate::tool_loop::tool_messages_from_events;
use crate::turn::TurnOutcome;

pub struct SessionEngine {
    adapter: Arc<dyn ProviderAdapter>,
    provider_session: Box<dyn ProviderSession>,
    provider_capabilities: ProviderCapabilitySet,
    model_capabilities: ModelCapabilitySet,
    model_ref: ModelRef,
    system: Option<String>,
    history: Vec<InternalMessage>,
    continuation: Option<ContinuationHandle>,
    tool_runtime: ToolRuntime,
    tool_execution_policy: ToolExecutionPolicy,
    tool_approver: Arc<dyn ToolApprover>,
}

impl SessionEngine {
    pub async fn new(
        adapter: Arc<dyn ProviderAdapter>,
        model_ref: ModelRef,
        system: Option<String>,
    ) -> Result<Self, ProviderError> {
        let provider_capabilities = adapter.provider_capabilities();
        let model_capabilities = adapter.model_capabilities(&model_ref);
        let provider_session = adapter.new_session().await?;
        Ok(Self {
            adapter,
            provider_session,
            provider_capabilities,
            model_capabilities,
            model_ref,
            system,
            history: Vec::new(),
            continuation: None,
            tool_runtime: ToolRuntime::default(),
            tool_execution_policy: ToolExecutionPolicy::default(),
            tool_approver: Arc::new(PolicyApprover),
        })
    }

    pub fn register_runtime_tool(
        &mut self,
        def: InternalToolDef,
        executor: Arc<dyn RuntimeToolExecutor>,
    ) {
        self.tool_runtime.register_tool(def, executor);
    }

    pub fn set_tool_execution_policy(&mut self, policy: ToolExecutionPolicy) {
        self.tool_execution_policy = policy;
    }

    pub fn set_tool_approver(&mut self, approver: Arc<dyn ToolApprover>) {
        self.tool_approver = approver;
    }

    pub fn continuation(&self) -> Option<&ContinuationHandle> {
        self.continuation.as_ref()
    }

    pub fn provider_capabilities(&self) -> &ProviderCapabilitySet {
        &self.provider_capabilities
    }

    pub fn model_capabilities(&self) -> &ModelCapabilitySet {
        &self.model_capabilities
    }

    pub fn history(&self) -> &[InternalMessage] {
        &self.history
    }

    pub async fn compact_history(&mut self) -> Result<(), ProviderError> {
        match self.provider_capabilities.compaction_backend {
            logos_runtime_types::CompactionBackendKind::Disabled => Err(ProviderError::new(
                self.model_ref.provider_id.clone(),
                ProviderErrorKind::UnsupportedCapability,
                "provider does not support compaction",
            )),
            logos_runtime_types::CompactionBackendKind::PortableCheckpoint => {
                Err(ProviderError::new(
                    self.model_ref.provider_id.clone(),
                    ProviderErrorKind::UnsupportedCapability,
                    "portable compaction is deferred beyond phase-2",
                ))
            }
            logos_runtime_types::CompactionBackendKind::NativeRemoteRewrite => {
                let result = self
                    .adapter
                    .compact_history(logos_runtime_types::InternalCompactionRequest {
                        model_ref: self.model_ref.clone(),
                        system: self.system.clone(),
                        messages: self.history.clone(),
                        tools: self.tool_runtime.declared_tools(),
                    })
                    .await?;
                self.history = result.replacement_messages;
                self.continuation = None;
                Ok(())
            }
        }
    }

    pub async fn run_turn(
        &mut self,
        new_messages: Vec<InternalMessage>,
    ) -> Result<TurnOutcome, ProviderError> {
        self.run_turn_with_tools(new_messages, Vec::new()).await
    }

    pub async fn run_turn_with_tools(
        &mut self,
        new_messages: Vec<InternalMessage>,
        tools: Vec<InternalToolDef>,
    ) -> Result<TurnOutcome, ProviderError> {
        let result = self.run_turn_with_tools_inner(new_messages, tools).await;
        if result.is_err() {
            self.provider_session.finish_turn().await?;
        }
        result
    }

    async fn run_turn_with_tools_inner(
        &mut self,
        new_messages: Vec<InternalMessage>,
        tools: Vec<InternalToolDef>,
    ) -> Result<TurnOutcome, ProviderError> {
        let mut pending_messages = new_messages;
        let mut all_events = Vec::new();
        let mut latest_continuation = self.continuation.clone();
        let mut request_tools = tools;
        request_tools.extend(self.tool_runtime.declared_tools());
        self.validate_declared_tools(&request_tools)?;

        loop {
            let request = InternalTurnRequest {
                model_ref: self.model_ref.clone(),
                system: self.system.clone(),
                messages: self
                    .history
                    .iter()
                    .cloned()
                    .chain(pending_messages.iter().cloned())
                    .collect(),
                tools: request_tools.clone(),
                output_mode: OutputMode::Text,
                reasoning_policy: ReasoningPolicy::Default,
                continuation: self.continuation.clone(),
                limits: TurnLimits::default(),
                metadata: Default::default(),
            };

            let mut stream = self.provider_session.stream_turn(request).await?;
            let mut events = Vec::new();
            let mut assistant_text = String::new();
            let mut assistant_message_id = None;
            while let Some(event) = stream.next().await {
                let event = event?;
                if let InternalEvent::TextDelta { text } = &event {
                    assistant_text.push_str(text);
                }
                if let InternalEvent::MessageCompleted { message_id } = &event
                    && message_id.is_some()
                {
                    assistant_message_id = message_id.clone();
                }
                if let InternalEvent::ContinuationUpdated { handle } = &event {
                    latest_continuation = Some(handle.clone());
                }
                events.push(event);
            }

            self.history.extend(pending_messages);
            if !assistant_text.is_empty() {
                self.history.push(InternalMessage::Assistant {
                    message_id: assistant_message_id,
                    parts: vec![ContentPart::Text {
                        text: assistant_text,
                    }],
                });
            }
            self.history.extend(tool_messages_from_events(&events));
            self.continuation = latest_continuation.clone();
            let pending_calls = tool_calls_from_events(&events);
            all_events.extend(events);

            let mut tool_results = Vec::new();
            for call in pending_calls {
                if let Some(tool_def) = self.tool_runtime.tool_def(&call.name) {
                    let request = ToolApprovalRequest {
                        call_id: call.call_id.clone(),
                        tool_name: call.name.clone(),
                        execution: tool_def.execution.clone(),
                    };
                    all_events.push(InternalEvent::ApprovalRequested {
                        request: request.clone(),
                    });
                    let outcome = self
                        .tool_approver
                        .decide(
                            &request,
                            self.tool_execution_policy
                                .approval_mode_for(&request.tool_name, &request.execution),
                        )
                        .await;
                    all_events.push(InternalEvent::ApprovalResolved {
                        call_id: request.call_id.clone(),
                        outcome: outcome.clone(),
                    });
                    match outcome {
                        ToolApprovalOutcome::Approved => {
                            if let Some(result) = self.tool_runtime.execute_call(&call).await {
                                tool_results.push(result);
                            }
                        }
                        ToolApprovalOutcome::Denied => {
                            tool_results.push(InternalMessage::ToolResult {
                                call_id: request.call_id,
                                output: serde_json::json!({"error": "approval denied"}),
                                is_error: true,
                            });
                        }
                    }
                }
            }

            if tool_results.is_empty() {
                self.provider_session.finish_turn().await?;
                return Ok(TurnOutcome {
                    events: all_events,
                    continuation: latest_continuation,
                });
            }

            pending_messages = tool_results;
        }
    }

    fn validate_declared_tools(&self, tools: &[InternalToolDef]) -> Result<(), ProviderError> {
        if !tools.is_empty() && !self.model_capabilities.supports_tool_calling {
            return Err(ProviderError::new(
                self.model_ref.provider_id.clone(),
                ProviderErrorKind::UnsupportedCapability,
                "model does not support tool calling",
            ));
        }

        for tool in tools {
            match tool.execution {
                ToolExecutionKind::RuntimeOwned | ToolExecutionKind::McpBackedRuntime => {
                    if !self.provider_capabilities.supports_runtime_owned_tools {
                        return Err(ProviderError::new(
                            self.model_ref.provider_id.clone(),
                            ProviderErrorKind::UnsupportedCapability,
                            format!(
                                "provider does not support runtime-owned tool '{}'",
                                tool.name
                            ),
                        ));
                    }
                    if !self.model_capabilities.supports_runtime_tool_reinjection {
                        return Err(ProviderError::new(
                            self.model_ref.provider_id.clone(),
                            ProviderErrorKind::UnsupportedCapability,
                            format!(
                                "model does not support runtime tool reinjection for '{}'",
                                tool.name
                            ),
                        ));
                    }
                }
                ToolExecutionKind::ProviderNative => {
                    if !self.provider_capabilities.supports_provider_builtin_tools
                        || !self.model_capabilities.supports_provider_native_tool_calls
                        || !self.tool_execution_policy.allow_provider_native
                    {
                        return Err(ProviderError::new(
                            self.model_ref.provider_id.clone(),
                            ProviderErrorKind::UnsupportedCapability,
                            format!(
                                "provider-native tool '{}' is not enabled for this session",
                                tool.name
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
