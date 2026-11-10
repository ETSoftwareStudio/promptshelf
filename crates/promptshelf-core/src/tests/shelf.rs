use super::*;
use crate::config::Writes;

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn lib(name: &str, root: &Path) -> LibraryConfig {
    LibraryConfig {
        name: name.into(),
        path: root.to_string_lossy().into_owned(),
        remote: None,
        writes: Writes::Commit,
        personal: false,
        trusted: None,
        forge: None,
        api: None,
    }
}

fn fixture() -> (tempfile::TempDir, Shelf) {
    let dir = tempfile::tempdir().unwrap();
    let personal = dir.path().join("personal");
    let acme = dir.path().join("acme");
    write(
        &personal,
        "templates/strict-pr-review.md",
        "---\ntags: [review]\n---\nReview {{diff}}.\n",
    );
    write(&personal, "snippets/tone.md", "Be brief.\n");
    write(&acme, "templates/strict-pr-review.md", "Team version.\n");
    write(
        &acme,
        "templates/coroutine-audit.md",
        "---\ntitle: Coroutine audit\ntags: [kotlin, review]\n---\nAudit.\n",
    );
    write(&acme, "templates/release.md", "Release.\n");
    write(&acme, "inbox/release.md", "Duplicate.\n");
    write(&acme, "templates/broken.md", "---\ntags: [a\n---\n");
    let config = Config {
        libraries: vec![lib("personal", &personal), lib("acme", &acme)],
        agents: Default::default(),
    };
    (dir, Shelf::load(config))
}

#[test]
fn resolves_ids() {
    let (_dir, shelf) = fixture();
    assert_eq!(
        shelf.resolve("strict-pr-review").unwrap().library,
        "personal"
    );
    assert_eq!(
        shelf.resolve("acme/strict-pr-review").unwrap().body,
        "Team version.\n"
    );
    assert_eq!(shelf.resolve("coroutine-audit").unwrap().library, "acme");
    assert!(matches!(shelf.resolve("nope"), Err(Error::NotFound(_))));
    assert!(matches!(
        shelf.resolve("x/nope"),
        Err(Error::UnknownLibrary(_))
    ));
}

#[test]
fn ambiguous_outside_personal() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    write(&a, "templates/x.md", "a");
    write(&b, "templates/x.md", "b");
    let mut pa = lib("a", &a);
    pa.personal = false;
    let config = Config {
        libraries: vec![lib("p", &dir.path().join("p")), pa, lib("b", &b)],
        agents: Default::default(),
    };
    let shelf = Shelf::load(config);
    match shelf.resolve("x") {
        Err(Error::Ambiguous { matches, .. }) => assert_eq!(matches, ["a/x", "b/x"]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn reports_bad_and_duplicate_files() {
    let (_dir, shelf) = fixture();
    let messages: Vec<&str> = shelf
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("duplicate ID \"release\""))
    );
    assert!(messages.iter().any(|m| m.contains("unclosed bracket")));
}

#[test]
fn lists_with_filters() {
    let (_dir, shelf) = fixture();
    let all = shelf.list(&ListFilter::default()).unwrap();
    assert_eq!(all.len(), 5);
    let review = shelf
        .list(&ListFilter {
            tag: Some("review".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(review.len(), 2);
    let snippets = shelf
        .list(&ListFilter {
            kind: Some(Kind::Snippet),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(snippets[0].id, "tone");
}

#[test]
fn search_ranks_id_and_title_first() {
    let (_dir, shelf) = fixture();
    let hits = shelf.search("review", &ListFilter::default()).unwrap();
    let ids: Vec<String> = hits.iter().map(|h| h.prompt.full_id()).collect();
    assert_eq!(ids[0], "acme/strict-pr-review");
    assert!(ids.contains(&"acme/coroutine-audit".to_string()));
    assert!(
        shelf
            .search("zzz", &ListFilter::default())
            .unwrap()
            .is_empty()
    );
    let both = shelf
        .search("audit kotlin", &ListFilter::default())
        .unwrap();
    assert_eq!(both.len(), 1);
}
