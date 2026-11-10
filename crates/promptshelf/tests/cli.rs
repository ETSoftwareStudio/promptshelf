//! End-to-end tests of the `shelf` command against the example library.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serde_json::Value;

struct Env {
    _dir: tempfile::TempDir,
    config: PathBuf,
    state: PathBuf,
    personal: PathBuf,
    home: PathBuf,
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A config with Personal (empty) and `examples` (a copy of examples/library).
fn setup() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let state = dir.path().join("state");
    let personal = dir.path().join("personal");
    let examples_copy = dir.path().join("examples");
    let home = dir.path().join("home");
    copy_dir(&examples(), &examples_copy);
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("config.toml"),
        format!(
            "[[library]]\nname = \"personal\"\npath = {:?}\n\n[[library]]\nname = \"examples\"\npath = {:?}\n",
            personal.display().to_string(),
            examples_copy.display().to_string()
        ),
    )
    .unwrap();
    for f in ["templates", "snippets", "context", "inbox"] {
        std::fs::create_dir_all(personal.join(f)).unwrap();
    }
    Env {
        _dir: dir,
        config,
        state,
        personal,
        home,
    }
}

impl Env {
    fn cmd(&self, agent: bool) -> Command {
        let mut cmd = Command::cargo_bin("shelf").unwrap();
        cmd.env("PROMPTSHELF_CONFIG_DIR", &self.config)
            .env("PROMPTSHELF_STATE_DIR", &self.state)
            .env("PROMPTSHELF_AGENT", if agent { "1" } else { "0" })
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env_remove("XDG_CONFIG_HOME");
        cmd
    }

    fn json(&self, args: &[&str], code: i32) -> Value {
        let out = self.cmd(true).args(args).arg("--json").output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(code),
            "args {args:?}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["schema"], 1);
        v
    }
}

fn ids(v: &Value) -> Vec<String> {
    v["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn list_and_filters() {
    let env = setup();
    let v = env.json(&["list"], 0);
    assert_eq!(
        ids(&v),
        [
            "examples/explain-like-a-colleague",
            "examples/strict-pr-review",
            "examples/house-rules",
            "examples/about-me",
            "examples/rough-idea",
        ]
    );
    assert_eq!(v["diagnostics"].as_array().unwrap().len(), 0);
    let review = env.json(&["list", "--tag", "review"], 0);
    assert_eq!(
        ids(&review),
        ["examples/strict-pr-review", "examples/house-rules"]
    );
    let drafts = env.json(&["list", "--kind", "draft"], 0);
    assert_eq!(ids(&drafts), ["examples/rough-idea"]);
    let blanks = &v["prompts"][1]["blanks"];
    assert_eq!(blanks, &serde_json::json!(["focus", "findings", "diff"]));
}

#[test]
fn search_ranks() {
    let env = setup();
    let v = env.json(&["search", "review"], 0);
    assert_eq!(ids(&v)[0], "examples/strict-pr-review");
    let none = env.json(&["search", "nothing-like-this"], 0);
    assert!(ids(&none).is_empty());
}

#[test]
fn show_includes_blanks() {
    let env = setup();
    let v = env.json(&["show", "strict-pr-review"], 0);
    let p = &v["prompt"];
    assert_eq!(p["id"], "examples/strict-pr-review");
    assert_eq!(p["kind"], "template");
    assert_eq!(p["blanks"][0]["name"], "focus");
    assert_eq!(p["blanks"][0]["from"], "conversation");
    assert_eq!(p["blanks"][1]["choices"][1], "Everything");
}

#[test]
fn fill_open_blanks_exit_3() {
    let env = setup();
    let v = env.json(&["fill", "strict-pr-review"], 3);
    assert_eq!(v["complete"], false);
    let missing: Vec<&str> = v["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(missing, ["focus", "diff"]);
    assert_eq!(v["missing"][0]["hint"], "what the user is worried about");
    assert!(v["instruction"].as_str().unwrap().contains("conversation"));
    assert_eq!(v["values"][0]["name"], "findings");
    assert_eq!(v["values"][0]["source"], "default");
    assert!(v["text"].as_str().unwrap().contains("{{focus}}"));
}

#[test]
fn fill_complete() {
    let env = setup();
    let v = env.json(
        &[
            "fill",
            "examples/strict-pr-review",
            "--var",
            "focus=error handling",
            "--var",
            "diff=+ a line",
            "--var",
            "findings=Everything",
        ],
        0,
    );
    assert_eq!(v["complete"], true);
    assert!(v.get("instruction").is_none());
    let text = v["text"].as_str().unwrap();
    assert!(text.starts_with("Review this diff like a strict senior reviewer. Focus on error handling.\nReport Everything"));
    assert!(text.contains("Flag missing tests."));
    assert!(text.ends_with("The diff:\n+ a line\n"));
}

#[test]
fn fill_errors() {
    let env = setup();
    let bad_choice = env.json(&["fill", "strict-pr-review", "--var", "findings=Some"], 2);
    assert_eq!(bad_choice["error"]["code"], "invalid");
    let bad_var = env.json(&["fill", "strict-pr-review", "--var", "nope"], 2);
    assert_eq!(bad_var["error"]["code"], "invalid");
    let missing = env.json(&["fill", "no-such-prompt"], 4);
    assert_eq!(missing["error"]["code"], "not_found");
    let usage = env.json(&["fill"], 2);
    assert_eq!(usage["error"]["code"], "usage");
}

#[test]
fn ambiguous_ids_list_matches() {
    let env = setup();
    let other = env.personal.parent().unwrap().join("other");
    std::fs::create_dir_all(other.join("templates")).unwrap();
    std::fs::write(other.join("templates/about-me.md"), "Other.\n").unwrap();
    let mut config = std::fs::read_to_string(env.config.join("config.toml")).unwrap();
    config.push_str(&format!(
        "\n[[library]]\nname = \"other\"\npath = {:?}\n",
        other.display().to_string()
    ));
    std::fs::write(env.config.join("config.toml"), config).unwrap();
    let v = env.json(&["show", "about-me"], 4);
    assert_eq!(v["error"]["code"], "ambiguous");
    assert_eq!(
        v["error"]["matches"],
        serde_json::json!(["examples/about-me", "other/about-me"])
    );
}

#[test]
fn agents_save_drafts_only() {
    let env = setup();
    let v = env
        .cmd(true)
        .args(["save", "--title", "Meeting notes to actions", "--json"])
        .write_stdin("Turn notes into actions.\n")
        .output()
        .unwrap();
    assert_eq!(v.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&v.stdout).unwrap();
    assert_eq!(v["id"], "personal/meeting-notes-to-actions");
    assert_eq!(v["kind"], "draft");
    assert!(
        env.personal
            .join("inbox/meeting-notes-to-actions.md")
            .exists()
    );

    let refused = env
        .cmd(true)
        .args(["save", "--title", "X", "--lib", "personal", "--json"])
        .write_stdin("x")
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(5));
    let refused: Value = serde_json::from_slice(&refused.stdout).unwrap();
    assert_eq!(refused["error"]["code"], "refused");

    let empty = env
        .cmd(true)
        .args(["save", "--title", "Empty", "--json"])
        .write_stdin("  \n")
        .output()
        .unwrap();
    assert_eq!(empty.status.code(), Some(2));
}

#[test]
fn people_save_into_a_library() {
    let env = setup();
    let out = env
        .cmd(false)
        .args(["save", "--title", "Commit message", "--lib", "personal"])
        .write_stdin("Write a commit message.\n")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(env.personal.join("templates/commit-message.md").exists());
}

#[test]
fn agent_writes_need_permission() {
    let env = setup();
    let v = env.json(&["new", "Something"], 5);
    assert_eq!(v["error"]["code"], "refused");
    let v = env.json(&["skill", "install"], 5);
    assert_eq!(v["error"]["code"], "refused");
    // --user-action lets a UI client write for a person.
    let out = env
        .cmd(true)
        .args(["skill", "install", "--user-action", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(
        env.home
            .join(".claude/skills/promptshelf/SKILL.md")
            .exists()
    );
}

#[test]
fn init_creates_config_and_library() {
    let dir = tempfile::tempdir().unwrap();
    let lib = dir.path().join("Prompts/personal");
    let out = Command::cargo_bin("shelf")
        .unwrap()
        .env("PROMPTSHELF_CONFIG_DIR", dir.path().join("cfg"))
        .env("PROMPTSHELF_AGENT", "0")
        .args(["init", "--path", lib.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["created"], true);
    assert!(lib.join("templates").is_dir());
    assert!(lib.join("inbox").is_dir());
    assert!(dir.path().join("cfg/config.toml").exists());
}

#[test]
fn no_config_is_a_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("shelf")
        .unwrap()
        .env("PROMPTSHELF_CONFIG_DIR", dir.path())
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "no_config");
}

#[test]
fn check_reports_bad_files() {
    let env = setup();
    std::fs::write(
        env.personal.join("templates/bad.md"),
        "---\ntags: [x\n---\n",
    )
    .unwrap();
    let v = env.json(&["check"], 1);
    assert_eq!(v["ok"], false);
    assert!(
        v["diagnostics"][0]["path"]
            .as_str()
            .unwrap()
            .ends_with("bad.md")
    );
}

#[test]
fn skill_print_has_frontmatter() {
    let env = setup();
    let out = env.cmd(true).args(["skill", "print"]).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("---\nname: promptshelf\n"));
}
