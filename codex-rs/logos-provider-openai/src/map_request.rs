use codex_core::Prompt;
use codex_tools::ConfiguredToolSpec;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use codex_tools::parse_tool_input_schema;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;
use logos_runtime_types::ToolExecutionKind;

use crate::extensions::OPENAI_PROVIDER_ID;
use crate::map_messages::map_messages;

pub fn map_turn_request(request: &InternalTurnRequest) -> Result<Prompt, ProviderError> {
    let mut prompt = Prompt::default();
    prompt.input = map_messages(&request.messages);
    if let Some(system) = &request.system {
        prompt.base_instructions.text = system.clone();
    }
    prompt.previous_response_id = map_continuation(request)?;
    prompt.set_configured_tools(
        request
            .tools
            .iter()
            .map(map_tool_def)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(prompt)
}

fn map_continuation(request: &InternalTurnRequest) -> Result<Option<String>, ProviderError> {
    match request.continuation.as_ref() {
        None => Ok(None),
        Some(ContinuationHandle::OpenAiPreviousResponseId(id)) => Ok(Some(id.clone())),
        Some(other) => Err(ProviderError::new(
            OPENAI_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            format!("unsupported continuation handle for OpenAI boundary: {other:?}"),
        )),
    }
}

pub(crate) fn map_tool_def(tool: &InternalToolDef) -> Result<ConfiguredToolSpec, ProviderError> {
    if tool.execution == ToolExecutionKind::ProviderNative {
        return Err(ProviderError::new(
            OPENAI_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            format!(
                "provider-native tool '{}' is not supported by the OpenAI boundary yet",
                tool.name
            ),
        ));
    }

    Ok(ConfiguredToolSpec::new(
        ToolSpec::Function(ResponsesApiTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            strict: false,
            defer_loading: None,
            parameters: parse_tool_input_schema(&tool.input_schema).map_err(|err| {
                ProviderError::new(
                    OPENAI_PROVIDER_ID,
                    ProviderErrorKind::InvalidRequest,
                    format!("tool '{}' has invalid input schema: {err}", tool.name),
                )
            })?,
            output_schema: tool.output_schema.clone(),
        }),
        /*supports_parallel_tool_calls*/ true,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use logos_runtime_types::ContinuationHandle;
    use logos_runtime_types::InternalMessage;
    use logos_runtime_types::InternalTurnRequest;
    use logos_runtime_types::ModelRef;
    use logos_runtime_types::OutputMode;
    use logos_runtime_types::ProviderErrorKind;
    use logos_runtime_types::ReasoningPolicy;
    use logos_runtime_types::TurnLimits;

    use super::map_turn_request;

    #[test]
    fn explicit_openai_continuation_maps_to_prompt() {
        let request = InternalTurnRequest {
            model_ref: ModelRef {
                provider_id: "openai".to_string(),
                model_id: "gpt-5".to_string(),
            },
            system: None,
            messages: vec![InternalMessage::user_text("hello")],
            tools: Vec::new(),
            output_mode: OutputMode::Text,
            reasoning_policy: ReasoningPolicy::Default,
            continuation: Some(ContinuationHandle::OpenAiPreviousResponseId(
                "resp-1".to_string(),
            )),
            limits: TurnLimits::default(),
            metadata: BTreeMap::new(),
        };

        let prompt = map_turn_request(&request).expect("map should succeed");

        assert_eq!(prompt.previous_response_id.as_deref(), Some("resp-1"));
    }

    #[test]
    fn non_openai_continuation_is_rejected() {
        let request = InternalTurnRequest {
            model_ref: ModelRef {
                provider_id: "openai".to_string(),
                model_id: "gpt-5".to_string(),
            },
            system: None,
            messages: vec![InternalMessage::user_text("hello")],
            tools: Vec::new(),
            output_mode: OutputMode::Text,
            reasoning_policy: ReasoningPolicy::Default,
            continuation: Some(ContinuationHandle::ProviderSessionId(
                "opaque-session".to_string(),
            )),
            limits: TurnLimits::default(),
            metadata: BTreeMap::new(),
        };

        let err = map_turn_request(&request).expect_err("map should fail");

        assert_eq!(err.kind, ProviderErrorKind::UnsupportedCapability);
        assert!(err.message.contains("unsupported continuation handle"));
    }
}
