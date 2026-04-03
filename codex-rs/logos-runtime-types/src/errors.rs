use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::model::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderErrorKind {
    Authentication,
    RateLimited,
    ContextLimit,
    InvalidRequest,
    Transport,
    ProviderUnavailable,
    UnsupportedCapability,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderError {
    pub provider_id: ProviderId,
    pub kind: ProviderErrorKind,
    pub message: String,
    pub retryable: bool,
    pub metadata: BTreeMap<String, Value>,
}

impl ProviderError {
    pub fn new(
        provider_id: impl Into<ProviderId>,
        kind: ProviderErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            kind,
            message: message.into(),
            retryable: false,
            metadata: BTreeMap::new(),
        }
    }
}
