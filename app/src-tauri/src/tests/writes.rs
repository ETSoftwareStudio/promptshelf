use promptshelf_core::Kind;
use promptshelf_core::pins::Pins;

use super::*;
use crate::tests_common::setup;

#[test]
fn pin_and_unpin_write_the_pin_file() {
    let (_dir, shelf) = setup();
    let root = shelf.personal().unwrap().root.clone();
    let pins = pin(&shelf, "review", None).unwrap();
    assert_eq!(pins.pins, ["personal/review"]);
    pin(&shelf, "personal/rough", Some(1)).unwrap();
    assert_eq!(
        Pins::load(&root).unwrap().pins,
        ["personal/rough", "personal/review"]
    );
    let pins = unpin(&shelf, "personal/review").unwrap();
    assert_eq!(pins.pins, ["personal/rough"]);
}

#[test]
fn save_file_refuses_broken_frontmatter() {
    let (_dir, shelf) = setup();
    let path = shelf.resolve("personal/review").unwrap().path.clone();
    let before = std::fs::read_to_string(&path).unwrap();
    let err = save_file(&shelf, "personal/review", "---\nvars: [\n---\nBody\n").unwrap_err();
    assert!(err.to_string().contains("frontmatter"), "{err}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn save_file_writes_and_reports_a_new_id() {
    let (_dir, shelf) = setup();
    let id = save_file(
        &shelf,
        "personal/review",
        "---\ntitle: Same\n---\nNew body\n",
    )
    .unwrap();
    assert_eq!(id, "personal/review");
    let path = shelf.resolve("personal/review").unwrap().path.clone();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "---\ntitle: Same\n---\nNew body\n"
    );

    let id = save_file(&shelf, "personal/review", "---\nid: strict\n---\nBody\n").unwrap();
    assert_eq!(id, "personal/strict");

    let err = save_file(&shelf, "personal/rough", "---\nid: release-notes\n---\nX\n").unwrap_err();
    assert!(err.to_string().contains("taken"), "{err}");
}

#[test]
fn new_prompt_goes_into_personal() {
    let (_dir, shelf) = setup();
    let id = new_prompt(&shelf, "  Strict review ", Kind::Template, None).unwrap();
    // "strict-review" is free: the existing file's ID is "review".
    assert_eq!(id, "personal/strict-review");
    let id = new_prompt(&shelf, "", Kind::Snippet, Some("personal")).unwrap();
    assert_eq!(id, "personal/untitled");
    assert!(new_prompt(&shelf, "x", Kind::Snippet, Some("nope")).is_err());
}

#[test]
fn sync_skips_folders_that_are_not_git_repos() {
    let (_dir, shelf) = setup();
    let reports = sync(&shelf, None, "shelf").unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skipped.as_deref(), Some("not a git repo"));
    assert!(sync(&shelf, Some("nope"), "shelf").is_err());
}
