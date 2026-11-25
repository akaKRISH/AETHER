#![forbid(unsafe_code)]

mod native;

use std::ffi::OsString;
use std::{env, process::ExitCode};

use native::Error;

const HELP: &str = "AETHER native experiment CLI
Usage: aether <command>

  init                   Initialize the current local project
  run-fixture            Run the deterministic classification fixture
  verify <run-id>         Check a recorded run's integrity and replay
  report <campaign-id>    List the fixture campaign's validated runs
  help                   Show this help

Exit codes: 0 success, 2 invalid input or missing ID, 3 policy denial,
            4 I/O or execution failure, 5 integrity failure.";

fn run(arguments: &[OsString]) -> Result<String, Error> {
    match arguments {
        [] => Ok(HELP.into()),
        [command] if command == "help" || command == "--help" || command == "-h" => Ok(HELP.into()),
        [command] if command == "init" => native::init(),
        [command] if command == "run-fixture" => native::run_fixture(),
        [command, id] if command == "verify" => native::verify(
            id.to_str()
                .ok_or_else(|| Error::Input("invalid run ID".into()))?,
        ),
        [command, id] if command == "report" => native::report(
            id.to_str()
                .ok_or_else(|| Error::Input("invalid campaign ID".into()))?,
        ),
        _ => Err(Error::Input(
            "invalid command or arguments, see aether help".into(),
        )),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    match run(&arguments) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("aether: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
