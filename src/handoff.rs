use crate::models::{
    ArchitectureNote, CurrentState, Decision, DecisionStatus, FileReference, MemoryInput,
    PromptItem, TodoItem,
};
use anyhow::{bail, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Summary,
    CurrentState,
    Decisions,
    Architecture,
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
        if line.is_empty() {
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
            Section::Files => {
                if let Some(item) = strip_bullet(line) {
                    let (path, reason) = split_label(item);
                    memory.files.push(FileReference { path, reason });
                }
            }
            Section::Todos => {
                if let Some(item) = strip_bullet(line) {
                    memory.todos.push(TodoItem {
                        text: item.to_string(),
                        priority: "normal".to_string(),
                    });
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
    if let Some(reason) = line.strip_prefix("Reason:") {
        decision.reason = Some(reason.trim().to_string());
    } else if let Some(status) = line.strip_prefix("Status:") {
        decision.status = Some(parse_status(status.trim())?);
    } else if let Some(tradeoff) = strip_bullet(line) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
