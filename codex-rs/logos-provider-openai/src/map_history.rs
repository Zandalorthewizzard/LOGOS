use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use logos_runtime_types::ContentPart;
use logos_runtime_types::InternalMessage;

pub fn map_compacted_history(items: Vec<ResponseItem>) -> Vec<InternalMessage> {
    items.into_iter().filter_map(map_compacted_item).collect()
}

fn map_compacted_item(item: ResponseItem) -> Option<InternalMessage> {
    match item {
        ResponseItem::Message { role, content, .. } if role == "user" => {
            Some(InternalMessage::User {
                parts: map_parts(content),
            })
        }
        ResponseItem::Message {
            role, content, id, ..
        } if role == "assistant" => Some(InternalMessage::Assistant {
            message_id: id,
            parts: map_parts(content),
        }),
        ResponseItem::Compaction { encrypted_content } => {
            Some(InternalMessage::CompactionCheckpoint {
                summary: encrypted_content,
            })
        }
        ResponseItem::Message { role, .. } if role == "developer" => None,
        _ => None,
    }
}

fn map_parts(content: Vec<ContentItem>) -> Vec<ContentPart> {
    content
        .into_iter()
        .filter_map(|item| match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                Some(ContentPart::Text { text })
            }
            ContentItem::InputImage { image_url } => Some(ContentPart::ImageRef {
                uri: image_url,
                mime_type: None,
            }),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use codex_protocol::models::ContentItem;
    use codex_protocol::models::ResponseItem;
    use logos_runtime_types::ContentPart;
    use logos_runtime_types::InternalMessage;
    use pretty_assertions::assert_eq;

    use super::map_compacted_history;

    #[test]
    fn compacted_history_normalizes_compaction_and_messages() {
        let mapped = map_compacted_history(vec![
            ResponseItem::Message {
                id: None,
                role: "developer".to_string(),
                content: vec![ContentItem::InputText {
                    text: "ignore".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: Some("msg-1".to_string()),
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "assistant".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: "{}".to_string(),
                call_id: "call-1".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "call-1".to_string(),
                output: codex_protocol::models::FunctionCallOutputPayload::from_text(
                    "ignored".to_string(),
                ),
            },
            ResponseItem::Reasoning {
                id: "rsn-1".to_string(),
                summary: Vec::new(),
                content: None,
                encrypted_content: None,
            },
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Compaction {
                encrypted_content: "summary".to_string(),
            },
        ]);

        assert_eq!(
            mapped,
            vec![
                InternalMessage::Assistant {
                    message_id: Some("msg-1".to_string()),
                    parts: vec![ContentPart::Text {
                        text: "assistant".to_string(),
                    }],
                },
                InternalMessage::User {
                    parts: vec![ContentPart::Text {
                        text: "hello".to_string(),
                    }],
                },
                InternalMessage::CompactionCheckpoint {
                    summary: "summary".to_string(),
                },
            ]
        );
    }
}
