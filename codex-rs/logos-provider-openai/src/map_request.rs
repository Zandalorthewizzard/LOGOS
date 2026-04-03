use codex_core::Prompt;
use codex_tools::ConfiguredToolSpec;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use codex_tools::parse_tool_input_schema;
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
    prompt.set_configured_tools(
        request
            .tools
            .iter()
            .map(map_tool_def)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(prompt)
}

fn map_tool_def(tool: &InternalToolDef) -> Result<ConfiguredToolSpec, ProviderError> {
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
