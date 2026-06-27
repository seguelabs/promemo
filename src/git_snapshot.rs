use crate::models::{RepoSnapshot, TodoComment};
use crate::store;
use ignore::WalkBuilder;
use std::path::Path;
use std::process::Command;

pub fn collect(repo: &Path, feature: &str, dry_run: bool) -> anyhow::Result<RepoSnapshot> {
    let mut warnings = Vec::new();
    let status_short = git(repo, &["status", "--short"], &mut warnings)?;
    let changed_files = changed_files(repo, &status_short, &mut warnings)?;
    let recent_commits = git(repo, &["log", "--oneline", "-5"], &mut warnings)?
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let todos = collect_todos(repo)?;
    let existing_memory = collect_existing_memory(repo, feature)?;
    warnings.extend(secret_warnings(repo, &changed_files)?);

    Ok(RepoSnapshot {
        feature: feature.to_string(),
        dry_run,
        branch: current_branch(repo, &mut warnings)?,
        status_short,
        diff_stat: git(repo, &["diff", "--stat"], &mut warnings)?,
        changed_files,
        recent_commits,
        todos,
        existing_memory,
        warnings,
    })
}

fn current_branch(repo: &Path, warnings: &mut Vec<String>) -> anyhow::Result<Option<String>> {
    let branch = git(repo, &["branch", "--show-current"], warnings)?;
    let branch = branch.trim();
    if branch.is_empty() {
        Ok(None)
    } else {
        Ok(Some(branch.to_string()))
    }
}

fn changed_files(
    repo: &Path,
    status_short: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<String>> {
    let mut files = git(repo, &["diff", "--name-only"], warnings)?
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    for line in status_short.lines() {
        if let Some(path) = status_path(line) {
            if !files.contains(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn status_path(line: &str) -> Option<String> {
    let path = line.get(3..)?.trim();
    let path = path.split(" -> ").last().unwrap_or(path).trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn collect_todos(repo: &Path) -> anyhow::Result<Vec<TodoComment>> {
    let mut todos = Vec::new();
    for entry in WalkBuilder::new(repo)
        .hidden(false)
        .filter_entry(|entry| !is_ignored_dir(entry.path()))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
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

fn collect_existing_memory(repo: &Path, feature: &str) -> anyhow::Result<Vec<String>> {
    let feature_dir = store::memory_root(repo)
        .join("features")
        .join(normalize_feature(feature));
    let mut entries = Vec::new();
    if !feature_dir.exists() {
        return Ok(entries);
    }

    for entry in WalkBuilder::new(&feature_dir)
        .hidden(false)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
    {
        entries.push(entry.path().strip_prefix(repo)?.display().to_string());
    }
    entries.sort();
    Ok(entries)
}

fn normalize_feature(feature: &str) -> String {
    feature.trim().to_lowercase().replace(' ', "-")
}

fn is_ignored_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| matches!(name, ".git" | "target" | "cache"))
        .unwrap_or(false)
}

fn git(repo: &Path, args: &[&str], warnings: &mut Vec<String>) -> anyhow::Result<String> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let command = format!("git {}", args.join(" "));
        if stderr.is_empty() {
            warnings.push(format!("{} failed", command));
        } else {
            warnings.push(format!("{} failed: {}", command, stderr));
        }
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
