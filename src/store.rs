use crate::config::Config;
use crate::index;
use crate::models::{
    DoctorCheck, DoctorReport, FeatureEntry, LoadedContext, MemoryInput, SaveReport,
};
use crate::render;
use anyhow::{bail, Context};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const GENERATED_START: &str = "<!-- promem:generated:start -->";
const GENERATED_END: &str = "<!-- promem:generated:end -->";

pub fn find_repo_root(start: &Path) -> anyhow::Result<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join(".promem").is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }

    bail!("not a Promem repository; run `promem init` from the repository root first");
}

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
    write_if_missing(
        root.join("shared/notes.md"),
        "# Shared Notes\n\nHuman-maintained project memory that Promem will not overwrite.\n",
    )?;
    write_if_missing(root.join("index.json"), "{\n  \"features\": []\n}\n")?;
    write_if_missing(
        root.join("config.toml"),
        &toml::to_string_pretty(&Config::default())?,
    )?;
    Ok(())
}

pub fn save_memory(repo: &Path, feature: &str, memory: &MemoryInput) -> anyhow::Result<()> {
    save_memory_with_report(repo, feature, memory).map(|_| ())
}

pub fn save_memory_with_report(
    repo: &Path,
    feature: &str,
    memory: &MemoryInput,
) -> anyhow::Result<SaveReport> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let feature_dir = repo.join(".promem/features").join(&slug);
    fs::create_dir_all(&feature_dir)?;
    let mut files_written = Vec::new();
    let mut warnings = Vec::new();

    for (name, contents) in render::render_feature_files(memory) {
        let path = feature_dir.join(name);
        write_generated_file(&path, &contents, &mut warnings)?;
        files_written.push(path.strip_prefix(repo)?.display().to_string());
    }
    write_if_missing(
        feature_dir.join("notes.md"),
        "# Notes\n\nHuman-maintained notes for this feature. Promem will not overwrite this file.\n",
    )?;

    index::upsert_feature(
        repo,
        FeatureEntry {
            name: slug.clone(),
            title: memory.title.clone(),
            summary: memory.summary.clone(),
        },
    )?;
    files_written.push(".promem/index.json".to_string());
    files_written.sort();
    Ok(SaveReport {
        feature: slug,
        title: memory.title.clone(),
        dry_run: false,
        files_written,
        warnings,
    })
}

pub fn preview_memory_save(
    repo: &Path,
    feature: &str,
    memory: &MemoryInput,
) -> anyhow::Result<SaveReport> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let mut files_written = render::render_feature_files(memory)
        .into_iter()
        .map(|(name, _)| {
            repo.join(".promem/features")
                .join(&slug)
                .join(name)
                .strip_prefix(repo)
                .map(|path| path.display().to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    files_written.push(".promem/index.json".to_string());
    files_written.sort();

    Ok(SaveReport {
        feature: slug,
        title: memory.title.clone(),
        dry_run: true,
        files_written,
        warnings: Vec::new(),
    })
}

pub fn feature_dir(repo: &Path, feature: &str) -> anyhow::Result<PathBuf> {
    ensure_initialized(repo)?;
    Ok(repo
        .join(".promem/features")
        .join(normalize_feature(feature)?))
}

pub fn load_context(repo: &Path, feature: &str) -> anyhow::Result<LoadedContext> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let mut text = String::new();
    append_if_exists(&mut text, repo.join(".promem/project.md"))?;
    append_if_exists(&mut text, repo.join(".promem/shared/architecture.md"))?;
    append_if_exists(&mut text, repo.join(".promem/shared/coding-guidelines.md"))?;
    append_if_exists(&mut text, repo.join(".promem/shared/notes.md"))?;

    let feature_dir = repo.join(".promem/features").join(&slug);
    if !feature_dir.exists() {
        bail!(
            "feature `{}` does not exist; save it first with `promem save-json {}`",
            slug,
            slug
        );
    }
    if feature_dir.join("memory.md").exists() {
        append_if_exists(&mut text, feature_dir.join("memory.md"))?;
    } else {
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
    }
    append_if_exists(&mut text, feature_dir.join("notes.md"))?;

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

fn write_generated_file(
    path: &Path,
    contents: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<()> {
    let generated = wrap_generated(contents);
    if !path.exists() {
        fs::write(path, generated)?;
        return Ok(());
    }

    let existing = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if existing.contains(GENERATED_START) && existing.contains(GENERATED_END) {
        fs::write(path, replace_generated_region(&existing, &generated))?;
    } else if existing.trim().is_empty() {
        fs::write(path, generated)?;
    } else {
        warnings.push(format!(
            "{} had no Promem generated markers; preserved existing content below the generated block",
            path.display()
        ));
        fs::write(
            path,
            format!(
                "{}\n\n<!-- promem:manual:preserved -->\n{}",
                generated,
                existing.trim_start()
            ),
        )?;
    }
    Ok(())
}

fn wrap_generated(contents: &str) -> String {
    format!(
        "{}\n{}\n{}\n",
        GENERATED_START,
        contents.trim_end(),
        GENERATED_END
    )
}

fn replace_generated_region(existing: &str, generated: &str) -> String {
    let Some(start) = existing.find(GENERATED_START) else {
        return generated.to_string();
    };
    let Some(end) = existing.find(GENERATED_END) else {
        return generated.to_string();
    };
    let after_end = end + GENERATED_END.len();
    format!(
        "{}{}{}",
        &existing[..start],
        generated.trim_end(),
        &existing[after_end..]
    )
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
    fn find_repo_root_walks_up_from_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path()).unwrap();
        let nested = dir.path().join("src/auth");
        fs::create_dir_all(&nested).unwrap();

        let root = find_repo_root(&nested).unwrap();

        assert_eq!(root, dir.path());
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
    fn save_preserves_unmarked_existing_feature_files() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path()).unwrap();
        let feature_dir = dir.path().join(".promem/features/auth");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("context.md"),
            "# Manual Context\n\nKeep me.\n",
        )
        .unwrap();

        let report = save_memory_with_report(dir.path(), "auth", &sample_memory()).unwrap();

        let context = fs::read_to_string(feature_dir.join("context.md")).unwrap();
        assert!(context.contains("<!-- promem:generated:start -->"));
        assert!(context.contains("# Manual Context"));
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("had no Promem generated markers")));
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
