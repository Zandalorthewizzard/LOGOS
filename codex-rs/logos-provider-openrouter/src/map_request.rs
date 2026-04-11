use serde::Serialize;
use serde_json::Value;

use logos_runtime_types::ContentPart;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use logos_runtime_types::ReasoningPolicy;

const OPENROUTER_PROVIDER_ID: &str = "openrouter";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenRouterResponsesRequest {
    pub model: String,
    pub input: Vec<OpenRouterInputItem>,
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

    if !request.tools.is_empty() {
        return Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter does not support tools in the current text-only scope",
        ));
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
        InternalMessage::AssistantToolCall { .. }
        | InternalMessage::CompactionCheckpoint { .. }
        | InternalMessage::ToolResult { .. } => Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter validation adapter cannot replay tool or compaction records in the current scope",
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
    use logos_runtime_types::InternalTurnRequest;
    use logos_runtime_types::ModelRef;
    use logos_runtime_types::OutputMode;
    use logos_runtime_types::ProviderErrorKind;
    use logos_runtime_types::ReasoningPolicy;
    use logos_runtime_types::TurnLimits;
    use pretty_assertions::assert_eq;

    use super::OpenRouterContentPart;
    use super::OpenRouterInputItem;
    use super::OpenRouterResponsesRequest;
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
