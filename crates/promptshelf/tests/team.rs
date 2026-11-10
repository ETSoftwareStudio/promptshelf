//! A team library on a bare remote: proposals, merges and Inbox updates.

use std::path::{Path, PathBuf};
use std::process::Command as Std;

use assert_cmd::Command;
use serde_json::Value;

struct Team {
    dir: tempfile::TempDir,
    git_config: PathBuf,
    /// The maintainer's clone, standing in for the host's merge button.
    host: PathBuf,
    /// Alice's clone: her team library.
    alice: PathBuf,
}

fn git(t: &Team, dir: &Path, args: &[&str]) -> String {
    let out = Std::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Maintainer",
            "-c",
            "user.email=m@example.com",
        ])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", &t.git_config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn setup() -> Team {
    let dir = tempfile::tempdir().unwrap();
    let git_config = dir.path().join("gitconfig");
    std::fs::write(&git_config, "[user]\n\tuseConfigOnly = true\n").unwrap();
    let t = Team {
        host: dir.path().join("host"),
        alice: dir.path().join("alice/team"),
        git_config,
        dir,
    };
    let root = t.dir.path().to_path_buf();
    git(
        &t,
        &root,
        &["init", "-q", "--bare", "-b", "main", "team.git"],
    );
    git(&t, &root, &["clone", "-q", "team.git", "host"]);
    std::fs::create_dir_all(t.host.join("templates")).unwrap();
    std::fs::write(t.host.join("templates/review.md"), "Review the team way.\n").unwrap();
    std::fs::write(t.host.join("templates/notes.md"), "Write notes.\n").unwrap();
    git(&t, &t.host, &["add", "-A"]);
    git(&t, &t.host, &["commit", "-q", "-m", "start"]);
    git(&t, &t.host, &["push", "-q", "-u", "origin", "main"]);
    std::fs::create_dir_all(root.join("alice")).unwrap();
    git(
        &t,
        &root.join("alice"),
        &[
            "clone",
            "-q",
            root.join("team.git").to_str().unwrap(),
            "team",
        ],
    );
    let personal = root.join("alice/personal");
    std::fs::create_dir_all(personal.join("templates")).unwrap();
    std::fs::create_dir_all(root.join("alice/config")).unwrap();
    std::fs::write(
        root.join("alice/config/config.toml"),
        format!(
            "[[library]]\nname = \"personal\"\npath = {:?}\n\n[[library]]\nname = \"team\"\npath = {:?}\nwrites = \"pull-request\"\n",
            personal.display().to_string(),
            t.alice.display().to_string()
        ),
    )
    .unwrap();
    t
}

impl Team {
    fn shelf(&self, args: &[&str]) -> (i32, Value) {
        let base = self.dir.path().join("alice");
        let out = Command::cargo_bin("shelf")
            .unwrap()
            .args(args)
            .arg("--json")
            .current_dir(&base)
            .env("PROMPTSHELF_CONFIG_DIR", base.join("config"))
            .env("PROMPTSHELF_STATE_DIR", base.join("state"))
            .env("PROMPTSHELF_AGENT", "0")
            .env("GIT_CONFIG_GLOBAL", &self.git_config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("HOME", &base)
            .output()
            .unwrap();
        let v = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        (out.status.code().unwrap(), v)
    }

    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.shelf(args);
        assert_eq!(code, 0, "shelf {args:?}: {v}");
        v
    }

    fn text(&self, id: &str, main: bool) -> String {
        let mut args = vec!["fill", id];
        if main {
            args.push("--main");
        }
        self.ok(&args)["text"].as_str().unwrap().to_string()
    }
}

#[test]
fn propose_merge_and_clean_up() {
    let t = setup();
    // Edit: a worktree on its own branch; the main folder stays on main.
    let v = t.ok(&["edit", "team/review"]);
    let path = PathBuf::from(v["path"].as_str().unwrap());
    assert!(!path.starts_with(&t.alice));
    std::fs::write(&path, "Review Alice's way.\n").unwrap();
    assert_eq!(t.text("team/review", false), "Review Alice's way.\n");
    assert_eq!(t.text("team/review", true), "Review the team way.\n");

    let v = t.ok(&["propose", "team/review"]);
    assert_eq!(v["proposal"]["status"], "sent");
    let branch = v["proposal"]["branch"].as_str().unwrap().to_string();
    assert!(
        branch.starts_with("shelf/") && branch.ends_with("/review"),
        "{branch}"
    );

    // The maintainer merges it with a merge commit.
    git(&t, &t.host, &["fetch", "-q", "origin"]);
    git(
        &t,
        &t.host,
        &[
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "merge",
            &format!("origin/{branch}"),
        ],
    );
    git(&t, &t.host, &["push", "-q", "origin", "main"]);

    let v = t.ok(&["sync"]);
    assert_eq!(v["proposals"][0]["outcome"], "merged", "{v}");
    assert_eq!(t.ok(&["proposals"])["proposals"], serde_json::json!([]));
    assert_eq!(t.text("team/review", false), "Review Alice's way.\n");
    assert!(!path.exists());
}

#[test]
fn squash_merges_are_found_by_content() {
    let t = setup();
    let v = t.ok(&["edit", "notes"]);
    std::fs::write(v["path"].as_str().unwrap(), "Write short notes.\n").unwrap();
    let v = t.ok(&["propose", "notes"]);
    let branch = v["proposal"]["branch"].as_str().unwrap().to_string();
    git(&t, &t.host, &["fetch", "-q", "origin"]);
    git(
        &t,
        &t.host,
        &["merge", "-q", "--squash", &format!("origin/{branch}")],
    );
    git(&t, &t.host, &["commit", "-q", "-m", "squashed"]);
    git(&t, &t.host, &["push", "-q", "origin", "main"]);
    git(&t, &t.host, &["push", "-q", "origin", "--delete", &branch]);
    let v = t.ok(&["sync"]);
    assert_eq!(v["proposals"][0]["outcome"], "merged", "{v}");
}

#[test]
fn team_updates_go_to_the_inbox() {
    let t = setup();
    std::fs::write(
        t.host.join("templates/review.md"),
        "---\nvars:\n  diff: { from: \"git diff\" }\n---\nReview {{diff}}.\n",
    )
    .unwrap();
    git(&t, &t.host, &["commit", "-q", "-am", "add a command"]);
    git(&t, &t.host, &["push", "-q", "origin", "main"]);

    let v = t.ok(&["sync"]);
    assert_eq!(v["libraries"][1]["updates"], 1, "{v}");
    let inbox = t.ok(&["inbox"]);
    let u = &inbox["updates"][0];
    assert_eq!(u["id"], "review");
    assert_eq!(u["change"], "edited");
    assert_eq!(u["new_commands"], serde_json::json!(["git diff"]));

    // The new command stays off until Alice allows it.
    let (code, v) = t.shelf(&["fill", "team/review"]);
    assert_eq!(code, 3, "{v}");
    assert!(
        v["missing"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("not allowed")
    );

    let v = t.ok(&["keep-previous", "team/review"]);
    assert_eq!(v["id"], "personal/review");
    assert_eq!(t.text("personal/review", false), "Review the team way.\n");
    assert_eq!(t.ok(&["inbox", "--done"])["cleared"], 1);
}

#[test]
fn new_prompts_in_a_team_library_are_proposals() {
    let t = setup();
    let v = t.ok(&["new", "Release checklist", "--lib", "team"]);
    assert_eq!(v["id"], "team/release-checklist");
    assert!(!t.alice.join("templates/release-checklist.md").exists());
    let list = t.ok(&["proposals"]);
    assert_eq!(list["proposals"][0]["id"], "release-checklist");
    assert_eq!(list["proposals"][0]["status"], "not-sent");
    assert_eq!(
        t.ok(&["withdraw", "release-checklist"])["withdrawn"],
        "team/release-checklist"
    );
}
