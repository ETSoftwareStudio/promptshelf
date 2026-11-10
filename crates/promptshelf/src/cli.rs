//! The command-line arguments.

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "shelf",
    version,
    about = "Save, fill and share prompts.",
    long_about = "Save, fill and share prompts.\n\nPrompts are Markdown files in libraries you own, synced with git. Every command takes --json for agents."
)]
pub struct Cli {
    /// Print JSON instead of text.
    #[arg(long, global = true)]
    pub json: bool,

    /// A UI client runs this for a person (for example Pin from Raycast).
    #[arg(long, global = true, hide = true)]
    pub user_action: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create the config and the Personal library.
    Init(InitArgs),
    /// List prompts.
    List(FilterArgs),
    /// Search prompts by ID, title, tags and text.
    Search(SearchArgs),
    /// Show one prompt.
    Show(IdArgs),
    /// Fill a prompt's blanks and print the result.
    Fill(FillArgs),
    /// Save text from stdin as a new prompt (a draft in the Inbox by default).
    Save(SaveArgs),
    /// Create a new prompt and open it in $EDITOR.
    New(NewArgs),
    /// List or add libraries.
    Libs(LibsArgs),
    /// Install or print the Claude Code skill for `shelf`.
    Skill(SkillArgs),
    /// Report files that could not be read.
    Check,
    /// Pin a prompt. Pins come first everywhere (⌘1 to ⌘9 in the apps).
    Pin(PinArgs),
    /// Remove a pin.
    Unpin(IdArgs),
    /// List the pins in order.
    Pins,
    /// Commit, pull and push the libraries.
    Sync(SyncArgs),
    /// Allow the shell commands of a prompt from a library you do not trust.
    Allow(IdArgs),
    /// Edit a prompt in $EDITOR. In a team library this starts a proposal.
    Edit(IdArgs),
    /// Send a proposal to the team as a pull request, or update it.
    Propose(ProposeArgs),
    /// List your proposals to team libraries.
    Proposals {
        /// Ask the host (or git) for each proposal's state, and clean up merged ones.
        #[arg(long)]
        check: bool,
    },
    /// Withdraw a proposal: close its pull request and remove your version.
    Withdraw(IdArgs),
    /// Drafts and updates pulled from team libraries.
    Inbox(InboxArgs),
    /// Copy the text a team update replaced into Personal.
    KeepPrevious(IdArgs),
    /// Copy a prompt into Personal, to change it for yourself.
    Copy(IdArgs),
    /// Git merge driver for the pin list (set up by `shelf sync`).
    #[command(hide = true)]
    MergePins {
        base: std::path::PathBuf,
        ours: std::path::PathBuf,
        theirs: std::path::PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct ProposeArgs {
    /// The prompt ID.
    pub id: String,
    /// The pull request title. Defaults to "Edit <id>" or "Add <id>".
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct InboxArgs {
    /// Clear all team updates.
    #[arg(long)]
    pub done: bool,
    /// Clear the update for one prompt.
    #[arg(long = "done-id", value_name = "ID")]
    pub done_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct PinArgs {
    pub id: String,
    /// The position, 1 to 9. Defaults to the end.
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=9))]
    pub at: Option<u8>,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Only this library.
    #[arg(long = "lib")]
    pub library: Option<String>,
    /// Commit and pull, but do not push.
    #[arg(long)]
    pub no_push: bool,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// The folder for the Personal library.
    #[arg(long, default_value = "~/Prompts/personal")]
    pub path: String,
}

#[derive(Debug, Args, Default)]
pub struct FilterArgs {
    /// Only this library.
    #[arg(long = "lib")]
    pub library: Option<String>,
    /// Only this kind.
    #[arg(long, value_enum)]
    pub kind: Option<KindArg>,
    /// Only prompts with this tag.
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum KindArg {
    Template,
    Snippet,
    Context,
    Persona,
    Workflow,
    Draft,
}

impl From<KindArg> for promptshelf_core::Kind {
    fn from(k: KindArg) -> Self {
        use promptshelf_core::Kind;
        match k {
            KindArg::Template => Kind::Template,
            KindArg::Snippet => Kind::Snippet,
            KindArg::Context => Kind::Context,
            KindArg::Persona => Kind::Persona,
            KindArg::Workflow => Kind::Workflow,
            KindArg::Draft => Kind::Draft,
        }
    }
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Words to search for.
    #[arg(required = true, num_args = 1..)]
    pub query: Vec<String>,
    #[command(flatten)]
    pub filter: FilterArgs,
}

#[derive(Debug, Args)]
pub struct IdArgs {
    /// The prompt ID: `library/id`, or a bare `id`.
    pub id: String,
}

#[derive(Debug, Args)]
pub struct FillArgs {
    pub id: String,
    /// A value for a blank: `--var name=value`. Repeat for each blank.
    #[arg(long = "var", value_name = "NAME=VALUE")]
    pub vars: Vec<String>,
    /// The folder for `branch` and shell commands. Defaults to the current folder.
    #[arg(short = 'C', long, value_name = "DIR")]
    pub cwd: Option<std::path::PathBuf>,
    /// Do not run shell commands from `from:`.
    #[arg(long)]
    pub no_run: bool,
    /// Use the team's version, not your open proposal.
    #[arg(long)]
    pub main: bool,
}

#[derive(Debug, Args)]
pub struct SaveArgs {
    /// The title. The ID comes from it.
    #[arg(long, alias = "name")]
    pub title: String,
    /// Save straight into this library instead of the Inbox (people only).
    #[arg(long = "lib")]
    pub library: Option<String>,
    /// The kind, when saving into a library.
    #[arg(long, value_enum, default_value = "template")]
    pub kind: KindArg,
    #[arg(long)]
    pub tag: Vec<String>,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// The title. The ID comes from it.
    pub title: Option<String>,
    #[arg(long = "lib")]
    pub library: Option<String>,
    #[arg(long, value_enum, default_value = "template")]
    pub kind: KindArg,
}

#[derive(Debug, Args)]
pub struct LibsArgs {
    #[command(subcommand)]
    pub command: Option<LibsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum LibsCommand {
    /// List the libraries.
    List,
    /// Attach a library: a folder, or a git URL to clone.
    Add(LibsAddArgs),
}

#[derive(Debug, Args)]
pub struct LibsAddArgs {
    /// A folder, or a git URL.
    pub source: String,
    /// The library name, used in prompt IDs.
    #[arg(long)]
    pub name: String,
    /// Where to clone a git URL. Defaults to ~/Prompts/<name>.
    #[arg(long)]
    pub path: Option<String>,
    /// How your changes go in.
    #[arg(long, value_enum)]
    pub writes: Option<WritesArg>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum WritesArg {
    Commit,
    PullRequest,
}

#[derive(Debug, Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    /// Print SKILL.md.
    Print,
    /// Write SKILL.md to ~/.claude/skills/promptshelf/.
    Install {
        /// Install into ./.claude/skills/ in this folder instead.
        #[arg(long)]
        project: bool,
    },
}
