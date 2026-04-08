use serde::Deserialize;
use serde::Serialize;

use crate::continuation::ContinuationMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionBackendKind {
    Disabled,
    NativeRemoteRewrite,
    PortableCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilitySet {
    pub supports_server_side_state: bool,
    pub supports_client_side_history: bool,
    pub supports_structured_output: bool,
    pub supports_context_cache: bool,
    pub supports_runtime_owned_tools: bool,
    pub supports_provider_builtin_tools: bool,
    pub supports_approval_control: bool,
    pub compaction_backend: CompactionBackendKind,
    pub continuation_mode: ContinuationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilitySet {
    pub supports_tool_calling: bool,
    pub supports_parallel_tools: bool,
    pub supports_tool_arg_streaming: bool,
    pub supports_provider_native_tool_calls: bool,
    pub supports_runtime_tool_reinjection: bool,
    pub supports_reasoning_deltas: bool,
    pub supports_opaque_reasoning_carryover: bool,
    pub supports_images: bool,
}
