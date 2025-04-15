use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use anyhow::bail;
use tracing::Level;
use tracing_indicatif::span_ext::IndicatifSpanExt as _;
use tracing_indicatif::style::ProgressStyle;
use url::Url;
use wdl_analysis::Analyzer;
use wdl_analysis::DiagnosticsConfig;
use wdl_analysis::path_to_uri;
use wdl_ast::Validator;
use wdl_lint::LintVisitor;
use wdl_lint::Rule as LintRule;
use wdl_lint::rules::ShellCheckRule;

use crate::PROGRESS_BAR_DELAY_BEFORE_RENDER;

mod results;

pub use results::AnalysisResults;

/// An analysis.
pub struct Analysis {
    /// The set of root nodes to analyze.
    ///
    /// Can be files, directories, or URLs.
    sources: Vec<String>,

    /// A list of rules to except.
    exceptions: HashSet<String>,

    /// Whether or not to enable linting.
    lint: bool,

    /// Whether or not to enable `shellcheck`.
    shellcheck: bool,

    /// The style of the progress bar.
    progress_style: ProgressStyle,
}

impl Analysis {
    /// Adds a source to the analysis.
    pub fn add_source(mut self, source: impl Into<String>) -> Self {
        self.sources.push(source.into());
        self
    }

    /// Adds multiple sources to the analysis.
    pub fn extend_sources(mut self, source: impl IntoIterator<Item = String>) -> Self {
        self.sources.extend(source.into_iter());
        self
    }

    /// Adds a rule to the excepted rules list.
    pub fn add_exception(mut self, rule: impl Into<String>) -> Self {
        self.exceptions.insert(rule.into());
        self
    }

    /// Adds multiple rules to the excepted rules list.
    pub fn extend_exceptions(mut self, rules: impl IntoIterator<Item = String>) -> Self {
        self.exceptions.extend(rules.into_iter());
        self
    }

    /// Sets whether linting is enabled.
    pub fn lint(mut self, value: bool) -> Self {
        self.lint = value;
        self
    }

    /// Sets whether `shellcheck` is enabled.
    pub fn shellcheck(mut self, value: bool) -> Self {
        self.shellcheck = value;
        self
    }

    /// Runs the analysis and returns all results (if any exist).
    pub async fn run(self) -> Result<AnalysisResults> {
        let config = get_diagnostics_config(&self.exceptions);

        let span = tracing::span!(Level::WARN, "progress");
        span.pb_set_style(&self.progress_style);

        let start = Instant::now();

        let mut analyzer = Analyzer::new_with_validator(
            config,
            move |_: (), kind, completed, total| {
                let span = span.clone();
                async move {
                    if start.elapsed() < PROGRESS_BAR_DELAY_BEFORE_RENDER {
                        return;
                    }

                    if completed == 0 {
                        span.pb_start();
                        span.pb_set_length(total.try_into().unwrap());
                        span.pb_set_message(&format!("{kind}"));
                    }

                    span.pb_set_position(completed.try_into().unwrap());
                }
            },
            move || {
                let mut validator = Validator::default();

                if self.lint {
                    let visitor = get_lint_visitor(&self.exceptions);
                    validator.add_visitor(visitor);

                    if self.shellcheck {
                        let rule: Vec<Box<dyn LintRule>> = vec![Box::<ShellCheckRule>::default()];
                        let visitor = LintVisitor::new(rule);
                        validator.add_visitor(visitor);
                    }
                }

                validator
            },
        );

        for source in self.sources {
            register_source(&mut analyzer, &source).await?;
        }

        let results = analyzer.analyze(()).await?;

        Ok(AnalysisResults::from(results))
    }
}

impl Default for Analysis {
    fn default() -> Self {
        Self {
            sources: Default::default(),
            exceptions: Default::default(),
            lint: Default::default(),
            shellcheck: Default::default(),
            progress_style: ProgressStyle::with_template(
                "[{elapsed_precise:.cyan/blue}] {bar:40.cyan/blue} {msg} {pos}/{len}",
            )
            // SAFETY: this is statically tested to always unwrap.
            .unwrap(),
        }
    }
}

/// Gets the rules as a diagnositics configuration with the excepted rules
/// removed.
fn get_diagnostics_config(exceptions: &HashSet<String>) -> DiagnosticsConfig {
    DiagnosticsConfig::new(
        wdl_analysis::rules()
            .into_iter()
            .filter(|rule| !exceptions.contains(rule.id())),
    )
}

/// Gets a lint visitor with the excepted rules removed.
fn get_lint_visitor(exceptions: &HashSet<String>) -> LintVisitor {
    LintVisitor::new(
        wdl_lint::rules()
            .into_iter()
            .filter(|rule| !exceptions.contains(rule.id())),
    )
}

/// Registers a source within an analyzer to be analyzed.
async fn register_source<T: Send + Clone + 'static>(
    analyzer: &mut Analyzer<T>,
    source: &str,
) -> Result<()> {
    if let Ok(url) = Url::parse(source) {
        analyzer.add_document(url).await?;
        return Ok(());
    }

    let path = Path::new(source);

    if path.is_dir() {
        analyzer.add_directory(path.into()).await?;
    } else if let Some(url) = path_to_uri(source) {
        if !Path::new(source).is_file() {
            bail!("source file `{source}` does not exist");
        }

        analyzer.add_document(url).await?;
    } else {
        bail!("failed to convert `{source}` to a URI", source = source)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Analysis;

    #[test]
    fn default() {
        let _ = std::hint::black_box(Analysis::default());
    }
}
