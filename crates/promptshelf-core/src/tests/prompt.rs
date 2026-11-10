use super::*;

const SAMPLE: &str = "---\nid: strict-pr-review\nkind: template\ntags: [review, code]\nvars:\n  focus: { from: conversation, hint: \"what the user is worried about\" }\n  findings: { choices: [Blockers only, Everything] }\n---\nReview this diff. Focus on {{focus}}.\n";

#[test]
fn parses_frontmatter() {
    let p = Prompt::parse(
        SAMPLE,
        Path::new("x/whatever.md"),
        "personal",
        Kind::Snippet,
    )
    .unwrap();
    assert_eq!(p.id, "strict-pr-review");
    assert_eq!(p.kind, Kind::Template);
    assert_eq!(p.tags, vec!["review", "code"]);
    assert_eq!(p.vars["focus"].from.as_deref(), Some("conversation"));
    assert_eq!(p.vars["findings"].choices.len(), 2);
    assert_eq!(p.body, "Review this diff. Focus on {{focus}}.\n");
    assert_eq!(p.title, "Strict pr review");
    assert_eq!(p.full_id(), "personal/strict-pr-review");
}

#[test]
fn no_frontmatter_uses_file_name_and_heading() {
    let p = Prompt::parse(
        "# Release checklist\n\nDo it.\n",
        Path::new("templates/Release Checklist.md"),
        "acme",
        Kind::Template,
    )
    .unwrap();
    assert_eq!(p.id, "release-checklist");
    assert_eq!(p.title, "Release checklist");
    assert!(p.vars.is_empty());
}

#[test]
fn windows_line_endings() {
    let p = Prompt::parse(
        &SAMPLE.replace('\n', "\r\n"),
        Path::new("a.md"),
        "personal",
        Kind::Template,
    )
    .unwrap();
    assert_eq!(p.id, "strict-pr-review");
}

#[test]
fn bad_yaml_is_an_error() {
    let err = Prompt::parse(
        "---\ntags: [a\n---\nx",
        Path::new("a.md"),
        "p",
        Kind::Template,
    )
    .unwrap_err();
    assert_eq!(err.code(), "frontmatter");
}

#[test]
fn bad_id_is_an_error() {
    let err = Prompt::parse(
        "---\nid: Bad ID\n---\nx",
        Path::new("a.md"),
        "p",
        Kind::Template,
    )
    .unwrap_err();
    assert_eq!(err.code(), "frontmatter");
}

#[test]
fn unclosed_frontmatter_is_body() {
    let (fm, body) = split_frontmatter("---\nid: x\nno end");
    assert!(fm.is_none());
    assert_eq!(body, "---\nid: x\nno end");
}

#[test]
fn slugs() {
    assert_eq!(slugify("Strict PR review!"), "strict-pr-review");
    assert_eq!(slugify("  --  "), "untitled");
    assert_eq!(slugify("Café au lait"), "caf-au-lait");
    assert!(is_valid_id("a-b-1"));
    assert!(!is_valid_id("a--b"));
    assert!(!is_valid_id("-a"));
    assert!(!is_valid_id("A"));
}
