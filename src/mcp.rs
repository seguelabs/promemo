use crate::{config, git_snapshot, index, models, provider, search, store};
use anyhow::{bail, Context};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::Path;

const PROTOCOL_VERSION: &str = "2024-11-05";

pub fn serve(cwd: &Path, stdin: &mut impl Read, stdout: &mut impl Write) -> anyhow::Result<()> {
    while let Some(message) = read_message(stdin)? {
        let response = handle_message(cwd, &message)?;
        if let Some(response) = response {
            write_message(stdout, &response)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn handle_message(cwd: &Path, message: &str) -> anyhow::Result<Option<Value>> {
    let request: JsonRpcMessage =
        serde_json::from_str(message).context("invalid JSON-RPC message")?;
    let Some(id) = request.id else {
        return Ok(None);
    };

    let result = match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => call_tool(cwd, request.params.unwrap_or_else(|| json!({}))),
        "resources/list" => list_resources(cwd),
        "resources/read" => read_resource(cwd, request.params.unwrap_or_else(|| json!({}))),
        "prompts/list" => Ok(json!({ "prompts": prompts() })),
        "prompts/get" => get_prompt(request.params.unwrap_or_else(|| json!({}))),
        other => Err(anyhow::anyhow!("unsupported MCP method `{}`", other)),
    };

    Ok(Some(match result {
        Ok(result) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32000,
                "message": error.to_string()
            }
        }),
    }))
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "promemo",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn call_tool(cwd: &Path, params: Value) -> anyhow::Result<Value> {
    let request: ToolCallRequest = serde_json::from_value(params)?;
    let arguments = request.arguments.unwrap_or_else(|| json!({}));
    let repo = store::find_repo_root(cwd)?;
    let result = match request.name.as_str() {
        "promemo_load_context" => {
            let args: FeatureArgs = serde_json::from_value(arguments)?;
            serde_json::to_value(store::load_context(&repo, &args.feature)?)?
        }
        "promemo_search" => {
            let args: SearchArgs = serde_json::from_value(arguments)?;
            serde_json::to_value(search::keyword_search(&repo, &args.query)?)?
        }
        "promemo_save_memory" => {
            let args: SaveMemoryArgs = serde_json::from_value(arguments)?;
            let report = if args.dry_run.unwrap_or(false) {
                store::preview_memory_save(&repo, &args.feature, &args.memory)?
            } else {
                store::save_memory_with_report(&repo, &args.feature, &args.memory)?
            };
            serde_json::to_value(report)?
        }
        "promemo_preview_memory" => {
            let args: PreviewMemoryArgs = serde_json::from_value(arguments)?;
            serde_json::to_value(store::preview_memory_save(
                &repo,
                &args.feature,
                &args.memory,
            )?)?
        }
        "promemo_extract_memory" => {
            let args: ExtractMemoryArgs = serde_json::from_value(arguments)?;
            let config = config::Config::load(&repo).context("failed to load Promemo config")?;
            let provider = provider::provider_from_config(&config)?;
            let memory = provider.extract_memory(&provider::ExtractionRequest {
                feature: args.feature.clone(),
                source_text: args.source_text,
            })?;
            let report = if args.dry_run.unwrap_or(true) {
                store::preview_memory_save(&repo, &args.feature, &memory)?
            } else {
                store::save_memory_with_report(&repo, &args.feature, &memory)?
            };
            extraction_result(&memory, &report)?
        }
        "promemo_memory_schema" => models::memory_input_schema(),
        "promemo_list_features" => serde_json::to_value(index::load_features(&repo)?)?,
        "promemo_tree" => serde_json::to_value(store::memory_tree(&repo)?)?,
        "promemo_doctor" => serde_json::to_value(store::doctor(&repo)?)?,
        "promemo_read_feature_file" => {
            let args: ReadFeatureFileArgs = serde_json::from_value(arguments)?;
            serde_json::to_value(read_feature_file(&repo, &args.feature, &args.file)?)?
        }
        "promemo_snapshot" => {
            let args: SnapshotArgs = serde_json::from_value(arguments)?;
            serde_json::to_value(git_snapshot::collect(
                &repo,
                &args.feature,
                args.dry_run.unwrap_or(true),
            )?)?
        }
        other => bail!("unknown Promemo MCP tool `{}`", other),
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result)?
        }]
    }))
}

fn extraction_result(
    memory: &models::MemoryInput,
    report: &models::SaveReport,
) -> anyhow::Result<Value> {
    Ok(json!({
        "memory": memory,
        "report": report
    }))
}

fn read_feature_file(repo: &Path, feature: &str, file: &str) -> anyhow::Result<Value> {
    if !allowed_feature_file(file) {
        bail!("unsupported Promemo feature file `{}`", file);
    }
    let path = store::feature_dir(repo, feature)?.join(file);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(json!({
        "feature": feature,
        "file": file,
        "path": store::relative_path(repo, &path)?,
        "text": text
    }))
}

fn allowed_feature_file(file: &str) -> bool {
    matches!(
        file,
        "memory.md"
            | "notes.md"
            | "context.md"
            | "architecture.md"
            | "decisions.md"
            | "api.md"
            | "data-model.md"
            | "todos.md"
            | "prompts.md"
    )
}

fn list_resources(cwd: &Path) -> anyhow::Result<Value> {
    let repo = store::find_repo_root(cwd)?;
    let mut resources = vec![
        json!({
            "uri": "promemo://project",
            "name": "Project Memory",
            "mimeType": "text/markdown"
        }),
        json!({
            "uri": "promemo://features",
            "name": "Feature Index",
            "mimeType": "application/json"
        }),
    ];
    for feature in index::load_features(&repo)? {
        resources.push(json!({
            "uri": format!("promemo://features/{}", feature.name),
            "name": feature.title,
            "description": feature.summary,
            "mimeType": "text/markdown"
        }));
    }
    Ok(json!({ "resources": resources }))
}

fn read_resource(cwd: &Path, params: Value) -> anyhow::Result<Value> {
    let args: ResourceArgs = serde_json::from_value(params)?;
    let repo = store::find_repo_root(cwd)?;
    let (mime_type, text) = if args.uri == "promemo://project" {
        (
            "text/markdown",
            std::fs::read_to_string(store::memory_root(&repo).join("project.md"))?,
        )
    } else if args.uri == "promemo://features" {
        (
            "application/json",
            serde_json::to_string_pretty(&index::load_features(&repo)?)?,
        )
    } else if let Some(feature) = args.uri.strip_prefix("promemo://features/") {
        ("text/markdown", store::load_context(&repo, feature)?.text)
    } else {
        bail!("unknown Promemo MCP resource `{}`", args.uri);
    };

    Ok(json!({
        "contents": [{
            "uri": args.uri,
            "mimeType": mime_type,
            "text": text
        }]
    }))
}

fn get_prompt(params: Value) -> anyhow::Result<Value> {
    let args: PromptArgs = serde_json::from_value(params)?;
    let (description, text) = match args.name.as_str() {
        "promemo_load_feature_context" => (
            "Load project memory before working on a feature.",
            "Use promemo_load_context with the target feature, then keep decisions and TODOs aligned with the loaded memory.",
        ),
        "promemo_save_distilled_memory" => (
            "Save durable memory after a work session.",
            "Distill the session into MemoryInput JSON. Do not store raw transcript text. Use promemo_save_memory with dry_run first, then save when the preview looks right.",
        ),
        other => bail!("unknown Promemo MCP prompt `{}`", other),
    };
    Ok(json!({
        "description": description,
        "messages": [{
            "role": "user",
            "content": {
                "type": "text",
                "text": text
            }
        }]
    }))
}

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "promemo_load_context",
            "description": "Load prompt-ready Promemo context for a feature.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug")
            }), vec!["feature"])
        }),
        json!({
            "name": "promemo_search",
            "description": "Search saved Promemo Markdown memory.",
            "inputSchema": object_schema(json!({
                "query": string_schema("Keyword search query")
            }), vec!["query"])
        }),
        json!({
            "name": "promemo_save_memory",
            "description": "Save structured MemoryInput JSON for a feature. Writes by default; set dry_run to true to preview first.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug"),
                "memory": models::memory_input_schema(),
                "dry_run": { "type": "boolean", "default": false }
            }), vec!["feature", "memory"])
        }),
        json!({
            "name": "promemo_preview_memory",
            "description": "Preview where structured MemoryInput JSON would be saved without writing files.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug"),
                "memory": models::memory_input_schema()
            }), vec!["feature", "memory"])
        }),
        json!({
            "name": "promemo_extract_memory",
            "description": "Extract structured memory from unstructured text with the configured provider. Returns the exact MemoryInput and a preview/save report, previewing by default.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug"),
                "source_text": string_schema("Unstructured notes, handoff text, or snapshot text to extract from"),
                "dry_run": { "type": "boolean", "default": true }
            }), vec!["feature", "source_text"])
        }),
        json!({
            "name": "promemo_memory_schema",
            "description": "Return the Promemo MemoryInput JSON schema.",
            "inputSchema": object_schema(json!({}), Vec::<&str>::new())
        }),
        json!({
            "name": "promemo_list_features",
            "description": "List Promemo features from the memory index.",
            "inputSchema": object_schema(json!({}), Vec::<&str>::new())
        }),
        json!({
            "name": "promemo_tree",
            "description": "List files in the Promemo memory tree.",
            "inputSchema": object_schema(json!({}), Vec::<&str>::new())
        }),
        json!({
            "name": "promemo_doctor",
            "description": "Run Promemo repository health checks.",
            "inputSchema": object_schema(json!({}), Vec::<&str>::new())
        }),
        json!({
            "name": "promemo_read_feature_file",
            "description": "Read a specific generated or notes Markdown file from a feature memory folder.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug"),
                "file": {
                    "type": "string",
                    "enum": [
                        "memory.md",
                        "notes.md",
                        "context.md",
                        "architecture.md",
                        "decisions.md",
                        "api.md",
                        "data-model.md",
                        "todos.md",
                        "prompts.md"
                    ]
                }
            }), vec!["feature", "file"])
        }),
        json!({
            "name": "promemo_snapshot",
            "description": "Collect git state, TODOs, and existing memory for a feature.",
            "inputSchema": object_schema(json!({
                "feature": string_schema("Feature name or slug"),
                "dry_run": { "type": "boolean", "default": true }
            }), vec!["feature"])
        }),
    ]
}

fn prompts() -> Vec<Value> {
    vec![
        json!({
            "name": "promemo_load_feature_context",
            "description": "Load Promemo context before feature work"
        }),
        json!({
            "name": "promemo_save_distilled_memory",
            "description": "Distill and save durable Promemo memory"
        }),
    ]
}

fn object_schema(properties: Value, required: Vec<&str>) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
        "required": required
    })
}

fn string_schema(description: &str) -> Value {
    json!({ "type": "string", "description": description, "minLength": 1 })
}

fn read_message(reader: &mut impl Read) -> anyhow::Result<Option<String>> {
    let mut header = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match reader.read(&mut byte)? {
            0 if header.is_empty() || header.iter().all(u8::is_ascii_whitespace) => {
                return Ok(None)
            }
            0 => bail!("incomplete MCP message header"),
            _ => {
                if header.is_empty() && byte[0].is_ascii_whitespace() {
                    continue;
                }
                header.push(byte[0]);
                if header.ends_with(b"\r\n\r\n") || header.ends_with(b"\n\n") {
                    break;
                }
            }
        }
    }

    let header = String::from_utf8(header).context("MCP header was not UTF-8")?;
    let content_length = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("Content-Length").then_some(value)
        })
        .map(str::trim)
        .context("MCP message missing Content-Length header")?
        .parse::<usize>()
        .context("invalid MCP Content-Length header")?;

    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(
        String::from_utf8(body).context("MCP body was not UTF-8")?,
    ))
}

fn write_message(writer: &mut impl Write, value: &Value) -> anyhow::Result<()> {
    let body = serde_json::to_string(value)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct JsonRpcMessage {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    name: String,
    arguments: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct FeatureArgs {
    feature: String,
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
}

#[derive(Debug, Deserialize)]
struct SaveMemoryArgs {
    feature: String,
    memory: models::MemoryInput,
    dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PreviewMemoryArgs {
    feature: String,
    memory: models::MemoryInput,
}

#[derive(Debug, Deserialize)]
struct ExtractMemoryArgs {
    feature: String,
    source_text: String,
    dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ReadFeatureFileArgs {
    feature: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotArgs {
    feature: String,
    dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ResourceArgs {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct PromptArgs {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_initialize_uses_content_length_framing() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })
        .to_string();
        let input = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
        let mut output = Vec::new();

        serve(Path::new("."), &mut input.as_bytes(), &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Content-Length:"));
        assert!(output.contains("\"serverInfo\""));
        assert!(output.contains("\"promemo\""));
    }

    #[test]
    fn mcp_reader_tolerates_blank_input_before_header() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })
        .to_string();
        let input = format!("\n\ncontent-length: {}\r\n\r\n{}", request.len(), request);
        let mut output = Vec::new();

        serve(Path::new("."), &mut input.as_bytes(), &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Content-Length:"));
        assert!(output.contains("\"serverInfo\""));
    }

    #[test]
    fn mcp_reader_treats_blank_input_as_clean_shutdown() {
        let input = "\n\n";
        let mut output = Vec::new();

        serve(Path::new("."), &mut input.as_bytes(), &mut output).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn extraction_result_includes_exact_memory_and_report() {
        let memory = models::MemoryInput {
            title: "Authentication".to_string(),
            summary: "JWT authentication memory.".to_string(),
            ..models::MemoryInput::default()
        };
        let report = models::SaveReport {
            feature: "authentication".to_string(),
            title: "Authentication".to_string(),
            dry_run: true,
            files_written: vec![".promemo/features/authentication/memory.md".to_string()],
            warnings: Vec::new(),
        };

        let result = extraction_result(&memory, &report).unwrap();

        assert_eq!(result["memory"]["title"], "Authentication");
        assert_eq!(result["report"]["dry_run"], true);
    }
}
