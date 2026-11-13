use super::*;

#[test]
fn installs_the_skill() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!skill_current(dir.path()));
    let path = install_skill(dir.path()).unwrap();
    assert!(path.ends_with(".claude/skills/promptshelf/SKILL.md"));
    assert!(skill_current(dir.path()));
}

/// `npx skills add` reads skills/promptshelf/SKILL.md from the repo.
#[test]
fn the_repo_skill_matches_the_built_in_skill() {
    let repo_skill =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/promptshelf/SKILL.md");
    // Windows checkouts can turn LF into CRLF.
    let text = std::fs::read_to_string(repo_skill)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        text, SKILL_MD,
        "copy SKILL_MD to skills/promptshelf/SKILL.md"
    );
}

#[test]
fn adds_the_mcp_server_and_keeps_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("claude_desktop_config.json");
    std::fs::write(
        &path,
        r#"{"theme":"dark","mcpServers":{"other":{"command":"x","args":[]}}}"#,
    )
    .unwrap();
    let setup = add_mcp_server(&path, "/bin/shelf").unwrap();
    assert!(setup.changed);
    let backup = setup.backup.unwrap();
    assert!(
        std::fs::read_to_string(backup)
            .unwrap()
            .contains("\"theme\"")
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["theme"], "dark");
    assert_eq!(v["mcpServers"]["other"]["command"], "x");
    assert_eq!(
        v["mcpServers"]["promptshelf"],
        json!({ "command": "/bin/shelf", "args": ["mcp"] })
    );
    assert_eq!(mcp_server_command(&path).as_deref(), Some("/bin/shelf"));

    let again = add_mcp_server(&path, "/bin/shelf").unwrap();
    assert!(!again.changed);
    assert_eq!(again.backup, None);
}

#[test]
fn makes_a_new_config_and_refuses_a_broken_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Claude/claude_desktop_config.json");
    let setup = add_mcp_server(&path, "shelf").unwrap();
    assert_eq!(setup.backup, None);
    assert!(path.exists());

    std::fs::write(&path, "[1, 2").unwrap();
    assert!(add_mcp_server(&path, "shelf").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1, 2");
    std::fs::write(&path, r#"{"mcpServers": 3}"#).unwrap();
    assert!(add_mcp_server(&path, "shelf").is_err());
}

#[cfg(unix)]
#[test]
fn links_the_command_and_keeps_real_files() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("bin");
    let target = dir.path().join("app");
    std::fs::write(&target, "").unwrap();
    let link = install_command(&target, &bin).unwrap();
    assert_eq!(std::fs::read_link(&link).unwrap(), target);
    install_command(&target, &bin).unwrap();
    std::fs::remove_file(&link).unwrap();
    std::fs::write(&link, "real").unwrap();
    assert!(install_command(&target, &bin).is_err());
    assert_eq!(std::fs::read_to_string(&link).unwrap(), "real");
}

#[test]
fn adds_local_bin_to_the_profile_once() {
    let home = tempfile::tempdir().unwrap();
    let changed = ensure_on_path(home.path(), "/bin/zsh").unwrap();
    let profile = home.path().join(".zprofile");
    assert_eq!(changed.as_deref(), Some(profile.as_path()));
    assert!(
        std::fs::read_to_string(&profile)
            .unwrap()
            .contains("export PATH=\"$HOME/.local/bin:$PATH\"")
    );
    assert_eq!(ensure_on_path(home.path(), "/bin/zsh").unwrap(), None);

    let other = tempfile::tempdir().unwrap();
    std::fs::write(other.path().join(".zshrc"), "path+=(~/.local/bin)\n").unwrap();
    assert_eq!(ensure_on_path(other.path(), "/bin/zsh").unwrap(), None);
    assert!(profile_for(other.path(), "/usr/bin/fish").ends_with("conf.d/promptshelf.fish"));
}
