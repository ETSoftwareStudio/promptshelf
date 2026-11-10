//! Settings → About, and the card that asks for a star.

use super::CmdResult;
use crate::about::{self, AboutView};
use crate::tips::{Answer, Tips};

#[tauri::command]
pub async fn about() -> CmdResult<AboutView> {
    Ok(about::view())
}

/// True when the sidebar shows the star card. The first call marks the
/// first start.
#[tauri::command]
pub async fn star_card() -> CmdResult<bool> {
    let mut ask = false;
    Tips::update(|t, now| {
        t.started(now);
        ask = t.should_ask(now);
    });
    Ok(ask)
}

/// Saves the answer. Star also opens the repo; the app never learns if the
/// person starred it.
#[tauri::command]
pub async fn answer_star(answer: Answer) -> CmdResult<()> {
    Tips::update(|t, now| t.answer(answer, now));
    if answer == Answer::Star {
        about::open(about::REPO)?;
    }
    Ok(())
}
