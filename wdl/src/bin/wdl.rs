//! The `wdl` command line tool.
//!
//! If you're here and not a developer of the `wdl` family of crates, you're
//! probably looking for
//! [Sprocket](https://github.com/stjude-rust-labs/sprocket) instead.

use std::io::IsTerminal as _;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use clap_verbosity_flag::WarnLevel;
use colored::Colorize as _;
use tracing_subscriber::layer::SubscriberExt as _;
use wdl::Command;

/// A tool for parsing, validating, and linting WDL source code.
///
/// This command line tool is intended as an entrypoint to work with and develop
/// the `wdl` family of crates. It is not intended to be used by the broader
/// community. If you are interested in a command line tool designed to work
/// with WDL documents more generally, have a look at the `sprocket` command
/// line tool.
///
/// Link: https://github.com/stjude-rust-labs/sprocket
#[derive(Parser)]
#[clap(
    bin_name = "wdl",
    version,
    propagate_version = true,
    arg_required_else_help = true
)]
pub struct App {
    /// The command to run.
    #[command(subcommand)]
    command: Command,

    /// The verbosity flags.
    #[command(flatten)]
    verbosity: Verbosity<WarnLevel>,
}

/// Configures the logging for the application.
fn configure_logging(app: &App) -> Result<()> {
    let indicatif_layer = tracing_indicatif::IndicatifLayer::new();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(app.verbosity)
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_ansi(std::io::stderr().is_terminal())
        .finish()
        .with(indicatif_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();

    configure_logging(&app)?;

    if let Err(e) = match app.command {
        Command::Check(args) => wdl::command::check::main(args).await,
        Command::Run(args) => wdl::command::run::main(args).await,
        Command::Validate(args) => wdl::command::validate::main(args).await,
    } {
        eprintln!(
            "{error}: {e:?}",
            error = if std::io::stderr().is_terminal() {
                "error".red().bold()
            } else {
                "error".normal()
            }
        );
        std::process::exit(1);
    }

    Ok(())
}
