use std::error::Error;

use clap::{self, Parser};
use clio::ClioPath;

mod build;
mod gen;
mod image_data;
mod output;
mod paths;
mod scan;
mod scanner;
mod static_file_data;

/// Tool for diagnosing Terraria Resource Packs.
#[derive(Parser)]
#[clap(version, about, long_about)]
struct CliArgs {
    /// The action to be performed.
    action: String,

    /// Input path directory. Not used by all commands.
    #[clap(short, long,
        value_parser = clap::value_parser!(ClioPath).exists().is_dir(),
        default_value = ".",
    )]
    input: ClioPath,

    /// Output path directory. Not used by all commands.
    #[clap(short, long,
        value_parser = clap::value_parser!(ClioPath),
        default_value = ".",
    )]
    output: ClioPath,

    /// Reference path directory. Not used by all commands.
    #[clap(short, long,
        value_parser = clap::value_parser!(ClioPath).exists().is_dir(),
        default_value = ".",
    )]
    reference: ClioPath,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    output::info("Started diagnostic.");

    let input = args.input.path().to_path_buf();
    let output = args.output.to_path_buf();
    let reference = args.reference.to_path_buf();

    match args.action.as_str() {
        "gen" => gen::generate_references(&input, &output)?,
        "scan" => scan::scan_resource_pack(&input, &reference)?,
        "build" => build::build_resource_pack(&input, &output, &reference)?,
        a => panic!("invalid action `{a}`, run with `--help` for info"),
    }

    output::info("Diagnostic complete!");
    Ok(())
}
