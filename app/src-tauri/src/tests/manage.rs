use std::path::Path;

use super::*;
use crate::tests_common::setup;

#[test]
fn status_of_a_folder_and_a_repo() {
    let (_dir, shelf) = setup();
    let mut last = LastSyncs::new();
    last.insert(
        "personal".into(),
        crate::views::LastSync {
            at: 1,
            error: Some("boom".into()),
            ..Default::default()
        },
    );
    let status = library_status(&shelf, &last);
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].remote, None);
    assert_eq!(status[0].changes, 0, "not a repo");
    assert_eq!(
        status[0].last_sync.as_ref().unwrap().error.as_deref(),
        Some("boom")
    );

    let root = &shelf.libraries[0].root;
    git::run(root, &["init", "-q", "-b", "main"]).unwrap();
    git::run(
        root,
        &["remote", "add", "origin", "https://example.com/a/b.git"],
    )
    .unwrap();
    let status = library_status(&shelf, &LastSyncs::new());
    assert_eq!(status[0].branch, None, "no commit yet");
    assert_eq!(
        status[0].remote.as_deref(),
        Some("https://example.com/a/b.git")
    );
    assert_eq!(status[0].changes, 3, "three new files");
}

#[test]
fn merge_driver_is_shelf_or_its_full_path() {
    let d = merge_driver();
    assert!(d == "shelf" || Path::new(&d).is_absolute(), "{d}");
}
