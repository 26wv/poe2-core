//! The `poe2` binary; all logic lives in `poe2_cli::run`.
#![deny(unsafe_code)]

fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(poe2_cli::run(
        std::env::args_os(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
