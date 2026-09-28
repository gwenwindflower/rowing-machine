use std::process::ExitCode;

use clap::Parser;
use rowing_machine::cli::Cli;

fn main() -> ExitCode {
    match Cli::parse()
        .config()
        .and_then(|config| rowing_machine::run(&config))
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
