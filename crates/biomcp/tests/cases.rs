//! YAML cases run the binary Cargo built, one independently named test per file.
#![expect(
    clippy::disallowed_methods,
    reason = "The test entry point reads arguments and locates its own executable"
)]
// Each target compiles only its own entry points from the shared runner modules.
macro_rules! tool_only {
    ($($tokens:tt)*) => {};
}
mod runner;

fn main() -> std::process::ExitCode {
    runner::case_main::main(std::env::args().collect())
}
