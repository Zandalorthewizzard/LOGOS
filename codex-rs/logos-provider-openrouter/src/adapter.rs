use std::sync::Arc;

use async_trait::async_trait;
use codex_model_provider_info::OPENROUTER_VALIDATED_MODEL_ID;
use futures::stream;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderEventStream;
use logos_provider_api::ProviderSession;
use logos_runtime_types::CompactionBackendKind;
use logos_runtime_types::ContinuationMode;
use logos_runtime_types::InternalCompactionRequest;
use logos_runtime_types::InternalCompactionResult;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelMeta;
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use reqwest::Client;

use crate::map_request::map_turn_request;
use crate::responses::OpenRouterErrorBody;
use crate::responses::OpenRouterResponsesEnvelope;
use crate::responses::map_http_error_fallback;
use crate::responses::map_http_error_response;
use crate::responses::map_response;

pub const OPENROUTER_PROVIDER_ID: &str = "openrouter";

#[derive(Clone)]
pub struct OpenRouterAdapterConfig {
    pub client: Client,
    pub base_url: String,
    pub api_key: String,
    pub model_ref: ModelRef,
}

#[derive(Clone)]
pub struct OpenRouterAdapter {
    config: Arc<OpenRouterAdapterConfig>,
}

impl OpenRouterAdapter {
    pub fn new(config: OpenRouterAdapterConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

struct OpenRouterSession {
    config: Arc<OpenRouterAdapterConfig>,
}

#[async_trait]
impl ProviderAdapter for OpenRouterAdapter {
    fn provider_id(&self) -> &'static str {
        OPENROUTER_PROVIDER_ID
    }

    fn provider_capabilities(&self) -> ProviderCapabilitySet {
        ProviderCapabilitySet {
            supports_server_side_state: false,
            supports_client_side_history: true,
            supports_structured_output: false,
            supports_context_cache: false,
            supports_runtime_owned_tools: true,
            supports_provider_builtin_tools: false,
            supports_approval_control: true,
            compaction_backend: CompactionBackendKind::Disabled,
            continuation_mode: ContinuationMode::ClientManaged,
        }
    }

    fn model_capabilities(&self, _model: &ModelRef) -> ModelCapabilitySet {
        ModelCapabilitySet {
            supports_tool_calling: true,
            supports_parallel_tools: false,
            supports_tool_arg_streaming: false,
            supports_provider_native_tool_calls: false,
            supports_runtime_tool_reinjection: true,
            supports_reasoning_deltas: false,
            supports_opaque_reasoning_carryover: false,
            supports_images: false,
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError> {
        Ok(vec![ModelMeta {
            model_ref: self.config.model_ref.clone(),
            display_name: self.config.model_ref.model_id.clone(),
            provider_label: format!("OpenRouter beta (validated: {OPENROUTER_VALIDATED_MODEL_ID})"),
            context_window: None,
            default_output_mode: OutputMode::Text,
        }])
    }

    async fn compact_history(
        &self,
        _request: InternalCompactionRequest,
    ) -> Result<InternalCompactionResult, ProviderError> {
        Err(ProviderError::new(
            OPENROUTER_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "OpenRouter beta adapter does not implement native compaction",
        ))
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        Ok(Box::new(OpenRouterSession {
            config: Arc::clone(&self.config),
        }))
    }
}

#[async_trait]
impl ProviderSession for OpenRouterSession {
    async fn stream_turn(
        &mut self,
        request: InternalTurnRequest,
    ) -> Result<ProviderEventStream, ProviderError> {
        let payload = map_turn_request(&request)?;
        let response = self
            .config
            .client
            .post(format!(
                "{}/responses",
                self.config.base_url.trim_end_matches('/')
            ))
            .bearer_auth(&self.config.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|error| transport_error(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .map_err(|error| transport_error(error.to_string()))?;
            return match serde_json::from_str::<OpenRouterErrorBody>(&body) {
                Ok(error) => Err(map_http_error_response(status.as_u16(), error)),
                Err(parse_error) => Err(map_http_error_fallback(
                    status.as_u16(),
                    &body,
                    &parse_error.to_string(),
                )),
            };
        }

        let envelope = response
            .json::<OpenRouterResponsesEnvelope>()
            .await
            .map_err(|error| transport_error(error.to_string()))?;
        let events = map_response(envelope)?;
        Ok(Box::pin(stream::iter(events.into_iter().map(Ok))))
    }
}

fn transport_error(message: String) -> ProviderError {
    let mut error = ProviderError::new(
        OPENROUTER_PROVIDER_ID,
        ProviderErrorKind::Transport,
        message,
    );
    error.retryable = true;
    error
}

#[cfg(test)]
mod tests {
    use logos_provider_api::ProviderAdapter;
    use logos_runtime_types::ContinuationMode;

    #[test]
    fn reports_client_managed_validation_capabilities() {
        let provider = super::OpenRouterAdapter::new(super::OpenRouterAdapterConfig {
            client: reqwest::Client::new(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            api_key: "test".to_string(),
            model_ref: logos_runtime_types::ModelRef {
                provider_id: "openrouter".to_string(),
                model_id: "openai/o4-mini".to_string(),
            },
        });

        assert_eq!(
            provider.provider_capabilities().continuation_mode,
            ContinuationMode::ClientManaged
        );
        assert!(!provider.provider_capabilities().supports_structured_output);
        assert!(
            provider
                .provider_capabilities()
                .supports_runtime_owned_tools
        );
        assert!(provider.provider_capabilities().supports_approval_control);
        assert!(
            provider
                .model_capabilities(&provider.config.model_ref)
                .supports_tool_calling
        );
        assert!(
            provider
                .model_capabilities(&provider.config.model_ref)
                .supports_runtime_tool_reinjection
        );
    }
}
