use anyhow::Result;
use anyhow::bail;
use wdl_analysis::AnalysisResult;
use wdl_ast::AstNode as _;

use crate::emit_diagnostics;

/// A set of analysis results.
#[derive(Debug)]
pub struct AnalysisResults(Vec<AnalysisResult>);

impl AnalysisResults {
    /// Emits any diagnostics found in the analysis results.
    ///
    /// If any analysis returned an error, that error is returned here and
    /// emitted diagnostics should be considered potentially incomplete.
    pub fn emit_diagnostics(&self) -> Result<()> {
        for result in &self.0 {
            let document = result.document();

            if let Some(e) = result.error() {
                bail!(e.to_owned());
            }

            let diagnostics = document.diagnostics();

            if !diagnostics.is_empty() {
                let source = document.root().text().to_string();
                emit_diagnostics(&document.path(), &source, diagnostics)?;
            }
        }

        Ok(())
    }
}

impl From<Vec<AnalysisResult>> for AnalysisResults {
    fn from(value: Vec<AnalysisResult>) -> Self {
        Self(value)
    }
}
