use super::*;
use crate::Config;
use crate::config::LibraryConfig;
use crate::git;

fn write_file(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn lib_config(name: &str, root: &Path, writes: Writes) -> LibraryConfig {
    LibraryConfig {
        name: name.into(),
        path: root.display().to_string(),
        remote: None,
        writes,
        personal: name == "personal",
        trusted: None,
        forge: None,
        api: None,
    }
}

/// Personal with two prompts (one includes the other) and a pin.
fn setup() -> (tempfile::TempDir, Shelf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("personal");
    write_file(
        &root,
        "templates/review.md",
        "---\ntitle: Review\n---\nReview it.\n",
    );
    write_file(
        &root,
        "templates/full.md",
        "Start. {{> review}} {{>personal/review}}\n",
    );
    write_file(
        &root,
        "inbox/idea.md",
        "---\nid: idea\nkind: draft\n---\nAn idea.\n",
    );
    Pins {
        pins: vec!["personal/review".into(), "personal/idea".into()],
    }
    .save(&root)
    .unwrap();
    let config = Config {
        libraries: vec![lib_config("personal", &root, Writes::Commit)],
        agents: Default::default(),
    };
    (dir, Shelf::load_main(config))
}

fn pins(shelf: &Shelf) -> Vec<String> {
    Pins::load(&shelf.personal().unwrap().root).unwrap().pins
}

#[test]
fn set_field_keeps_every_other_byte() {
    assert_eq!(
        set_field("---\ntitle: A\ntags: [x]\n---\n\nBody\n", "title", "B"),
        "---\ntitle: B\ntags: [x]\n---\n\nBody\n"
    );
    assert_eq!(
        set_field("---\ntitle: A\n---\nBody\n", "id", "a"),
        "---\ntitle: A\nid: a\n---\nBody\n"
    );
    assert_eq!(set_field("Body\n", "id", "a"), "---\nid: a\n---\nBody\n");
    // `title_x:` is another key.
    assert_eq!(
        set_field("---\ntitle_x: 1\n---\n", "title", "T"),
        "---\ntitle_x: 1\ntitle: T\n---\n"
    );
}

#[test]
fn save_text_checks_the_text_and_the_id() {
    let (_dir, shelf) = setup();
    let path = shelf.resolve("review").unwrap().path.clone();
    assert!(save_text(&shelf, "review", "---\nvars: [\n---\nX\n").is_err());
    let err = save_text(&shelf, "review", "---\nid: full\n---\nX\n").unwrap_err();
    assert!(err.to_string().contains("taken"), "{err}");
    assert_eq!(
        save_text(&shelf, "review", "---\ntitle: R\n---\nNew\n").unwrap(),
        "personal/review"
    );
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "---\ntitle: R\n---\nNew\n"
    );
}

#[test]
fn rename_moves_the_file_pins_and_includes() {
    let (dir, shelf) = setup();
    let r = rename(&shelf, "review", "Strict review", "strict").unwrap();
    assert_eq!(
        (r.old.as_str(), r.new.as_str()),
        ("personal/review", "personal/strict")
    );
    assert_eq!(r.includes, 1);
    let root = dir.path().join("personal");
    assert!(!root.join("templates/review.md").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("templates/strict.md")).unwrap(),
        "---\ntitle: Strict review\nid: strict\n---\nReview it.\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("templates/full.md")).unwrap(),
        "Start. {{> strict}} {{> personal/strict}}\n"
    );
    assert_eq!(pins(&shelf), ["personal/strict", "personal/idea"]);
}

#[test]
fn rename_title_only_and_bad_ids() {
    let (_dir, shelf) = setup();
    let r = rename(&shelf, "review", "Kind review", "review").unwrap();
    assert_eq!(r.includes, 0);
    assert!(
        std::fs::read_to_string(&r.path)
            .unwrap()
            .contains("title: Kind review")
    );
    assert!(rename(&shelf, "review", "X", "Bad ID").is_err());
    assert!(rename(&shelf, "review", " ", "review").is_err());
    assert!(rename(&shelf, "review", "X", "full").is_err());
}

#[test]
fn delete_removes_the_file_and_the_pin() {
    let (_dir, shelf) = setup();
    let path = delete(&shelf, "review").unwrap();
    assert!(!path.exists());
    assert_eq!(pins(&shelf), ["personal/idea"]);
    assert!(delete(&Shelf::load_main(shelf.config.clone()), "review").is_err());
}

#[test]
fn move_a_draft_into_the_library() {
    let (dir, shelf) = setup();
    let id = move_to(&shelf, "idea", "personal", Kind::Snippet).unwrap();
    assert_eq!(id, "personal/idea");
    let root = dir.path().join("personal");
    assert!(!root.join("inbox/idea.md").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("snippets/idea.md")).unwrap(),
        "---\nid: idea\nkind: snippet\n---\nAn idea.\n"
    );
    assert_eq!(pins(&shelf), ["personal/review", "personal/idea"]);
}

#[test]
fn team_libraries_refuse_delete_and_rename() {
    let (dir, mut shelf) = setup();
    let team = dir.path().join("team");
    write_file(&team, "templates/t.md", "T\n");
    shelf
        .config
        .libraries
        .push(lib_config("team", &team, Writes::PullRequest));
    let shelf = Shelf::load_main(shelf.config);
    assert!(matches!(delete(&shelf, "team/t"), Err(Error::Refused(_))));
    assert!(matches!(
        rename(&shelf, "team/t", "T", "u"),
        Err(Error::Refused(_))
    ));
    assert!(in_a_library(&shelf, &team.join("templates/t.md")));
    assert!(!in_a_library(&shelf, Path::new("/etc/hosts")));
}

/// Runs a test in a child process with its own state folder, because
/// proposals keep a record there.
fn in_child(name: &str) {
    let dir = tempfile::tempdir().unwrap();
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--ignored", "--test-threads=1"])
        .env(crate::config::STATE_DIR_ENV, dir.path().join("state"))
        .env(crate::config::CONFIG_DIR_ENV, dir.path().join("config"))
        .env(CHILD_ENV, dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("1 passed"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const CHILD_ENV: &str = "PROMPTSHELF_TEST_CHILD";

#[test]
fn team_edits_go_into_a_proposal() {
    in_child("edit::tests::team_edits_child");
}

#[test]
#[ignore = "runs in a child process with its own state folder"]
fn team_edits_child() {
    let Some(base) = std::env::var_os(CHILD_ENV).map(PathBuf::from) else {
        return;
    };
    let id = ["-c", "user.name=T", "-c", "user.email=t@example.com"];
    git::run(&base, &["init", "-q", "--bare", "-b", "main", "team.git"]).unwrap();
    git::run(&base, &["clone", "-q", "team.git", "team"]).unwrap();
    let team = base.join("team");
    write_file(&team, "templates/t.md", "Team text.\n");
    git::run(&team, &["add", "-A"]).unwrap();
    git::run(&team, &[&id[..], &["commit", "-q", "-m", "t"]].concat()).unwrap();
    git::run(&team, &["push", "-q", "origin", "main"]).unwrap();
    let personal = base.join("personal");
    write_file(&personal, "inbox/idea.md", "An idea.\n");
    let config = Config {
        libraries: vec![
            lib_config("personal", &personal, Writes::Commit),
            lib_config("team", &team, Writes::PullRequest),
        ],
        agents: Default::default(),
    };

    let shelf = Shelf::load(config.clone());
    save_text(&shelf, "team/t", "My text.\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(team.join("templates/t.md")).unwrap(),
        "Team text.\n",
        "the main folder keeps the team's text"
    );
    let shelf = Shelf::load(config.clone());
    let p = shelf.resolve("team/t").unwrap();
    assert!(p.proposal);
    assert_eq!(p.body, "My text.\n");
    // A second save goes into the same proposal.
    save_text(&shelf, "team/t", "My text, again.\n").unwrap();
    assert_eq!(proposals::load().len(), 1);

    let moved = move_to(&shelf, "personal/idea", "team", Kind::Template).unwrap();
    assert_eq!(moved, "team/idea");
    assert!(!team.join("templates/idea.md").exists());
    assert!(proposals::find("team", "idea").is_some());
}
