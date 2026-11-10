//! Two machines share a Personal library through a bare git remote.

use std::path::{Path, PathBuf};
use std::process::Command as Std;

use assert_cmd::Command;
use serde_json::Value;

struct World {
    dir: tempfile::TempDir,
    remote: PathBuf,
    git_config: PathBuf,
}

struct Machine<'a> {
    world: &'a World,
    name: &'static str,
    lib: PathBuf,
}

fn git(world: &World, dir: &Path, args: &[&str]) -> String {
    let out = Std::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", &world.git_config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", world.dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn world() -> World {
    let dir = tempfile::tempdir().unwrap();
    let git_config = dir.path().join("gitconfig");
    // Like a CI runner: no name or email, and git must not guess one.
    std::fs::write(&git_config, "[user]\n\tuseConfigOnly = true\n").unwrap();
    let remote = dir.path().join("remote.git");
    let w = World {
        dir,
        remote,
        git_config,
    };
    git(
        &w,
        w.dir.path(),
        &["init", "-q", "--bare", "-b", "main", "remote.git"],
    );
    w
}

impl World {
    /// A machine with its own config and a clone of the remote as Personal.
    fn machine(&self, name: &'static str) -> Machine<'_> {
        let base = self.dir.path().join(name);
        let lib = base.join("personal");
        std::fs::create_dir_all(&base).unwrap();
        git(
            self,
            &base,
            &["clone", "-q", self.remote.to_str().unwrap(), "personal"],
        );
        std::fs::create_dir_all(base.join("config")).unwrap();
        std::fs::write(
            base.join("config/config.toml"),
            format!(
                "[[library]]\nname = \"personal\"\npath = {:?}\n",
                lib.display().to_string()
            ),
        )
        .unwrap();
        Machine {
            world: self,
            name,
            lib,
        }
    }
}

impl Machine<'_> {
    fn shelf(&self, agent: bool, args: &[&str]) -> (i32, Value) {
        let bin = assert_cmd::cargo::cargo_bin("shelf");
        let path = std::env::join_paths(
            std::iter::once(bin.parent().unwrap().to_path_buf())
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        let base = self.world.dir.path().join(self.name);
        let out = Command::new(&bin)
            .args(args)
            .arg("--json")
            .current_dir(&base)
            .env("PATH", path)
            .env("PROMPTSHELF_CONFIG_DIR", base.join("config"))
            .env("PROMPTSHELF_STATE_DIR", base.join("state"))
            .env("PROMPTSHELF_AGENT", if agent { "1" } else { "0" })
            .env("GIT_CONFIG_GLOBAL", &self.world.git_config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("HOME", &base)
            .output()
            .unwrap();
        let v = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        (out.status.code().unwrap(), v)
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.lib.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}

#[test]
fn push_pull_pins_and_conflicts() {
    let w = world();
    let a = w.machine("a");
    a.write("templates/review.md", "Review it.\n");
    let (code, v) = a.shelf(false, &["sync"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["libraries"][0]["committed"], "Update 2 files");
    assert!(v["libraries"][0]["pushed"].as_u64().unwrap() >= 1);

    // The second machine pulls the prompt.
    let b = w.machine("b");
    let (code, v) = b.shelf(false, &["show", "review"]);
    assert_eq!(code, 0, "{v}");

    // Each machine pins a different prompt; the merge driver keeps both.
    a.write("templates/notes.md", "Notes.\n");
    b.write("templates/plan.md", "Plan.\n");
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    assert_eq!(b.shelf(false, &["sync"]).0, 0);
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    assert_eq!(a.shelf(false, &["pin", "notes"]).0, 0);
    assert_eq!(b.shelf(false, &["pin", "plan"]).0, 0);
    let (code, v) = a.shelf(false, &["sync"]);
    assert_eq!(code, 0, "{v}");
    let (code, v) = b.shelf(false, &["sync"]);
    assert_eq!(code, 0, "{v}");
    assert!(v["libraries"][0]["conflicts"].is_null(), "{v}");
    // A rebase replays B's change on top of A's, so A's order comes first and
    // B's new pin follows.
    let (_, pins) = b.shelf(false, &["pins"]);
    assert_eq!(
        pins["pins"],
        serde_json::json!(["personal/notes", "personal/plan"])
    );

    // Both machines edit the same line: B keeps its version, nothing half-merged.
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    a.write("templates/review.md", "Review it from A.\n");
    b.write("templates/review.md", "Review it from B.\n");
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    let (code, v) = b.shelf(false, &["sync"]);
    assert_eq!(code, 1, "{v}");
    assert_eq!(
        v["libraries"][0]["conflicts"],
        serde_json::json!(["templates/review.md"])
    );
    assert_eq!(
        std::fs::read_to_string(b.lib.join("templates/review.md")).unwrap(),
        "Review it from B.\n"
    );
    assert!(!b.lib.join(".git/rebase-merge").exists());
    assert!(!b.lib.join(".git/rebase-apply").exists());
    let status = git(&w, &b.lib, &["status", "--porcelain"]);
    assert!(status.is_empty(), "{status}");
}

#[test]
fn agents_only_pull() {
    let w = world();
    let a = w.machine("a");
    a.write("templates/one.md", "One.\n");
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    let b = w.machine("b");
    b.write("templates/local.md", "Local.\n");
    a.write("templates/two.md", "Two.\n");
    assert_eq!(a.shelf(false, &["sync"]).0, 0);
    let (code, v) = b.shelf(true, &["sync"]);
    assert_eq!(code, 0, "{v}");
    assert!(v["libraries"][0]["committed"].is_null());
    assert_eq!(v["libraries"][0]["pushed"], 0);
    assert_eq!(v["libraries"][0]["pulled"], 1);
    assert_eq!(v["libraries"][0]["changes"], 1);
    let (code, _) = b.shelf(true, &["pin", "one"]);
    assert_eq!(code, 5);
}

#[test]
fn commands_need_trust_or_allow() {
    let w = world();
    let a = w.machine("a");
    let team = w.dir.path().join("team");
    std::fs::create_dir_all(team.join("templates")).unwrap();
    std::fs::write(
        team.join("templates/greet.md"),
        "---\nvars:\n  who: { from: \"echo world\" }\n---\nHello {{who}}.\n",
    )
    .unwrap();
    let config = a.lib.parent().unwrap().join("config/config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str(&format!(
        "\n[[library]]\nname = \"team\"\npath = {:?}\nwrites = \"pull-request\"\n",
        team.display().to_string()
    ));
    std::fs::write(&config, text).unwrap();

    let (code, v) = a.shelf(true, &["fill", "team/greet"]);
    assert_eq!(code, 3, "{v}");
    assert!(
        v["missing"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("shelf allow team/greet")
    );
    assert_eq!(a.shelf(true, &["allow", "team/greet"]).0, 5);
    assert_eq!(a.shelf(false, &["allow", "team/greet"]).0, 0);
    let (code, v) = a.shelf(true, &["fill", "team/greet"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["text"].as_str().unwrap().trim(), "Hello world.");
    assert_eq!(v["values"][0]["source"], "command");

    // A changed command needs a new allow.
    std::fs::write(
        team.join("templates/greet.md"),
        "---\nvars:\n  who: { from: \"echo moon\" }\n---\nHello {{who}}.\n",
    )
    .unwrap();
    assert_eq!(a.shelf(true, &["fill", "team/greet"]).0, 3);
    assert_eq!(a.shelf(true, &["fill", "team/greet", "--no-run"]).0, 3);
}
