//! Shared test set-up: a Personal library in a temporary folder.

use std::path::Path;

use promptshelf_core::config::{LibraryConfig, Writes};
use promptshelf_core::{Config, Shelf};

pub fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A Personal library with three prompts.
pub fn setup() -> (tempfile::TempDir, Shelf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("personal");
    write(
        &root,
        "templates/review.md",
        "---\ntitle: Strict review\nvars:\n  focus: { from: conversation, hint: what worries you }\n  audience: { default: developers }\n---\nReview for {{audience}}. Focus on {{focus}}.\n",
    );
    write(
        &root,
        "templates/release-notes.md",
        "---\ntitle: Release notes\ntags: [git]\n---\nWrite release notes.\n",
    );
    write(&root, "inbox/rough.md", "# Rough idea\nSomething.\n");
    let config = Config {
        libraries: vec![LibraryConfig {
            name: "personal".into(),
            path: root.display().to_string(),
            remote: None,
            writes: Writes::Commit,
            personal: true,
            trusted: None,
            forge: None,
            api: None,
        }],
        agents: Default::default(),
    };
    (dir, Shelf::load(config))
}
