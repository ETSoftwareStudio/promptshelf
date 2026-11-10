use std::collections::BTreeMap;

use promptshelf_core::Kind;
use promptshelf_core::pins::Pins;
use promptshelf_core::resolve::Options;
use promptshelf_core::state::{Usage, UsageLog};
use promptshelf_core::template::{LATE_BINDING, Source, Value};

use super::*;
use crate::tests_common::setup;

fn no_commands() -> Options {
    Options {
        cwd: std::env::temp_dir(),
        run_commands: false,
    }
}

#[test]
fn list_puts_pins_first() {
    let (_dir, shelf) = setup();
    let pins = Pins {
        pins: vec!["personal/rough".into()],
    };
    let out = list(&shelf, &pins, &UsageLog::default(), &Query::default()).unwrap();
    let ids: Vec<&str> = out.iter().map(|s| s.full_id.as_str()).collect();
    assert_eq!(ids[0], "personal/rough");
    assert_eq!(out[0].pin, Some(1));
    assert_eq!(ids.len(), 3);
}

#[test]
fn search_and_kind_filter() {
    let (_dir, shelf) = setup();
    let pins = Pins::default();
    let usage = UsageLog::default();
    let q = Query {
        text: "release".into(),
        ..Default::default()
    };
    let out = list(&shelf, &pins, &usage, &q).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].title, "Release notes");

    let q = Query {
        kind: Some(Kind::Draft),
        ..Default::default()
    };
    let out = list(&shelf, &pins, &usage, &q).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "rough");
}

#[test]
fn recent_sorts_by_last_use() {
    let (_dir, shelf) = setup();
    let mut usage = UsageLog::default();
    usage.prompts.insert(
        "personal/review".into(),
        Usage {
            count: 3,
            last_used: Some("2026-10-01".into()),
            values: BTreeMap::new(),
        },
    );
    usage.prompts.insert(
        "personal/rough".into(),
        Usage {
            count: 1,
            last_used: Some("2026-11-01".into()),
            values: BTreeMap::new(),
        },
    );
    let q = Query {
        view: View::Recent,
        ..Default::default()
    };
    let out = list(&shelf, &Pins::default(), &usage, &q).unwrap();
    let ids: Vec<&str> = out.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["rough", "review"]);
    assert_eq!(out[1].uses, 3);
}

#[test]
fn overview_counts_kinds_and_pins() {
    let (_dir, shelf) = setup();
    let pins = Pins {
        pins: vec!["personal/review".into(), "personal/gone".into()],
    };
    let o = overview(&shelf, &pins, &LastSyncs::new());
    assert_eq!(o.total, 3);
    assert_eq!(o.personal.as_deref(), Some("personal"));
    assert_eq!(o.libraries[0].count, 3);
    assert!(o.libraries[0].personal);
    let drafts = o.kinds.iter().find(|k| k.kind == Kind::Draft).unwrap();
    assert_eq!(drafts.count, 1);
    assert_eq!(o.pins[0].title, "Strict review");
    assert!(!o.pins[1].exists);
}

#[test]
fn fill_leaves_conversation_blanks_open_with_the_instruction() {
    let (_dir, shelf) = setup();
    let v = fill(&shelf, "personal/review", &BTreeMap::new(), &no_commands()).unwrap();
    assert!(!v.complete);
    assert_eq!(v.missing.len(), 1);
    assert_eq!(v.missing[0].name, "focus");
    assert_eq!(v.missing[0].spec.hint.as_deref(), Some("what worries you"));
    assert!(v.pieces.contains(&PieceView::Open {
        name: "focus".into()
    }));
    let audience = v
        .pieces
        .iter()
        .find_map(|p| match p {
            PieceView::Value { name, value } if name == "audience" => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(audience.source, Source::Default);
    assert!(v.send_text.ends_with(&format!("{LATE_BINDING}\n")));
    assert!(
        v.send_text
            .starts_with("Review for developers. Focus on {{focus}}.")
    );
}

#[test]
fn fill_ignores_empty_typed_values() {
    let (_dir, shelf) = setup();
    let given = BTreeMap::from([("focus".to_string(), String::new())]);
    let v = fill(&shelf, "personal/review", &given, &no_commands()).unwrap();
    assert!(!v.complete);
}

#[test]
fn render_with_typed_values_completes_without_resolvers() {
    let (_dir, shelf) = setup();
    let values = BTreeMap::from([
        ("focus".to_string(), Value::given("retries")),
        (
            "audience".to_string(),
            Value {
                value: "developers".into(),
                source: Source::Default,
                detail: None,
            },
        ),
    ]);
    let v = render(&shelf, "personal/review", &values).unwrap();
    assert!(v.complete);
    assert_eq!(v.text, "Review for developers. Focus on retries.\n");
    assert_eq!(v.send_text, v.text);
    let text = prepare_send(&shelf, "personal/review", &values, false).unwrap();
    assert_eq!(text, v.text);
}

#[test]
fn values_come_back_from_json() {
    // The UI sends values back as JSON; the source must survive the trip.
    let json = r#"{"value":"main","source":"branch"}"#;
    let v: Value = serde_json::from_str(json).unwrap();
    assert_eq!(v.source, Source::Branch);
    assert_eq!(v.detail, None);
}

#[test]
fn send_text_adds_the_instruction_only_when_open() {
    assert_eq!(send_text("Hi\n", true), "Hi\n");
    assert_eq!(
        send_text("Hi {{x}}\n", false),
        format!("Hi {{{{x}}}}\n\n{LATE_BINDING}\n")
    );
}
