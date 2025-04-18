//! Facilities for more elegantly exposing `wdl` crate functionality from the
//! command line.

use std::io::IsTerminal as _;
use std::time::Duration;

use anyhow::Context as _;
use anyhow::Result;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::Config;
use codespan_reporting::term::emit;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;
use wdl_ast::Diagnostic;
use wdl_ast::Severity;

pub mod analysis;
pub mod eval;
pub mod input;

pub use analysis::Analysis;
pub use input::Input;
pub use input::Inputs;

/// The delay in showing the progress bar.
///
/// This is to prevent the progress bar from flashing on the screen for
/// very short analyses.
const PROGRESS_BAR_DELAY_BEFORE_RENDER: Duration = Duration::from_secs(2);

/// Emits the given diagnostics to the output stream.
///
/// The use of color is determined by the presence of a terminal.
///
/// In the future, we might want the color choice to be a CLI argument.
fn emit_diagnostics(path: &str, source: &str, diagnostics: &[Diagnostic]) -> Result<usize> {
    let file = SimpleFile::new(path, source);
    let mut stream = StandardStream::stdout(if std::io::stdout().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    let mut errors = 0;
    for diagnostic in diagnostics.iter() {
        if diagnostic.severity() == Severity::Error {
            errors += 1;
        }

        emit(
            &mut stream,
            &Config::default(),
            &file,
            &diagnostic.to_codespan(),
        )
        .context("failed to emit diagnostic")?;
    }

    Ok(errors)
}
