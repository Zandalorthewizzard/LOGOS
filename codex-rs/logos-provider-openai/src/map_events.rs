use codex_core::ResponseEvent;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ReasoningItemReasoningSummary;
use codex_protocol::models::ResponseItem;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;

pub fn map_response_event(event: ResponseEvent) -> Vec<InternalEvent> {
    match event {
        ResponseEvent::Created
        | ResponseEvent::ServerModel(_)
        | ResponseEvent::ServerReasoningIncluded(_)
        | ResponseEvent::RateLimits(_)
        | ResponseEvent::ModelsEtag(_) => Vec::new(),
        ResponseEvent::OutputTextDelta(text) => vec![InternalEvent::TextDelta { text }],
        ResponseEvent::ReasoningSummaryDelta { delta, .. }
        | ResponseEvent::ReasoningContentDelta { delta, .. } => {
            vec![InternalEvent::ReasoningDelta { text: delta }]
        }
        ResponseEvent::ReasoningSummaryPartAdded { .. } => Vec::new(),
        ResponseEvent::OutputItemAdded(item) => map_output_item_added(item),
        ResponseEvent::OutputItemDone(item) => map_output_item_done(item),
        ResponseEvent::Completed {
            response_id,
            token_usage,
        } => {
            let mut events = Vec::new();
            if let Some(usage) = token_usage {
                events.push(InternalEvent::Usage {
                    input_tokens: usage.input_tokens.max(0) as u64,
                    output_tokens: usage.output_tokens.max(0) as u64,
                });
            }
            events.push(InternalEvent::ContinuationUpdated {
                handle: ContinuationHandle::OpenAiPreviousResponseId(response_id),
            });
            events.push(InternalEvent::Completed);
            events
        }
    }
}

fn map_output_item_added(item: ResponseItem) -> Vec<InternalEvent> {
    match item {
        ResponseItem::Message { id, .. } => vec![InternalEvent::MessageStarted { message_id: id }],
        ResponseItem::Reasoning {
            summary, content, ..
        } => {
            let mut text = String::new();
            append_reasoning_summary(&mut text, &summary);
            append_reasoning_content(&mut text, content.as_deref());
            if text.is_empty() {
                Vec::new()
            } else {
                vec![InternalEvent::ReasoningDelta { text }]
            }
        }
        ResponseItem::FunctionCall { call_id, name, .. }
        | ResponseItem::CustomToolCall { call_id, name, .. } => {
            vec![InternalEvent::ToolCallStarted { call_id, name }]
        }
        ResponseItem::ToolSearchCall {
            call_id: Some(call_id),
            ..
        } => {
            vec![InternalEvent::ToolCallStarted {
                call_id,
                name: "tool_search".to_string(),
            }]
        }
        ResponseItem::LocalShellCall {
            call_id: Some(call_id),
            ..
        } => {
            vec![InternalEvent::ToolCallStarted {
                call_id,
                name: "local_shell".to_string(),
            }]
        }
        ResponseItem::LocalShellCall {
            id: Some(call_id), ..
        } => {
            vec![InternalEvent::ToolCallStarted {
                call_id,
                name: "local_shell".to_string(),
            }]
        }
        _ => Vec::new(),
    }
}

fn map_output_item_done(item: ResponseItem) -> Vec<InternalEvent> {
    match item {
        ResponseItem::Message { id, .. } => {
            vec![InternalEvent::MessageCompleted { message_id: id }]
        }
        ResponseItem::FunctionCall {
            call_id,
            name,
            arguments,
            ..
        } => vec![
            InternalEvent::ToolCallStarted {
                call_id: call_id.clone(),
                name,
            },
            InternalEvent::ToolCallArgumentsDelta {
                call_id: call_id.clone(),
                delta: arguments,
            },
            InternalEvent::ToolCallFinished { call_id },
        ],
        ResponseItem::CustomToolCall {
            call_id,
            name,
            input,
            ..
        } => vec![
            InternalEvent::ToolCallStarted {
                call_id: call_id.clone(),
                name,
            },
            InternalEvent::ToolCallArgumentsDelta {
                call_id: call_id.clone(),
                delta: input,
            },
            InternalEvent::ToolCallFinished { call_id },
        ],
        ResponseItem::ToolSearchCall {
            call_id: Some(call_id),
            arguments,
            ..
        } => vec![
            InternalEvent::ToolCallStarted {
                call_id: call_id.clone(),
                name: "tool_search".to_string(),
            },
            InternalEvent::ToolCallArgumentsDelta {
                call_id: call_id.clone(),
                delta: arguments.to_string(),
            },
            InternalEvent::ToolCallFinished { call_id },
        ],
        ResponseItem::LocalShellCall {
            id,
            call_id,
            action,
            ..
        } => {
            let stable_call_id = call_id.or(id).unwrap_or_default();
            vec![
                InternalEvent::ToolCallStarted {
                    call_id: stable_call_id.clone(),
                    name: "local_shell".to_string(),
                },
                InternalEvent::ToolCallArgumentsDelta {
                    call_id: stable_call_id.clone(),
                    delta: serde_json::to_string(&action).unwrap_or_default(),
                },
                InternalEvent::ToolCallFinished {
                    call_id: stable_call_id,
                },
            ]
        }
        ResponseItem::Reasoning {
            summary, content, ..
        } => {
            let mut text = String::new();
            append_reasoning_summary(&mut text, &summary);
            append_reasoning_content(&mut text, content.as_deref());
            if text.is_empty() {
                Vec::new()
            } else {
                vec![InternalEvent::ReasoningDelta { text }]
            }
        }
        _ => Vec::new(),
    }
}

fn append_reasoning_summary(buffer: &mut String, summary: &[ReasoningItemReasoningSummary]) {
    for entry in summary {
        let ReasoningItemReasoningSummary::SummaryText { text } = entry;
        buffer.push_str(text);
    }
}

fn append_reasoning_content(buffer: &mut String, content: Option<&[ReasoningItemContent]>) {
    let Some(content) = content else {
        return;
    };
    for entry in content {
        match entry {
            ReasoningItemContent::ReasoningText { text } | ReasoningItemContent::Text { text } => {
                buffer.push_str(text)
            }
        }
    }
}
