use super::*;
use crate::config::LibraryConfig;
use crate::shelf::Library;

const OPTS: Options = Options {
    commit: true,
    push: true,
    merge_driver: "shelf",
};

fn git(dir: &Path, args: &[&str]) -> String {
    let mut all = vec!["-c", "user.name=T", "-c", "user.email=t@example.com"];
    all.extend_from_slice(args);
    git::run(dir, &all).unwrap()
}

fn library(root: &Path) -> Library {
    Library::scan(
        LibraryConfig {
            name: "personal".into(),
            path: root.display().to_string(),
            remote: None,
            writes: Writes::Commit,
            personal: false,
            trusted: None,
            forge: None,
            api: None,
        },
        &mut Vec::new(),
    )
}

/// Two clones of one remote. Each edits the same line of `templates/a.md`,
/// and "other" pushes first, so a sync of "here" conflicts.
fn conflict() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path();
    git::run(base, &["init", "-q", "--bare", "-b", "main", "r.git"]).unwrap();
    git::run(base, &["clone", "-q", "r.git", "here"]).unwrap();
    let here = base.join("here");
    std::fs::create_dir_all(here.join("templates")).unwrap();
    std::fs::write(here.join("templates/a.md"), "---\nid: a\n---\nOne.\n").unwrap();
    git(&here, &["add", "-A"]);
    git(&here, &["commit", "-q", "-m", "a"]);
    git(&here, &["push", "-q", "-u", "origin", "main"]);
    git::run(base, &["clone", "-q", "r.git", "other"]).unwrap();
    let other = base.join("other");
    std::fs::write(other.join("templates/a.md"), "---\nid: a\n---\nTheirs.\n").unwrap();
    git(&other, &["commit", "-q", "-am", "theirs"]);
    git(&other, &["push", "-q"]);
    std::fs::write(here.join("templates/a.md"), "---\nid: a\n---\nMine.\n").unwrap();
    let report = sync(&library(&here), false, &OPTS);
    assert_eq!(report.conflicts, ["templates/a.md"]);
    (dir, here, other)
}

/// Reads a file with LF line endings: git on Windows checks files out with CRLF.
fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel))
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn resolve_with_mine_pushes_mine() {
    let (_dir, here, other) = conflict();
    let report = resolve(&library(&here), false, Resolution::Mine, &OPTS);
    assert_eq!(report.error, None, "{report:?}");
    assert!(report.conflicts.is_empty());
    assert_eq!(read(&here, "templates/a.md"), "---\nid: a\n---\nMine.\n");
    git(&other, &["pull", "-q"]);
    assert_eq!(read(&other, "templates/a.md"), "---\nid: a\n---\nMine.\n");
}

#[test]
fn resolve_with_theirs_keeps_the_remote() {
    let (_dir, here, _other) = conflict();
    let report = resolve(&library(&here), false, Resolution::Theirs, &OPTS);
    assert_eq!(report.error, None, "{report:?}");
    assert_eq!(read(&here, "templates/a.md"), "---\nid: a\n---\nTheirs.\n");
    assert_eq!(git::ahead_behind(&here), Some((0, 0)));
}

#[test]
fn resolve_with_both_saves_a_copy() {
    let (_dir, here, other) = conflict();
    let report = resolve(&library(&here), false, Resolution::Both, &OPTS);
    assert_eq!(report.error, None, "{report:?}");
    assert_eq!(read(&here, "templates/a.md"), "---\nid: a\n---\nTheirs.\n");
    assert_eq!(
        read(&here, "templates/a-2.md"),
        "---\nid: a-2\n---\nMine.\n"
    );
    git(&other, &["pull", "-q"]);
    assert!(other.join("templates/a-2.md").exists());
}

#[test]
fn resolve_without_a_conflict_just_syncs() {
    let (_dir, here, _other) = conflict();
    resolve(&library(&here), false, Resolution::Theirs, &OPTS);
    let report = resolve(&library(&here), false, Resolution::Both, &OPTS);
    assert_eq!(report.error, None);
    assert!(!here.join("templates/a-2.md").exists());
}
