use super::*;

fn spec(from: &str, pattern: Option<&str>) -> VarSpec {
    VarSpec {
        from: Some(from.into()),
        pattern: pattern.map(Into::into),
        ..Default::default()
    }
}

fn facts(allowed: bool) -> Facts<'static> {
    Facts {
        last: Some("last value"),
        command_allowed: allowed,
        allow_hint: "run `shelf allow x`",
    }
}

fn opts(dir: &Path) -> Options {
    Options {
        cwd: dir.to_path_buf(),
        run_commands: true,
    }
}

#[test]
fn runs_allowed_commands() {
    let dir = tempfile::tempdir().unwrap();
    match resolve(
        &spec("echo hello-world", None),
        &opts(dir.path()),
        &facts(true),
    ) {
        Outcome::Value(v) => {
            assert_eq!(v.value.trim(), "hello-world");
            assert_eq!(v.source, Source::Command);
            assert_eq!(v.detail.as_deref(), Some("echo hello-world"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn blocks_commands_that_are_not_allowed() {
    let dir = tempfile::tempdir().unwrap();
    match resolve(&spec("echo hi", None), &opts(dir.path()), &facts(false)) {
        Outcome::Open(Some(reason)) => assert!(reason.contains("not allowed"), "{reason}"),
        other => panic!("{other:?}"),
    }
    let off = Options {
        run_commands: false,
        ..opts(dir.path())
    };
    match resolve(&spec("echo hi", None), &off, &facts(true)) {
        Outcome::Open(Some(reason)) => assert!(reason.contains("commands are off")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn reports_failing_and_empty_commands() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        run_shell("exit 3", dir.path())
            .unwrap_err()
            .contains("failed")
    );
    let silent = if cfg!(windows) { "rem" } else { "true" };
    assert!(
        run_shell(silent, dir.path())
            .unwrap_err()
            .contains("printed nothing")
    );
}

#[test]
fn applies_patterns() {
    assert_eq!(
        pick(r"[A-Z]+-\d+", "feature/ABC-123-login").unwrap(),
        "ABC-123"
    );
    assert_eq!(pick(r"v(\d+)", "release v42").unwrap(), "42");
    assert!(pick(r"\d+", "none").unwrap_err().contains("no match"));
    assert!(pick(r"(", "x").unwrap_err().contains("bad pattern"));
}

#[test]
fn reads_the_branch() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    crate::git::run(d, &["init", "-q", "-b", "feature/ABC-7-thing"]).unwrap();
    crate::git::run(
        d,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.com",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "x",
        ],
    )
    .unwrap();
    match resolve(&spec("branch", Some(r"[A-Z]+-\d+")), &opts(d), &facts(true)) {
        Outcome::Value(v) => {
            assert_eq!(v.value, "ABC-7");
            assert_eq!(v.source, Source::Branch);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn conversation_stays_open_and_last_is_used() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        resolve(&spec("conversation", None), &opts(dir.path()), &facts(true)),
        Outcome::Open(None)
    ));
    match resolve(&spec("last", None), &opts(dir.path()), &facts(true)) {
        Outcome::Value(v) => assert_eq!(v.value, "last value"),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        resolve(&spec("selection", None), &opts(dir.path()), &facts(true)),
        Outcome::Open(Some(_))
    ));
    assert!(is_command("git diff --staged"));
    assert!(!is_command("branch"));
}
