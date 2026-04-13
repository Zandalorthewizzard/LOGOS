use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;

const OPENROUTER_PROVIDER_ID: &str = "openrouter";

#[derive(Debug, Deserialize)]
pub struct OpenRouterResponsesEnvelope {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub output: Vec<OpenRouterOutputItem>,
    #[serde(default)]
    pub usage: Option<OpenRouterUsage>,
    #[serde(default)]
    pub error: Option<OpenRouterErrorDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OpenRouterOutputItem {
    #[serde(rename = "message")]
    Message {
        id: Option<String>,
        role: String,
        content: Vec<OpenRouterOutputContentPart>,
    },
    #[serde(rename = "function_call")]
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OpenRouterOutputContentPart {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterErrorBody {
    pub error: OpenRouterErrorDetail,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterErrorDetail {
    pub code: Value,
    pub message: String,
    #[serde(default)]
    pub metadata: Option<BTreeMap<String, Value>>,
}

pub fn map_response(
    envelope: OpenRouterResponsesEnvelope,
) -> Result<Vec<InternalEvent>, ProviderError> {
    if let Some(error) = envelope.error {
        return Err(map_api_error(200, error, None));
    }

    let mut events = Vec::new();
    let mut saw_output = false;
    for item in envelope.output {
        match item {
            OpenRouterOutputItem::Message { id, role, content } => {
                if role != "assistant" {
                    continue;
                }
                let message_id = id.ok_or_else(|| {
                    ProviderError::new(
                        OPENROUTER_PROVIDER_ID,
                        ProviderErrorKind::InvalidRequest,
                        "OpenRouter response omitted assistant message id required for replay",
                    )
                })?;
                events.push(InternalEvent::MessageStarted {
                    message_id: Some(message_id.clone()),
                });
                let mut saw_text = false;
                for part in content {
                    if let OpenRouterOutputContentPart::OutputText { text } = part
                        && !text.is_empty()
                    {
                        saw_text = true;
                        events.push(InternalEvent::TextDelta { text });
                    }
                }
                if !saw_text {
                    return Err(ProviderError::new(
                        OPENROUTER_PROVIDER_ID,
                        ProviderErrorKind::UnsupportedCapability,
                        "OpenRouter beta adapter expected assistant text output when an assistant message is present",
                    ));
                }
                events.push(InternalEvent::MessageCompleted {
                    message_id: Some(message_id),
                });
                saw_output = true;
            }
            OpenRouterOutputItem::FunctionCall {
                call_id,
                name,
                arguments,
            } => {
                events.push(InternalEvent::ToolCallStarted {
                    call_id: call_id.clone(),
                    name,
                });
                if !arguments.is_empty() {
                    events.push(InternalEvent::ToolCallArgumentsDelta {
                        call_id: call_id.clone(),
                        delta: arguments,
                    });
                }
                events.push(InternalEvent::ToolCallFinished { call_id });
                saw_output = true;
            }
            OpenRouterOutputItem::Unknown => {}
        }
    }

    if !saw_output {
        return Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::InvalidRequest,
            format!(
                "OpenRouter response '{}' did not contain an assistant message or tool call",
                envelope.id.unwrap_or_else(|| "<unknown>".to_string())
            ),
        ));
    }

    if let Some(usage) = envelope.usage {
        events.push(InternalEvent::Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        });
    }
    events.push(InternalEvent::ContinuationUpdated {
        handle: ContinuationHandle::ClientManagedHistory,
    });
    events.push(InternalEvent::Completed);
    Ok(events)
}

pub fn map_http_error_response(status: u16, body: OpenRouterErrorBody) -> ProviderError {
    map_api_error(status, body.error, body.metadata)
}

pub fn map_http_error_fallback(status: u16, body: &str, parse_error: &str) -> ProviderError {
    let kind = map_status_kind(status);
    let retryable = matches!(
        kind,
        ProviderErrorKind::RateLimited
            | ProviderErrorKind::Transport
            | ProviderErrorKind::ProviderUnavailable
    );
    let mut provider_error = ProviderError::new(
        OPENROUTER_PROVIDER_ID,
        kind,
        format!("OpenRouter returned HTTP {status} with an unparseable error body"),
    );
    provider_error.retryable = retryable;
    provider_error
        .metadata
        .insert("http_status".to_string(), Value::from(status));
    provider_error
        .metadata
        .insert("body_parse_failed".to_string(), Value::Bool(true));
    provider_error.metadata.insert(
        "body_parse_error".to_string(),
        Value::String(parse_error.to_string()),
    );
    if !body.is_empty() {
        provider_error.metadata.insert(
            "raw_body_preview".to_string(),
            Value::String(raw_body_preview(body)),
        );
    }
    provider_error
}

fn map_api_error(
    status: u16,
    error: OpenRouterErrorDetail,
    metadata: Option<Value>,
) -> ProviderError {
    let code = value_to_code(&error.code);
    let kind = match code.as_deref() {
        Some("invalid_api_key") => ProviderErrorKind::Authentication,
        Some("rate_limit_exceeded") => ProviderErrorKind::RateLimited,
        Some(
            "context_length_exceeded"
            | "max_tokens_exceeded"
            | "token_limit_exceeded"
            | "string_too_long",
        ) => ProviderErrorKind::ContextLimit,
        Some("invalid_prompt") => ProviderErrorKind::InvalidRequest,
        Some("server_error") => ProviderErrorKind::ProviderUnavailable,
        _ => map_status_kind(status),
    };

    let retryable = matches!(
        kind,
        ProviderErrorKind::RateLimited
            | ProviderErrorKind::Transport
            | ProviderErrorKind::ProviderUnavailable
    );
    let mut provider_error = ProviderError::new(OPENROUTER_PROVIDER_ID, kind, error.message);
    provider_error.retryable = retryable;
    provider_error
        .metadata
        .insert("http_status".to_string(), Value::from(status));
    if let Some(code) = code {
        provider_error
            .metadata
            .insert("openrouter_error_code".to_string(), Value::String(code));
    }
    if let Some(metadata) = metadata {
        provider_error
            .metadata
            .insert("response_metadata".to_string(), metadata);
    }
    if let Some(extra) = error.metadata {
        provider_error.metadata.insert(
            "error_metadata".to_string(),
            Value::Object(extra.into_iter().collect()),
        );
    }
    provider_error
}

fn value_to_code(value: &Value) -> Option<String> {
    match value {
        Value::String(string) => Some(string.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn map_status_kind(status: u16) -> ProviderErrorKind {
    match status {
        400 => ProviderErrorKind::InvalidRequest,
        401 => ProviderErrorKind::Authentication,
        402 | 429 => ProviderErrorKind::RateLimited,
        408 => ProviderErrorKind::Transport,
        413 => ProviderErrorKind::ContextLimit,
        502..=504 => ProviderErrorKind::ProviderUnavailable,
        500..=599 => ProviderErrorKind::ProviderUnavailable,
        _ => ProviderErrorKind::InvalidRequest,
    }
}

fn raw_body_preview(body: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 240;
    let preview: String = body.chars().take(MAX_PREVIEW_CHARS).collect();
    if body.chars().count() > MAX_PREVIEW_CHARS {
        format!("{preview}...")
    } else {
        preview
    }
}

#[cfg(test)]
mod tests {
    use logos_runtime_types::ContinuationHandle;
    use logos_runtime_types::InternalEvent;
    use logos_runtime_types::ProviderErrorKind;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::OpenRouterErrorBody;
    use super::OpenRouterResponsesEnvelope;
    use super::map_http_error_fallback;
    use super::map_http_error_response;
    use super::map_response;

    #[test]
    fn maps_text_response_into_internal_events() {
        let response: OpenRouterResponsesEnvelope = serde_json::from_value(json!({
            "id": "resp-1",
            "output": [{
                "type": "message",
                "id": "msg-1",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "Hello from OpenRouter."
                }]
            }],
            "usage": {
                "input_tokens": 12,
                "output_tokens": 4
            }
        }))
        .expect("response should parse");

        assert_eq!(
            map_response(response).expect("response should map"),
            vec![
                InternalEvent::MessageStarted {
                    message_id: Some("msg-1".to_string()),
                },
                InternalEvent::TextDelta {
                    text: "Hello from OpenRouter.".to_string(),
                },
                InternalEvent::MessageCompleted {
                    message_id: Some("msg-1".to_string()),
                },
                InternalEvent::Usage {
                    input_tokens: 12,
                    output_tokens: 4,
                },
                InternalEvent::ContinuationUpdated {
                    handle: ContinuationHandle::ClientManagedHistory,
                },
                InternalEvent::Completed,
            ]
        );
    }

    #[test]
    fn maps_tool_call_response_into_internal_events() {
        let response: OpenRouterResponsesEnvelope = serde_json::from_value(json!({
            "id": "resp-tool-1",
            "output": [{
                "type": "function_call",
                "call_id": "call-1",
                "name": "logos_echo",
                "arguments": "{\"text\":\"hi\"}"
            }],
            "usage": {
                "input_tokens": 21,
                "output_tokens": 5
            }
        }))
        .expect("response should parse");

        assert_eq!(
            map_response(response).expect("response should map"),
            vec![
                InternalEvent::ToolCallStarted {
                    call_id: "call-1".to_string(),
                    name: "logos_echo".to_string(),
                },
                InternalEvent::ToolCallArgumentsDelta {
                    call_id: "call-1".to_string(),
                    delta: "{\"text\":\"hi\"}".to_string(),
                },
                InternalEvent::ToolCallFinished {
                    call_id: "call-1".to_string(),
                },
                InternalEvent::Usage {
                    input_tokens: 21,
                    output_tokens: 5,
                },
                InternalEvent::ContinuationUpdated {
                    handle: ContinuationHandle::ClientManagedHistory,
                },
                InternalEvent::Completed,
            ]
        );
    }

    #[test]
    fn normalizes_rate_limit_status() {
        let error: OpenRouterErrorBody = serde_json::from_value(json!({
            "error": {
                "code": "rate_limit_exceeded",
                "message": "Too many requests"
            }
        }))
        .expect("error body should parse");

        let mapped = map_http_error_response(429, error);
        assert_eq!(mapped.kind, ProviderErrorKind::RateLimited);
        assert!(mapped.retryable);
    }

    #[test]
    fn malformed_400_falls_back_to_invalid_request() {
        let mapped = map_http_error_fallback(400, "not-json", "expected value");

        assert_eq!(mapped.kind, ProviderErrorKind::InvalidRequest);
        assert!(!mapped.retryable);
        assert_eq!(mapped.metadata["http_status"], json!(400));
        assert_eq!(mapped.metadata["body_parse_failed"], json!(true));
    }

    #[test]
    fn malformed_401_falls_back_to_authentication() {
        let mapped = map_http_error_fallback(401, "nope", "expected value");

        assert_eq!(mapped.kind, ProviderErrorKind::Authentication);
        assert!(!mapped.retryable);
    }

    #[test]
    fn malformed_429_falls_back_to_rate_limited() {
        let mapped = map_http_error_fallback(429, "try again later", "expected value");

        assert_eq!(mapped.kind, ProviderErrorKind::RateLimited);
        assert!(mapped.retryable);
        assert_eq!(
            mapped.metadata["raw_body_preview"],
            json!("try again later")
        );
    }
}
