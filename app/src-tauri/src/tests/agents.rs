use std::ffi::OsStr;

use super::*;

#[test]
fn runs_as_the_cli_only_under_the_name_shelf() {
    assert!(is_cli_name(OsStr::new("/Users/a/.local/bin/shelf")));
    assert!(is_cli_name(OsStr::new("shelf.exe")));
    assert!(!is_cli_name(OsStr::new(
        "/Applications/PromptShelf.app/Contents/MacOS/PromptShelf"
    )));
    assert!(!is_cli_name(OsStr::new("promptshelf-desktop")));
}

#[test]
fn a_translocated_app_can_not_be_the_command() {
    let note = command_note(Path::new(
        "/private/var/folders/x/AppTranslocation/1/d/PromptShelf.app/Contents/MacOS/PromptShelf",
    ));
    assert!(note.is_some_and(|n| n.contains("Applications")));
    if !cfg!(windows) {
        assert_eq!(
            command_note(Path::new(
                "/Applications/PromptShelf.app/Contents/MacOS/PromptShelf"
            )),
            None
        );
    }
}
