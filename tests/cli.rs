use std::io::Write;
use std::process::{Command, Stdio};

fn promem() -> Command {
    Command::new(env!("CARGO_BIN_EXE_promem"))
}

#[test]
fn binary_saves_lists_loads_searches_and_outputs_tree_json() {
    let dir = tempfile::tempdir().unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let mut save = promem()
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

    let list = promem()
        .arg("list")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        "authentication"
    );

    let load = promem()
        .args(["load", "authentication"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let loaded = String::from_utf8_lossy(&load.stdout);
    assert!(loaded.contains("# Authentication"));
    assert!(loaded.contains("JWT access tokens"));

    let search = promem()
        .args(["search", "passkeys"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&search.stdout).contains("Support passkeys"));

    let tree = promem()
        .args(["tree", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let entries: Vec<String> = serde_json::from_slice(&tree.stdout).unwrap();
    assert!(entries.contains(&".promem/features/authentication/context.md".to_string()));
}

#[test]
fn binary_save_json_prints_human_success_output() {
    let dir = tempfile::tempdir().unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promem()
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
    assert!(text.contains("promem open authentication"));
    assert!(text.contains("less .promem/features/authentication/memory.md"));
}

#[test]
fn binary_commands_work_from_repo_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("src/auth");
    std::fs::create_dir_all(&nested).unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let mut save = promem()
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
        .join(".promem/features/authentication/context.md")
        .exists());

    let load = promem()
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

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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
        .any(|path| path == ".promem/features/product-direction/context.md"));

    let loaded = promem()
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
    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promem()
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
        .join(".promem/features/product-direction/decisions.md")
        .exists());
}

#[test]
fn binary_import_handoff_alias_writes_memory_and_json_report() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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
        .join(".promem/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_save_handoff_dry_run_previews_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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
        .join(".promem/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_import_handoff_dry_run_json_reports_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let handoff = dir.path().join("handoff.md");
    std::fs::write(&handoff, include_str!("../examples/handoff.md")).unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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
        .join(".promem/features/product-direction/memory.md")
        .exists());
}

#[test]
fn binary_search_reports_no_matches_in_human_mode() {
    let dir = tempfile::tempdir().unwrap();

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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

    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promem()
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

    let output = promem()
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
    assert!(text.contains(".promem/features/authentication/memory.md"));
}

#[test]
fn binary_reports_missing_feature() {
    let dir = tempfile::tempdir().unwrap();
    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let output = promem()
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
    let init = promem()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());

    let mut save = promem()
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
