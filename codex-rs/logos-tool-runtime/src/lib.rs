use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::InternalToolDef;
use logos_runtime_types::McpToolCallCorrelation;
use logos_runtime_types::ToolApprovalMode;
use logos_runtime_types::ToolApprovalOutcome;
use logos_runtime_types::ToolApprovalRequest;
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::ToolCallResultDisposition;
use logos_runtime_types::ToolCallTerminalState;
use logos_runtime_types::ToolExecutionKind;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments_json: String,
    pub execution: ToolExecutionKind,
    pub mcp_correlation: Option<McpToolCallCorrelation>,
}

impl PendingToolCall {
    pub fn arguments(&self) -> Value {
        serde_json::from_str(&self.arguments_json)
            .unwrap_or_else(|_| Value::String(self.arguments_json.clone()))
    }

    pub fn into_message(self) -> InternalMessage {
        let arguments = self.arguments();
        InternalMessage::AssistantToolCall {
            call_id: self.call_id,
            name: self.name,
            arguments,
        }
    }
}

#[derive(Debug, Default)]
pub struct ToolCallCollector {
    calls: BTreeMap<String, PendingToolCall>,
}

impl ToolCallCollector {
    pub fn observe(&mut self, event: &InternalEvent) {
        match event {
            InternalEvent::ToolCallStarted { call_id, name } => {
                self.calls.insert(
                    call_id.clone(),
                    PendingToolCall {
                        call_id: call_id.clone(),
                        name: name.clone(),
                        arguments_json: String::new(),
                        execution: ToolExecutionKind::RuntimeOwned,
                        mcp_correlation: None,
                    },
                );
            }
            InternalEvent::ToolCallArgumentsDelta { call_id, delta } => {
                if let Some(call) = self.calls.get_mut(call_id) {
                    call.arguments_json.push_str(delta);
                }
            }
            InternalEvent::ToolCallFinished { .. }
            | InternalEvent::ApprovalRequested { .. }
            | InternalEvent::ApprovalResolved { .. }
            | InternalEvent::ToolCallTerminal { .. }
            | InternalEvent::MessageStarted { .. }
            | InternalEvent::TextDelta { .. }
            | InternalEvent::MessageCompleted { .. }
            | InternalEvent::ReasoningDelta { .. }
            | InternalEvent::Usage { .. }
            | InternalEvent::ContinuationUpdated { .. }
            | InternalEvent::ProviderExtension { .. }
            | InternalEvent::RawProviderEvent { .. }
            | InternalEvent::Completed
            | InternalEvent::Failed(_) => {}
        }
    }

    pub fn into_calls(self) -> Vec<PendingToolCall> {
        self.calls.into_values().collect()
    }

    pub fn into_messages(self) -> Vec<InternalMessage> {
        self.into_calls()
            .into_iter()
            .map(PendingToolCall::into_message)
            .collect()
    }
}

pub trait McpCorrelationIngress: Send + Sync {
    fn begin_call(&self, call_id: &str, tool_name: &str) -> McpToolCallCorrelation;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolExecutionOutput {
    pub output: Value,
    pub mcp_correlation: Option<McpToolCallCorrelation>,
}

impl ToolExecutionOutput {
    pub fn success(output: Value) -> Self {
        Self {
            output,
            mcp_correlation: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolExecutionOutcome {
    pub message: InternalMessage,
    pub record: ToolCallRecord,
}

#[async_trait]
pub trait RuntimeToolExecutor: Send + Sync {
    async fn execute(&self, call: &PendingToolCall) -> Result<ToolExecutionOutput, String>;
}

#[async_trait]
pub trait ToolApprover: Send + Sync {
    async fn decide(
        &self,
        request: &ToolApprovalRequest,
        mode: ToolApprovalMode,
    ) -> ToolApprovalOutcome;
}

pub struct PolicyApprover;

#[async_trait]
impl ToolApprover for PolicyApprover {
    async fn decide(
        &self,
        _request: &ToolApprovalRequest,
        mode: ToolApprovalMode,
    ) -> ToolApprovalOutcome {
        match mode {
            ToolApprovalMode::AutoApprove => ToolApprovalOutcome::Approved,
            ToolApprovalMode::RequireApproval | ToolApprovalMode::Deny => {
                ToolApprovalOutcome::Denied
            }
        }
    }
}

#[derive(Clone)]
struct RegisteredRuntimeTool {
    def: InternalToolDef,
    executor: Arc<dyn RuntimeToolExecutor>,
}

#[derive(Clone, Default)]
pub struct ToolRuntime {
    tools: BTreeMap<String, RegisteredRuntimeTool>,
    mcp_correlation_ingress: Option<Arc<dyn McpCorrelationIngress>>,
}

impl ToolRuntime {
    pub fn register_tool(&mut self, def: InternalToolDef, executor: Arc<dyn RuntimeToolExecutor>) {
        self.tools
            .insert(def.name.clone(), RegisteredRuntimeTool { def, executor });
    }

    pub fn set_mcp_correlation_ingress(&mut self, ingress: Arc<dyn McpCorrelationIngress>) {
        self.mcp_correlation_ingress = Some(ingress);
    }

    pub fn declared_tools(&self) -> Vec<InternalToolDef> {
        self.tools.values().map(|tool| tool.def.clone()).collect()
    }

    pub fn tool_def(&self, name: &str) -> Option<InternalToolDef> {
        self.tools.get(name).map(|tool| tool.def.clone())
    }

    pub fn prepare_call(
        &self,
        mut call: PendingToolCall,
        tool_def: &InternalToolDef,
    ) -> Result<PendingToolCall, String> {
        call.execution = tool_def.execution.clone();
        if call.execution == ToolExecutionKind::McpBackedRuntime && call.mcp_correlation.is_none() {
            let ingress = self
                .mcp_correlation_ingress
                .as_ref()
                .ok_or_else(|| format!("missing MCP correlation ingress for '{}'", call.name))?;
            call.mcp_correlation = Some(ingress.begin_call(&call.call_id, &call.name));
        }
        Ok(call)
    }

    pub async fn execute_call(&self, call: &PendingToolCall) -> Option<ToolExecutionOutcome> {
        let tool = self.tools.get(&call.name)?;
        let outcome = match tool.executor.execute(call).await {
            Ok(output) => ToolExecutionOutcome {
                message: tool_result_message(
                    call.call_id.clone(),
                    output.output,
                    /*is_error*/ false,
                ),
                record: ToolCallRecord {
                    call_id: call.call_id.clone(),
                    tool_name: call.name.clone(),
                    execution: call.execution.clone(),
                    terminal_state: ToolCallTerminalState::CompletedWithResult,
                    mcp_correlation: output
                        .mcp_correlation
                        .or_else(|| call.mcp_correlation.clone()),
                    result_disposition: ToolCallResultDisposition::Success,
                },
            },
            Err(message) => ToolExecutionOutcome {
                message: tool_result_message(
                    call.call_id.clone(),
                    Value::Object(serde_json::Map::from_iter([(
                        "error".to_string(),
                        Value::String(message),
                    )])),
                    /*is_error*/ true,
                ),
                record: ToolCallRecord {
                    call_id: call.call_id.clone(),
                    tool_name: call.name.clone(),
                    execution: call.execution.clone(),
                    terminal_state: ToolCallTerminalState::FailedBeforeResult,
                    mcp_correlation: call.mcp_correlation.clone(),
                    result_disposition: ToolCallResultDisposition::Error,
                },
            },
        };
        Some(outcome)
    }
}

pub fn tool_result_message(
    call_id: impl Into<String>,
    output: Value,
    is_error: bool,
) -> InternalMessage {
    InternalMessage::ToolResult {
        call_id: call_id.into(),
        output,
        is_error,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use logos_runtime_types::InternalToolDef;
    use logos_runtime_types::McpToolCallCorrelation;
    use logos_runtime_types::ToolExecutionKind;
    use serde_json::json;

    use super::McpCorrelationIngress;
    use super::PendingToolCall;
    use super::ToolRuntime;

    struct DeterministicIngress;

    impl McpCorrelationIngress for DeterministicIngress {
        fn begin_call(&self, call_id: &str, _tool_name: &str) -> McpToolCallCorrelation {
            McpToolCallCorrelation {
                request_id: format!("req-{call_id}"),
                submission_id: format!("sub-{call_id}"),
                thread_id: None,
            }
        }
    }

    #[test]
    fn mcp_call_requires_explicit_ingress() {
        let runtime = ToolRuntime::default();

        let error = runtime
            .prepare_call(
                PendingToolCall {
                    call_id: "call-1".to_string(),
                    name: "logos_mcp".to_string(),
                    arguments_json: "{}".to_string(),
                    execution: ToolExecutionKind::RuntimeOwned,
                    mcp_correlation: None,
                },
                &tool_def("logos_mcp", ToolExecutionKind::McpBackedRuntime),
            )
            .expect_err("mcp call should require ingress");

        assert_eq!(error, "missing MCP correlation ingress for 'logos_mcp'");
    }

    #[test]
    fn mcp_ingress_attaches_request_and_submission_ids() {
        let mut runtime = ToolRuntime::default();
        runtime.set_mcp_correlation_ingress(Arc::new(DeterministicIngress));

        let call = runtime
            .prepare_call(
                PendingToolCall {
                    call_id: "call-1".to_string(),
                    name: "logos_mcp".to_string(),
                    arguments_json: json!({"text": "hi"}).to_string(),
                    execution: ToolExecutionKind::RuntimeOwned,
                    mcp_correlation: None,
                },
                &tool_def("logos_mcp", ToolExecutionKind::McpBackedRuntime),
            )
            .expect("mcp ingress should attach ids");

        assert_eq!(
            call.mcp_correlation,
            Some(McpToolCallCorrelation {
                request_id: "req-call-1".to_string(),
                submission_id: "sub-call-1".to_string(),
                thread_id: None,
            })
        );
        assert_eq!(call.execution, ToolExecutionKind::McpBackedRuntime);
    }

    fn tool_def(name: &str, execution: ToolExecutionKind) -> InternalToolDef {
        InternalToolDef {
            name: name.to_string(),
            description: format!("tool {name}"),
            input_schema: json!({"type": "object"}),
            output_schema: None,
            execution,
            metadata: Default::default(),
        }
    }
}
