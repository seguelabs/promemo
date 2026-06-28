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
