//! The terminal app: `shelf` with no arguments, at a terminal.
//!
//! It draws on stderr, so stdout stays free: `shelf | pbcopy` prints the chosen
//! prompt to the pipe. At a terminal, the chosen prompt goes to the clipboard.

mod app;
mod view;

use std::io::{IsTerminal, Write};

use promptshelf_core::resolve::{self, Options};
use promptshelf_core::{Config, Result, Shelf, pins};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

pub use app::{Action, App};

fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stderr(), LeaveAlternateScreen);
}

pub fn run() -> Result<u8> {
    let shelf = Shelf::load(Config::load()?);
    let pins = pins::load(&shelf).unwrap_or_default();
    let mut app = App::new(&shelf, pins, Options::default());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous_hook(info);
    }));
    enable_raw_mode().map_err(|e| promptshelf_core::Error::Invalid(e.to_string()))?;
    let _ = execute!(std::io::stderr(), EnterAlternateScreen);
    let result = event_loop(&mut app);
    restore();

    match result? {
        Action::Done { id, text } => {
            if std::io::stdout().is_terminal() {
                match resolve::set_clipboard(&text) {
                    Ok(()) => eprintln!("Copied {id} to the clipboard."),
                    Err(e) => {
                        eprintln!("shelf: {e}. The prompt:\n");
                        print!("{text}");
                    }
                }
            } else {
                print!("{text}");
                let _ = std::io::stdout().flush();
            }
            Ok(crate::exit::OK)
        }
        _ => Ok(crate::exit::OK),
    }
}

fn event_loop(app: &mut App) -> Result<Action> {
    let backend = CrosstermBackend::new(std::io::stderr());
    let mut terminal =
        Terminal::new(backend).map_err(|e| promptshelf_core::Error::Invalid(e.to_string()))?;
    loop {
        terminal
            .draw(|f| view::draw(f, app))
            .map_err(|e| promptshelf_core::Error::Invalid(e.to_string()))?;
        let ev = event::read().map_err(|e| promptshelf_core::Error::Invalid(e.to_string()))?;
        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match app.on_key(key) {
                Action::None => {}
                other => return Ok(other),
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/tui.rs"]
mod tests;
