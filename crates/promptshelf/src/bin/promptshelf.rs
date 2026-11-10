//! The long name for `shelf`, for systems where another tool owns `shelf`.

fn main() -> std::process::ExitCode {
    promptshelf::run(std::env::args().skip(1))
}
