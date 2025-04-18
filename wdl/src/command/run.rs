//! The `run` subcommand.

use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use colored::Colorize as _;
use indexmap::IndexSet;
use tracing_indicatif::span_ext::IndicatifSpanExt as _;
use url::Url;
use wdl_cli::Analysis;
use wdl_cli::Inputs;
use wdl_engine::Inputs as EngineInputs;
use wdl_engine::v1::ProgressKind;

/// Arguments to the `run` subcommand.
#[derive(Parser)]
#[clap(disable_version_flag = true)]
pub struct Args {
    /// A source WDL file or URL.
    #[clap(value_name = "PATH or URL")]
    pub source: String,

    /// The name of the task or workflow to run against.
    ///
    /// If inputs are provided, this will be attempted to be inferred from the
    /// prefixed names of the inputs (e.g, `<name>.<input-name>`).
    ///
    /// If no inputs are provided and this argument is not provided, it will be
    /// assumed you're trying to run the workflow present in the specified WDL
    /// document.
    #[clap(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// The inputs passed in on the command line.
    pub inputs: Vec<String>,
}

// let run_kind = match &inputs {
//         Inputs::Task(_) => "task",
//         Inputs::Workflow(_) => "workflow",
//     };

//     let pb = tracing::warn_span!("progress");
//     pb.pb_set_style(
//         &ProgressStyle::with_template(&format!(
//             "[{{elapsed_precise:.cyan/blue}}] {{spinner:.cyan/blue}} {running} {run_kind} \
//              {name}{{msg}}",
//             running = "running".cyan(),
//             name = name.magenta().bold()
//         ))
//         .unwrap(),
//     );

//     let state = Mutex::<State>::default();

/// Helper for displaying task ids.
struct Ids<'a>(&'a IndexSet<String>);

impl std::fmt::Display for Ids<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /// The maximum number of executing task names to display at a time
        const MAX_TASKS: usize = 10;

        let mut first = true;
        for (i, id) in self.0.iter().enumerate() {
            if i == MAX_TASKS {
                write!(f, "...")?;
                break;
            }

            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }

            write!(f, "{id}", id = id.magenta().bold())?;
        }

        Ok(())
    }
}

/// Represents state for reporting evaluation progress.
#[derive(Default)]
struct State {
    /// The set of currently executing task identifiers.
    ids: IndexSet<String>,
    /// The number of completed tasks.
    completed: usize,
    /// The number of tasks awaiting execution.
    ready: usize,
    /// The number of currently executing tasks.
    executing: usize,
}

/// A callback for updating state based on engine events.
fn progress(kind: ProgressKind<'_>, pb: &tracing::Span, state: &Mutex<State>) {
    pb.pb_start();

    let message = {
        let mut state = state.lock().expect("failed to lock progress mutex");
        match kind {
            ProgressKind::TaskStarted { .. } => {
                state.ready += 1;
            }
            ProgressKind::TaskExecutionStarted { id, attempt } => {
                // If this is the first attempt, remove it from the ready set
                if attempt == 0 {
                    state.ready -= 1;
                }

                state.executing += 1;
                state.ids.insert(id.to_string());
            }
            ProgressKind::TaskExecutionCompleted { id, .. } => {
                state.executing -= 1;
                state.ids.swap_remove(id);
            }
            ProgressKind::TaskCompleted { .. } => {
                state.completed += 1;
            }
            _ => {}
        }

        format!(
            " - {c} {completed} task{s1}, {r} {ready} task{s2}, {e} {executing} task{s3}: \
                 {ids}",
            c = state.completed,
            completed = "completed".cyan(),
            s1 = if state.completed == 1 { "" } else { "s" },
            r = state.ready,
            ready = "ready".cyan(),
            s2 = if state.ready == 1 { "" } else { "s" },
            e = state.executing,
            executing = "executing".cyan(),
            s3 = if state.executing == 1 { "" } else { "s" },
            ids = Ids(&state.ids)
        )
    };

    pb.pb_set_message(&message);
}

/// The main function for the `run` subcommand.
pub async fn main(args: Args) -> Result<()> {
    let uri = Url::parse(&args.source)
        .with_context(|| format!("path `{path}` must be a file or URL", path = args.source))?;

    let results = Analysis::default().add_source(uri.as_str()).run().await?;

    // SAFETY: this must exist, as we added it as the only source to be analyzed above.
    let document = results.find_result(uri).unwrap().document();

    let inferred = Inputs::coalesce(args.inputs)?.into_engine_inputs(document)?;

    let (name, inputs) = if let Some(inputs) = inferred {
        inputs
    } else {
        if let Some(name) = args.name {
            match (document.task_by_name(&name), document.workflow()) {
                (Some(_), _) => (name, EngineInputs::Task(Default::default())),
                (None, Some(workflow)) => {
                    if workflow.name() == &name {
                        (name, EngineInputs::Workflow(Default::default()))
                    } else {
                        bail!("no task or workflow with name `{name}` was found")
                    }
                }
                (None, None) => bail!("no task or workflow with name `{name}` was found"),
            }
        } else {
            if let Some(workflow) = document.workflow() {
                (
                    workflow.name().to_owned(),
                    EngineInputs::Workflow(Default::default()),
                )
            } else {
                bail!(
                    "no workflow was found in `{path}`; either specify a document with \
                a workflow or use the `-n` option to refer to a specific task or \
                workflow by name",
                    path = args.source
                )
            }
        }
    };

    todo!();

    Ok(())
}
