use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Result;
use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalMessage;
use serde::Deserialize;
use serde::Serialize;

const HISTORY_DIR: &str = "thread-history";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HistoryEntry {
    #[serde(rename = "message")]
    Message { seq: u64, data: InternalMessage },
    #[serde(rename = "continuation")]
    Continuation {
        seq: u64,
        data: Option<ContinuationHandle>,
    },
    #[serde(rename = "elicitation")]
    Elicitation { seq: u64, count: u64 },
}

pub struct HistoryFile {
    path: PathBuf,
}

impl HistoryFile {
    pub fn new(codex_home: &Path, thread_id: &str) -> Self {
        let dir = codex_home.join(HISTORY_DIR);
        let path = dir.join(format!("{thread_id}.jsonl"));
        Self { path }
    }

    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn ensure_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("failed to create history directory {}", parent.display())
            })?;
        }
        Ok(())
    }

    pub async fn append(&self, entry: &HistoryEntry) -> Result<()> {
        self.ensure_dir().await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .with_context(|| format!("failed to open history file {}", self.path.display()))?;
        let mut line = serde_json::to_string(entry)?;
        line.push('\n');
        tokio::io::AsyncWriteExt::write_all(&mut file, line.as_bytes())
            .await
            .with_context(|| format!("failed to append to history file {}", self.path.display()))?;
        tokio::io::AsyncWriteExt::flush(&mut file)
            .await
            .with_context(|| format!("failed to flush history file {}", self.path.display()))?;
        Ok(())
    }

    pub async fn write_all(&self, entries: &[HistoryEntry]) -> Result<()> {
        self.ensure_dir().await?;
        let mut lines = Vec::with_capacity(entries.len());
        for entry in entries {
            let mut line = serde_json::to_string(entry)?;
            line.push('\n');
            lines.push(line);
        }
        tokio::fs::write(&self.path, lines.join(""))
            .await
            .with_context(|| format!("failed to write history file {}", self.path.display()))?;
        Ok(())
    }

    pub async fn read_all(&self) -> Result<Vec<HistoryEntry>> {
        let data = match tokio::fs::read_to_string(&self.path).await {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("failed to read history file {}", self.path.display())
                });
            }
        };
        let mut entries = Vec::new();
        for line in data.lines() {
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<HistoryEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(_) => break,
            }
        }
        Ok(entries)
    }

    pub async fn into_history_parts(
        entries: Vec<HistoryEntry>,
    ) -> (Vec<InternalMessage>, Option<ContinuationHandle>, u64) {
        let mut messages = Vec::new();
        let mut continuation = None;
        let mut elicitation_count = 0;
        for entry in entries {
            match entry {
                HistoryEntry::Message { data, .. } => {
                    messages.push(data);
                }
                HistoryEntry::Continuation { data, .. } => {
                    continuation = data;
                }
                HistoryEntry::Elicitation { count, .. } => {
                    elicitation_count = count;
                }
            }
        }
        (messages, continuation, elicitation_count)
    }
}

#[cfg(test)]
mod tests {
    use super::HistoryEntry;
    use super::HistoryFile;
    use anyhow::Result;
    use tempfile::TempDir;

    #[tokio::test]
    async fn history_file_round_trips_entries() -> Result<()> {
        let home = TempDir::new()?;
        let hf = HistoryFile::new(home.path(), "test-thread-id");

        hf.append(&HistoryEntry::Elicitation { seq: 1, count: 3 })
            .await?;
        hf.append(&HistoryEntry::Elicitation { seq: 2, count: 4 })
            .await?;

        let entries = hf.read_all().await?;
        assert_eq!(entries.len(), 2);

        let (msgs, cont, count) = HistoryFile::into_history_parts(entries).await;
        assert!(msgs.is_empty());
        assert!(cont.is_none());
        assert_eq!(count, 4);

        Ok(())
    }

    #[tokio::test]
    async fn history_file_handles_missing_file() -> Result<()> {
        let home = TempDir::new()?;
        let hf = HistoryFile::new(home.path(), "nonexistent-thread");
        let entries = hf.read_all().await?;
        assert!(entries.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn history_file_write_all_overwrites() -> Result<()> {
        let home = TempDir::new()?;
        let hf = HistoryFile::new(home.path(), "test-thread-id");

        hf.write_all(&[HistoryEntry::Elicitation { seq: 1, count: 10 }])
            .await?;

        hf.write_all(&[
            HistoryEntry::Elicitation { seq: 1, count: 1 },
            HistoryEntry::Elicitation { seq: 2, count: 2 },
        ])
        .await?;

        let entries = hf.read_all().await?;
        assert_eq!(entries.len(), 2);
        let (_, _, count) = HistoryFile::into_history_parts(entries).await;
        assert_eq!(count, 2);
        Ok(())
    }
}
