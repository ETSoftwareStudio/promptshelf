//! Settings → About.

use super::CmdResult;
use crate::about::{self, AboutView};

#[tauri::command]
pub async fn about() -> CmdResult<AboutView> {
    Ok(about::view())
}
