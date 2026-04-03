use std::sync::Arc;

use futures::StreamExt;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderSession;
use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ReasoningPolicy;
use logos_runtime_types::TurnLimits;

use crate::tool_loop::tool_messages_from_events;
use crate::turn::TurnOutcome;

pub struct SessionEngine {
    provider_session: Box<dyn ProviderSession>,
    model_ref: ModelRef,
    system: Option<String>,
    history: Vec<InternalMessage>,
    continuation: Option<ContinuationHandle>,
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
        })
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
        let request = InternalTurnRequest {
            model_ref: self.model_ref.clone(),
            system: self.system.clone(),
            messages: self
                .history
                .iter()
                .cloned()
                .chain(new_messages.iter().cloned())
                .collect(),
            tools: Vec::new(),
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
        let mut latest_continuation = self.continuation.clone();
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

        self.history.extend(new_messages);
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

        Ok(TurnOutcome {
            events,
            continuation: latest_continuation,
        })
    }
}
