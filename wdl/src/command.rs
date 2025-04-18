//! Subcommands for the `wdl` command line tool.

use std::io::Read as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Result;
use clap::Subcommand;

pub mod check;
pub mod run;
pub mod validate;

/// The command to run.
#[derive(Subcommand)]
pub enum Command {
    /// Runs an analysis on a WDL file or set of WDL files within a directory.
    Check(check::Args),

    /// Runs the specified WDL task or workflow.
    Run(run::Args),

    /// Validates the inputs with respect to the specified WDL task or workflow.
    Validate(validate::Args),
}
