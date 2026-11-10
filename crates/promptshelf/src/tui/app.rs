//! The terminal app's state and key handling. No drawing here, so tests can drive
//! it with key events.

use std::collections::{BTreeMap, HashMap};

use promptshelf_core::pins::Pins;
use promptshelf_core::resolve::Options;
use promptshelf_core::template::{self, Piece, Rendered};
use promptshelf_core::{ListFilter, Prompt, Shelf, VarSpec, fill};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What the event loop does after a key.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    /// A filled prompt is ready: copy or print it.
    Done {
        id: String,
        text: String,
    },
}

pub struct Field {
    pub name: String,
    pub spec: VarSpec,
    pub value: String,
    pub reason: Option<String>,
}

pub enum Mode {
    Browse,
    Fill { fields: Vec<Field>, focus: usize },
}

/// The preview of one prompt: the filled pieces and the fill result.
pub struct Preview {
    pub pieces: Vec<Piece>,
    pub rendered: Rendered,
}

pub struct App<'a> {
    pub shelf: &'a Shelf,
    pub pins: Pins,
    pub query: String,
    pub items: Vec<&'a Prompt>,
    pub selected: usize,
    pub mode: Mode,
    pub status: Option<String>,
    /// Record usage on this machine when a prompt is used. Tests turn it off.
    pub record: bool,
    opts: Options,
    previews: HashMap<String, Result<Preview, String>>,
}

impl<'a> App<'a> {
    pub fn new(shelf: &'a Shelf, pins: Pins, opts: Options) -> Self {
        let mut app = App {
            shelf,
            pins,
            query: String::new(),
            items: Vec::new(),
            selected: 0,
            mode: Mode::Browse,
            status: None,
            record: true,
            opts,
            previews: HashMap::new(),
        };
        app.refresh();
        app
    }

    /// Pins first, then everything else; or search results for a query.
    fn refresh(&mut self) {
        let all = self.shelf.list(&ListFilter::default()).unwrap_or_default();
        self.items = if self.query.trim().is_empty() {
            let mut items: Vec<&Prompt> = self
                .pins
                .pins
                .iter()
                .filter_map(|id| all.iter().find(|p| &p.full_id() == id).copied())
                .collect();
            for p in all {
                if self.pins.position(&p.full_id()).is_none() {
                    items.push(p);
                }
            }
            items
        } else {
            self.shelf
                .search(&self.query, &ListFilter::default())
                .unwrap_or_default()
                .into_iter()
                .map(|h| h.prompt)
                .collect()
        };
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    pub fn current(&self) -> Option<&'a Prompt> {
        self.items.get(self.selected).copied()
    }

    /// The preview of the selected prompt, filled from context. Cached, because
    /// resolvers can run commands.
    pub fn preview(&mut self) -> Option<&Result<Preview, String>> {
        let p = self.current()?;
        let id = p.full_id();
        if !self.previews.contains_key(&id) {
            let result = self.fill(p, &BTreeMap::new());
            self.previews.insert(id.clone(), result);
        }
        self.previews.get(&id)
    }

    fn fill(&self, p: &Prompt, given: &BTreeMap<String, String>) -> Result<Preview, String> {
        let rendered = fill::fill(self.shelf, p, given, &self.opts).map_err(|e| e.to_string())?;
        let values = rendered
            .values
            .iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect();
        let pieces = template::pieces(self.shelf, p, &values).map_err(|e| e.to_string())?;
        Ok(Preview { pieces, rendered })
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Action {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }
        self.status = None;
        match self.mode {
            Mode::Browse => self.on_browse_key(key.code, ctrl),
            Mode::Fill { .. } => self.on_fill_key(key.code),
        }
    }

    fn on_browse_key(&mut self, code: KeyCode, ctrl: bool) -> Action {
        match code {
            KeyCode::Esc if !self.query.is_empty() => {
                self.query.clear();
                self.refresh();
            }
            KeyCode::Esc => return Action::Quit,
            KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('p') if ctrl => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down => self.move_down(),
            KeyCode::Char('n') if ctrl => self.move_down(),
            KeyCode::Char('t') if ctrl => self.toggle_pin(),
            KeyCode::Char('u') if ctrl => {
                self.query.clear();
                self.refresh();
            }
            KeyCode::Enter => return self.choose(),
            KeyCode::Backspace => {
                self.query.pop();
                self.refresh();
            }
            // With an empty query, 1 to 9 open the pinned prompts.
            KeyCode::Char(c @ '1'..='9') if self.query.is_empty() && !ctrl => {
                let n = c as usize - '0' as usize;
                if n <= self.pins.pins.len() {
                    self.selected = n - 1;
                    return self.choose();
                }
            }
            KeyCode::Char(c) if !ctrl => {
                self.query.push(c);
                self.selected = 0;
                self.refresh();
            }
            _ => {}
        }
        Action::None
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    fn toggle_pin(&mut self) {
        let Some(p) = self.current() else { return };
        let Some(root) = self.shelf.personal().map(|l| l.root.clone()) else {
            self.status = Some("No Personal library for pins.".into());
            return;
        };
        let id = p.full_id();
        let result = if self.pins.unpin(&id) {
            Ok(format!("Unpinned {id}"))
        } else {
            self.pins.pin(&id, None).map(|_| format!("Pinned {id}"))
        };
        self.status = Some(
            match result.and_then(|msg| self.pins.save(&root).map(|_| msg)) {
                Ok(msg) => msg,
                Err(e) => e.to_string(),
            },
        );
        self.refresh();
    }

    /// Enter on a prompt: done when nothing is open, else ask for the open blanks.
    fn choose(&mut self) -> Action {
        let Some(p) = self.current() else {
            return Action::None;
        };
        let record = self.record;
        match self.preview() {
            Some(Ok(preview)) if preview.rendered.complete => {
                if record {
                    let _ = fill::record_use(p, &preview.rendered);
                }
                Action::Done {
                    id: p.full_id(),
                    text: preview.rendered.text.clone(),
                }
            }
            Some(Ok(preview)) => {
                let fields = preview
                    .rendered
                    .missing
                    .iter()
                    .map(|m| Field {
                        name: m.name.clone(),
                        value: m.spec.choices.first().cloned().unwrap_or_default(),
                        spec: m.spec.clone(),
                        reason: m.reason.clone(),
                    })
                    .collect();
                self.mode = Mode::Fill { fields, focus: 0 };
                Action::None
            }
            Some(Err(e)) => {
                self.status = Some(e.clone());
                Action::None
            }
            None => Action::None,
        }
    }
}

// The fill form's keys are in form.rs, a child module that can use the
// private parts of App.
#[path = "form.rs"]
mod form;
