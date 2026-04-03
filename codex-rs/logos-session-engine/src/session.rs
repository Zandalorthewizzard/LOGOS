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
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ReasoningPolicy;
use logos_runtime_types::TurnLimits;
use logos_tool_runtime::RuntimeToolExecutor;
use logos_tool_runtime::ToolRuntime;

use crate::tool_loop::tool_calls_from_events;
use crate::tool_loop::tool_messages_from_events;
use crate::turn::TurnOutcome;

pub struct SessionEngine {
    provider_session: Box<dyn ProviderSession>,
    model_ref: ModelRef,
    system: Option<String>,
    history: Vec<InternalMessage>,
    continuation: Option<ContinuationHandle>,
    tool_runtime: ToolRuntime,
}

impl SessionEngine {
    pub async fn new(
        adapter: Arc<dyn ProviderAdapter>,
        model_ref: ModelRef,
        system: Option<String>,
    ) -> Result<Self, ProviderError> {
        Ok(Self {
            provider_session: adapter.new_session().await?,
            model_ref,
            system,
            history: Vec::new(),
            continuation: None,
            tool_runtime: ToolRuntime::default(),
        })
    }

    pub fn register_runtime_tool(
        &mut self,
        def: InternalToolDef,
        executor: Arc<dyn RuntimeToolExecutor>,
    ) {
        self.tool_runtime.register_tool(def, executor);
    }

    pub fn continuation(&self) -> Option<&ContinuationHandle> {
        self.continuation.as_ref()
    }

    pub fn history(&self) -> &[InternalMessage] {
        &self.history
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
        let mut pending_messages = new_messages;
        let mut all_events = Vec::new();
        let mut latest_continuation = self.continuation.clone();
        let mut request_tools = tools;
        request_tools.extend(self.tool_runtime.declared_tools());

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
                if let Some(result) = self.tool_runtime.execute_call(&call).await {
                    tool_results.push(result);
                }
            }

            if tool_results.is_empty() {
                return Ok(TurnOutcome {
                    events: all_events,
                    continuation: latest_continuation,
                });
            }

            pending_messages = tool_results;
        }
    }
}
