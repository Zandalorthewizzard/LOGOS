use codex_core::Prompt;
use logos_runtime_types::InternalTurnRequest;
use logos_runtime_types::ProviderError;
use logos_runtime_types::ProviderErrorKind;

use crate::extensions::OPENAI_PROVIDER_ID;
use crate::map_messages::map_messages;

pub fn map_turn_request(request: &InternalTurnRequest) -> Result<Prompt, ProviderError> {
    if !request.tools.is_empty() {
        return Err(ProviderError::new(
            OPENAI_PROVIDER_ID,
            ProviderErrorKind::UnsupportedCapability,
            "phase-1 adapter does not yet map tool definitions into Prompt::tools",
        ));
    }

    let mut prompt = Prompt::default();
    prompt.input = map_messages(&request.messages);
    if let Some(system) = &request.system {
        prompt.base_instructions.text = system.clone();
    }
    Ok(prompt)
}
