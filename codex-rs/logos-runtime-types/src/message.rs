use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::continuation::ContinuationHandle;
use crate::model::CallId;
use crate::model::ModelRef;
use crate::tools::InternalToolDef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    Text,
    Structured,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningPolicy {
    Off,
    Default,
    Low,
    Medium,
    High,
    ProviderDefault,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentPart {
    Text {
        text: String,
    },
    ImageRef {
        uri: String,
        mime_type: Option<String>,
    },
    FileRef {
        uri: String,
        mime_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InternalMessage {
    User {
        parts: Vec<ContentPart>,
    },
    Assistant {
        message_id: Option<String>,
        parts: Vec<ContentPart>,
    },
    AssistantToolCall {
        call_id: CallId,
        name: String,
        arguments: Value,
    },
    CompactionCheckpoint {
        summary: String,
    },
    ToolResult {
        call_id: CallId,
        output: Value,
        is_error: bool,
    },
}

impl InternalMessage {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self::User {
            parts: vec![ContentPart::Text { text: text.into() }],
        }
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self::Assistant {
            message_id: None,
            parts: vec![ContentPart::Text { text: text.into() }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TurnLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalTurnRequest {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub messages: Vec<InternalMessage>,
    pub tools: Vec<InternalToolDef>,
    pub output_mode: OutputMode,
    pub reasoning_policy: ReasoningPolicy,
    pub continuation: Option<ContinuationHandle>,
    pub limits: TurnLimits,
    pub metadata: BTreeMap<String, Value>,
}
