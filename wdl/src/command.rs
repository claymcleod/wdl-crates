//! Subcommands for the `wdl` command line tool.

use std::io::Read as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Result;
use clap::Subcommand;

pub mod check;
pub mod validate;

/// The command to run.
#[derive(Subcommand)]
pub enum Command {
    /// Runs an analysis on a file or set of files within a directory.
    Check(check::Args),

    /// Validates the inputs with respect to the specified WDL task or workflow.
    Validate(validate::Args),
}

/// Reads source from the given path.
///
/// If the path is simply `-`, the source is read from STDIN.
fn read_source(path: &Path) -> Result<String> {
    if path.as_os_str() == "-" {
        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .context("failed to read source from stdin")?;
        Ok(source)
    } else {
        Ok(std::fs::read_to_string(path).with_context(|| {
            format!("failed to read source file `{path}`", path = path.display())
        })?)
    }
}
