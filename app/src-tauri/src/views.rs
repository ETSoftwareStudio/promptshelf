//! The shapes the commands send to the UI. Field names are camelCase for
//! TypeScript; `app/src/lib/types.ts` mirrors them.

use std::collections::BTreeMap;

use promptshelf_core::template::{Filled, Missing, Piece, Value};
use promptshelf_core::{Kind, VarSpec};
use serde::{Deserialize, Serialize};

/// One library, for the sidebar and Settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub name: String,
    pub path: String,
    pub remote: Option<String>,
    /// "commit" or "pull-request".
    pub writes: &'static str,
    pub personal: bool,
    pub trusted: bool,
    pub git: bool,
    pub exists: bool,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub kind: Kind,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinInfo {
    pub full_id: String,
    pub title: String,
    pub position: usize,
    /// False when the pinned prompt no longer exists.
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticInfo {
    pub path: String,
    pub message: String,
}

/// Everything the sidebar needs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub config_path: String,
    pub has_config: bool,
    pub personal: Option<String>,
    pub libraries: Vec<LibraryInfo>,
    pub kinds: Vec<KindCount>,
    pub total: usize,
    pub pins: Vec<PinInfo>,
    pub diagnostics: Vec<DiagnosticInfo>,
}

/// A prompt in the list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub full_id: String,
    pub id: String,
    pub library: String,
    pub kind: Kind,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    /// The 1-based pin position, for ⌘1 to ⌘9.
    pub pin: Option<usize>,
    pub uses: u64,
    /// `YYYY-MM-DD`, UTC.
    pub last_used: Option<String>,
}

/// A prompt with its file, for the detail column and the editor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    #[serde(flatten)]
    pub summary: Summary,
    pub path: String,
    pub body: String,
    pub vars: BTreeMap<String, VarSpec>,
    pub forked_from: Option<String>,
    /// The whole file, frontmatter included, for the Markdown editor.
    pub file: String,
}

/// Which prompts the list shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    /// Pins first, then every other prompt.
    #[default]
    All,
    /// Prompts used on this machine, the most recent first.
    Recent,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Query {
    pub text: String,
    pub library: Option<String>,
    pub kind: Option<Kind>,
    pub view: View,
}

/// A piece of the filled sentence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PieceView {
    Text {
        text: String,
    },
    Value {
        name: String,
        #[serde(flatten)]
        value: Value,
    },
    Open {
        name: String,
    },
}

impl From<Piece> for PieceView {
    fn from(p: Piece) -> Self {
        match p {
            Piece::Text(text) => PieceView::Text { text },
            Piece::Value { name, value } => PieceView::Value { name, value },
            Piece::Open { name } => PieceView::Open { name },
        }
    }
}

/// A filled prompt: the sentence in pieces and the text to send.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FillView {
    pub full_id: String,
    pub pieces: Vec<PieceView>,
    /// The filled text. Open blanks stay as `{{name}}`.
    pub text: String,
    /// What Send copies: the text, plus the instruction to fill open blanks
    /// from the conversation when some are open.
    pub send_text: String,
    pub complete: bool,
    pub values: Vec<Filled>,
    pub missing: Vec<Missing>,
    /// The spec of every blank, the includes' too: hints and choices.
    pub specs: BTreeMap<String, VarSpec>,
}
