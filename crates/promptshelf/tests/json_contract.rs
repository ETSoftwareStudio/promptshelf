//! Snapshots of the `--json` output. These outputs are a contract with agents:
//! a snapshot change is a breaking change unless it only adds fields.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serde_json::Value;

fn run(args: &[&str]) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let examples: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    std::fs::write(
        dir.path().join("config.toml"),
        format!(
            "[[library]]\nname = \"examples\"\npath = {:?}\n",
            examples.display().to_string()
        ),
    )
    .unwrap();
    let out = Command::cargo_bin("shelf")
        .unwrap()
        .env("PROMPTSHELF_CONFIG_DIR", dir.path())
        .env("PROMPTSHELF_STATE_DIR", dir.path().join("state"))
        .env("PROMPTSHELF_AGENT", "1")
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn list() {
    insta::assert_json_snapshot!(run(&["list"]), {
        ".prompts[].path" => "[path]",
    });
}

#[test]
fn search() {
    insta::assert_json_snapshot!(run(&["search", "explain"]), {
        ".prompts[].path" => "[path]",
    });
}

#[test]
fn show() {
    insta::assert_json_snapshot!(run(&["show", "strict-pr-review"]), {
        ".prompt.path" => "[path]",
    });
}

#[test]
fn fill_open() {
    insta::assert_json_snapshot!(run(&["fill", "explain-like-a-colleague"]));
}

#[test]
fn fill_complete() {
    insta::assert_json_snapshot!(run(&[
        "fill",
        "explain-like-a-colleague",
        "--var",
        "topic=the sync loop"
    ]));
}

#[test]
fn not_found() {
    insta::assert_json_snapshot!(run(&["show", "nope"]));
}
