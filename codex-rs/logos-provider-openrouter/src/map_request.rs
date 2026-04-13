use serde::Serialize;
use serde_json::Value;

use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use logos_runtime_types::ReasoningPolicy;
use logos_runtime_types::ToolExecutionKind;

const OPENROUTER_PROVIDER_ID: &str = "openrouter";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenRouterResponsesRequest {
    pub model: String,
    pub input: Vec<OpenRouterInputItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<OpenRouterTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum OpenRouterInputItem {
    #[serde(rename = "message")]
    Message {
        role: String,
        content: Vec<OpenRouterContentPart>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    #[serde(rename = "function_call")]
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "function_call_output")]
    FunctionCallOutput { call_id: String, output: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum OpenRouterContentPart {
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        annotations: Vec<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum OpenRouterTool {
    #[serde(rename = "function")]
    Function {
        name: String,
        description: String,
        parameters: Value,
    },
}

pub fn map_turn_request(
    request: &InternalTurnRequest,
) -> Result<OpenRouterResponsesRequest, ProviderError> {
    reject_unsupported_controls(request)?;

    let mut input = Vec::new();
    if let Some(system) = &request.system {
        input.push(OpenRouterInputItem::Message {
            role: "system".to_string(),
            content: vec![OpenRouterContentPart::InputText {
                text: system.clone(),
            }],
            id: None,
            status: None,
        });
    }
    for message in &request.messages {
        input.push(map_message(message)?);
    }

    Ok(OpenRouterResponsesRequest {
        model: request.model_ref.model_id.clone(),
        input,
        tools: request
            .tools
            .iter()
            .map(map_tool_def)
            .collect::<Result<Vec<_>, _>>()?,
        max_output_tokens: request.limits.max_output_tokens,
    })
}

fn reject_unsupported_controls(request: &InternalTurnRequest) -> Result<(), ProviderError> {
    match request.continuation.as_ref() {
        None | Some(ContinuationHandle::ClientManagedHistory) => {}
        Some(other) => {
            return Err(ProviderError::new(
                OPENROUTER_PROVIDER_ID,
                ProviderErrorKind::UnsupportedCapability,
                format!(
                    "OpenRouter stateless validation adapter does not accept continuation handle {other:?}"
                ),
            ));
        }
    }

    if request.output_mode != OutputMode::Text {
        return Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter currently supports text output only",
        ));
    }

    if request.reasoning_policy != ReasoningPolicy::Default {
        return Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter reasoning semantics remain UNKNOWN in the current phase-5 scope",
        ));
    }

    for tool in &request.tools {
        if tool.execution == ToolExecutionKind::ProviderNative {
            return Err(ProviderError::new(
                OPENROUTER_PROVIDER_ID,
                ProviderErrorKind::UnsupportedCapability,
                format!(
                    "OpenRouter beta adapter does not support provider-native tool '{}'",
                    tool.name
                ),
            ));
        }
    }

    if request.limits.max_input_tokens.is_some() || request.limits.timeout_ms.is_some() {
        return Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter only supports max_output_tokens in the current scope",
        ));
    }

    Ok(())
}

fn map_message(message: &InternalMessage) -> Result<OpenRouterInputItem, ProviderError> {
    match message {
        InternalMessage::User { parts } => Ok(OpenRouterInputItem::Message {
            role: "user".to_string(),
            content: map_input_parts(parts)?,
            id: None,
            status: None,
        }),
        InternalMessage::Assistant { message_id, parts } => {
            let id = message_id.clone().ok_or_else(|| {
                ProviderError::new(
                    OPENROUTER_PROVIDER_ID,
                    ProviderErrorKind::InvalidRequest,
                    "OpenRouter replay requires assistant message ids in history",
                )
            })?;
            Ok(OpenRouterInputItem::Message {
                role: "assistant".to_string(),
                content: map_output_parts(parts)?,
                id: Some(id),
                status: Some("completed".to_string()),
            })
        }
        InternalMessage::AssistantToolCall {
            call_id,
            name,
            arguments,
        } => Ok(OpenRouterInputItem::FunctionCall {
            call_id: call_id.clone(),
            name: name.clone(),
            arguments: arguments.to_string(),
        }),
        InternalMessage::ToolResult {
            call_id, output, ..
        } => Ok(OpenRouterInputItem::FunctionCallOutput {
            call_id: call_id.clone(),
            output: output.to_string(),
        }),
        InternalMessage::CompactionCheckpoint { .. } => Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter beta adapter cannot replay compaction records yet",
        )),
    }
}

fn map_tool_def(tool: &InternalToolDef) -> Result<OpenRouterTool, ProviderError> {
    match tool.execution {
        ToolExecutionKind::RuntimeOwned | ToolExecutionKind::McpBackedRuntime => {
            Ok(OpenRouterTool::Function {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            })
        }
        ToolExecutionKind::ProviderNative => Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            format!(
                "OpenRouter beta adapter does not support provider-native tool '{}'",
                tool.name
            ),
        )),
    }
}

fn map_input_parts(parts: &[ContentPart]) -> Result<Vec<OpenRouterContentPart>, ProviderError> {
    parts.iter().map(map_input_part).collect()
}

fn map_input_part(part: &ContentPart) -> Result<OpenRouterContentPart, ProviderError> {
    match part {
        ContentPart::Text { text } => Ok(OpenRouterContentPart::InputText { text: text.clone() }),
        ContentPart::ImageRef { .. } | ContentPart::FileRef { .. } => Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter currently supports text message parts only",
        )),
    }
}

fn map_output_parts(parts: &[ContentPart]) -> Result<Vec<OpenRouterContentPart>, ProviderError> {
    parts.iter().map(map_output_part).collect()
}

fn map_output_part(part: &ContentPart) -> Result<OpenRouterContentPart, ProviderError> {
    match part {
        ContentPart::Text { text } => Ok(OpenRouterContentPart::OutputText {
            text: text.clone(),
            annotations: Vec::new(),
        }),
        ContentPart::ImageRef { .. } | ContentPart::FileRef { .. } => Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter currently supports text message parts only",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use logos_runtime_types::ContinuationHandle;
    use logos_runtime_types::InternalMessage;
    use logos_runtime_types::InternalToolDef;
    use logos_runtime_types::InternalTurnRequest;
    use logos_runtime_types::ModelRef;
    use logos_runtime_types::OutputMode;
    use logos_runtime_types::ProviderErrorKind;
    use logos_runtime_types::ReasoningPolicy;
    use logos_runtime_types::ToolExecutionKind;
    use logos_runtime_types::TurnLimits;
    use pretty_assertions::assert_eq;

    use super::OpenRouterContentPart;
    use super::OpenRouterInputItem;
    use super::OpenRouterResponsesRequest;
    use super::OpenRouterTool;
    use super::map_turn_request;

    #[test]
    fn maps_text_history_without_previous_response_id() {
        let request = request();

        assert_eq!(
            map_turn_request(&request).expect("map should succeed"),
            OpenRouterResponsesRequest {
                model: "openai/o4-mini".to_string(),
                input: vec![
                    OpenRouterInputItem::Message {
                        role: "system".to_string(),
                        content: vec![OpenRouterContentPart::InputText {
                            text: "You are helpful".to_string(),
                        }],
                        id: None,
                        status: None,
                    },
                    OpenRouterInputItem::Message {
                        role: "user".to_string(),
                        content: vec![OpenRouterContentPart::InputText {
                            text: "hello".to_string(),
                        }],
                        id: None,
                        status: None,
                    },
                    OpenRouterInputItem::Message {
                        role: "assistant".to_string(),
                        content: vec![OpenRouterContentPart::OutputText {
                            text: "hi".to_string(),
                            annotations: Vec::new(),
                        }],
                        id: Some("msg-1".to_string()),
                        status: Some("completed".to_string()),
                    },
                ],
                tools: Vec::new(),
                max_output_tokens: Some(256),
            }
        );
    }

    #[test]
    fn maps_runtime_tools_and_tool_replay_items() {
        let mut request = request();
        request.tools = vec![InternalToolDef {
            name: "logos_echo".to_string(),
            description: "Echo text".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                },
                "required": ["text"]
            }),
            output_schema: None,
            execution: ToolExecutionKind::RuntimeOwned,
            metadata: BTreeMap::new(),
        }];
        request.messages.extend([
            InternalMessage::AssistantToolCall {
                call_id: "call-1".to_string(),
                name: "logos_echo".to_string(),
                arguments: serde_json::json!({"text": "hi"}),
            },
            InternalMessage::ToolResult {
                call_id: "call-1".to_string(),
                output: serde_json::json!({"text": "hi"}),
                is_error: false,
            },
        ]);

        assert_eq!(
            map_turn_request(&request).expect("map should succeed"),
            OpenRouterResponsesRequest {
                model: "openai/o4-mini".to_string(),
                input: vec![
                    OpenRouterInputItem::Message {
                        role: "system".to_string(),
                        content: vec![OpenRouterContentPart::InputText {
                            text: "You are helpful".to_string(),
                        }],
                        id: None,
                        status: None,
                    },
                    OpenRouterInputItem::Message {
                        role: "user".to_string(),
                        content: vec![OpenRouterContentPart::InputText {
                            text: "hello".to_string(),
                        }],
                        id: None,
                        status: None,
                    },
                    OpenRouterInputItem::Message {
                        role: "assistant".to_string(),
                        content: vec![OpenRouterContentPart::OutputText {
                            text: "hi".to_string(),
                            annotations: Vec::new(),
                        }],
                        id: Some("msg-1".to_string()),
                        status: Some("completed".to_string()),
                    },
                    OpenRouterInputItem::FunctionCall {
                        call_id: "call-1".to_string(),
                        name: "logos_echo".to_string(),
                        arguments: "{\"text\":\"hi\"}".to_string(),
                    },
                    OpenRouterInputItem::FunctionCallOutput {
                        call_id: "call-1".to_string(),
                        output: "{\"text\":\"hi\"}".to_string(),
                    },
                ],
                tools: vec![OpenRouterTool::Function {
                    name: "logos_echo".to_string(),
                    description: "Echo text".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": {"type": "string"}
                        },
                        "required": ["text"]
                    }),
                }],
                max_output_tokens: Some(256),
            }
        );
    }

    #[test]
    fn rejects_non_client_managed_continuation() {
        let mut request = request();
        request.continuation = Some(ContinuationHandle::OpenAiPreviousResponseId(
            "resp-1".to_string(),
        ));

        let error = map_turn_request(&request).expect_err("map should fail");
        assert_eq!(error.kind, ProviderErrorKind::UnsupportedCapability);
        assert!(
            error
                .message
                .contains("does not accept continuation handle")
        );
    }

    #[test]
    fn rejects_unknown_reasoning_controls() {
        let mut request = request();
        request.reasoning_policy = ReasoningPolicy::High;

        let error = map_turn_request(&request).expect_err("map should fail");
        assert_eq!(error.kind, ProviderErrorKind::UnsupportedCapability);
        assert!(error.message.contains("UNKNOWN"));
    }

    #[test]
    fn rejects_provider_native_tools() {
        let mut request = request();
        request.tools = vec![InternalToolDef {
            name: "provider_search".to_string(),
            description: "Search".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            execution: ToolExecutionKind::ProviderNative,
            metadata: BTreeMap::new(),
        }];

        let error = map_turn_request(&request).expect_err("map should fail");
        assert_eq!(error.kind, ProviderErrorKind::UnsupportedCapability);
        assert!(
            error
                .message
                .contains("provider-native tool 'provider_search'")
        );
    }

    fn request() -> InternalTurnRequest {
        InternalTurnRequest {
            model_ref: ModelRef {
                provider_id: "openrouter".to_string(),
                model_id: "openai/o4-mini".to_string(),
            },
            system: Some("You are helpful".to_string()),
            messages: vec![
                InternalMessage::user_text("hello"),
                InternalMessage::Assistant {
                    message_id: Some("msg-1".to_string()),
                    parts: vec![logos_runtime_types::ContentPart::Text {
                        text: "hi".to_string(),
                    }],
                },
            ],
            tools: Vec::new(),
            output_mode: OutputMode::Text,
            reasoning_policy: ReasoningPolicy::Default,
            continuation: Some(ContinuationHandle::ClientManagedHistory),
            limits: TurnLimits {
                max_input_tokens: None,
                max_output_tokens: Some(256),
                timeout_ms: None,
            },
            metadata: BTreeMap::new(),
        }
    }
}
