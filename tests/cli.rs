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
