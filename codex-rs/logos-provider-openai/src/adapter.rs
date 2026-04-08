use std::sync::Arc;
use std::sync::Mutex as StdMutex;

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

use crate::extensions::OPENAI_PROVIDER_ID;
use crate::map_events::map_response_event;
use crate::map_history::map_compacted_history;
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
    active_turn_state: Arc<StdMutex<Option<String>>>,
}

#[async_trait]
impl ProviderAdapter for OpenAiAdapter {
    fn provider_id(&self) -> &'static str {
        OPENAI_PROVIDER_ID
    }

    fn provider_capabilities(&self) -> ProviderCapabilitySet {
        openai_provider_capabilities()
    }

    fn model_capabilities(&self, _model: &ModelRef) -> ModelCapabilitySet {
        openai_model_capabilities()
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

    async fn compact_history(
        &self,
        request: InternalCompactionRequest,
    ) -> Result<InternalCompactionResult, ProviderError> {
        let mut prompt = codex_core::Prompt::default();
        prompt.input = crate::map_messages::map_messages(&request.messages);
        prompt.set_configured_tools(
            request
                .tools
                .iter()
                .map(crate::map_request::map_tool_def)
                .collect::<Result<Vec<_>, _>>()?,
        );
        if let Some(system) = request.system {
            prompt.base_instructions.text = system;
        }
        let compacted = self
            .config
            .client
            .compact_conversation_history(
                &prompt,
                &self.config.model_info,
                self.config.reasoning_effort,
                self.config.reasoning_summary,
                &self.config.session_telemetry,
            )
            .await
            .map_err(map_core_error)?;

        Ok(InternalCompactionResult {
            replacement_messages: map_compacted_history(compacted),
        })
    }

    async fn new_session(&self) -> Result<Box<dyn ProviderSession>, ProviderError> {
        Ok(Box::new(OpenAiSession {
            config: Arc::clone(&self.config),
            active_turn_state: Arc::new(StdMutex::new(None)),
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
        session.set_turn_state(lock_turn_state(&self.active_turn_state).clone());
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
        let active_turn_state = Arc::clone(&self.active_turn_state);

        let mapped = try_stream! {
            pin_mut!(stream);
            while let Some(event) = stream.next().await {
                let event = match event {
                    Ok(event) => event,
                    Err(error) => {
                        *lock_turn_state(&active_turn_state) = None;
                        Err(map_core_error(error))?;
                        unreachable!();
                    }
                };
                for internal_event in map_response_event(event) {
                    yield internal_event;
                }
            }
            *lock_turn_state(&active_turn_state) = session.current_turn_state();
        };

        Ok(Box::pin(mapped))
    }

    async fn finish_turn(&mut self) -> Result<(), ProviderError> {
        *lock_turn_state(&self.active_turn_state) = None;
        Ok(())
    }
}

fn map_core_error(error: codex_protocol::error::CodexErr) -> ProviderError {
    ProviderError::new(
        OPENAI_PROVIDER_ID,
        ProviderErrorKind::Transport,
        error.to_string(),
    )
}

fn openai_provider_capabilities() -> ProviderCapabilitySet {
    ProviderCapabilitySet {
        supports_server_side_state: true,
        supports_client_side_history: true,
        supports_structured_output: true,
        supports_context_cache: false,
        supports_runtime_owned_tools: true,
        supports_provider_builtin_tools: true,
        supports_approval_control: true,
        compaction_backend: CompactionBackendKind::NativeRemoteRewrite,
        continuation_mode: ContinuationMode::ServerManaged,
    }
}

fn openai_model_capabilities() -> ModelCapabilitySet {
    ModelCapabilitySet {
        supports_tool_calling: true,
        supports_parallel_tools: true,
        supports_tool_arg_streaming: false,
        supports_provider_native_tool_calls: false,
        supports_runtime_tool_reinjection: true,
        supports_reasoning_deltas: true,
        supports_opaque_reasoning_carryover: true,
        supports_images: true,
    }
}

fn lock_turn_state(
    active_turn_state: &Arc<StdMutex<Option<String>>>,
) -> std::sync::MutexGuard<'_, Option<String>> {
    match active_turn_state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use logos_runtime_types::CompactionBackendKind;

    #[test]
    fn reports_honest_openai_tool_streaming_and_compaction_capabilities() {
        let provider = super::openai_provider_capabilities();
        let model = super::openai_model_capabilities();

        assert_eq!(
            provider.compaction_backend,
            CompactionBackendKind::NativeRemoteRewrite
        );
        assert!(!model.supports_tool_arg_streaming);
    }
}
