use crate::models::{FeatureEntry, MemoryIndex};
use anyhow::Context;
use std::fs;
use std::path::Path;

pub fn load_index(repo: &Path) -> anyhow::Result<MemoryIndex> {
    let path = repo.join(".promem/index.json");
    if !path.exists() {
        return Ok(MemoryIndex::default());
    }
    let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save_index(repo: &Path, index: &MemoryIndex) -> anyhow::Result<()> {
    let path = repo.join(".promem/index.json");
    fs::write(path, serde_json::to_string_pretty(index)? + "\n")?;
    Ok(())
}

pub fn upsert_feature(repo: &Path, entry: FeatureEntry) -> anyhow::Result<()> {
    let mut index = load_index(repo)?;
    if let Some(existing) = index
        .features
        .iter_mut()
        .find(|item| item.name == entry.name)
    {
        *existing = entry;
    } else {
        index.features.push(entry);
        index
            .features
            .sort_by(|left, right| left.name.cmp(&right.name));
    }
    save_index(repo, &index)
}

pub fn load_features(repo: &Path) -> anyhow::Result<Vec<FeatureEntry>> {
    Ok(load_index(repo)?.features)
}
