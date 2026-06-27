use crate::models::{
    ApiItem, ArchitectureNote, CurrentState, DataModelItem, Decision, DecisionStatus,
    FileReference, MemoryInput, PromptItem, TodoItem,
};
use anyhow::{bail, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Summary,
    CurrentState,
    Decisions,
    Architecture,
    Api,
    DataModel,
    Files,
    Todos,
    OpenQuestions,
    FutureWork,
    Prompts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentStatePart {
    Implemented,
    Pending,
}

#[derive(Debug, Default)]
struct DecisionBuilder {
    title: String,
    reason: Option<String>,
    tradeoffs: Vec<String>,
    status: Option<DecisionStatus>,
}

impl DecisionBuilder {
    fn finish(self) -> Decision {
        Decision {
            title: self.title,
            reason: self.reason,
            tradeoffs: self.tradeoffs,
            status: self.status.unwrap_or(DecisionStatus::Proposed),
        }
    }
}

pub fn parse_markdown(input: &str) -> anyhow::Result<MemoryInput> {
    let mut memory = MemoryInput::default();
    let mut section = Section::None;
    let mut current_state_part = CurrentStatePart::Implemented;
    let mut current_decision: Option<DecisionBuilder> = None;
    let mut summary_lines = Vec::new();
    let mut architecture_title: Option<String> = None;
    let mut architecture_lines = Vec::new();
    let mut prompts_title: Option<String> = None;
    let mut prompts_lines = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("<!--") {
            continue;
        }

        if let Some(title) = line.strip_prefix("# ") {
            memory.title = title.trim().to_string();
            continue;
        }

        if let Some(title) = line.strip_prefix("## ") {
            finish_decision(&mut memory, &mut current_decision);
            finish_architecture(
                &mut memory,
                &mut architecture_title,
                &mut architecture_lines,
            );
            finish_prompt(&mut memory, &mut prompts_title, &mut prompts_lines);
            section = match normalize_heading(title).as_str() {
                "summary" => Section::Summary,
                "current state" => Section::CurrentState,
                "decisions" => Section::Decisions,
                "architecture" => Section::Architecture,
                "api" => Section::Api,
                "data model" | "data-model" | "data models" => Section::DataModel,
                "files" => Section::Files,
                "todos" | "todo" | "tasks" => Section::Todos,
                "open questions" => Section::OpenQuestions,
                "future work" => Section::FutureWork,
                "prompts" => Section::Prompts,
                _ => Section::None,
            };
            current_state_part = CurrentStatePart::Implemented;
            continue;
        }

        if let Some(title) = line.strip_prefix("#### ") {
            let _ = title;
            continue;
        }

        if let Some(title) = line.strip_prefix("### ") {
            match section {
                Section::CurrentState => {
                    current_state_part = match normalize_heading(title).as_str() {
                        "pending" => CurrentStatePart::Pending,
                        _ => CurrentStatePart::Implemented,
                    };
                }
                Section::Decisions => {
                    finish_decision(&mut memory, &mut current_decision);
                    current_decision = Some(DecisionBuilder {
                        title: title.trim().to_string(),
                        ..DecisionBuilder::default()
                    });
                }
                Section::Architecture => {
                    finish_architecture(
                        &mut memory,
                        &mut architecture_title,
                        &mut architecture_lines,
                    );
                    architecture_title = Some(title.trim().to_string());
                }
                Section::Prompts => {
                    finish_prompt(&mut memory, &mut prompts_title, &mut prompts_lines);
                    prompts_title = Some(title.trim().to_string());
                }
                Section::Todos | Section::OpenQuestions | Section::FutureWork => {
                    section = match normalize_heading(title).as_str() {
                        "open questions" => Section::OpenQuestions,
                        "future work" => Section::FutureWork,
                        _ => Section::Todos,
                    };
                }
                _ => {}
            }
            continue;
        }

        match section {
            Section::Summary => summary_lines.push(line.to_string()),
            Section::CurrentState => {
                let item = strip_bullet(line).unwrap_or(line).trim();
                if item.is_empty() {
                    continue;
                }
                let state = memory
                    .current_state
                    .get_or_insert_with(CurrentState::default);
                match current_state_part {
                    CurrentStatePart::Implemented => state.implemented.push(item.to_string()),
                    CurrentStatePart::Pending => state.pending.push(item.to_string()),
                }
            }
            Section::Decisions => parse_decision_line(line, &mut current_decision)?,
            Section::Architecture => {
                if architecture_title.is_none() {
                    architecture_title = Some("Notes".to_string());
                }
                architecture_lines.push(line.to_string());
            }
            Section::Api => {
                if let Some(item) = strip_bullet(line) {
                    let (endpoint, description) = split_label(item);
                    let mut parts = endpoint.splitn(2, char::is_whitespace);
                    let method = parts.next().unwrap_or_default().trim().to_string();
                    let path = parts.next().unwrap_or_default().trim().to_string();
                    if !method.is_empty() && !path.is_empty() {
                        memory.api.push(ApiItem {
                            method,
                            path,
                            description,
                        });
                    }
                }
            }
            Section::DataModel => {
                if let Some(item) = strip_bullet(line) {
                    let (name, description) = split_label(item);
                    memory.data_model.push(DataModelItem { name, description });
                }
            }
            Section::Files => {
                if let Some(item) = strip_bullet(line) {
                    let (path, reason) = split_label(item);
                    memory.files.push(FileReference { path, reason });
                }
            }
            Section::Todos => {
                if let Some(item) = strip_bullet(line) {
                    let (priority, text) = parse_todo_item(item);
                    memory.todos.push(TodoItem { text, priority });
                }
            }
            Section::OpenQuestions => {
                if let Some(item) = strip_bullet(line) {
                    memory.open_questions.push(item.to_string());
                }
            }
            Section::FutureWork => {
                if let Some(item) = strip_bullet(line) {
                    memory.future_work.push(item.to_string());
                }
            }
            Section::Prompts => {
                if prompts_title.is_none() {
                    prompts_title = Some("Useful Prompt".to_string());
                }
                prompts_lines.push(line.to_string());
            }
            Section::None => {}
        }
    }

    finish_decision(&mut memory, &mut current_decision);
    finish_architecture(
        &mut memory,
        &mut architecture_title,
        &mut architecture_lines,
    );
    finish_prompt(&mut memory, &mut prompts_title, &mut prompts_lines);

    memory.summary = summary_lines.join("\n").trim().to_string();
    if memory.title.trim().is_empty() {
        bail!("handoff Markdown must start with a `# Title` heading");
    }
    if memory.summary.trim().is_empty() {
        bail!("handoff Markdown must include a non-empty `## Summary` section");
    }

    Ok(memory)
}

fn parse_decision_line(
    line: &str,
    current_decision: &mut Option<DecisionBuilder>,
) -> anyhow::Result<()> {
    let decision = current_decision
        .as_mut()
        .context("decision details must appear under a `### Decision title` heading")?;
    if let Some((label, value)) = line.split_once(':') {
        match label.trim().to_lowercase().as_str() {
            "reason" => {
                decision.reason = Some(value.trim().to_string());
                return Ok(());
            }
            "status" => {
                decision.status = Some(parse_status(value.trim())?);
                return Ok(());
            }
            _ => {}
        }
    }

    if let Some(tradeoff) = strip_bullet(line) {
        decision.tradeoffs.push(tradeoff.to_string());
    } else if let Some(reason) = &mut decision.reason {
        reason.push('\n');
        reason.push_str(line);
    } else {
        decision.reason = Some(line.to_string());
    }
    Ok(())
}

fn parse_status(status: &str) -> anyhow::Result<DecisionStatus> {
    let status = status.trim().trim_matches('`');
    match status.to_lowercase().replace('-', "_").as_str() {
        "proposed" => Ok(DecisionStatus::Proposed),
        "accepted" => Ok(DecisionStatus::Accepted),
        "deprecated" => Ok(DecisionStatus::Deprecated),
        other => bail!("unsupported decision status `{}`", other),
    }
}

fn finish_decision(memory: &mut MemoryInput, decision: &mut Option<DecisionBuilder>) {
    if let Some(decision) = decision.take() {
        memory.decisions.push(decision.finish());
    }
}

fn finish_architecture(
    memory: &mut MemoryInput,
    title: &mut Option<String>,
    lines: &mut Vec<String>,
) {
    if let Some(title) = title.take() {
        let details = lines.join("\n").trim().to_string();
        if !details.is_empty() {
            memory
                .architecture
                .push(ArchitectureNote { title, details });
        }
    }
    lines.clear();
}

fn finish_prompt(memory: &mut MemoryInput, title: &mut Option<String>, lines: &mut Vec<String>) {
    if let Some(title) = title.take() {
        let prompt = lines.join("\n").trim().to_string();
        if !prompt.is_empty() {
            memory.prompts.push(PromptItem { title, prompt });
        }
    }
    lines.clear();
}

fn normalize_heading(value: &str) -> String {
    value.trim().to_lowercase()
}

fn strip_bullet(line: &str) -> Option<&str> {
    line.strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .map(str::trim)
}

fn split_label(item: &str) -> (String, String) {
    if let Some((left, right)) = item.split_once(':') {
        (
            left.trim().trim_matches('`').to_string(),
            right.trim().to_string(),
        )
    } else {
        (item.trim().trim_matches('`').to_string(), String::new())
    }
}

fn parse_todo_item(item: &str) -> (String, String) {
    if let Some(rest) = item.strip_prefix('[') {
        if let Some((priority, text)) = rest.split_once(']') {
            return (
                priority.trim().to_string(),
                text.trim_start().trim().to_string(),
            );
        }
    }
    ("normal".to_string(), item.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApiItem, DataModelItem};
    use crate::render;

    #[test]
    fn parses_structured_handoff_markdown() {
        let memory = parse_markdown(
            r#"# Authentication

## Summary
Authentication supports JWT access tokens.

## Current State
### Implemented
- Email login
### Pending
- MFA

## Decisions
### Use refresh token rotation
Reason: limits stolen-token lifetime.
Status: accepted
- More moving parts

## TODOs
- Add MFA enrollment.

## Open Questions
- Should refresh tokens be device-scoped?
"#,
        )
        .unwrap();

        assert_eq!(memory.title, "Authentication");
        assert!(memory.summary.contains("JWT access tokens"));
        let state = memory.current_state.unwrap();
        assert_eq!(state.implemented, vec!["Email login"]);
        assert_eq!(state.pending, vec!["MFA"]);
        assert_eq!(memory.decisions[0].title, "Use refresh token rotation");
        assert!(matches!(
            memory.decisions[0].status,
            DecisionStatus::Accepted
        ));
        assert_eq!(memory.todos[0].text, "Add MFA enrollment.");
        assert_eq!(
            memory.open_questions[0],
            "Should refresh tokens be device-scoped?"
        );
    }

    #[test]
    fn parses_canonical_generated_markdown_back_into_memory() {
        let original = MemoryInput {
            title: "Authentication".to_string(),
            summary: "Authentication supports JWT access tokens.".to_string(),
            current_state: Some(CurrentState {
                implemented: vec!["Email login".to_string()],
                pending: vec!["MFA".to_string()],
            }),
            architecture: vec![ArchitectureNote {
                title: "Token validation".to_string(),
                details: "Protected routes validate access tokens.".to_string(),
            }],
            decisions: vec![Decision {
                title: "Use JWT access tokens".to_string(),
                reason: Some("Keeps API requests stateless.".to_string()),
                tradeoffs: vec!["Harder immediate revocation".to_string()],
                status: DecisionStatus::Accepted,
            }],
            api: vec![ApiItem {
                method: "POST".to_string(),
                path: "/auth/login".to_string(),
                description: "Authenticates a user.".to_string(),
            }],
            data_model: vec![DataModelItem {
                name: "refresh_tokens".to_string(),
                description: "Stores hashed refresh tokens.".to_string(),
            }],
            todos: vec![TodoItem {
                text: "Add MFA enrollment.".to_string(),
                priority: "high".to_string(),
            }],
            open_questions: vec!["Should refresh tokens be device-scoped?".to_string()],
            future_work: vec!["Support passkeys.".to_string()],
            prompts: vec![PromptItem {
                title: "Auth risk review".to_string(),
                prompt: "Review token logic.".to_string(),
            }],
            files: vec![FileReference {
                path: "src/auth.rs".to_string(),
                reason: "Auth entrypoint.".to_string(),
            }],
        };

        let markdown = render::memory_markdown(&original);
        let parsed = parse_markdown(&markdown).unwrap();

        assert_eq!(parsed.title, original.title);
        assert_eq!(parsed.summary, original.summary);
        assert_eq!(parsed.decisions[0].status, DecisionStatus::Accepted);
        assert_eq!(parsed.api[0].path, "/auth/login");
        assert_eq!(parsed.data_model[0].name, "refresh_tokens");
        assert_eq!(parsed.todos[0].priority, "high");
        assert_eq!(parsed.open_questions, original.open_questions);
        assert_eq!(parsed.future_work, original.future_work);
    }

    #[test]
    fn tolerates_human_edited_markdown_variants() {
        let memory = parse_markdown(
            r#"# Authentication

<!-- promemo:generated:start -->

## Summary
Auth notes.

## Decisions
### Use JWT Tokens
status: `Accepted`
Reason:
They keep requests stateless.

#### Tradeoffs
* Harder revocation

## TODOs
### Tasks
- [high] Add MFA.
### Open Questions
* Should tokens be device-scoped?
<!-- promemo:generated:end -->
"#,
        )
        .unwrap();

        assert_eq!(memory.decisions[0].title, "Use JWT Tokens");
        assert_eq!(memory.decisions[0].status, DecisionStatus::Accepted);
        assert!(memory.decisions[0]
            .reason
            .as_ref()
            .unwrap()
            .contains("stateless"));
        assert_eq!(memory.todos[0].priority, "high");
        assert_eq!(memory.open_questions[0], "Should tokens be device-scoped?");
    }
}
