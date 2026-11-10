//! The fill form: one field per open blank.

use std::collections::BTreeMap;

use promptshelf_core::fill;
use ratatui::crossterm::event::KeyCode;

use super::{Action, App, Mode};

impl App<'_> {
    pub(super) fn on_fill_key(&mut self, code: KeyCode) -> Action {
        let Mode::Fill { fields, focus } = &mut self.mode else {
            return Action::None;
        };
        let field = &mut fields[*focus];
        match code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Tab | KeyCode::Down => *focus = (*focus + 1).min(fields.len() - 1),
            KeyCode::BackTab | KeyCode::Up => *focus = focus.saturating_sub(1),
            KeyCode::Left | KeyCode::Right if !field.spec.choices.is_empty() => {
                let choices = &field.spec.choices;
                let i = choices.iter().position(|c| *c == field.value).unwrap_or(0);
                let next = if code == KeyCode::Right {
                    (i + 1) % choices.len()
                } else {
                    (i + choices.len() - 1) % choices.len()
                };
                field.value = choices[next].clone();
            }
            KeyCode::Backspace if field.spec.choices.is_empty() => {
                field.value.pop();
            }
            KeyCode::Char(c) if field.spec.choices.is_empty() => field.value.push(c),
            KeyCode::Enter if *focus + 1 < fields.len() => *focus += 1,
            KeyCode::Enter => return self.finish_fill(),
            _ => {}
        }
        Action::None
    }

    fn finish_fill(&mut self) -> Action {
        let Mode::Fill { fields, .. } = &self.mode else {
            return Action::None;
        };
        if let Some(empty) = fields.iter().find(|f| f.value.trim().is_empty()) {
            self.status = Some(format!("{} is empty", empty.name));
            return Action::None;
        }
        let given: BTreeMap<String, String> = fields
            .iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect();
        let Some(p) = self.current() else {
            return Action::None;
        };
        match self.fill(p, &given) {
            Ok(preview) if preview.rendered.complete => {
                if self.record {
                    let _ = fill::record_use(p, &preview.rendered);
                }
                Action::Done {
                    id: p.full_id(),
                    text: preview.rendered.text,
                }
            }
            Ok(_) => {
                self.status = Some("Some blanks are still open.".into());
                Action::None
            }
            Err(e) => {
                self.status = Some(e);
                Action::None
            }
        }
    }
}
