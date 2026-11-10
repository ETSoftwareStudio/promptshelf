fn main() -> std::process::ExitCode {
    promptshelf::run(std::env::args().skip(1))
}
