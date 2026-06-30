use crate::models::SearchMatch;
use crate::store;
use anyhow::Context;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const INDEX_VERSION: u32 = 1;
const EMBEDDING_DIMENSIONS: usize = 64;
const CHUNK_TARGET_WORDS: usize = 120;
const CHUNK_OVERLAP_WORDS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Keyword,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub version: u32,
    pub generated_by: String,
    pub chunks: Vec<MemoryChunk>,
    pub graph: RelationshipGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    pub id: String,
    pub feature: Option<String>,
    pub path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub text: String,
    pub keywords: Vec<String>,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipGraph {
    pub features: BTreeMap<String, FeatureNode>,
    pub files: BTreeMap<String, BTreeSet<String>>,
    pub decisions: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureNode {
    #[serde(default)]
    pub files: BTreeSet<String>,
    #[serde(default)]
    pub decisions: BTreeSet<String>,
    #[serde(default)]
    pub related_features: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub exists: bool,
    pub path: String,
    pub version: Option<u32>,
    pub chunks: usize,
    pub features: usize,
    pub files: usize,
    pub decisions: usize,
}

pub fn keyword_search(repo: &Path, query: &str) -> anyhow::Result<Vec<SearchMatch>> {
    search(repo, query, SearchMode::Keyword, None)
}

pub fn search(
    repo: &Path,
    query: &str,
    mode: SearchMode,
    limit: Option<usize>,
) -> anyhow::Result<Vec<SearchMatch>> {
    let index = load_or_rebuild_index(repo)?;
    if mode == SearchMode::Keyword {
        return keyword_line_search(repo, query, limit);
    }
    let query_terms = keywords(query);
    let query_embedding = embedding(&query_terms);
    let max_results = limit.unwrap_or(20);
    let mut scored = Vec::new();

    for chunk in &index.chunks {
        let keyword_score = keyword_score(&query_terms, &chunk.keywords, &chunk.text);
        let semantic_score = cosine_similarity(&query_embedding, &chunk.embedding);
        let score = match mode {
            SearchMode::Keyword => keyword_score,
            SearchMode::Semantic => semantic_score,
            SearchMode::Hybrid => (keyword_score * 0.65) + (semantic_score * 0.35),
        };
        if score > 0.0 {
            scored.push((score, chunk));
        }
    }

    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.1.path.cmp(&right.1.path))
            .then_with(|| left.1.line_start.cmp(&right.1.line_start))
    });

    Ok(scored
        .into_iter()
        .take(max_results)
        .map(|(score, chunk)| SearchMatch {
            path: chunk.path.clone(),
            line: chunk.line_start,
            line_end: Some(chunk.line_end),
            feature: chunk.feature.clone(),
            score: Some((score * 1000.0).round() / 1000.0),
            snippet: snippet(&chunk.text),
        })
        .collect())
}

fn keyword_line_search(
    repo: &Path,
    query: &str,
    limit: Option<usize>,
) -> anyhow::Result<Vec<SearchMatch>> {
    let query_terms = keywords(query);
    if query_terms.is_empty() {
        return Ok(Vec::new());
    }
    let max_results = limit.unwrap_or(20);
    let root = store::memory_root(repo);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    for entry in WalkBuilder::new(&root)
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
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let data = fs::read_to_string(path)?;
        let relative = store::relative_path(repo, path)?;
        let feature = feature_from_path(Path::new(&relative));
        for (line_index, line) in data.lines().enumerate() {
            let line_terms = keywords(line);
            if keyword_score(&query_terms, &line_terms, line) > 0.0 {
                matches.push(SearchMatch {
                    path: relative.clone(),
                    line: line_index + 1,
                    line_end: None,
                    feature: feature.clone(),
                    score: None,
                    snippet: line.trim().to_string(),
                });
                if matches.len() >= max_results {
                    return Ok(matches);
                }
            }
        }
    }
    Ok(matches)
}

pub fn rebuild_index(repo: &Path) -> anyhow::Result<SearchIndex> {
    let index = build_index(repo)?;
    let path = index_path(repo);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&index)?)?;
    Ok(index)
}

pub fn index_status(repo: &Path) -> anyhow::Result<IndexStatus> {
    let path = index_path(repo);
    if !path.exists() {
        return Ok(IndexStatus {
            exists: false,
            path: store::relative_path(repo, &path)?,
            version: None,
            chunks: 0,
            features: 0,
            files: 0,
            decisions: 0,
        });
    }
    let index = load_index(repo)?;
    status_from_index(repo, &path, true, &index)
}

pub fn related_features(repo: &Path, feature: &str) -> anyhow::Result<Vec<String>> {
    let index = load_or_rebuild_index(repo)?;
    let slug = normalize_feature_for_lookup(feature);
    let Some(node) = index.graph.features.get(&slug) else {
        return Ok(Vec::new());
    };
    Ok(node.related_features.iter().cloned().collect())
}

fn load_or_rebuild_index(repo: &Path) -> anyhow::Result<SearchIndex> {
    match load_index(repo) {
        Ok(index) if index.version == INDEX_VERSION && index_is_fresh(repo)? => Ok(index),
        _ => rebuild_index(repo),
    }
}

fn load_index(repo: &Path) -> anyhow::Result<SearchIndex> {
    let path = index_path(repo);
    let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&data)?)
}

fn build_index(repo: &Path) -> anyhow::Result<SearchIndex> {
    let root = store::memory_root(repo);
    let mut chunks = Vec::new();
    let mut graph = RelationshipGraph::default();
    if !root.exists() {
        return Ok(SearchIndex {
            version: INDEX_VERSION,
            generated_by: format!("promemo {}", env!("CARGO_PKG_VERSION")),
            chunks,
            graph,
        });
    }

    for entry in WalkBuilder::new(&root)
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
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let data = fs::read_to_string(path)?;
        let relative = store::relative_path(repo, path)?;
        let feature = feature_from_path(Path::new(&relative));
        update_graph(&mut graph, feature.as_deref(), &data);
        chunks.extend(chunk_markdown(&relative, feature, &data));
    }

    link_related_features(&mut graph);
    Ok(SearchIndex {
        version: INDEX_VERSION,
        generated_by: format!("promemo {}", env!("CARGO_PKG_VERSION")),
        chunks,
        graph,
    })
}

fn chunk_markdown(path: &str, feature: Option<String>, data: &str) -> Vec<MemoryChunk> {
    let lines: Vec<&str> = data.lines().collect();
    let mut positions = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        for word in line.split_whitespace() {
            positions.push((line_index + 1, word.to_string()));
        }
    }
    if positions.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    while start < positions.len() {
        let end = (start + CHUNK_TARGET_WORDS).min(positions.len());
        let line_start = positions[start].0;
        let line_end = positions[end - 1].0;
        let text = lines[line_start - 1..line_end].join("\n");
        let terms = keywords(&text);
        let vector = embedding(&terms);
        chunks.push(MemoryChunk {
            id: format!("{}:{}-{}", path, line_start, line_end),
            feature: feature.clone(),
            path: path.to_string(),
            line_start,
            line_end,
            text,
            keywords: terms,
            embedding: vector,
        });
        if end == positions.len() {
            break;
        }
        start = end.saturating_sub(CHUNK_OVERLAP_WORDS);
    }
    chunks
}

fn feature_from_path(path: &Path) -> Option<String> {
    let mut parts = path
        .components()
        .filter_map(|part| part.as_os_str().to_str());
    if parts.next()? != store::MEMORY_DIR {
        return None;
    }
    if parts.next()? != "features" {
        return None;
    }
    parts.next().map(ToString::to_string)
}

fn update_graph(graph: &mut RelationshipGraph, feature: Option<&str>, data: &str) {
    let Some(feature) = feature else {
        return;
    };
    let node = graph.features.entry(feature.to_string()).or_default();

    for line in data.lines() {
        let trimmed = line.trim();
        if let Some(path) = parse_markdown_code_reference(trimmed) {
            node.files.insert(path.clone());
            graph
                .files
                .entry(path)
                .or_default()
                .insert(feature.to_string());
        }
        if let Some(decision) = trimmed.strip_prefix("### ") {
            let decision = decision.trim().to_string();
            if !decision.is_empty() {
                node.decisions.insert(decision.clone());
                graph
                    .decisions
                    .entry(decision)
                    .or_default()
                    .insert(feature.to_string());
            }
        }
    }
}

fn link_related_features(graph: &mut RelationshipGraph) {
    let mut related: HashMap<String, BTreeSet<String>> = HashMap::new();
    for features in graph.files.values().chain(graph.decisions.values()) {
        for feature in features {
            for other in features {
                if feature != other {
                    related
                        .entry(feature.clone())
                        .or_default()
                        .insert(other.clone());
                }
            }
        }
    }
    for (feature, related_features) in related {
        graph
            .features
            .entry(feature)
            .or_default()
            .related_features
            .extend(related_features);
    }
}

fn parse_markdown_code_reference(line: &str) -> Option<String> {
    let trimmed = line.strip_prefix("- `")?;
    let end = trimmed.find('`')?;
    let value = &trimmed[..end];
    if value.contains('/') || value.contains('.') {
        Some(value.to_string())
    } else {
        None
    }
}

fn keyword_score(query_terms: &[String], chunk_terms: &[String], text: &str) -> f32 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let chunk_set: BTreeSet<&str> = chunk_terms.iter().map(String::as_str).collect();
    let mut matches = 0.0;
    for term in query_terms {
        if chunk_set.contains(term.as_str()) {
            matches += 1.0;
        }
    }
    let phrase_bonus = if text.to_lowercase().contains(&query_terms.join(" ")) {
        0.4
    } else {
        0.0
    };
    (matches / query_terms.len() as f32) + phrase_bonus
}

fn keywords(input: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            current.push(ch.to_ascii_lowercase());
        } else {
            push_keyword(&mut terms, &mut current);
        }
    }
    push_keyword(&mut terms, &mut current);
    terms
}

fn push_keyword(terms: &mut Vec<String>, current: &mut String) {
    if current.len() > 2 && !is_stopword(current) {
        terms.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

fn is_stopword(value: &str) -> bool {
    matches!(
        value,
        "the"
            | "and"
            | "for"
            | "with"
            | "that"
            | "this"
            | "from"
            | "into"
            | "when"
            | "then"
            | "will"
            | "should"
            | "would"
            | "could"
            | "can"
            | "are"
            | "was"
            | "were"
            | "has"
            | "have"
            | "had"
            | "not"
            | "but"
            | "you"
            | "your"
    )
}

fn embedding(terms: &[String]) -> Vec<f32> {
    let mut vector = vec![0.0; EMBEDDING_DIMENSIONS];
    for term in terms {
        let index = stable_bucket(term) % EMBEDDING_DIMENSIONS;
        vector[index] += 1.0;
    }
    normalize(&mut vector);
    vector
}

fn stable_bucket(value: &str) -> usize {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash as usize
}

fn normalize(vector: &mut [f32]) {
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in vector {
            *value /= magnitude;
        }
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f32>()
}

fn snippet(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= 180 {
        collapsed
    } else {
        format!("{}...", &collapsed[..180])
    }
}

fn index_path(repo: &Path) -> PathBuf {
    store::memory_root(repo).join("cache/search-index.json")
}

fn index_is_fresh(repo: &Path) -> anyhow::Result<bool> {
    let path = index_path(repo);
    let index_modified = fs::metadata(&path)?.modified()?;
    let root = store::memory_root(repo);
    for entry in WalkBuilder::new(&root)
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
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        if fs::metadata(path)?.modified()? > index_modified {
            return Ok(false);
        }
    }
    Ok(true)
}

fn status_from_index(
    repo: &Path,
    path: &Path,
    exists: bool,
    index: &SearchIndex,
) -> anyhow::Result<IndexStatus> {
    Ok(IndexStatus {
        exists,
        path: store::relative_path(repo, path)?,
        version: Some(index.version),
        chunks: index.chunks.len(),
        features: index.graph.features.len(),
        files: index.graph.files.len(),
        decisions: index.graph.decisions.len(),
    })
}

fn normalize_feature_for_lookup(feature: &str) -> String {
    feature.trim().to_lowercase().replace(' ', "-")
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

        assert_eq!(matches.len(), 2);
        assert!(matches[0].snippet.contains("refresh tokens"));
        assert!(dir.path().join(".promemo/cache/search-index.json").exists());
    }

    #[test]
    fn semantic_search_uses_local_embeddings() {
        let dir = tempfile::tempdir().unwrap();
        store::init_repo(dir.path()).unwrap();
        store::save_memory(
            dir.path(),
            "auth",
            &MemoryInput {
                title: "Authentication".to_string(),
                summary: "Passkey login and identity verification.".to_string(),
                ..MemoryInput::default()
            },
        )
        .unwrap();

        let matches = search(
            dir.path(),
            "identity passkey",
            SearchMode::Semantic,
            Some(3),
        )
        .unwrap();

        assert!(!matches.is_empty());
        assert_eq!(matches[0].feature.as_deref(), Some("auth"));
    }
}
