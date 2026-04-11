use std::fs;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use uuid::Uuid;

use crate::error::JournalError;
use crate::records::JournalRecord;
use crate::records::JournalRecordKind;
use crate::records::SessionOpenedRecord;
use crate::view::ApprovalView;
use crate::view::JournalSessionView;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JournalConfig {
    pub store_provider_snapshots: bool,
}

#[derive(Debug, Clone)]
pub struct JournalStore {
    root: PathBuf,
    config: JournalConfig,
}

impl JournalStore {
    pub fn new(root: impl Into<PathBuf>, config: JournalConfig) -> Result<Self, JournalError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|source| JournalError::CreateDir {
            path: root.clone(),
            source,
        })?;
        Ok(Self { root, config })
    }

    pub fn create_session(
        &self,
        opened: SessionOpenedRecord,
    ) -> Result<JournalSession, JournalError> {
        let session_id = Uuid::new_v4().to_string();
        let session = JournalSession {
            path: self.path_for_session(&session_id),
            session_id,
            config: self.config.clone(),
        };
        session.append(JournalRecordKind::SessionOpened(opened))?;
        Ok(session)
    }

    pub fn open_session(&self, session_id: &str) -> JournalSession {
        JournalSession {
            path: self.path_for_session(session_id),
            session_id: session_id.to_string(),
            config: self.config.clone(),
        }
    }

    fn path_for_session(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.jsonl"))
    }
}

#[derive(Debug, Clone)]
pub struct JournalSession {
    path: PathBuf,
    session_id: String,
    config: JournalConfig,
}

impl JournalSession {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn config(&self) -> &JournalConfig {
        &self.config
    }

    pub fn append(&self, kind: JournalRecordKind) -> Result<JournalRecord, JournalError> {
        let sequence_id = self
            .records()?
            .last()
            .map_or(1, |record| record.sequence_id + 1);
        let record = JournalRecord::new(self.session_id.clone(), sequence_id, kind);
        let serialized = serde_json::to_string(&record).map_err(|source| JournalError::Parse {
            path: self.path.clone(),
            source,
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| JournalError::Append {
                path: self.path.clone(),
                source,
            })?;
        writeln!(file, "{serialized}").map_err(|source| JournalError::Append {
            path: self.path.clone(),
            source,
        })?;
        file.flush().map_err(|source| JournalError::Append {
            path: self.path.clone(),
            source,
        })?;
        Ok(record)
    }

    pub fn view(&self) -> Result<JournalSessionView, JournalError> {
        let records = self.records()?;
        let opened = records.iter().find_map(|record| match &record.kind {
            JournalRecordKind::SessionOpened(opened) => Some(opened.clone()),
            _ => None,
        });
        let Some(opened) = opened else {
            return Err(JournalError::MissingSessionOpened {
                session_id: self.session_id.clone(),
            });
        };

        let mut view = JournalSessionView::from_opened(self.session_id.clone(), opened);
        let mut message_sequences: Vec<(u64, logos_runtime_types::InternalMessage)> = Vec::new();
        for record in records {
            match record.kind {
                JournalRecordKind::SessionOpened(_) => {}
                JournalRecordKind::TurnStarted { turn_id, .. } => {
                    view.last_turn_id = Some(turn_id);
                }
                JournalRecordKind::MessageAppended { message, .. } => {
                    message_sequences.push((record.sequence_id, message));
                }
                JournalRecordKind::ContinuationUpdated { handle, .. } => {
                    view.continuation = handle;
                }
                JournalRecordKind::ApprovalStateChanged { approval, .. } => {
                    view.approvals.insert(
                        approval.request.call_id.clone(),
                        ApprovalView {
                            request: approval.request,
                            state: approval.state,
                        },
                    );
                }
                JournalRecordKind::ToolCallTerminalRecorded { record, .. } => {
                    view.tool_call_records
                        .insert(record.call_id.clone(), record);
                }
                JournalRecordKind::TurnFinished {
                    turn_id,
                    reason,
                    error_message,
                } => {
                    view.last_turn_finish = Some((turn_id, reason, error_message));
                }
                JournalRecordKind::CompactionApplied {
                    first_retained_sequence_id,
                    ..
                } => {
                    message_sequences
                        .retain(|(sequence_id, _)| *sequence_id >= first_retained_sequence_id);
                }
                JournalRecordKind::ProviderSnapshotStored { kind, payload, .. } => {
                    if self.config.store_provider_snapshots {
                        view.snapshots.push((kind, payload));
                    }
                }
            }
        }
        view.history = message_sequences
            .into_iter()
            .map(|(_, message)| message)
            .collect();
        Ok(view)
    }

    pub fn records(&self) -> Result<Vec<JournalRecord>, JournalError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        read_jsonl(&self.path)
    }
}

fn read_jsonl(path: &Path) -> Result<Vec<JournalRecord>, JournalError> {
    let file = fs::File::open(path).map_err(|source| JournalError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|source| JournalError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let record = serde_json::from_str(&line).map_err(|source| JournalError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use logos_runtime_types::ContinuationMode;
    use logos_runtime_types::InternalMessage;
    use logos_runtime_types::ModelCapabilitySet;
    use logos_runtime_types::ModelRef;
    use logos_runtime_types::ProviderCapabilitySet;
    use logos_runtime_types::ToolApprovalRequest;
    use logos_runtime_types::ToolExecutionKind;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use crate::records::ApprovalViewRecord;
    use crate::records::JournalRecordKind;
    use crate::records::JournalTurnFinishReason;
    use crate::records::SessionOpenedRecord;

    use super::JournalConfig;
    use super::JournalStore;

    #[test]
    fn journal_records_roundtrip_and_apply_compaction_boundary() {
        let dir = TempDir::new().expect("tempdir should exist");
        let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("store");
        let session = store.create_session(opened()).expect("session");

        let turn_id = "turn-1".to_string();
        session
            .append(JournalRecordKind::TurnStarted {
                turn_id: turn_id.clone(),
                output_mode: logos_runtime_types::OutputMode::Text,
                reasoning_policy: logos_runtime_types::ReasoningPolicy::Default,
                limits: Default::default(),
            })
            .expect("turn started");
        let first = session
            .append(JournalRecordKind::MessageAppended {
                turn_id: Some(turn_id.clone()),
                message: InternalMessage::user_text("before"),
            })
            .expect("first message");
        let checkpoint = session
            .append(JournalRecordKind::MessageAppended {
                turn_id: Some(turn_id.clone()),
                message: InternalMessage::CompactionCheckpoint {
                    summary: "summary".to_string(),
                },
            })
            .expect("checkpoint");
        session
            .append(JournalRecordKind::CompactionApplied {
                turn_id: Some(turn_id),
                backend: logos_runtime_types::CompactionBackendKind::PortableCheckpoint,
                first_retained_sequence_id: checkpoint.sequence_id,
                checkpoint_sequence_id: checkpoint.sequence_id,
            })
            .expect("compaction applied");
        session
            .append(JournalRecordKind::TurnFinished {
                turn_id: "turn-1".to_string(),
                reason: JournalTurnFinishReason::Completed,
                error_message: None,
            })
            .expect("turn finished");

        let view = session.view().expect("view should load");
        assert_eq!(
            view.history,
            vec![InternalMessage::CompactionCheckpoint {
                summary: "summary".to_string(),
            }]
        );
        assert_eq!(first.sequence_id, 3);
    }

    #[test]
    fn pending_approval_survives_view_reconstruction() {
        let dir = TempDir::new().expect("tempdir should exist");
        let store = JournalStore::new(dir.path(), JournalConfig::default()).expect("store");
        let session = store.create_session(opened()).expect("session");

        session
            .append(JournalRecordKind::ApprovalStateChanged {
                turn_id: Some("turn-1".to_string()),
                approval: ApprovalViewRecord {
                    request: ToolApprovalRequest {
                        call_id: "call-1".to_string(),
                        tool_name: "logos_echo".to_string(),
                        execution: ToolExecutionKind::RuntimeOwned,
                        mcp_correlation: None,
                    },
                    state: crate::ApprovalJournalState::Pending,
                },
            })
            .expect("pending approval");

        let view = session.view().expect("view should load");
        assert_eq!(view.approvals.len(), 1);
        assert_eq!(
            view.approvals["call-1"].state,
            crate::ApprovalJournalState::Pending
        );
    }

    fn opened() -> SessionOpenedRecord {
        SessionOpenedRecord {
            model_ref: ModelRef {
                provider_id: "fake".to_string(),
                model_id: "model".to_string(),
            },
            system: Some("system".to_string()),
            provider_capabilities: ProviderCapabilitySet {
                supports_server_side_state: true,
                supports_client_side_history: true,
                supports_structured_output: false,
                supports_context_cache: false,
                supports_runtime_owned_tools: true,
                supports_provider_builtin_tools: false,
                supports_approval_control: true,
                compaction_backend: logos_runtime_types::CompactionBackendKind::NativeRemoteRewrite,
                continuation_mode: ContinuationMode::ServerManaged,
            },
            model_capabilities: ModelCapabilitySet {
                supports_tool_calling: true,
                supports_parallel_tools: false,
                supports_tool_arg_streaming: false,
                supports_provider_native_tool_calls: false,
                supports_runtime_tool_reinjection: true,
                supports_reasoning_deltas: false,
                supports_opaque_reasoning_carryover: false,
                supports_images: false,
            },
            continuation_mode: ContinuationMode::ServerManaged,
        }
    }
}
