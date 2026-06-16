use std::process::ExitCode;

mod argparse;
mod error;

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

fn run() -> error::Result<()> {
    let args = std::env::args_os();
    let (_operands, behavior) = argparse::parse(args)?;

    if behavior.show_help {
        let version = env!("CARGO_PKG_VERSION");
        let help = argparse::help_text(BIN_NAME, version);
        println!("{help}");
        return Ok(());
    }

    Ok(())
}
