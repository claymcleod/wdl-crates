use anyhow::Result;
use anyhow::bail;
use url::Url;
use wdl_analysis::AnalysisResult;
use wdl_ast::AstNode as _;

use crate::emit_diagnostics;

/// A set of analysis results.
#[derive(Debug)]
pub struct AnalysisResults(Vec<AnalysisResult>);

impl AnalysisResults {
    /// Creates a new set of analysis results.
    pub fn new(value: Vec<AnalysisResult>) -> Self {
        Self(value)
    }

    /// Attempts to find an analysis result for the matching document URI.
    pub fn find_result(&self, uri: Url) -> Option<&AnalysisResult> {
        self.0.iter().find(|r| **r.document().uri() == uri)
    }

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
