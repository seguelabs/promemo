use crate::models::{RepoSnapshot, TodoComment};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

pub fn collect(repo: &Path, feature: &str, dry_run: bool) -> anyhow::Result<RepoSnapshot> {
    let status_short = git(repo, &["status", "--short"])?;
    let changed_files = git(repo, &["diff", "--name-only"])?
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let recent_commits = git(repo, &["log", "--oneline", "-5"])
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let todos = collect_todos(repo)?;
    let warnings = secret_warnings(repo, &changed_files)?;

    Ok(RepoSnapshot {
        feature: feature.to_string(),
        dry_run,
        branch: current_branch(repo)?,
        status_short,
        diff_stat: git(repo, &["diff", "--stat"])?,
        changed_files,
        recent_commits,
        todos,
        warnings,
    })
}

fn current_branch(repo: &Path) -> anyhow::Result<Option<String>> {
    let branch = git(repo, &["branch", "--show-current"])?;
    let branch = branch.trim();
    if branch.is_empty() {
        Ok(None)
    } else {
        Ok(Some(branch.to_string()))
    }
}

fn collect_todos(repo: &Path) -> anyhow::Result<Vec<TodoComment>> {
    let mut todos = Vec::new();
    for entry in WalkDir::new(repo)
        .into_iter()
        .filter_entry(|entry| !is_ignored_dir(entry.path()))
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
    {
        let data = match std::fs::read_to_string(entry.path()) {
            Ok(data) => data,
            Err(_) => continue,
        };
        for (idx, line) in data.lines().enumerate() {
            if line.contains("TODO") || line.contains("FIXME") {
                todos.push(TodoComment {
                    path: entry.path().strip_prefix(repo)?.display().to_string(),
                    line: idx + 1,
                    text: line.trim().to_string(),
                });
            }
        }
    }
    Ok(todos)
}

fn secret_warnings(repo: &Path, files: &[String]) -> anyhow::Result<Vec<String>> {
    let mut warnings = Vec::new();
    for file in files {
        let path = repo.join(file);
        if !path.is_file() {
            continue;
        }
        let data = match std::fs::read_to_string(&path) {
            Ok(data) => data,
            Err(_) => continue,
        };
        let lowered = data.to_lowercase();
        if lowered.contains("api_key")
            || lowered.contains("secret")
            || lowered.contains("password")
            || lowered.contains("token")
        {
            warnings.push(format!("possible sensitive content in {}", file));
        }
    }
    Ok(warnings)
}

fn is_ignored_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| matches!(name, ".git" | "target" | "cache"))
        .unwrap_or(false)
}

fn git(repo: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    } else {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn snapshot_collects_git_state_and_todos() {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("main.rs"), "// TODO: wire auth\n").unwrap();

        let snapshot = collect(dir.path(), "auth", true).unwrap();

        assert!(snapshot.dry_run);
        assert_eq!(snapshot.feature, "auth");
        assert_eq!(snapshot.todos.len(), 1);
    }
}
