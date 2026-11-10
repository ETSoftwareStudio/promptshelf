//! The `shelf` command. The `shelf` and `promptshelf` binaries both call [`run`].

use std::process::ExitCode;

/// Runs the command with the given arguments, without the program name.
pub fn run<I>(args: I) -> ExitCode
where
    I: IntoIterator<Item = String>,
{
    match args.into_iter().next().as_deref() {
        Some("--version" | "-V") => {
            println!("shelf {}", promptshelf_core::VERSION);
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("shelf {}: no commands yet", promptshelf_core::VERSION);
            ExitCode::from(2)
        }
    }
}
