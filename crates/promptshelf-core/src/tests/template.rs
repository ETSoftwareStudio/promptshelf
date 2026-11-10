use super::*;
use crate::config::{Config, LibraryConfig, Writes};

#[test]
fn parses_segments() {
    use Segment::*;
    assert_eq!(
        parse("Hi {{ name }}, {{> about-me}} {{not a var}} {{"),
        vec![
            Text("Hi ".into()),
            Var("name".into()),
            Text(", ".into()),
            Include("about-me".into()),
            Text(" {{not a var}} {{".into()),
        ]
    );
    assert_eq!(parse(""), vec![]);
}

fn shelf_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Shelf) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in files {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let config = Config {
        libraries: vec![LibraryConfig {
            name: "personal".into(),
            path: dir.path().to_string_lossy().into_owned(),
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

#[test]
fn fills_and_lists_missing() {
    let (_d, shelf) = shelf_with(&[
        (
            "templates/review.md",
            "---\nvars:\n  focus: { from: conversation, hint: worries }\n  findings: { choices: [All, Blockers] }\n---\nFocus on {{focus}}. Report {{findings}}. {{> tone}}\n",
        ),
        (
            "snippets/tone.md",
            "---\nvars:\n  tone: { default: brief }\n---\nBe {{tone}}.\n",
        ),
    ]);
    let prompt = shelf.resolve("review").unwrap();
    let mut values = BTreeMap::new();
    values.insert("findings".into(), Value::given("All"));
    let r = render(&shelf, prompt, &values).unwrap();
    assert_eq!(r.text, "Focus on {{focus}}. Report All. Be {{tone}}.\n");
    assert!(!r.complete);
    let names: Vec<&str> = r.missing.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["focus", "tone"]);
    assert_eq!(r.missing[0].spec.hint.as_deref(), Some("worries"));
    assert_eq!(r.missing[1].spec.default.as_deref(), Some("brief"));
}

#[test]
fn rejects_a_value_outside_choices() {
    let (_d, shelf) = shelf_with(&[(
        "templates/t.md",
        "---\nvars:\n  x: { choices: [a, b] }\n---\n{{x}}",
    )]);
    let mut values = BTreeMap::new();
    values.insert("x".into(), Value::given("c"));
    let err = render(&shelf, shelf.resolve("t").unwrap(), &values).unwrap_err();
    assert_eq!(err.code(), "invalid");
}

#[test]
fn detects_include_loops_and_missing_includes() {
    let (_d, shelf) = shelf_with(&[
        ("templates/a.md", "{{> b}}"),
        ("snippets/b.md", "{{> a}}"),
        ("templates/c.md", "{{> nope}}"),
    ]);
    let e = render(&shelf, shelf.resolve("a").unwrap(), &BTreeMap::new()).unwrap_err();
    assert!(e.to_string().contains("include loop"), "{e}");
    let e = render(&shelf, shelf.resolve("c").unwrap(), &BTreeMap::new()).unwrap_err();
    assert!(e.to_string().contains("does not exist"), "{e}");
}

#[test]
fn pieces_mark_values_and_open_blanks() {
    let (_d, shelf) = shelf_with(&[
        ("templates/t.md", "A {{x}} B {{y}}.\n{{> empty}}end\n"),
        ("snippets/empty.md", ""),
    ]);
    let mut values = BTreeMap::new();
    values.insert("x".into(), Value::given("one"));
    let pieces = pieces(&shelf, shelf.resolve("t").unwrap(), &values).unwrap();
    assert_eq!(
        pieces,
        vec![
            Piece::Text("A ".into()),
            Piece::Value {
                name: "x".into(),
                value: Value::given("one")
            },
            Piece::Text(" B ".into()),
            Piece::Open { name: "y".into() },
            Piece::Text(".\nend\n".into()),
        ]
    );
    assert_eq!(to_text(&pieces), "A one B {{y}}.\nend\n");
}
