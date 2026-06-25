use crate::config::Config;
use crate::index;
use crate::models::{DoctorCheck, DoctorReport, FeatureEntry, LoadedContext, MemoryInput};
use crate::render;
use anyhow::{bail, Context};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn init_repo(repo: &Path) -> anyhow::Result<()> {
    let root = repo.join(".promem");
    fs::create_dir_all(root.join("features"))?;
    fs::create_dir_all(root.join("shared"))?;
    fs::create_dir_all(root.join("decisions"))?;
    fs::create_dir_all(root.join("cache"))?;

    write_if_missing(
        root.join("project.md"),
        "# Project Memory\n\nProject-level context for Promem.\n",
    )?;
    write_if_missing(
        root.join("shared/architecture.md"),
        "# Shared Architecture\n\n",
    )?;
    write_if_missing(
        root.join("shared/coding-guidelines.md"),
        "# Coding Guidelines\n\n",
    )?;
    write_if_missing(root.join("index.json"), "{\n  \"features\": []\n}\n")?;
    write_if_missing(
        root.join("config.toml"),
        &toml::to_string_pretty(&Config::default())?,
    )?;
    Ok(())
}

pub fn save_memory(repo: &Path, feature: &str, memory: &MemoryInput) -> anyhow::Result<()> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let feature_dir = repo.join(".promem/features").join(&slug);
    fs::create_dir_all(&feature_dir)?;

    for (name, contents) in render::render_feature_files(memory) {
        fs::write(feature_dir.join(name), contents)?;
    }

    index::upsert_feature(
        repo,
        FeatureEntry {
            name: slug,
            title: memory.title.clone(),
            summary: memory.summary.clone(),
        },
    )?;
    Ok(())
}

pub fn load_context(repo: &Path, feature: &str) -> anyhow::Result<LoadedContext> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let mut text = String::new();
    append_if_exists(&mut text, repo.join(".promem/project.md"))?;
    append_if_exists(&mut text, repo.join(".promem/shared/architecture.md"))?;
    append_if_exists(&mut text, repo.join(".promem/shared/coding-guidelines.md"))?;

    let feature_dir = repo.join(".promem/features").join(&slug);
    if !feature_dir.exists() {
        bail!(
            "feature `{}` does not exist; save it first with `promem save-json {}`",
            slug,
            slug
        );
    }
    for name in [
        "context.md",
        "architecture.md",
        "decisions.md",
        "api.md",
        "data-model.md",
        "todos.md",
        "prompts.md",
    ] {
        append_if_exists(&mut text, feature_dir.join(name))?;
    }

    Ok(LoadedContext {
        feature: slug,
        text: text.trim().to_string(),
    })
}

pub fn memory_tree(repo: &Path) -> anyhow::Result<Vec<String>> {
    ensure_initialized(repo)?;
    let root = repo.join(".promem");
    let mut entries = Vec::new();
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
    {
        let relative = entry.path().strip_prefix(repo)?.display().to_string();
        entries.push(relative);
    }
    entries.sort();
    Ok(entries)
}

pub fn doctor(repo: &Path) -> anyhow::Result<DoctorReport> {
    let checks = vec![
        check(repo.join(".git").exists(), "repository has .git"),
        check(repo.join(".promem").exists(), "repository has .promem"),
        check(
            repo.join(".promem/config.toml").exists(),
            "config.toml exists",
        ),
        check(
            repo.join(".promem/index.json").exists(),
            "index.json exists",
        ),
        check(
            repo.join(".gitignore").exists(),
            ".gitignore exists for generated files",
        ),
    ];
    Ok(DoctorReport { checks })
}

fn ensure_initialized(repo: &Path) -> anyhow::Result<()> {
    if !repo.join(".promem").exists() {
        bail!("not a Promem repository; run `promem init` first");
    }
    Ok(())
}

fn normalize_feature(feature: &str) -> anyhow::Result<String> {
    let slug = feature.trim().to_lowercase().replace(' ', "-");
    if slug.is_empty() || slug.contains('/') || slug.contains('\\') {
        bail!("feature must be a non-empty name without path separators");
    }
    Ok(slug)
}

fn write_if_missing(path: PathBuf, contents: &str) -> anyhow::Result<()> {
    if !path.exists() {
        fs::write(path, contents)?;
    }
    Ok(())
}

fn append_if_exists(out: &mut String, path: PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        out.push_str(&data);
        out.push_str("\n\n");
    }
    Ok(())
}

fn check(ok: bool, message: &str) -> DoctorCheck {
    DoctorCheck {
        ok,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArchitectureNote, CurrentState, Decision, DecisionStatus};

    #[test]
    fn init_creates_promem_structure() {
        let dir = tempfile::tempdir().unwrap();

        init_repo(dir.path()).unwrap();

        assert!(dir.path().join(".promem/project.md").exists());
        assert!(dir.path().join(".promem/config.toml").exists());
        assert!(dir.path().join(".promem/index.json").exists());
        assert!(dir.path().join(".promem/features").exists());
        assert!(dir.path().join(".promem/shared").exists());
    }

    #[test]
    fn save_json_writes_markdown_and_index() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path()).unwrap();
        let memory = sample_memory();

        save_memory(dir.path(), "Authentication", &memory).unwrap();

        let context = fs::read_to_string(
            dir.path()
                .join(".promem/features/authentication/context.md"),
        )
        .unwrap();
        assert!(context.contains("Email login"));
        let features = index::load_features(dir.path()).unwrap();
        assert_eq!(features[0].name, "authentication");
    }

    #[test]
    fn load_combines_project_shared_and_feature_context() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path()).unwrap();
        save_memory(dir.path(), "auth", &sample_memory()).unwrap();

        let loaded = load_context(dir.path(), "auth").unwrap();

        assert!(loaded.text.contains("# Project Memory"));
        assert!(loaded.text.contains("# Authentication"));
        assert!(loaded.text.contains("JWT access tokens"));
    }

    fn sample_memory() -> MemoryInput {
        MemoryInput {
            title: "Authentication".to_string(),
            summary: "Email login with JWT access tokens.".to_string(),
            current_state: Some(CurrentState {
                implemented: vec!["Email login".to_string()],
                pending: vec!["MFA".to_string()],
            }),
            architecture: vec![ArchitectureNote {
                title: "Token validation".to_string(),
                details: "Middleware validates access tokens.".to_string(),
            }],
            decisions: vec![Decision {
                title: "Use JWT access tokens".to_string(),
                reason: Some("Keeps API stateless.".to_string()),
                tradeoffs: vec!["Harder revocation".to_string()],
                status: DecisionStatus::Accepted,
            }],
            ..MemoryInput::default()
        }
    }
}
