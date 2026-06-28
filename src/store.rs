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

pub const MEMORY_DIR: &str = ".promemo";
pub const LEGACY_MEMORY_DIR: &str = ".promem";

const GENERATED_START: &str = "<!-- promemo:generated:start -->";
const GENERATED_END: &str = "<!-- promemo:generated:end -->";
const LEGACY_GENERATED_START: &str = "<!-- promem:generated:start -->";
const LEGACY_GENERATED_END: &str = "<!-- promem:generated:end -->";

pub fn find_repo_root(start: &Path) -> anyhow::Result<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join(MEMORY_DIR).is_dir() || candidate.join(LEGACY_MEMORY_DIR).is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }

    bail!("not a Promemo repository; run `promemo init` from the repository root first");
}

pub fn init_repo(repo: &Path) -> anyhow::Result<()> {
    let root = repo.join(MEMORY_DIR);
    fs::create_dir_all(root.join("features"))?;
    fs::create_dir_all(root.join("shared"))?;
    fs::create_dir_all(root.join("decisions"))?;
    fs::create_dir_all(root.join("cache"))?;

    write_if_missing(
        root.join("project.md"),
        "# Project Memory\n\nProject-level context for Promemo.\n",
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
        "# Shared Notes\n\nHuman-maintained project memory that Promemo will not overwrite.\n",
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
    memory.validate()?;
    let slug = normalize_feature(feature)?;
    let root = memory_root(repo);
    let feature_dir = root.join("features").join(&slug);
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
        "# Notes\n\nHuman-maintained notes for this feature. Promemo will not overwrite this file.\n",
    )?;

    index::upsert_feature(
        repo,
        FeatureEntry {
            name: slug.clone(),
            title: memory.title.clone(),
            summary: memory.summary.clone(),
        },
    )?;
    files_written.push(relative_path(repo, &root.join("index.json"))?);
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
    memory.validate()?;
    let slug = normalize_feature(feature)?;
    let root = memory_root(repo);
    let mut files_written = render::render_feature_files(memory)
        .into_iter()
        .map(|(name, _)| {
            root.join("features")
                .join(&slug)
                .join(name)
                .strip_prefix(repo)
                .map(|path| path.display().to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    files_written.push(relative_path(repo, &root.join("index.json"))?);
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
    Ok(memory_root(repo)
        .join("features")
        .join(normalize_feature(feature)?))
}

pub fn load_context(repo: &Path, feature: &str) -> anyhow::Result<LoadedContext> {
    ensure_initialized(repo)?;
    let slug = normalize_feature(feature)?;
    let root = memory_root(repo);
    let mut text = String::new();
    append_if_exists(&mut text, root.join("project.md"))?;
    append_if_exists(&mut text, root.join("shared/architecture.md"))?;
    append_if_exists(&mut text, root.join("shared/coding-guidelines.md"))?;
    append_if_exists(&mut text, root.join("shared/notes.md"))?;

    let feature_dir = root.join("features").join(&slug);
    if !feature_dir.exists() {
        bail!(
            "feature `{}` does not exist; save it first with `promemo save-json {}`",
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
    let root = memory_root(repo);
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
        check(
            memory_root(repo).exists(),
            "repository has memory directory",
        ),
        check(
            memory_root(repo).join("config.toml").exists(),
            "config.toml exists",
        ),
        check(
            memory_root(repo).join("index.json").exists(),
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
    if !memory_root(repo).exists() {
        bail!("not a Promemo repository; run `promemo init` first");
    }
    Ok(())
}

pub fn memory_root(repo: &Path) -> PathBuf {
    let current = repo.join(MEMORY_DIR);
    if current.exists() {
        current
    } else {
        repo.join(LEGACY_MEMORY_DIR)
    }
}

pub fn relative_path(repo: &Path, path: &Path) -> anyhow::Result<String> {
    Ok(path.strip_prefix(repo)?.display().to_string())
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
    if has_generated_markers(&existing) {
        fs::write(path, replace_generated_region(&existing, &generated))?;
    } else if existing.trim().is_empty() {
        fs::write(path, generated)?;
    } else {
        warnings.push(format!(
            "{} had no Promemo generated markers; preserved existing content below the generated block",
            path.display()
        ));
        fs::write(
            path,
            format!(
                "{}\n\n<!-- promemo:manual:preserved -->\n{}",
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
    let (start_marker, end_marker) = if existing.contains(GENERATED_START) {
        (GENERATED_START, GENERATED_END)
    } else {
        (LEGACY_GENERATED_START, LEGACY_GENERATED_END)
    };
    let Some(start) = existing.find(start_marker) else {
        return generated.to_string();
    };
    let Some(end) = existing.find(end_marker) else {
        return generated.to_string();
    };
    let after_end = end + end_marker.len();
    format!(
        "{}{}{}",
        &existing[..start],
        generated.trim_end(),
        &existing[after_end..]
    )
}

fn has_generated_markers(existing: &str) -> bool {
    (existing.contains(GENERATED_START) && existing.contains(GENERATED_END))
        || (existing.contains(LEGACY_GENERATED_START) && existing.contains(LEGACY_GENERATED_END))
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
    fn init_creates_promemo_structure() {
        let dir = tempfile::tempdir().unwrap();

        init_repo(dir.path()).unwrap();

        assert!(dir.path().join(".promemo/project.md").exists());
        assert!(dir.path().join(".promemo/config.toml").exists());
        assert!(dir.path().join(".promemo/index.json").exists());
        assert!(dir.path().join(".promemo/features").exists());
        assert!(dir.path().join(".promemo/shared").exists());
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
    fn legacy_promem_repositories_still_work() {
        let dir = tempfile::tempdir().unwrap();
        let legacy_root = dir.path().join(".promem");
        fs::create_dir_all(legacy_root.join("features/auth")).unwrap();
        fs::create_dir_all(legacy_root.join("shared")).unwrap();
        fs::write(legacy_root.join("index.json"), "{\n  \"features\": []\n}\n").unwrap();
        fs::write(
            legacy_root.join("features/auth/context.md"),
            "<!-- promem:generated:start -->\n# Old Context\n<!-- promem:generated:end -->\n",
        )
        .unwrap();

        save_memory(dir.path(), "auth", &sample_memory()).unwrap();

        let context = fs::read_to_string(legacy_root.join("features/auth/context.md")).unwrap();
        assert!(context.contains("<!-- promemo:generated:start -->"));
        assert!(context.contains("Email login"));
        assert!(!dir.path().join(".promemo").exists());
    }

    #[test]
    fn save_json_writes_markdown_and_index() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path()).unwrap();
        let memory = sample_memory();

        save_memory(dir.path(), "Authentication", &memory).unwrap();

        let context = fs::read_to_string(
            dir.path()
                .join(".promemo/features/authentication/context.md"),
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
        let feature_dir = dir.path().join(".promemo/features/auth");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("context.md"),
            "# Manual Context\n\nKeep me.\n",
        )
        .unwrap();

        let report = save_memory_with_report(dir.path(), "auth", &sample_memory()).unwrap();

        let context = fs::read_to_string(feature_dir.join("context.md")).unwrap();
        assert!(context.contains("<!-- promemo:generated:start -->"));
        assert!(context.contains("# Manual Context"));
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("had no Promemo generated markers")));
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
