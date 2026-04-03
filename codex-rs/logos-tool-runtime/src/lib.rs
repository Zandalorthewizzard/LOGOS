use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments_json: String,
}

impl PendingToolCall {
    pub fn arguments(&self) -> Value {
        serde_json::from_str(&self.arguments_json)
            .unwrap_or_else(|_| Value::String(self.arguments_json.clone()))
    }

    pub fn into_message(self) -> InternalMessage {
        let arguments = self.arguments();
        InternalMessage::AssistantToolCall {
            call_id: self.call_id,
            name: self.name,
            arguments,
        }
    }
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

    pub fn into_calls(self) -> Vec<PendingToolCall> {
        self.calls.into_values().collect()
    }

    pub fn into_messages(self) -> Vec<InternalMessage> {
        self.into_calls()
            .into_iter()
            .map(PendingToolCall::into_message)
            .collect()
    }
}

#[async_trait]
pub trait RuntimeToolExecutor: Send + Sync {
    async fn execute(&self, arguments: Value) -> Result<Value, String>;
}

#[derive(Clone)]
struct RegisteredRuntimeTool {
    def: InternalToolDef,
    executor: Arc<dyn RuntimeToolExecutor>,
}

#[derive(Clone, Default)]
pub struct ToolRuntime {
    tools: BTreeMap<String, RegisteredRuntimeTool>,
}

impl ToolRuntime {
    pub fn register_tool(&mut self, def: InternalToolDef, executor: Arc<dyn RuntimeToolExecutor>) {
        self.tools
            .insert(def.name.clone(), RegisteredRuntimeTool { def, executor });
    }

    pub fn declared_tools(&self) -> Vec<InternalToolDef> {
        self.tools.values().map(|tool| tool.def.clone()).collect()
    }

    pub async fn execute_call(&self, call: &PendingToolCall) -> Option<InternalMessage> {
        let tool = self.tools.get(&call.name)?;
        let output = match tool.executor.execute(call.arguments()).await {
            Ok(output) => {
                tool_result_message(call.call_id.clone(), output, /*is_error*/ false)
            }
            Err(message) => tool_result_message(
                call.call_id.clone(),
                Value::Object(serde_json::Map::from_iter([(
                    "error".to_string(),
                    Value::String(message),
                )])),
                /*is_error*/ true,
            ),
        };
        Some(output)
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
