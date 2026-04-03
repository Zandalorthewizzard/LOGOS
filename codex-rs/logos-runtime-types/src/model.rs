use serde::Deserialize;
use serde::Serialize;

use crate::message::OutputMode;

pub type ProviderId = String;
pub type ModelId = String;
pub type CallId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider_id: ProviderId,
    pub model_id: ModelId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMeta {
    pub model_ref: ModelRef,
    pub display_name: String,
    pub provider_label: String,
    pub context_window: Option<u64>,
    pub default_output_mode: OutputMode,
}
