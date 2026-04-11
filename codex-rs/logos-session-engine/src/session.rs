use std::sync::Arc;

use futures::StreamExt;
use logos_journal::ApprovalJournalState;
use logos_journal::ApprovalViewRecord;
use logos_journal::JournalRecordKind;
use logos_journal::JournalSession;
use logos_journal::JournalStore;
use logos_journal::JournalTurnFinishReason;
use logos_journal::SessionOpenedRecord;
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
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::ToolCallResultDisposition;
use logos_runtime_types::ToolCallTerminalState;
use logos_runtime_types::ToolExecutionKind;
use logos_runtime_types::ToolExecutionPolicy;
use logos_runtime_types::TurnLimits;
use logos_tool_runtime::McpCorrelationIngress;
use logos_tool_runtime::PolicyApprover;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolApprover;
use logos_tool_runtime::ToolRuntime;
use uuid::Uuid;

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
    tool_call_records: Vec<ToolCallRecord>,
    approvals: Vec<logos_journal::ApprovalView>,
    journal: Option<JournalSession>,
    session_id: String,
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
            tool_call_records: Vec::new(),
            approvals: Vec::new(),
            journal: None,
            session_id: Uuid::new_v4().to_string(),
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

    pub fn set_mcp_correlation_ingress(&mut self, ingress: Arc<dyn McpCorrelationIngress>) {
        self.tool_runtime.set_mcp_correlation_ingress(ingress);
    }

    pub fn continuation(&self) -> Option<&ContinuationHandle> {
        self.continuation.as_ref()
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
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

    pub fn tool_call_records(&self) -> &[ToolCallRecord] {
        &self.tool_call_records
    }

    pub fn approvals(&self) -> &[logos_journal::ApprovalView] {
        &self.approvals
    }

    pub fn attach_journal(&mut self, journal: JournalSession) {
        self.session_id = journal.session_id().to_string();
        self.journal = Some(journal);
    }

    pub fn create_and_attach_journal(&mut self, store: &JournalStore) -> Result<(), ProviderError> {
        let journal = store
            .create_session(self.session_opened_record())
            .map_err(|error| self.journal_error(error))?;
        self.attach_journal(journal);
        Ok(())
    }

    pub async fn resume_from_journal(
        adapter: Arc<dyn ProviderAdapter>,
        journal: JournalSession,
    ) -> Result<Self, ProviderError> {
        let view = journal.view().map_err(|error| {
            ProviderError::new(
                adapter.provider_id(),
                ProviderErrorKind::Internal,
                error.to_string(),
            )
        })?;
        if adapter.provider_id() != view.model_ref.provider_id {
            return Err(ProviderError::new(
                adapter.provider_id(),
                ProviderErrorKind::InvalidRequest,
                format!(
                    "journal provider '{}' does not match adapter '{}'",
                    view.model_ref.provider_id,
                    adapter.provider_id()
                ),
            ));
        }

        let mut engine = Self::new(adapter, view.model_ref, view.system).await?;
        engine.history = view.history;
        engine.continuation = view.continuation;
        engine.tool_call_records = view.tool_call_records.into_values().collect();
        engine.approvals = view.approvals.into_values().collect();
        engine.attach_journal(journal);
        Ok(engine)
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
                let checkpoint_count = result
                    .replacement_messages
                    .iter()
                    .filter(|message| {
                        matches!(message, InternalMessage::CompactionCheckpoint { .. })
                    })
                    .count();
                if checkpoint_count != 1 {
                    return Err(ProviderError::new(
                        self.model_ref.provider_id.clone(),
                        ProviderErrorKind::InvalidRequest,
                        format!(
                            "native compaction must return exactly one compaction checkpoint, got {checkpoint_count}"
                        ),
                    ));
                }
                let mut first_retained_sequence_id = None;
                let mut checkpoint_sequence_id = None;
                for message in result.replacement_messages.iter().cloned() {
                    let is_checkpoint =
                        matches!(message, InternalMessage::CompactionCheckpoint { .. });
                    let record =
                        self.append_journal_record(JournalRecordKind::MessageAppended {
                            turn_id: None,
                            message,
                        })?;
                    if first_retained_sequence_id.is_none() {
                        first_retained_sequence_id = Some(record.sequence_id);
                    }
                    if is_checkpoint {
                        checkpoint_sequence_id = Some(record.sequence_id);
                    }
                }
                if let (Some(first_retained_sequence_id), Some(checkpoint_sequence_id)) =
                    (first_retained_sequence_id, checkpoint_sequence_id)
                {
                    self.append_journal_record(JournalRecordKind::CompactionApplied {
                        turn_id: None,
                        backend: logos_runtime_types::CompactionBackendKind::NativeRemoteRewrite,
                        first_retained_sequence_id,
                        checkpoint_sequence_id,
                    })?;
                }
                self.history = result.replacement_messages;
                self.continuation = None;
                self.append_journal_record(JournalRecordKind::ContinuationUpdated {
                    turn_id: None,
                    handle: None,
                })?;
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
        let turn_id = Uuid::new_v4().to_string();
        self.append_journal_record(JournalRecordKind::TurnStarted {
            turn_id: turn_id.clone(),
            output_mode: OutputMode::Text,
            reasoning_policy: ReasoningPolicy::Default,
            limits: TurnLimits::default(),
        })?;

        loop {
            self.append_messages_to_journal(
                Some(turn_id.as_str()),
                pending_messages.iter().cloned(),
            )?;
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

            let mut stream = match self.provider_session.stream_turn(request).await {
                Ok(stream) => stream,
                Err(error) => {
                    self.append_journal_record(JournalRecordKind::TurnFinished {
                        turn_id: turn_id.clone(),
                        reason: JournalTurnFinishReason::Failed,
                        error_message: Some(error.message.clone()),
                    })?;
                    return Err(error);
                }
            };
            let mut events = Vec::new();
            let mut assistant_text = String::new();
            let mut assistant_message_id = None;
            while let Some(event) = stream.next().await {
                let event = match event {
                    Ok(event) => event,
                    Err(error) => {
                        self.record_interrupted_calls(&events, Some(turn_id.as_str()))?;
                        self.append_journal_record(JournalRecordKind::TurnFinished {
                            turn_id: turn_id.clone(),
                            reason: JournalTurnFinishReason::Failed,
                            error_message: Some(error.message.clone()),
                        })?;
                        return Err(error);
                    }
                };
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
                    self.append_journal_record(JournalRecordKind::ContinuationUpdated {
                        turn_id: Some(turn_id.clone()),
                        handle: Some(handle.clone()),
                    })?;
                }
                events.push(event);
            }

            self.history.extend(pending_messages);
            if !assistant_text.is_empty() {
                let assistant_message = InternalMessage::Assistant {
                    message_id: assistant_message_id,
                    parts: vec![ContentPart::Text {
                        text: assistant_text,
                    }],
                };
                self.append_messages_to_journal(
                    Some(turn_id.as_str()),
                    [assistant_message.clone()],
                )?;
                self.history.push(assistant_message);
            }
            let tool_messages = tool_messages_from_events(&events);
            self.append_messages_to_journal(Some(turn_id.as_str()), tool_messages.iter().cloned())?;
            self.history.extend(tool_messages);
            self.continuation = latest_continuation.clone();
            let pending_calls = tool_calls_from_events(&events);
            all_events.extend(events);

            let mut tool_results = Vec::new();
            for call in pending_calls {
                if let Some(tool_def) = self.tool_runtime.tool_def(&call.name) {
                    let call = match self.tool_runtime.prepare_call(call, &tool_def) {
                        Ok(call) => call,
                        Err(message) => {
                            let error = ProviderError::new(
                                self.model_ref.provider_id.clone(),
                                ProviderErrorKind::Internal,
                                message,
                            );
                            self.append_journal_record(JournalRecordKind::TurnFinished {
                                turn_id: turn_id.clone(),
                                reason: JournalTurnFinishReason::Failed,
                                error_message: Some(error.message.clone()),
                            })?;
                            return Err(error);
                        }
                    };
                    let request = ToolApprovalRequest {
                        call_id: call.call_id.clone(),
                        tool_name: call.name.clone(),
                        execution: tool_def.execution.clone(),
                        mcp_correlation: call.mcp_correlation.clone(),
                    };
                    all_events.push(InternalEvent::ApprovalRequested {
                        request: request.clone(),
                    });
                    self.record_approval_state(
                        &turn_id,
                        request.clone(),
                        ApprovalJournalState::Pending,
                    )?;
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
                            self.record_approval_state(
                                &turn_id,
                                request.clone(),
                                ApprovalJournalState::Approved,
                            )?;
                            if let Some(outcome) = self.tool_runtime.execute_call(&call).await {
                                if self.record_tool_call_terminal(
                                    outcome.record.clone(),
                                    Some(turn_id.as_str()),
                                )? {
                                    all_events.push(InternalEvent::ToolCallTerminal {
                                        record: outcome.record,
                                    });
                                }
                                tool_results.push(outcome.message);
                            }
                        }
                        ToolApprovalOutcome::Denied => {
                            self.record_approval_state(
                                &turn_id,
                                request.clone(),
                                ApprovalJournalState::Denied,
                            )?;
                            let record = ToolCallRecord {
                                call_id: request.call_id,
                                tool_name: request.tool_name,
                                execution: request.execution,
                                terminal_state: ToolCallTerminalState::DeniedByApproval,
                                mcp_correlation: request.mcp_correlation,
                                result_disposition: ToolCallResultDisposition::Error,
                            };
                            if self
                                .record_tool_call_terminal(record.clone(), Some(turn_id.as_str()))?
                            {
                                all_events.push(InternalEvent::ToolCallTerminal {
                                    record: record.clone(),
                                });
                            }
                            tool_results.push(InternalMessage::ToolResult {
                                call_id: record.call_id,
                                output: serde_json::json!({"error": "approval denied"}),
                                is_error: true,
                            });
                        }
                    }
                }
            }

            if tool_results.is_empty() {
                self.append_journal_record(JournalRecordKind::TurnFinished {
                    turn_id,
                    reason: JournalTurnFinishReason::Completed,
                    error_message: None,
                })?;
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

    fn record_interrupted_calls(
        &mut self,
        events: &[InternalEvent],
        turn_id: Option<&str>,
    ) -> Result<(), ProviderError> {
        for call in tool_calls_from_events(events) {
            let Some(tool_def) = self.tool_runtime.tool_def(&call.name) else {
                continue;
            };
            let Ok(call) = self.tool_runtime.prepare_call(call, &tool_def) else {
                continue;
            };
            self.record_tool_call_terminal(
                ToolCallRecord {
                    call_id: call.call_id,
                    tool_name: call.name,
                    execution: call.execution,
                    terminal_state: ToolCallTerminalState::Interrupted,
                    mcp_correlation: call.mcp_correlation,
                    result_disposition: ToolCallResultDisposition::None,
                },
                turn_id,
            )?;
        }
        if let Some(turn_id) = turn_id {
            let pending_requests: Vec<_> = self
                .approvals
                .iter()
                .filter(|approval| {
                    approval.state == ApprovalJournalState::Pending
                        && approval.turn_id.as_deref() == Some(turn_id)
                })
                .map(|approval| approval.request.clone())
                .collect();
            for request in pending_requests {
                self.record_approval_state(turn_id, request, ApprovalJournalState::Interrupted)?;
            }
        }
        Ok(())
    }

    fn record_tool_call_terminal(
        &mut self,
        record: ToolCallRecord,
        turn_id: Option<&str>,
    ) -> Result<bool, ProviderError> {
        if !Self::push_unique_tool_call_record(&mut self.tool_call_records, record.clone()) {
            return Ok(false);
        }
        self.append_journal_record(JournalRecordKind::ToolCallTerminalRecorded {
            turn_id: turn_id.map(str::to_string),
            record,
        })?;
        Ok(true)
    }

    fn push_unique_tool_call_record(
        records: &mut Vec<ToolCallRecord>,
        record: ToolCallRecord,
    ) -> bool {
        if records
            .iter()
            .any(|existing| existing.call_id == record.call_id)
        {
            return false;
        }

        records.push(record);
        true
    }

    fn append_messages_to_journal(
        &mut self,
        turn_id: Option<&str>,
        messages: impl IntoIterator<Item = InternalMessage>,
    ) -> Result<Option<u64>, ProviderError> {
        let mut first_sequence_id = None;
        for message in messages {
            let record = self.append_journal_record(JournalRecordKind::MessageAppended {
                turn_id: turn_id.map(str::to_string),
                message,
            })?;
            if first_sequence_id.is_none() {
                first_sequence_id = Some(record.sequence_id);
            }
        }
        Ok(first_sequence_id)
    }

    fn record_approval_state(
        &mut self,
        turn_id: &str,
        request: ToolApprovalRequest,
        state: ApprovalJournalState,
    ) -> Result<(), ProviderError> {
        if let Some(existing) = self
            .approvals
            .iter_mut()
            .find(|approval| approval.request.call_id == request.call_id)
        {
            existing.turn_id = Some(turn_id.to_string());
            existing.state = state.clone();
            existing.request = request.clone();
        } else {
            self.approvals.push(logos_journal::ApprovalView {
                turn_id: Some(turn_id.to_string()),
                request: request.clone(),
                state: state.clone(),
            });
        }
        self.append_journal_record(JournalRecordKind::ApprovalStateChanged {
            turn_id: Some(turn_id.to_string()),
            approval: ApprovalViewRecord { request, state },
        })?;
        Ok(())
    }

    fn append_journal_record(
        &self,
        kind: JournalRecordKind,
    ) -> Result<logos_journal::JournalRecord, ProviderError> {
        match &self.journal {
            Some(journal) => journal
                .append(kind)
                .map_err(|error| self.journal_error(error)),
            None => Ok(logos_journal::JournalRecord::new(
                self.session_id.clone(),
                0,
                kind,
            )),
        }
    }

    fn session_opened_record(&self) -> SessionOpenedRecord {
        SessionOpenedRecord {
            model_ref: self.model_ref.clone(),
            system: self.system.clone(),
            provider_capabilities: self.provider_capabilities.clone(),
            model_capabilities: self.model_capabilities.clone(),
            continuation_mode: self.provider_capabilities.continuation_mode.clone(),
        }
    }

    fn journal_error(&self, error: logos_journal::JournalError) -> ProviderError {
        ProviderError::new(
            self.model_ref.provider_id.clone(),
            ProviderErrorKind::Internal,
            error.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use logos_runtime_types::ToolExecutionKind;

    use super::SessionEngine;
    use super::ToolCallRecord;
    use super::ToolCallResultDisposition;
    use super::ToolCallTerminalState;

    #[test]
    fn duplicate_terminal_records_are_ignored_by_call_id() {
        let mut records = Vec::new();

        assert!(SessionEngine::push_unique_tool_call_record(
            &mut records,
            tool_call_record(ToolCallTerminalState::CompletedWithResult),
        ));
        assert!(!SessionEngine::push_unique_tool_call_record(
            &mut records,
            tool_call_record(ToolCallTerminalState::Interrupted),
        ));
        assert_eq!(
            records,
            vec![tool_call_record(ToolCallTerminalState::CompletedWithResult)]
        );
    }

    fn tool_call_record(terminal_state: ToolCallTerminalState) -> ToolCallRecord {
        ToolCallRecord {
            call_id: "call-1".to_string(),
            tool_name: "logos_tool".to_string(),
            execution: ToolExecutionKind::RuntimeOwned,
            terminal_state,
            mcp_correlation: None,
            result_disposition: ToolCallResultDisposition::Success,
        }
    }
}
