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
    pub last_sync: Option<LastSync>,
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
    pub library: Option<String>,
    /// The path inside the library.
    pub file: String,
    /// What to do, in plain words.
    pub help: String,
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
    /// Team updates in the Inbox.
    pub updates: usize,
    pub proposals: usize,
    /// Libraries whose last sync had a conflict or an error.
    pub sync_problems: Vec<String>,
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
    /// True when this is your proposed version of a team prompt.
    pub proposal: bool,
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
    /// "commit", or "pull-request" for a team library.
    pub writes: &'static str,
    /// Your proposal for this prompt, in a team library.
    pub proposal_info: Option<ProposalView>,
    /// Shell commands from a library you do not trust that are not allowed yet.
    pub blocked: Vec<String>,
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

/// A library's git state, for Settings → Libraries.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStatus {
    pub name: String,
    /// `origin` as git has it; the config can be out of date.
    pub remote: Option<String>,
    pub branch: Option<String>,
    /// Commits not pushed yet, and commits on the remote not pulled yet.
    pub ahead: u32,
    pub behind: u32,
    /// Files with uncommitted changes.
    pub changes: usize,
    pub last_sync: Option<LastSync>,
}

/// The last sync of one library in this run of the app.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastSync {
    /// Seconds since 1970, UTC.
    pub at: u64,
    pub error: Option<String>,
    pub hint: Option<promptshelf_core::tools::Hint>,
    pub conflicts: Vec<String>,
}

/// A sync report with plain words for its error.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncView {
    #[serde(flatten)]
    pub report: promptshelf_core::sync::Report,
    pub hint: Option<promptshelf_core::tools::Hint>,
}

/// git and gh on this computer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub git: promptshelf_core::tools::GitInfo,
    pub gh: promptshelf_core::tools::Gh,
}

/// One proposal, for the Proposals list and the detail banner.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalView {
    pub full_id: String,
    pub library: String,
    pub id: String,
    pub title: String,
    /// "not-sent", "sent", "changes-requested", "not-merged" or "gone".
    pub status: promptshelf_core::proposals::Status,
    pub number: Option<u64>,
    pub url: Option<String>,
    pub branch: String,
    /// Edits made after the last send.
    pub unsent: bool,
}

/// One team update in the Inbox.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    #[serde(flatten)]
    pub update: promptshelf_core::inbox::Update,
    pub full_id: String,
    pub title: String,
    /// Commands of `new_commands` you allowed already.
    pub allowed: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxView {
    pub drafts: Vec<Summary>,
    pub updates: Vec<UpdateView>,
}

/// The connected tools in Settings → Agents.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsView {
    pub skill_path: String,
    pub skill_installed: bool,
    /// False when the installed skill is an older text.
    pub skill_current: bool,
    /// The `shelf` command PromptShelf finds, if any.
    pub command: Option<String>,
    /// Why this app can not install the command here, if it can not.
    pub command_note: Option<String>,
    pub claude_config: Option<String>,
    /// The command Claude Desktop starts for PromptShelf, if set up.
    pub claude_command: Option<String>,
}
