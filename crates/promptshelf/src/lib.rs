//! The `shelf` command. The `shelf` and `promptshelf` binaries both call [`run`].

mod caller;
mod cli;
mod commands;
mod mcp;
mod output;
mod skill;
mod tui;

use std::process::ExitCode;

use clap::Parser;

pub use caller::AGENT_ENV;
pub use cli::*;

/// Exit codes. They are part of the contract with agents; never change one.
pub mod exit {
    pub const OK: u8 = 0;
    pub const ERROR: u8 = 1;
    pub const USAGE: u8 = 2;
    pub const BLANKS_OPEN: u8 = 3;
    pub const NOT_FOUND: u8 = 4;
    pub const REFUSED: u8 = 5;
}

/// Runs the command with the given arguments, without the program name.
pub fn run<I>(args: I) -> ExitCode
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let json = args.iter().any(|a| a == "--json");
    let cli = match Cli::try_parse_from(std::iter::once("shelf".to_string()).chain(args)) {
        Ok(cli) => cli,
        Err(e) => {
            use clap::error::ErrorKind;
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                let _ = e.print();
                return ExitCode::SUCCESS;
            }
            if json {
                output::json_error("usage", &e.kind().to_string(), &[]);
            } else {
                let _ = e.print();
            }
            return ExitCode::from(exit::USAGE);
        }
    };
    ExitCode::from(commands::dispatch(cli))
}
