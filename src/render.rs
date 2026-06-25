use crate::models::{MemoryInput, RepoSnapshot};

pub fn render_feature_files(memory: &MemoryInput) -> Vec<(&'static str, String)> {
    vec![
        ("context.md", context(memory)),
        ("architecture.md", architecture(memory)),
        ("decisions.md", decisions(memory)),
        ("api.md", api(memory)),
        ("data-model.md", data_model(memory)),
        ("todos.md", todos(memory)),
        ("changelog.md", changelog(memory)),
        ("prompts.md", prompts(memory)),
    ]
}

pub fn context(memory: &MemoryInput) -> String {
    let mut out = format!("# {}\n\n## Summary\n\n{}\n", memory.title, memory.summary);
    if let Some(state) = &memory.current_state {
        out.push_str("\n## Current State\n\n");
        if !state.implemented.is_empty() {
            out.push_str("### Implemented\n\n");
            push_list(&mut out, &state.implemented);
        }
        if !state.pending.is_empty() {
            out.push_str("\n### Pending\n\n");
            push_list(&mut out, &state.pending);
        }
    }
    if !memory.files.is_empty() {
        out.push_str("\n## Files\n\n");
        for file in &memory.files {
            out.push_str(&format!("- `{}`: {}\n", file.path, file.reason));
        }
    }
    out
}

fn architecture(memory: &MemoryInput) -> String {
    let mut out = format!("# {} Architecture\n", memory.title);
    for item in &memory.architecture {
        out.push_str(&format!("\n## {}\n\n{}\n", item.title, item.details));
    }
    out
}

fn decisions(memory: &MemoryInput) -> String {
    let mut out = format!("# {} Decisions\n", memory.title);
    for item in &memory.decisions {
        out.push_str(&format!(
            "\n## {}\n\nStatus: `{:?}`\n",
            item.title, item.status
        ));
        if let Some(reason) = &item.reason {
            out.push_str(&format!("\n{}\n", reason));
        }
        if !item.tradeoffs.is_empty() {
            out.push_str("\n### Tradeoffs\n\n");
            push_list(&mut out, &item.tradeoffs);
        }
    }
    out
}

fn api(memory: &MemoryInput) -> String {
    let mut out = format!("# {} API\n", memory.title);
    for item in &memory.api {
        out.push_str(&format!(
            "\n## {} {}\n\n{}\n",
            item.method, item.path, item.description
        ));
    }
    out
}

fn data_model(memory: &MemoryInput) -> String {
    let mut out = format!("# {} Data Model\n", memory.title);
    for item in &memory.data_model {
        out.push_str(&format!("\n## {}\n\n{}\n", item.name, item.description));
    }
    out
}

fn todos(memory: &MemoryInput) -> String {
    let mut out = format!("# {} TODOs\n", memory.title);
    if !memory.todos.is_empty() {
        out.push_str("\n## Tasks\n\n");
        for item in &memory.todos {
            out.push_str(&format!("- [{}] {}\n", item.priority, item.text));
        }
    }
    if !memory.open_questions.is_empty() {
        out.push_str("\n## Open Questions\n\n");
        push_list(&mut out, &memory.open_questions);
    }
    if !memory.future_work.is_empty() {
        out.push_str("\n## Future Work\n\n");
        push_list(&mut out, &memory.future_work);
    }
    out
}

fn changelog(memory: &MemoryInput) -> String {
    format!("# {} Changelog\n\n- Memory saved.\n", memory.title)
}

fn prompts(memory: &MemoryInput) -> String {
    let mut out = format!("# {} Prompts\n", memory.title);
    for item in &memory.prompts {
        out.push_str(&format!("\n## {}\n\n{}\n", item.title, item.prompt));
    }
    out
}

pub fn render_snapshot(snapshot: &RepoSnapshot) -> String {
    let mut out = format!("# Snapshot: {}\n\n", snapshot.feature);
    out.push_str(&format!("Dry run: `{}`\n\n", snapshot.dry_run));
    if let Some(branch) = &snapshot.branch {
        out.push_str(&format!("Branch: `{}`\n\n", branch));
    }
    if !snapshot.status_short.is_empty() {
        out.push_str("## Status\n\n```txt\n");
        out.push_str(&snapshot.status_short);
        out.push_str("\n```\n");
    }
    if !snapshot.diff_stat.is_empty() {
        out.push_str("\n## Diff Stat\n\n```txt\n");
        out.push_str(&snapshot.diff_stat);
        out.push_str("\n```\n");
    }
    out
}

fn push_list(out: &mut String, items: &[String]) {
    for item in items {
        out.push_str(&format!("- {}\n", item));
    }
}
