use serde::Deserialize;
use serde::Serialize;

use crate::message::InternalMessage;
use crate::model::ModelRef;
use crate::tools::InternalToolDef;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalCompactionRequest {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub messages: Vec<InternalMessage>,
    pub tools: Vec<InternalToolDef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalCompactionResult {
    pub replacement_messages: Vec<InternalMessage>,
}
