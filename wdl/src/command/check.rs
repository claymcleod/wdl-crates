//! The `check` subcommand.

use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use wdl_cli::Analysis;

/// Arguments to the `check` subcommand.
#[derive(Parser)]
#[clap(disable_version_flag = true)]
pub struct Args {
    /// A set of source WDL files (files or URLs).
    #[clap(value_name = "PATH or URL")]
    pub sources: Vec<String>,

    /// Excepts (ignores) an analysis or lint rule.
    #[clap(long)]
    pub except: Vec<String>,

    /// Enables the default set of lints (everything but `shellcheck`).
    #[clap(long)]
    pub lint: bool,

    /// Enable `shellcheck` lints.
    #[clap(long)]
    pub shellcheck: bool,

    /// Whether or not to print the results.
    #[clap(short, long)]
    pub print_results: bool,
}

/// The main function for the `check` subcommand.
pub async fn main(args: Args) -> Result<()> {
    if args.sources.is_empty() {
        bail!("you must provide at least one source file, directory, or URL");
    }

    let results = Analysis::default()
        .extend_sources(args.sources)
        .extend_exceptions(args.except)
        .lint(args.lint)
        .shellcheck(args.shellcheck)
        .run()
        .await?;

    results.emit_diagnostics()?;

    if args.print_results {
        println!("{:#?}", results);
    }

    Ok(())
}
