use std::collections::BTreeMap;

use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelRef;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ToolCallRecord;

use crate::records::ApprovalJournalState;
use crate::records::JournalTurnFinishReason;
use crate::records::ProviderSnapshotKind;
use crate::records::SessionOpenedRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct JournalSessionView {
    pub session_id: String,
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub provider_capabilities: ProviderCapabilitySet,
    pub model_capabilities: ModelCapabilitySet,
    pub continuation: Option<ContinuationHandle>,
    pub history: Vec<InternalMessage>,
    pub approvals: BTreeMap<String, ApprovalView>,
    pub tool_call_records: BTreeMap<String, ToolCallRecord>,
    pub last_turn_id: Option<String>,
    pub last_turn_finish: Option<(String, JournalTurnFinishReason, Option<String>)>,
    pub snapshots: Vec<(ProviderSnapshotKind, serde_json::Value)>,
}

impl JournalSessionView {
    pub fn from_opened(session_id: String, opened: SessionOpenedRecord) -> Self {
        Self {
            session_id,
            model_ref: opened.model_ref,
            system: opened.system,
            provider_capabilities: opened.provider_capabilities,
            model_capabilities: opened.model_capabilities,
            continuation: None,
            history: Vec::new(),
            approvals: BTreeMap::new(),
            tool_call_records: BTreeMap::new(),
            last_turn_id: None,
            last_turn_finish: None,
            snapshots: Vec::new(),
        }
    }

    pub fn pending_approvals(&self) -> Vec<&ApprovalView> {
        self.approvals
            .values()
            .filter(|approval| approval.state == ApprovalJournalState::Pending)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalView {
    pub request: logos_runtime_types::ToolApprovalRequest,
    pub state: ApprovalJournalState,
}
