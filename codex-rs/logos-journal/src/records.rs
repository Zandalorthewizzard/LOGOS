use chrono::Utc;
use logos_runtime_types::CompactionBackendKind;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::ContinuationMode;
use logos_runtime_types::InternalMessage;
use logos_runtime_types::ModelCapabilitySet;
use logos_runtime_types::ModelRef;
use logos_runtime_types::OutputMode;
use logos_runtime_types::ProviderCapabilitySet;
use logos_runtime_types::ReasoningPolicy;
use logos_runtime_types::ToolApprovalRequest;
use logos_runtime_types::ToolCallRecord;
use logos_runtime_types::TurnLimits;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalRecord {
    pub session_id: String,
    pub sequence_id: u64,
    pub recorded_at: i64,
    pub kind: JournalRecordKind,
}

impl JournalRecord {
    pub fn new(session_id: String, sequence_id: u64, kind: JournalRecordKind) -> Self {
        Self {
            session_id,
            sequence_id,
            recorded_at: Utc::now().timestamp(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JournalRecordKind {
    SessionOpened(SessionOpenedRecord),
    TurnStarted {
        turn_id: String,
        output_mode: OutputMode,
        reasoning_policy: ReasoningPolicy,
        limits: TurnLimits,
    },
    MessageAppended {
        turn_id: Option<String>,
        message: InternalMessage,
    },
    ContinuationUpdated {
        turn_id: Option<String>,
        handle: Option<ContinuationHandle>,
    },
    ApprovalStateChanged {
        turn_id: Option<String>,
        approval: ApprovalViewRecord,
    },
    ToolCallTerminalRecorded {
        turn_id: Option<String>,
        record: ToolCallRecord,
    },
    TurnFinished {
        turn_id: String,
        reason: JournalTurnFinishReason,
        error_message: Option<String>,
    },
    CompactionApplied {
        turn_id: Option<String>,
        backend: CompactionBackendKind,
        first_retained_sequence_id: u64,
        checkpoint_sequence_id: u64,
    },
    ProviderSnapshotStored {
        turn_id: Option<String>,
        kind: ProviderSnapshotKind,
        payload: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionOpenedRecord {
    pub model_ref: ModelRef,
    pub system: Option<String>,
    pub provider_capabilities: ProviderCapabilitySet,
    pub model_capabilities: ModelCapabilitySet,
    pub continuation_mode: ContinuationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalJournalState {
    Pending,
    Approved,
    Denied,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalViewRecord {
    pub request: ToolApprovalRequest,
    pub state: ApprovalJournalState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalTurnFinishReason {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSnapshotKind {
    Request,
    Response,
    Event,
}
