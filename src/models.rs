use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInput {
    pub title: String,
    pub summary: String,
    pub current_state: Option<CurrentState>,
    #[serde(default)]
    pub architecture: Vec<ArchitectureNote>,
    #[serde(default)]
    pub decisions: Vec<Decision>,
    #[serde(default)]
    pub api: Vec<ApiItem>,
    #[serde(default)]
    pub data_model: Vec<DataModelItem>,
    #[serde(default)]
    pub files: Vec<FileReference>,
    #[serde(default)]
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub open_questions: Vec<String>,
    #[serde(default)]
    pub future_work: Vec<String>,
    #[serde(default)]
    pub prompts: Vec<PromptItem>,
}

impl MemoryInput {
    pub fn from_reader(reader: impl Read) -> anyhow::Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrentState {
    #[serde(default)]
    pub implemented: Vec<String>,
    #[serde(default)]
    pub pending: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureNote {
    pub title: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub title: String,
    pub reason: Option<String>,
    #[serde(default)]
    pub tradeoffs: Vec<String>,
    pub status: DecisionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiItem {
    pub method: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModelItem {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReference {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub text: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptItem {
    pub title: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryIndex {
    #[serde(default)]
    pub features: Vec<FeatureEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEntry {
    pub name: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveReport {
    pub feature: String,
    pub title: String,
    pub files_written: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedContext {
    pub feature: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub feature: String,
    pub dry_run: bool,
    pub branch: Option<String>,
    pub status_short: String,
    pub diff_stat: String,
    pub changed_files: Vec<String>,
    pub recent_commits: Vec<String>,
    pub todos: Vec<TodoComment>,
    pub warnings: Vec<String>,
    pub existing_memory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoComment {
    pub path: String,
    pub line: usize,
    pub text: String,
}
