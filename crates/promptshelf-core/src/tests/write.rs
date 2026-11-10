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
