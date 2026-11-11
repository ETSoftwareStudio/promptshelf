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

/// Reads the shelf again from its config, as the app does after a write.
fn reload(shelf: &Shelf) -> Shelf {
    Shelf::load(shelf.config.clone())
}

#[test]
fn create_prompt_writes_the_typed_body() {
    let (_dir, shelf) = setup();
    let body = "Review {{file}}.\n\n  Keep it short.";
    let id = create_prompt(&shelf, "  Strict review ", Kind::Template, None, body).unwrap();
    // "strict-review" is free: the existing file's ID is "review".
    assert_eq!(id, "personal/strict-review");
    let shelf = reload(&shelf);
    let p = shelf.resolve(&id).unwrap();
    assert_eq!(p.title, "Strict review");
    assert_eq!(p.kind, Kind::Template);
    assert!(
        p.path.ends_with("templates/strict-review.md"),
        "{:?}",
        p.path
    );
    let text = std::fs::read_to_string(&p.path).unwrap();
    assert!(
        text.ends_with("---\nReview {{file}}.\n\n  Keep it short.\n"),
        "{text}"
    );
    assert!(!text.contains("Write the prompt here"), "{text}");
}

#[test]
fn create_prompt_names_an_empty_title_untitled() {
    let (_dir, shelf) = setup();
    let id = create_prompt(&shelf, "  ", Kind::Snippet, Some("personal"), "").unwrap();
    assert_eq!(id, "personal/untitled");
    let shelf = reload(&shelf);
    let p = shelf.resolve(&id).unwrap();
    assert_eq!(p.title, "Untitled");
    assert_eq!(p.kind, Kind::Snippet);
    let id = create_prompt(&shelf, "", Kind::Snippet, None, "Text").unwrap();
    assert_eq!(id, "personal/untitled-2");
    assert!(create_prompt(&shelf, "x", Kind::Snippet, Some("nope"), "").is_err());
}

#[test]
fn create_prompt_puts_a_draft_in_the_inbox() {
    let (_dir, shelf) = setup();
    let id = create_prompt(&shelf, "Half an idea", Kind::Draft, None, "Maybe.").unwrap();
    assert_eq!(id, "personal/half-an-idea");
    let shelf = reload(&shelf);
    let p = shelf.resolve(&id).unwrap();
    assert_eq!(p.kind, Kind::Draft);
    let inbox = shelf.library("personal").unwrap().root.join("inbox");
    assert!(p.path.starts_with(&inbox), "{:?}", p.path);
    let drafts = shelf
        .list(&promptshelf_core::ListFilter {
            library: Some("personal".into()),
            kind: Some(Kind::Draft),
            tag: None,
        })
        .unwrap();
    assert!(drafts.iter().any(|d| d.full_id() == id));
}

#[test]
fn sync_skips_folders_that_are_not_git_repos() {
    let (_dir, shelf) = setup();
    let reports = sync(&shelf, None, "shelf").unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skipped.as_deref(), Some("not a git repo"));
    assert!(sync(&shelf, Some("nope"), "shelf").is_err());
}
