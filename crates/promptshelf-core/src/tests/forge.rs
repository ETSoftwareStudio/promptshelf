use super::*;
use crate::config::Writes;

fn lib(forge: Option<&str>) -> LibraryConfig {
    LibraryConfig {
        name: "acme".into(),
        path: "/tmp/acme".into(),
        remote: None,
        writes: Writes::PullRequest,
        personal: false,
        trusted: None,
        forge: forge.map(Into::into),
        api: None,
    }
}

#[test]
fn guesses_the_forge() {
    assert_eq!(
        Forge::of(&lib(None), "git@github.com:acme/prompts.git"),
        Forge::Github
    );
    assert_eq!(
        Forge::of(&lib(None), "https://gitlab.example.com/a/b.git"),
        Forge::Gitlab
    );
    assert_eq!(
        Forge::of(&lib(None), "https://codeberg.org/a/b.git"),
        Forge::Gitea
    );
    assert_eq!(Forge::of(&lib(None), "/srv/git/prompts.git"), Forge::Plain);
    assert_eq!(
        Forge::of(&lib(Some("forgejo")), "https://git.example.com/a/b"),
        Forge::Gitea
    );
}

#[test]
fn parses_repo_paths_and_web_urls() {
    assert_eq!(
        repo_path("git@github.com:acme/prompts.git").as_deref(),
        Some("acme/prompts")
    );
    assert_eq!(
        repo_path("https://user@gitlab.com/group/sub/prompts.git").as_deref(),
        Some("group/sub/prompts")
    );
    assert_eq!(repo_path("/srv/git/prompts.git"), None);
    assert_eq!(
        web_url("git@github.com:acme/prompts.git").as_deref(),
        Some("https://github.com/acme/prompts")
    );
    assert_eq!(
        web_url("https://user@git.example.com/a/b.git").as_deref(),
        Some("https://git.example.com/a/b")
    );
}
