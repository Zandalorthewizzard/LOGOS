use std::sync::Arc;

use async_stream::try_stream;
use async_trait::async_trait;
use codex_core::ModelClient;
use codex_otel::SessionTelemetry;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ReasoningEffort;
use futures::StreamExt;
use futures::pin_mut;
use logos_provider_api::ProviderAdapter;
use logos_provider_api::ProviderEventStream;
use logos_provider_api::ProviderSession;
use logos_runtime_types::ContinuationMode;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelMeta;
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;

use crate::extensions::OPENAI_PROVIDER_ID;
use crate::map_events::map_response_event;
use crate::map_request::map_turn_request;

#[derive(Clone)]
pub struct OpenAiAdapterConfig {
    pub client: ModelClient,
    pub model_info: ModelInfo,
    pub model_ref: ModelRef,
    pub session_telemetry: SessionTelemetry,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub reasoning_summary: ReasoningSummary,
    pub service_tier: Option<ServiceTier>,
}

#[derive(Clone)]
pub struct OpenAiAdapter {
    config: Arc<OpenAiAdapterConfig>,
}

impl OpenAiAdapter {
    pub fn new(config: OpenAiAdapterConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

struct OpenAiSession {
    config: Arc<OpenAiAdapterConfig>,
}

#[async_trait]
impl ProviderAdapter for OpenAiAdapter {
    fn provider_id(&self) -> &'static str {
        OPENAI_PROVIDER_ID
    }

    fn provider_capabilities(&self) -> ProviderCapabilitySet {
        ProviderCapabilitySet {
            supports_server_side_state: true,
            supports_client_side_history: true,
            supports_structured_output: true,
            supports_context_cache: false,
            supports_runtime_owned_tools: true,
            supports_provider_builtin_tools: true,
            supports_approval_control: true,
            supports_compaction: true,
            continuation_mode: ContinuationMode::ServerManaged,
        }
    }

    fn model_capabilities(&self, _model: &ModelRef) -> ModelCapabilitySet {
        ModelCapabilitySet {
            supports_tool_calling: true,
            supports_parallel_tools: true,
            supports_tool_arg_streaming: true,
            supports_provider_native_tool_calls: false,
            supports_runtime_tool_reinjection: true,
            supports_reasoning_deltas: true,
            supports_opaque_reasoning_carryover: true,
            supports_images: true,
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelMeta>, ProviderError> {
        Ok(vec![ModelMeta {
            model_ref: self.config.model_ref.clone(),
            display_name: self.config.model_info.display_name.clone(),
            provider_label: "OpenAI".to_string(),
            context_window: self
                .config
                .model_info
                .context_window
                .and_then(|window| u64::try_from(window).ok()),
            default_output_mode: OutputMode::Text,
        }])
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        Ok(Box::new(OpenAiSession {
            config: Arc::clone(&self.config),
        }))
    }
}

#[async_trait]
impl ProviderSession for OpenAiSession {
    async fn stream_turn(
        &mut self,
        request: InternalTurnRequest,
    ) -> Result<ProviderEventStream, ProviderError> {
        let prompt = map_turn_request(&request)?;
        let mut session = self.config.client.new_session();
        let stream = session
            .stream(
                &prompt,
                &self.config.model_info,
                &self.config.session_telemetry,
                self.config.reasoning_effort,
                self.config.reasoning_summary,
                self.config.service_tier,
                /*turn_metadata_header*/ None,
            )
            .await
            .map_err(map_core_error)?;

        let mapped = try_stream! {
            pin_mut!(stream);
            while let Some(event) = stream.next().await {
                let event = event.map_err(map_core_error)?;
                for internal_event in map_response_event(event) {
                    yield internal_event;
                }
            }
        };

        Ok(Box::pin(mapped))
    }
}

fn map_core_error(error: codex_protocol::error::CodexErr) -> ProviderError {
    ProviderError::new(
        OPENAI_PROVIDER_ID,
        ProviderErrorKind::Transport,
        error.to_string(),
    )
}
