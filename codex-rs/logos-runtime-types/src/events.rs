use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::continuation::ContinuationHandle;
use crate::errors::ProviderError;
use crate::model::CallId;
use crate::model::ProviderId;
use crate::tools::ToolApprovalOutcome;
use crate::tools::ToolApprovalRequest;
use crate::tools::ToolCallRecord;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InternalEvent {
    MessageStarted {
        message_id: Option<String>,
    },
    TextDelta {
        text: String,
    },
    MessageCompleted {
        message_id: Option<String>,
    },
    ToolCallStarted {
        call_id: CallId,
        name: String,
    },
    ToolCallArgumentsDelta {
        call_id: CallId,
        delta: String,
    },
    ToolCallFinished {
        call_id: CallId,
    },
    ApprovalRequested {
        request: ToolApprovalRequest,
    },
    ApprovalResolved {
        call_id: CallId,
        outcome: ToolApprovalOutcome,
    },
    ToolCallTerminal {
        record: ToolCallRecord,
    },
    ReasoningDelta {
        text: String,
    },
    Usage {
        input_tokens: u64,
        output_tokens: u64,
    },
    ContinuationUpdated {
        handle: ContinuationHandle,
    },
    ProviderExtension {
        key: String,
        payload: Value,
    },
    RawProviderEvent {
        provider_id: ProviderId,
        payload: Value,
    },
    Completed,
    Failed(ProviderError),
}
