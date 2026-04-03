use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolExecutionKind {
    RuntimeOwned,
    ProviderNative,
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
