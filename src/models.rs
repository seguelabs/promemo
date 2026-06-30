use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Read;
use std::path::Path;

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

    pub fn validate(&self) -> anyhow::Result<()> {
        let issues = self.validation_issues();
        if issues.is_empty() {
            return Ok(());
        }

        let details = issues
            .iter()
            .map(|issue| format!("{}: {}", issue.path, issue.message))
            .collect::<Vec<_>>()
            .join("; ");
        bail!("invalid memory input: {}", details)
    }

    pub fn validation_issues(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        require_non_empty(&mut issues, "title", &self.title);
        require_non_empty(&mut issues, "summary", &self.summary);

        if let Some(state) = &self.current_state {
            for (index, item) in state.implemented.iter().enumerate() {
                require_non_empty(
                    &mut issues,
                    &format!("current_state.implemented[{}]", index),
                    item,
                );
            }
            for (index, item) in state.pending.iter().enumerate() {
                require_non_empty(
                    &mut issues,
                    &format!("current_state.pending[{}]", index),
                    item,
                );
            }
        }

        for (index, item) in self.architecture.iter().enumerate() {
            require_non_empty(
                &mut issues,
                &format!("architecture[{}].title", index),
                &item.title,
            );
            require_non_empty(
                &mut issues,
                &format!("architecture[{}].details", index),
                &item.details,
            );
        }

        for (index, item) in self.decisions.iter().enumerate() {
            require_non_empty(
                &mut issues,
                &format!("decisions[{}].title", index),
                &item.title,
            );
            if let Some(reason) = &item.reason {
                require_non_empty(&mut issues, &format!("decisions[{}].reason", index), reason);
            }
            for (tradeoff_index, tradeoff) in item.tradeoffs.iter().enumerate() {
                require_non_empty(
                    &mut issues,
                    &format!("decisions[{}].tradeoffs[{}]", index, tradeoff_index),
                    tradeoff,
                );
            }
        }

        for (index, item) in self.api.iter().enumerate() {
            require_non_empty(&mut issues, &format!("api[{}].method", index), &item.method);
            require_non_empty(&mut issues, &format!("api[{}].path", index), &item.path);
            require_non_empty(
                &mut issues,
                &format!("api[{}].description", index),
                &item.description,
            );
        }

        for (index, item) in self.data_model.iter().enumerate() {
            require_non_empty(
                &mut issues,
                &format!("data_model[{}].name", index),
                &item.name,
            );
            require_non_empty(
                &mut issues,
                &format!("data_model[{}].description", index),
                &item.description,
            );
        }

        for (index, item) in self.files.iter().enumerate() {
            require_non_empty(&mut issues, &format!("files[{}].path", index), &item.path);
            require_non_empty(
                &mut issues,
                &format!("files[{}].reason", index),
                &item.reason,
            );
            validate_repo_relative_path(&mut issues, &format!("files[{}].path", index), &item.path);
        }

        for (index, item) in self.todos.iter().enumerate() {
            require_non_empty(&mut issues, &format!("todos[{}].text", index), &item.text);
            require_non_empty(
                &mut issues,
                &format!("todos[{}].priority", index),
                &item.priority,
            );
            if !matches!(item.priority.as_str(), "low" | "normal" | "high") {
                issues.push(ValidationIssue {
                    path: format!("todos[{}].priority", index),
                    message: "must be one of low, normal, or high".to_string(),
                });
            }
        }

        for (index, item) in self.open_questions.iter().enumerate() {
            require_non_empty(&mut issues, &format!("open_questions[{}]", index), item);
        }

        for (index, item) in self.future_work.iter().enumerate() {
            require_non_empty(&mut issues, &format!("future_work[{}]", index), item);
        }

        for (index, item) in self.prompts.iter().enumerate() {
            require_non_empty(
                &mut issues,
                &format!("prompts[{}].title", index),
                &item.title,
            );
            require_non_empty(
                &mut issues,
                &format!("prompts[{}].prompt", index),
                &item.prompt,
            );
        }

        issues
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

pub fn memory_input_schema() -> serde_json::Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Promemo MemoryInput",
        "type": "object",
        "additionalProperties": false,
        "required": ["title", "summary"],
        "properties": {
            "title": non_empty_string_schema(),
            "summary": non_empty_string_schema(),
            "current_state": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "implemented": string_array_schema(),
                    "pending": string_array_schema()
                }
            },
            "architecture": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["title", "details"],
                "properties": {
                    "title": non_empty_string_schema(),
                    "details": non_empty_string_schema()
                }
            })),
            "decisions": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["title", "status"],
                "properties": {
                    "title": non_empty_string_schema(),
                    "reason": non_empty_string_schema(),
                    "tradeoffs": string_array_schema(),
                    "status": {
                        "type": "string",
                        "enum": ["proposed", "accepted", "deprecated"]
                    }
                }
            })),
            "api": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["method", "path", "description"],
                "properties": {
                    "method": non_empty_string_schema(),
                    "path": non_empty_string_schema(),
                    "description": non_empty_string_schema()
                }
            })),
            "data_model": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "description"],
                "properties": {
                    "name": non_empty_string_schema(),
                    "description": non_empty_string_schema()
                }
            })),
            "files": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path", "reason"],
                "properties": {
                    "path": non_empty_string_schema(),
                    "reason": non_empty_string_schema()
                }
            })),
            "todos": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["text", "priority"],
                "properties": {
                    "text": non_empty_string_schema(),
                    "priority": {
                        "type": "string",
                        "enum": ["low", "normal", "high"]
                    }
                }
            })),
            "open_questions": string_array_schema(),
            "future_work": string_array_schema(),
            "prompts": array_schema(json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["title", "prompt"],
                "properties": {
                    "title": non_empty_string_schema(),
                    "prompt": non_empty_string_schema()
                }
            }))
        }
    })
}

fn require_non_empty(issues: &mut Vec<ValidationIssue>, path: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(ValidationIssue {
            path: path.to_string(),
            message: "must not be empty".to_string(),
        });
    }
}

fn validate_repo_relative_path(issues: &mut Vec<ValidationIssue>, path: &str, value: &str) {
    let candidate = Path::new(value);
    if candidate.is_absolute() || value.split('/').any(|part| part == "..") {
        issues.push(ValidationIssue {
            path: path.to_string(),
            message: "must be a repository-relative path".to_string(),
        });
    }
}

fn non_empty_string_schema() -> serde_json::Value {
    json!({ "type": "string", "minLength": 1 })
}

fn string_array_schema() -> serde_json::Value {
    array_schema(non_empty_string_schema())
}

fn array_schema(items: serde_json::Value) -> serde_json::Value {
    json!({
        "type": "array",
        "items": items,
        "default": []
    })
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub dry_run: bool,
    pub files_written: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySaveRequest {
    pub feature: String,
    pub memory: MemoryInput,
}

impl MemorySaveRequest {
    pub fn from_reader(reader: impl Read) -> anyhow::Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
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
    pub existing_memory: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoComment {
    pub path: String,
    pub line: usize,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_rejects_empty_required_fields_and_bad_priority() {
        let memory = MemoryInput {
            title: " ".to_string(),
            summary: " ".to_string(),
            todos: vec![TodoItem {
                text: "Ship extraction".to_string(),
                priority: "urgent".to_string(),
            }],
            files: vec![FileReference {
                path: "../secret.txt".to_string(),
                reason: "Should not escape repo".to_string(),
            }],
            ..MemoryInput::default()
        };

        let issues = memory.validation_issues();
        assert!(issues.iter().any(|issue| issue.path == "title"));
        assert!(issues.iter().any(|issue| issue.path == "summary"));
        assert!(issues.iter().any(|issue| issue.path == "todos[0].priority"));
        assert!(issues.iter().any(|issue| issue.path == "files[0].path"));
    }

    #[test]
    fn memory_input_schema_exposes_decision_and_priority_enums() {
        let schema = memory_input_schema();

        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["decisions"]["items"]["properties"]["status"]["enum"][1],
            "accepted"
        );
        assert_eq!(
            schema["properties"]["todos"]["items"]["properties"]["priority"]["enum"][2],
            "high"
        );
    }
}
