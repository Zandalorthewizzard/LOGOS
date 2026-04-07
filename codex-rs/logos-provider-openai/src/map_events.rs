use codex_core::ResponseEvent;
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
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use codex_core::ResponseEvent;
    use codex_protocol::models::ReasoningItemReasoningSummary;
    use codex_protocol::models::ResponseItem;
    use logos_runtime_types::InternalEvent;
    use pretty_assertions::assert_eq;

    use super::map_response_event;

    #[test]
    fn output_item_added_and_done_do_not_duplicate_tool_start() {
        let events = [
            ResponseEvent::OutputItemAdded(ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: "{\"command\":\"pwd\"}".to_string(),
                call_id: "call-1".to_string(),
            }),
            ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: "{\"command\":\"pwd\"}".to_string(),
                call_id: "call-1".to_string(),
            }),
        ]
        .into_iter()
        .flat_map(map_response_event)
        .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                InternalEvent::ToolCallStarted {
                    call_id: "call-1".to_string(),
                    name: "shell".to_string(),
                },
                InternalEvent::ToolCallArgumentsDelta {
                    call_id: "call-1".to_string(),
                    delta: "{\"command\":\"pwd\"}".to_string(),
                },
                InternalEvent::ToolCallFinished {
                    call_id: "call-1".to_string(),
                },
            ]
        );
    }

    #[test]
    fn reasoning_item_added_and_done_do_not_duplicate_reasoning_text() {
        let events = [
            ResponseEvent::OutputItemAdded(ResponseItem::Reasoning {
                id: "rs-1".to_string(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "summary".to_string(),
                }],
                content: None,
                encrypted_content: None,
            }),
            ResponseEvent::ReasoningSummaryDelta {
                summary_index: 0,
                delta: "summary".to_string(),
            },
            ResponseEvent::OutputItemDone(ResponseItem::Reasoning {
                id: "rs-1".to_string(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "summary".to_string(),
                }],
                content: None,
                encrypted_content: None,
            }),
        ]
        .into_iter()
        .flat_map(map_response_event)
        .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![InternalEvent::ReasoningDelta {
                text: "summary".to_string(),
            }]
        );
    }
}
