use super::*;

#[test]
fn parses_libraries() {
    let config: Config = toml::from_str(
        r#"
        [[library]]
        name = "personal"
        path = "~/Prompts/personal"
        remote = "git@example.com:you/prompts.git"

        [[library]]
        name = "acme"
        path = "/tmp/acme"
        writes = "pull-request"
        "#,
    )
    .unwrap();
    assert_eq!(config.libraries.len(), 2);
    assert_eq!(config.personal().unwrap().name, "personal");
    assert!(config.libraries[0].is_trusted());
    assert!(!config.libraries[1].is_trusted());
    assert_eq!(config.libraries[1].writes, Writes::PullRequest);
}

#[test]
fn rejects_duplicate_names() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "[[library]]\nname = \"a\"\npath = \"x\"\n[[library]]\nname = \"a\"\npath = \"y\"\n",
    )
    .unwrap();
    assert!(matches!(
        Config::load_from(&path),
        Err(Error::Config { .. })
    ));
}

#[test]
fn round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let config = Config {
        libraries: vec![LibraryConfig {
            name: "personal".into(),
            path: "~/Prompts/personal".into(),
            remote: None,
            writes: Writes::Commit,
            personal: false,
            trusted: None,
            forge: None,
            api: None,
        }],
        agents: AgentRules::default(),
    };
    config.save_to(&path).unwrap();
    assert_eq!(Config::load_from(&path).unwrap(), config);
}
