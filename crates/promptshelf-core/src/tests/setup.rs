use std::path::Path;

use super::*;
use crate::shelf::Library;
use crate::sync::{self, Options};

fn git(dir: &Path, args: &[&str]) -> String {
    let mut all = vec!["-c", "user.name=Test", "-c", "user.email=t@example.com"];
    all.extend_from_slice(args);
    git::run(dir, &all).unwrap()
}

fn bare(dir: &Path, name: &str) -> PathBuf {
    git::run(dir, &["init", "-q", "--bare", "-b", "main", name]).unwrap();
    dir.join(name)
}

fn personal_config(dir: &Path) -> Config {
    let mut config = Config::default();
    let path = dir.join("personal").display().to_string();
    init_personal(&mut config, &path, "shelf").unwrap();
    config
}

const OPTS: Options = Options {
    commit: true,
    push: true,
    merge_driver: "shelf",
};

#[test]
fn init_creates_a_git_library_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    let path = dir.path().join("p").display().to_string();
    let init = init_personal(&mut config, &path, "shelf").unwrap();
    assert!(init.created);
    assert_eq!(init.name, "personal");
    assert!(init.root.join("templates").is_dir());
    assert!(git::is_repo(&init.root));
    assert!(config.personal().unwrap().personal);

    let again = init_personal(&mut config, "/somewhere/else", "shelf").unwrap();
    assert!(!again.created);
    assert_eq!(again.root, init.root);
    assert_eq!(config.libraries.len(), 1);
}

#[test]
fn init_keeps_an_existing_library() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("mine");
    std::fs::create_dir_all(root.join("templates")).unwrap();
    std::fs::write(root.join("templates/a.md"), "Hello\n").unwrap();
    let mut config = Config::default();
    init_personal(&mut config, &root.display().to_string(), "shelf").unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("templates/a.md")).unwrap(),
        "Hello\n"
    );
}

#[test]
fn git_urls_and_folders() {
    assert!(looks_like_git_url("git@github.com:acme/prompts.git"));
    assert!(looks_like_git_url("https://example.com/acme/prompts"));
    assert!(looks_like_git_url("/srv/team.git"));
    assert!(!looks_like_git_url("~/Prompts/acme"));
}

#[test]
fn add_a_folder_and_a_clone() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = personal_config(dir.path());
    let folder = dir.path().join("notes");
    std::fs::create_dir_all(&folder).unwrap();
    let lib = add_library(
        &mut config,
        &AddLibrary {
            name: "notes",
            source: &folder.display().to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(lib.writes, Writes::Commit);
    assert!(Path::new(&lib.path).is_absolute());

    let remote = bare(dir.path(), "team.git");
    let dest = dir.path().join("clones/team").display().to_string();
    let lib = add_library(
        &mut config,
        &AddLibrary {
            name: "team",
            source: &remote.display().to_string(),
            path: Some(&dest),
            trusted: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(lib.writes, Writes::PullRequest);
    assert_eq!(lib.trusted, Some(false));
    assert!(git::is_repo(Path::new(&dest)));
    assert_eq!(config.libraries.len(), 3);
}

#[test]
fn add_refuses_bad_names_and_missing_sources() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = personal_config(dir.path());
    let add = |config: &mut Config, name: &str, source: &str| {
        add_library(
            config,
            &AddLibrary {
                name,
                source,
                ..Default::default()
            },
        )
    };
    let folder = dir.path().display().to_string();
    assert!(add(&mut config, "Bad Name", &folder).is_err());
    assert!(add(&mut config, "personal", &folder).is_err());
    assert!(add(&mut config, "gone", "/no/such/folder").is_err());
    let err = add(&mut config, "nope", "/no/such/repo.git").unwrap_err();
    assert!(err.to_string().contains("git clone"), "{err}");
    assert_eq!(config.libraries.len(), 1);
}

#[test]
fn update_and_remove() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = personal_config(dir.path());
    let folder = dir.path().join("notes");
    std::fs::create_dir_all(&folder).unwrap();
    let source = folder.display().to_string();
    add_library(
        &mut config,
        &AddLibrary {
            name: "notes",
            source: &source,
            ..Default::default()
        },
    )
    .unwrap();
    update_library(&mut config, "notes", Some(Writes::PullRequest), Some(true)).unwrap();
    let lib = config.library("notes").unwrap();
    assert_eq!(lib.writes, Writes::PullRequest);
    assert!(lib.is_trusted());
    assert!(update_library(&mut config, "nope", None, None).is_err());

    assert!(remove_library(&mut config, "personal").is_err());
    remove_library(&mut config, "notes").unwrap();
    assert!(config.library("notes").is_err());
    assert!(folder.is_dir(), "the files stay");
}

#[test]
fn connect_then_sync_pushes() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = personal_config(dir.path());
    let root = config.personal().unwrap().root();
    std::fs::write(root.join("templates/a.md"), "A\n").unwrap();
    let remote = bare(dir.path(), "mine.git");
    connect_remote(&mut config, "personal", &remote.display().to_string()).unwrap();
    assert_eq!(
        config.personal().unwrap().remote.as_deref(),
        Some(remote.display().to_string().as_str())
    );
    let lib = Library::scan(config.personal().unwrap().clone(), &mut Vec::new());
    let report = sync::sync(&lib, true, &OPTS);
    assert_eq!(report.error, None);
    assert!(report.pushed > 0);
    assert!(
        git::run(&remote, &["log", "--oneline"])
            .unwrap()
            .lines()
            .count()
            > 0
    );
    assert!(connect_remote(&mut config, "personal", " ").is_err());
}

#[test]
fn connect_to_a_remote_with_content_pulls_first() {
    let dir = tempfile::tempdir().unwrap();
    let remote = bare(dir.path(), "mine.git");
    let other = dir.path().join("other");
    git::run(dir.path(), &["clone", "-q", "mine.git", "other"]).unwrap();
    std::fs::create_dir_all(other.join("templates")).unwrap();
    std::fs::write(other.join("templates/b.md"), "B\n").unwrap();
    git(&other, &["add", "-A"]);
    git(&other, &["commit", "-q", "-m", "B"]);
    git(&other, &["push", "-q", "origin", "main"]);

    let mut config = personal_config(dir.path());
    let root = config.personal().unwrap().root();
    std::fs::write(root.join("templates/a.md"), "A\n").unwrap();
    connect_remote(&mut config, "personal", &remote.display().to_string()).unwrap();
    let lib = Library::scan(config.personal().unwrap().clone(), &mut Vec::new());
    let report = sync::sync(&lib, true, &OPTS);
    assert_eq!(report.error, None, "{report:?}");
    assert!(
        root.join("templates/b.md").exists(),
        "pulled the remote's file"
    );
    git(&other, &["pull", "-q"]);
    assert!(other.join("templates/a.md").exists(), "pushed ours");
}
