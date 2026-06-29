use std::io::Write;
use std::process::{Command, Stdio};

fn promemo() -> Command {
    Command::new(env!("CARGO_BIN_EXE_promemo"))
}

#[test]
fn binary_saves_lists_loads_searches_and_outputs_tree_json() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let mut save = promemo()
        .args(["save-json", "authentication"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/memory.json"))
        .unwrap();
    let save_output = save.wait_with_output().unwrap();
    assert!(
        save_output.status.success(),
        "{}",
        String::from_utf8_lossy(&save_output.stderr)
    );

    let list = promemo()
        .arg("list")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        "authentication"
    );

    let load = promemo()
        .args(["load", "authentication"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let loaded = String::from_utf8_lossy(&load.stdout);
    assert!(loaded.contains("# Authentication"));
    assert!(loaded.contains("JWT access tokens"));

    let search = promemo()
        .args(["search", "passkeys"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&search.stdout).contains("Support passkeys"));

    let tree = promemo()
        .args(["tree", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let entries: Vec<String> = serde_json::from_slice(&tree.stdout).unwrap();
    assert!(entries.contains(&".promemo/features/authentication/context.md".to_string()));
}

#[test]
fn binary_save_json_prints_human_success_output() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());
    let mut save = promemo()
        .args(["save-json", "authentication"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/memory.json"))
        .unwrap();
    let output = save.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Parsed memory: Authentication"));
    assert!(text.contains("Saved feature: authentication"));
    assert!(text.contains("promemo open authentication"));
    assert!(text.contains("less .promemo/features/authentication/memory.md"));
}

#[test]
fn binary_schema_memory_input_prints_json_schema() {
    let output = promemo().args(["schema", "memory-input"]).output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(schema["title"], "Promemo MemoryInput");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]["todos"]["items"]["properties"]["priority"]["enum"][2],
        "high"
    );
}

#[test]
fn binary_memory_preview_reads_request_json_without_writing() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let request = serde_json::json!({
        "feature": "authentication",
        "memory": serde_json::from_slice::<serde_json::Value>(include_bytes!("../examples/memory.json")).unwrap()
    });

    let mut preview = promemo()
        .args(["memory", "preview"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    preview
        .stdin
        .as_mut()
        .unwrap()
        .write_all(serde_json::to_string(&request).unwrap().as_bytes())
        .unwrap();
    let output = preview.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["feature"], "authentication");
    assert_eq!(report["dry_run"], true);
    assert!(!dir
        .path()
        .join(".promemo/features/authentication/memory.md")
        .exists());
}

#[test]
fn binary_memory_save_reads_request_json_and_writes_memory() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let request = serde_json::json!({
        "feature": "authentication",
        "memory": serde_json::from_slice::<serde_json::Value>(include_bytes!("../examples/memory.json")).unwrap()
    });

    let mut save = promemo()
        .args(["memory", "save"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(serde_json::to_string(&request).unwrap().as_bytes())
        .unwrap();
    let output = save.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["feature"], "authentication");
    assert_eq!(report["dry_run"], false);
    assert!(dir
        .path()
        .join(".promemo/features/authentication/memory.md")
        .exists());
}

#[test]
fn binary_extract_reports_missing_provider_api_key_before_network_call() {
    let dir = tempfile::tempdir().unwrap();
    let notes = dir.path().join("notes.md");
    std::fs::write(&notes, "We shipped JWT auth. Need passkeys later.").unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    std::fs::write(
        dir.path().join(".promemo/config.toml"),
        r#"[project]
name = "Promemo Test"

[provider]
kind = "openai-compatible"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
api_key_env = "PROMEMO_TEST_MISSING_API_KEY"
timeout_seconds = 1
"#,
    )
    .unwrap();

    let output = promemo()
        .args([
            "extract",
            "authentication",
            "--from",
            notes.to_str().unwrap(),
            "--dry-run",
        ])
        .env_remove("PROMEMO_TEST_MISSING_API_KEY")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("provider API key env var `PROMEMO_TEST_MISSING_API_KEY` is not set"));
}

#[test]
fn binary_commands_work_from_repo_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("src/auth");
    std::fs::create_dir_all(&nested).unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let mut save = promemo()
        .args(["save-json", "authentication"])
        .current_dir(&nested)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/memory.json"))
        .unwrap();
    let save_output = save.wait_with_output().unwrap();
    assert!(
        save_output.status.success(),
        "{}",
        String::from_utf8_lossy(&save_output.stderr)
    );

    assert!(dir
        .path()
        .join(".promemo/features/authentication/context.md")
        .exists());

    let load = promemo()
        .args(["load", "authentication"])
        .current_dir(&nested)
        .output()
        .unwrap();
    assert!(load.status.success());
    assert!(String::from_utf8_lossy(&load.stdout).contains("# Authentication"));
}

#[test]
fn binary_save_from_handoff_markdown_writes_memory_and_json_report() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args([
            "save",
            "product-direction",
            "--from",
            handoff.to_str().unwrap(),
            "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["feature"], "product-direction");
    assert_eq!(report["title"], "Product Direction");
    assert!(report["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == ".promemo/features/product-direction/context.md"));

    let loaded = promemo()
        .args(["load", "product-direction"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&loaded.stdout);
    assert!(text.contains("Use structured handoff Markdown before MCP"));
    assert!(text.contains("Add assistant usage docs"));
}

#[test]
fn binary_save_stdin_accepts_handoff_markdown() {
    let dir = tempfile::tempdir().unwrap();
    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promemo()
        .args(["save", "product-direction", "--stdin"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/handoff.md"))
        .unwrap();
    let output = save.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir
        .path()
        .join(".promemo/features/product-direction/decisions.md")
        .exists());
}

#[test]
fn binary_import_handoff_alias_writes_memory_and_json_report() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args([
            "import",
            "handoff",
            handoff.to_str().unwrap(),
            "--feature",
            "product-direction",
            "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["feature"], "product-direction");
    assert!(dir
        .path()
        .join(".promemo/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_save_handoff_dry_run_previews_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args([
            "save",
            "product-direction",
            "--from",
            handoff.to_str().unwrap(),
            "--dry-run",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Parsed handoff: Product Direction"));
    assert!(text.contains("Dry run: no files were written"));
    assert!(text.contains("Would save feature: product-direction"));
    assert!(!dir
        .path()
        .join(".promemo/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_import_handoff_dry_run_json_reports_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args([
            "import",
            "handoff",
            handoff.to_str().unwrap(),
            "--feature",
            "product-direction",
            "--dry-run",
            "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["feature"], "product-direction");
    assert_eq!(report["dry_run"], true);
    assert!(!dir
        .path()
        .join(".promemo/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_search_reports_no_matches_in_human_mode() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args(["search", "nothing-here"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "No memory found for \"nothing-here\"."
    );
}

#[test]
fn binary_open_missing_feature_reports_clear_error() {
    let dir = tempfile::tempdir().unwrap();

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args(["open", "missing"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("feature `missing` does not exist"));
}

#[test]
fn binary_snapshot_human_output_includes_handoff_context() {
    let dir = tempfile::tempdir().unwrap();
    let init_git = Command::new("git")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init_git.status.success());

    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promemo()
        .args(["save-json", "authentication"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/memory.json"))
        .unwrap();
    let save_output = save.wait_with_output().unwrap();
    assert!(save_output.status.success());

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/main.rs"),
        "fn main() {}\n// TODO: wire auth\n",
    )
    .unwrap();

    let output = promemo()
        .args(["snapshot", "authentication", "--dry-run"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("## Changed Files"));
    assert!(text.contains("src/main.rs"));
    assert!(text.contains("## Recent Commits") || text.contains("git log --oneline -5 failed"));
    assert!(text.contains("## TODO/FIXME Comments"));
    assert!(text.contains("wire auth"));
    assert!(text.contains("## Existing Feature Memory"));
    assert!(text.contains(".promemo/features/authentication/memory.md"));
}

#[test]
fn binary_reports_missing_feature() {
    let dir = tempfile::tempdir().unwrap();
    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args(["load", "missing"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("feature `missing` does not exist"));
}

#[test]
fn binary_reports_malformed_json_context() {
    let dir = tempfile::tempdir().unwrap();
    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promemo()
        .args(["save-json", "bad"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin.as_mut().unwrap().write_all(b"{ nope").unwrap();
    let output = save.wait_with_output().unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to read memory JSON from stdin")
    );
}

#[test]
fn binary_json_mode_errors_use_stable_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promemo()
        .args(["load", "missing", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["error"]["code"], "promemo_error");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("feature `missing` does not exist"));
}

#[test]
fn binary_mcp_lists_and_calls_tools() {
    let dir = tempfile::tempdir().unwrap();
    let init = promemo()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());
    std::fs::write(
        dir.path().join(".promemo/config.toml"),
        r#"[project]
name = "Promemo MCP Test"

[provider]
kind = "openai-compatible"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
api_key_env = "PROMEMO_TEST_MCP_MISSING_API_KEY"
timeout_seconds = 1
"#,
    )
    .unwrap();

    let mut save = promemo()
        .args(["save-json", "authentication"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    save.stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!("../examples/memory.json"))
        .unwrap();
    let save_output = save.wait_with_output().unwrap();
    assert!(save_output.status.success());

    let memory: serde_json::Value =
        serde_json::from_slice(include_bytes!("../examples/memory.json")).unwrap();
    let messages = [
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "promemo_search",
                "arguments": { "query": "passkeys" }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "promemo_load_context",
                "arguments": { "feature": "authentication" }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "promemo_save_memory",
                "arguments": {
                    "feature": "mcp-auth",
                    "memory": memory.clone()
                }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "promemo_list_features",
                "arguments": {}
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "promemo_snapshot",
                "arguments": { "feature": "authentication" }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "promemo_preview_memory",
                "arguments": {
                    "feature": "preview-auth",
                    "memory": memory
                }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call",
            "params": {
                "name": "promemo_memory_schema",
                "arguments": {}
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "promemo_tree",
                "arguments": {}
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "promemo_doctor",
                "arguments": {}
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 12,
            "method": "tools/call",
            "params": {
                "name": "promemo_read_feature_file",
                "arguments": {
                    "feature": "authentication",
                    "file": "memory.md"
                }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 13,
            "method": "tools/call",
            "params": {
                "name": "promemo_extract_memory",
                "arguments": {
                    "feature": "extracted-auth",
                    "source_text": "We shipped JWT auth and need passkeys later."
                }
            }
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 14,
            "method": "resources/list",
            "params": {}
        })),
        mcp_frame(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 15,
            "method": "prompts/list",
            "params": {}
        })),
    ]
    .join("");

    let mut mcp = promemo()
        .arg("mcp")
        .current_dir(dir.path())
        .env_remove("PROMEMO_TEST_MCP_MISSING_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    mcp.stdin
        .as_mut()
        .unwrap()
        .write_all(messages.as_bytes())
        .unwrap();
    drop(mcp.stdin.take());
    let output = mcp.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses = parse_mcp_frames(&output.stdout);
    assert_eq!(responses.len(), 15);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "promemo");
    assert!(responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "promemo_save_memory"));
    assert!(responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "promemo_extract_memory"));
    assert!(responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "promemo_read_feature_file"));
    assert!(responses[2]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Support passkeys"));
    assert!(responses[3]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("JWT access tokens"));
    assert!(responses[4]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"feature\": \"mcp-auth\""));
    assert!(responses[5]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"name\": \"mcp-auth\""));
    assert!(responses[6]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"feature\": \"authentication\""));
    assert!(responses[7]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"dry_run\": true"));
    assert!(responses[8]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"title\": \"Promemo MemoryInput\""));
    assert!(responses[9]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains(".promemo/features/authentication/memory.md"));
    assert!(responses[10]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"message\": \"config.toml exists\""));
    assert!(responses[11]["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"file\": \"memory.md\""));
    assert!(responses[12]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("PROMEMO_TEST_MCP_MISSING_API_KEY"));
    assert!(responses[13]["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| resource["uri"] == "promemo://features/authentication"));
    assert!(responses[14]["result"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|prompt| prompt["name"] == "promemo_save_distilled_memory"));
    assert!(dir
        .path()
        .join(".promemo/features/mcp-auth/memory.md")
        .exists());
    assert!(!dir
        .path()
        .join(".promemo/features/preview-auth/memory.md")
        .exists());
}

fn mcp_frame(value: serde_json::Value) -> String {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn parse_mcp_frames(output: &[u8]) -> Vec<serde_json::Value> {
    let text = String::from_utf8(output.to_vec()).unwrap();
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
