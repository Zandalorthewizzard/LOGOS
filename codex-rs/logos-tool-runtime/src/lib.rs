use std::collections::BTreeMap;

use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments_json: String,
}

#[derive(Debug, Default)]
pub struct ToolCallCollector {
    calls: BTreeMap<String, PendingToolCall>,
}

impl ToolCallCollector {
    pub fn observe(&mut self, event: &InternalEvent) {
        match event {
            InternalEvent::ToolCallStarted { call_id, name } => {
                self.calls.insert(
                    call_id.clone(),
                    PendingToolCall {
                        call_id: call_id.clone(),
                        name: name.clone(),
                        arguments_json: String::new(),
                    },
                );
            }
            InternalEvent::ToolCallArgumentsDelta { call_id, delta } => {
                if let Some(call) = self.calls.get_mut(call_id) {
                    call.arguments_json.push_str(delta);
                }
            }
            InternalEvent::ToolCallFinished { .. }
            | InternalEvent::MessageStarted { .. }
            | InternalEvent::TextDelta { .. }
            | InternalEvent::MessageCompleted { .. }
            | InternalEvent::ReasoningDelta { .. }
            | InternalEvent::Usage { .. }
            | InternalEvent::ContinuationUpdated { .. }
            | InternalEvent::ProviderExtension { .. }
            | InternalEvent::RawProviderEvent { .. }
            | InternalEvent::Completed
            | InternalEvent::Failed(_) => {}
        }
    }

    pub fn into_messages(self) -> Vec<InternalMessage> {
        self.calls
            .into_values()
            .map(|call| InternalMessage::AssistantToolCall {
                call_id: call.call_id,
                name: call.name,
                arguments: serde_json::from_str(&call.arguments_json)
                    .unwrap_or(Value::String(call.arguments_json)),
            })
            .collect()
    }
}

pub fn tool_result_message(
    call_id: impl Into<String>,
    output: Value,
    is_error: bool,
) -> InternalMessage {
    InternalMessage::ToolResult {
        call_id: call_id.into(),
        output,
        is_error,
    }
}
