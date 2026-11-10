use super::*;

#[test]
fn explains_common_failures() {
    let code = |m: &str| explain(m).map(|h| h.code);
    assert_eq!(
        code("could not run git: No such file or directory"),
        Some("git-missing")
    );
    assert_eq!(
        code("git push failed: git@github.com: Permission denied (publickey)."),
        Some("auth")
    );
    assert_eq!(
        code("fatal: could not read Username for 'https://github.com': terminal prompts disabled"),
        Some("auth")
    );
    assert_eq!(code("*** Please tell me who you are."), Some("no-identity"));
    assert_eq!(code("ERROR: Repository not found."), Some("no-repo"));
    assert_eq!(code("Could not resolve host: github.com"), Some("offline"));
    assert_eq!(code("Host key verification failed."), Some("host-key"));
    assert_eq!(code("something else"), None);
}

#[test]
fn path_env_keeps_the_own_path_first() {
    let own = std::env::var_os("PATH").unwrap_or_default();
    let first = std::env::split_paths(&own).next();
    assert_eq!(std::env::split_paths(path_env()).next(), first);
    assert!(find_program("git").is_some());
    assert!(find_program("no-such-program-anywhere").is_none());
}

#[test]
fn git_info_reads_the_version() {
    assert!(git_info().version.unwrap().starts_with("git version"));
}

#[test]
fn identity_needs_a_name_and_an_email() {
    assert!(set_git_identity(" ", "a@b.c").is_err());
    assert!(set_git_identity("Ana", "not-an-email").is_err());
}

#[cfg(unix)]
mod fake_gh {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// A `gh` that answers `auth status` and makes `repo create` a local bare
    /// repo next to itself. It never reaches GitHub.
    fn fake(dir: &Path, signed_in: bool) -> String {
        let path = dir.join("gh");
        let status = if signed_in {
            "echo 'github.com'; echo '  Logged in to github.com account tester (keyring)'; exit 0"
        } else {
            "echo 'You are not logged into any GitHub hosts.' >&2; exit 1"
        };
        let script = format!(
            r#"#!/bin/sh
here=$(cd "$(dirname "$0")" && pwd)
case "$1 $2" in
  "auth status") {status} ;;
  "repo create")
    name=$3; shift 3
    while [ $# -gt 0 ]; do
      case $1 in --source) src=$2; shift ;; esac
      shift
    done
    git init -q --bare "$here/$name.git" || exit 1
    git -C "$src" remote add origin "$here/$name.git" || exit 1
    echo "https://github.com/tester/$name"
    ;;
  *) echo "unknown: $*" >&2; exit 2 ;;
esac
"#
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path.display().to_string()
    }

    #[test]
    fn status() {
        let dir = tempfile::tempdir().unwrap();
        let gh = fake(dir.path(), true);
        assert_eq!(
            gh_status(&gh),
            Gh::SignedIn {
                account: Some("tester".into())
            }
        );
        let out = tempfile::tempdir().unwrap();
        assert_eq!(gh_status(&fake(out.path(), false)), Gh::SignedOut);
        assert_eq!(gh_status("/no/such/gh"), Gh::Missing);
    }

    #[test]
    fn create_repo_adds_origin() {
        let dir = tempfile::tempdir().unwrap();
        let gh = fake(dir.path(), true);
        let root = dir.path().join("lib");
        std::fs::create_dir_all(&root).unwrap();
        let url = create_github_repo(&gh, &root, "my-prompts").unwrap();
        assert!(url.ends_with("my-prompts.git"), "{url}");
        assert!(crate::git::is_repo(&root));
        let again = create_github_repo(&gh, &root, "my-prompts").unwrap_err();
        assert!(again.to_string().contains("remote already"), "{again}");
        assert!(create_github_repo(&gh, &root, "Bad Name").is_err());
    }
}
