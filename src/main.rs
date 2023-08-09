//! Main file of TPackDiagnostic.

mod build;
mod gen;
mod input;
mod output;
mod scan;
mod slop;
mod slopx;

use std::{path::PathBuf, str::FromStr, error::Error};

use clap::{arg, Parser};

fn get_default_arg_path() -> PathBuf {
    PathBuf::from_str(".").unwrap()
}

/// Tool for diagnosing Terraria Resource Packs.
#[derive(Parser)]
struct CommandArgs {
    /// The action to be performed. See `README.md` for info.
    action: String,
    /// Input path directory. Not used by all commands.
    #[arg(short, long, default_value = get_default_arg_path().into_os_string())]
    input: PathBuf,
    /// Output path directory. Not used by all commands.
    #[arg(short, long, default_value = get_default_arg_path().into_os_string())]
    output: PathBuf,
    /// Reference path directory. Not used by all commands.
    #[arg(short, long, default_value = get_default_arg_path().into_os_string())]
    reference: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CommandArgs::parse();
    output::info("Started diagnostic.");

    dispatch_action(&args)?;

    output::info("Diagnostic complete!");
    Ok(())
}

fn dispatch_action(args: &CommandArgs) -> Result<(), Box<dyn Error>> {
    match args.action.as_str() {
        "gen" => gen::generate_references(&args.input, &args.output)?,
        "scan" => scan::scan_directory(&args.input, &args.reference)?,
        "build" => build::build_resource_pack(&args.input, &args.output, &args.reference)?,
        action => panic!("Invalid action `{action}`"),
    }
    Ok(())
}
