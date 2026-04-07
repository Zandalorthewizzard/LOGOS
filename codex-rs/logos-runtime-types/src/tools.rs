use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::model::CallId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolExecutionKind {
    RuntimeOwned,
    ProviderNative,
    McpBackedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolApprovalMode {
    AutoApprove,
    RequireApproval,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolApprovalOutcome {
    Approved,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolApprovalRequest {
    pub call_id: CallId,
    pub tool_name: String,
    pub execution: ToolExecutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolExecutionPolicy {
    pub runtime_owned_default: ToolApprovalMode,
    pub mcp_runtime_default: ToolApprovalMode,
    pub allow_provider_native: bool,
    pub per_tool_modes: BTreeMap<String, ToolApprovalMode>,
}

impl Default for ToolExecutionPolicy {
    fn default() -> Self {
        Self {
            runtime_owned_default: ToolApprovalMode::AutoApprove,
            mcp_runtime_default: ToolApprovalMode::RequireApproval,
            allow_provider_native: false,
            per_tool_modes: BTreeMap::new(),
        }
    }
}

impl ToolExecutionPolicy {
    pub fn approval_mode_for(
        &self,
        tool_name: &str,
        execution: &ToolExecutionKind,
    ) -> ToolApprovalMode {
        if let Some(mode) = self.per_tool_modes.get(tool_name) {
            return mode.clone();
        }

        match execution {
            ToolExecutionKind::RuntimeOwned => self.runtime_owned_default.clone(),
            ToolExecutionKind::McpBackedRuntime => self.mcp_runtime_default.clone(),
            ToolExecutionKind::ProviderNative => {
                if self.allow_provider_native {
                    ToolApprovalMode::AutoApprove
                } else {
                    ToolApprovalMode::Deny
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub execution: ToolExecutionKind,
    pub metadata: BTreeMap<String, Value>,
}
