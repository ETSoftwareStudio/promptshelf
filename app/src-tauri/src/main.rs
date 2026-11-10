// No console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    // Under the name `shelf` (the link Settings → Agents installs), the app is
    // the command-line tool.
    if std::env::args_os()
        .next()
        .is_some_and(|a| promptshelf_desktop::agents::is_cli_name(&a))
    {
        return promptshelf::run(std::env::args().skip(1));
    }
    promptshelf_desktop::run();
    ExitCode::SUCCESS
}
