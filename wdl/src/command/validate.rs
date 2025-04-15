//! The `validate` subcommand.

use anyhow::Result;
use clap::Parser;
use wdl_cli::Inputs;

/// Arguments to the `validate` subcommand.
#[derive(Parser)]
#[clap(disable_version_flag = true)]
pub struct Args {
    /// A source WDL file or URL.
    #[clap(value_name = "PATH or URL")]
    pub sources: String,

    /// The inputs passed in on the command line.
    pub inputs: Vec<String>,
}

/// The main function for the `check` command.
pub async fn main(args: Args) -> Result<()> {
    let inputs = Inputs::coalesce(args.inputs)?;

    dbg!(inputs);

    Ok(())
}
