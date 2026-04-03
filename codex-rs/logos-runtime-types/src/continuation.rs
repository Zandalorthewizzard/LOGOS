use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationHandle {
    OpenAiPreviousResponseId(String),
    ProviderSessionId(String),
    ClientManagedHistory,
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationMode {
    ServerManaged,
    ClientManaged,
    Hybrid,
}
