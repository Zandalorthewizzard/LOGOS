use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use logos_runtime_types::ContentPart;
use logos_runtime_types::InternalMessage;

pub fn map_messages(messages: &[InternalMessage]) -> Vec<ResponseItem> {
    messages.iter().map(map_message).collect()
}

fn map_message(message: &InternalMessage) -> ResponseItem {
    match message {
        InternalMessage::User { parts } => ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: map_user_parts(parts),
            end_turn: None,
            phase: None,
        },
        InternalMessage::Assistant { message_id, parts } => ResponseItem::Message {
            id: message_id.clone(),
            role: "assistant".to_string(),
            content: map_assistant_parts(parts),
            end_turn: None,
            phase: None,
        },
        InternalMessage::AssistantToolCall {
            call_id,
            name,
            arguments,
        } => ResponseItem::FunctionCall {
            id: None,
            name: name.clone(),
            namespace: None,
            arguments: arguments.to_string(),
            call_id: call_id.clone(),
        },
        InternalMessage::ToolResult {
            call_id,
            output,
            is_error: _,
        } => ResponseItem::FunctionCallOutput {
            call_id: call_id.clone(),
            output: FunctionCallOutputPayload::from_text(output.to_string()),
        },
    }
}

fn map_user_parts(parts: &[ContentPart]) -> Vec<ContentItem> {
    parts.iter().map(map_user_part).collect()
}

fn map_user_part(part: &ContentPart) -> ContentItem {
    match part {
        ContentPart::Text { text } => ContentItem::InputText { text: text.clone() },
        ContentPart::ImageRef { uri, .. } => ContentItem::InputImage {
            image_url: uri.clone(),
        },
        ContentPart::FileRef { uri, .. } => ContentItem::InputText { text: uri.clone() },
    }
}

fn map_assistant_parts(parts: &[ContentPart]) -> Vec<ContentItem> {
    parts.iter().map(map_assistant_part).collect()
}

fn map_assistant_part(part: &ContentPart) -> ContentItem {
    match part {
        ContentPart::Text { text } => ContentItem::OutputText { text: text.clone() },
        ContentPart::ImageRef { uri, .. } | ContentPart::FileRef { uri, .. } => {
            ContentItem::OutputText { text: uri.clone() }
        }
    }
}
