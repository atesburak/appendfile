mod cli;
mod fileops;
mod naming;
mod storage;

use std::process;

use cli::CliAction;

fn main() {
    match cli::parse_cli(std::env::args().skip(1)) {
        Ok(CliAction::Help) => println!("{}", cli::USAGE),
        Ok(CliAction::Run {
            input,
            prefix,
            target,
            persist_target,
        }) => {
            if let Err(err) = storage::run_with_target(input, prefix, target, persist_target) {
                eprintln!("Error: {err}\n\n{}", cli::USAGE);
                process::exit(2);
            }
        }
        Err(message) => {
            eprintln!("Error: {message}\n\n{}", cli::USAGE);
            process::exit(2);
        }
    }
}
