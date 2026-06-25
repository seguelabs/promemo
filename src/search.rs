use crate::models::SearchMatch;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn keyword_search(repo: &Path, query: &str) -> anyhow::Result<Vec<SearchMatch>> {
    let needle = query.to_lowercase();
    let root = repo.join(".promem");
    let mut matches = Vec::new();
    if !root.exists() {
        return Ok(matches);
    }

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let data = fs::read_to_string(path)?;
        for (idx, line) in data.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                matches.push(SearchMatch {
                    path: path.strip_prefix(repo)?.display().to_string(),
                    line: idx + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::MemoryInput, store};

    #[test]
    fn search_finds_markdown_snippets() {
        let dir = tempfile::tempdir().unwrap();
        store::init_repo(dir.path()).unwrap();
        store::save_memory(
            dir.path(),
            "auth",
            &MemoryInput {
                title: "Authentication".to_string(),
                summary: "Uses refresh tokens.".to_string(),
                ..MemoryInput::default()
            },
        )
        .unwrap();

        let matches = keyword_search(dir.path(), "refresh").unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].snippet.contains("refresh tokens"));
    }
}
