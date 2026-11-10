use std::path::Path;

use promptshelf_core::Shelf;
use promptshelf_core::config::{Config, LibraryConfig, Writes};
use promptshelf_core::pins::Pins;
use promptshelf_core::resolve::Options;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{Action, App, Mode};

fn shelf() -> Shelf {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    Shelf::load(Config {
        libraries: vec![LibraryConfig {
            name: "examples".into(),
            path: examples.display().to_string(),
            remote: None,
            writes: Writes::Commit,
            personal: true,
            trusted: None,
            forge: None,
            api: None,
        }],
        agents: Default::default(),
    })
}

fn app<'a>(shelf: &'a Shelf, pins: &[&str]) -> App<'a> {
    let dir = std::env::temp_dir();
    let mut app = App::new(
        shelf,
        Pins {
            pins: pins.iter().map(|s| s.to_string()).collect(),
        },
        Options {
            cwd: dir,
            run_commands: false,
        },
    );
    app.record = false;
    app
}

fn key(app: &mut App, code: KeyCode) -> Action {
    app.on_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn typed(app: &mut App, s: &str) {
    for c in s.chars() {
        key(app, KeyCode::Char(c));
    }
}

#[test]
fn pins_come_first_and_search_filters() {
    let shelf = shelf();
    let mut a = app(&shelf, &["examples/house-rules"]);
    assert_eq!(a.items[0].full_id(), "examples/house-rules");
    assert_eq!(a.items.len(), 5);
    typed(&mut a, "explain");
    assert_eq!(a.items.len(), 1);
    assert_eq!(a.items[0].id, "explain-like-a-colleague");
    key(&mut a, KeyCode::Esc);
    assert!(a.query.is_empty());
    assert_eq!(key(&mut a, KeyCode::Esc), Action::Quit);
}

#[test]
fn a_complete_prompt_is_done_at_once() {
    let shelf = shelf();
    let mut a = app(&shelf, &["examples/about-me"]);
    match key(&mut a, KeyCode::Char('1')) {
        Action::Done { id, text } => {
            assert_eq!(id, "examples/about-me");
            assert!(text.starts_with("I write Rust"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn open_blanks_open_the_form() {
    let shelf = shelf();
    let mut a = app(&shelf, &[]);
    typed(&mut a, "strict");
    assert_eq!(key(&mut a, KeyCode::Enter), Action::None);
    let Mode::Fill { fields, .. } = &a.mode else {
        panic!("not in fill mode");
    };
    let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["focus", "diff"]);
    // An empty value is not accepted.
    key(&mut a, KeyCode::Enter);
    assert_eq!(key(&mut a, KeyCode::Enter), Action::None);
    assert!(a.status.as_deref().unwrap().contains("empty"));
    key(&mut a, KeyCode::BackTab);
    typed(&mut a, "errors");
    key(&mut a, KeyCode::Tab);
    typed(&mut a, "+ fix");
    match key(&mut a, KeyCode::Enter) {
        Action::Done { text, .. } => {
            assert!(text.contains("Focus on errors."));
            assert!(text.ends_with("+ fix\n"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn ctrl_c_quits_anywhere() {
    let shelf = shelf();
    let mut a = app(&shelf, &[]);
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(a.on_key(ctrl_c), Action::Quit);
}

#[test]
fn draws_list_preview_and_footnotes() {
    use ratatui::{Terminal, backend::TestBackend};
    let shelf = shelf();
    let mut a = app(&shelf, &["examples/strict-pr-review"]);
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    terminal.draw(|f| super::view::draw(f, &mut a)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let screen: String = buffer
        .content()
        .chunks(100)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
        .collect();
    if std::env::var_os("SHOW_TUI").is_some() {
        println!("{screen}");
    }
    assert!(screen.contains("1 Strict PR review"));
    assert!(screen.contains("Prompts (5)"));
    assert!(screen.contains("╌focus╌"));
    assert!(screen.contains("findings: default"));
    assert!(screen.contains("↵ use"));
}
