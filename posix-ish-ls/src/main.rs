use posix_ish_utils::error::Result;
use std::process::ExitCode;

mod arg;
mod color;
mod file;
mod output;
mod traverse;
use traverse::traverse;

const BIN_NAME: &str = "ls";

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args = std::env::args_os();
    let (operands, behavior) = arg::parse(args)?;

    if behavior.show_help {
        let version = env!("CARGO_PKG_VERSION");
        let help = arg::help_text(BIN_NAME, version);
        println!("{help}");
        return Ok(());
    }

    for dir in operands {
        let entries = traverse(&dir, &behavior)?;
        output::print(entries, &behavior)?;
    }

    Ok(())
}
