use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelMeta;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ProviderError;

pub type ProviderEventStream =
    Pin<Box<dyn Stream<Item = Result<InternalEvent, ProviderError>> + Send>>;

/// Provider-owned streaming session that preserves transport-specific state across turns.
#[async_trait]
pub trait ProviderSession: Send {
    async fn stream_turn(
        &mut self,
        request: InternalTurnRequest,
    ) -> Result<ProviderEventStream, ProviderError>;
}

/// Adapter contract for a Logos provider implementation.
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn provider_capabilities(&self) -> ProviderCapabilitySet;

    fn model_capabilities(&self, model: &ModelRef) -> ModelCapabilitySet;

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError>;

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError>;
}
