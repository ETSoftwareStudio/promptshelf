use super::*;
use crate::config::{LibraryConfig, Writes};
use crate::prompt::Prompt;

fn library(root: &Path) -> Library {
    let mut diags = Vec::new();
    Library::scan(
        LibraryConfig {
            name: "personal".into(),
            path: root.to_string_lossy().into_owned(),
            remote: None,
            writes: Writes::Commit,
            personal: true,
            trusted: None,
            forge: None,
            api: None,
        },
        &mut diags,
    )
}

#[test]
fn creates_drafts_with_unique_ids() {
    let dir = tempfile::tempdir().unwrap();
    init_library(dir.path()).unwrap();
    let lib = library(dir.path());
    let new = NewPrompt {
        title: "Yes: a \"quoted\" title",
        kind: Kind::Draft,
        body: "Body",
        tags: &["x y".to_string()],
    };
    let (id1, p1) = create_prompt(&lib, &new).unwrap();
    let (id2, _) = create_prompt(&lib, &new).unwrap();
    assert_eq!(id1, "yes-a-quoted-title");
    assert_eq!(id2, "yes-a-quoted-title-2");
    assert!(p1.starts_with(dir.path().join("inbox")));
    let prompt = Prompt::load(&p1, "personal", Kind::Template).unwrap();
    assert_eq!(prompt.title, "Yes: a \"quoted\" title");
    assert_eq!(prompt.kind, Kind::Draft);
    assert_eq!(prompt.tags, ["x y"]);
    assert_eq!(prompt.body, "Body\n");
}

#[test]
fn quotes_yaml_when_needed() {
    assert_eq!(yaml_string("plain title"), "plain title");
    assert_eq!(yaml_string("yes"), "\"yes\"");
    assert_eq!(yaml_string("1.5"), "\"1.5\"");
    assert_eq!(yaml_string("a: b"), "\"a: b\"");
}

fn shelf(root: &Path) -> Shelf {
    let config = crate::Config {
        libraries: vec![library(root).config],
        agents: Default::default(),
    };
    Shelf::load_main(config)
}

#[test]
fn copies_into_personal_with_forked_from() {
    let dir = tempfile::tempdir().unwrap();
    init_library(dir.path()).unwrap();
    std::fs::write(
        dir.path().join("templates/review.md"),
        "---\nid: review\nforked_from: old\ntitle: Review\n---\nBody\n",
    )
    .unwrap();
    let s = shelf(dir.path());
    let c = copy_prompt(&s, "review").unwrap();
    assert_eq!(c.id, "personal/review-2");
    assert_eq!(c.forked_from, "personal/review");
    let text = std::fs::read_to_string(&c.path).unwrap();
    assert_eq!(
        text,
        "---\nid: review-2\nforked_from: personal/review\ntitle: Review\n---\nBody\n"
    );
}

#[test]
fn create_in_a_commit_library_writes_to_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    init_library(dir.path()).unwrap();
    let lib = library(dir.path());
    let (id, path) = create_in(
        &lib,
        &NewPrompt {
            title: "Notes",
            kind: Kind::Snippet,
            body: "B",
            tags: &[],
        },
    )
    .unwrap();
    assert_eq!(id, "notes");
    assert!(path.starts_with(dir.path().join("snippets")));
}
