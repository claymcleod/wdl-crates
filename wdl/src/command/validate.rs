//! The `validate` subcommand.

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use url::Url;
use wdl_cli::Analysis;
use wdl_cli::Inputs;
use wdl_engine::Inputs as EngineInputs;

/// Arguments to the `validate` subcommand.
#[derive(Parser)]
#[clap(disable_version_flag = true)]
pub struct Args {
    /// A source WDL file or URL.
    #[clap(value_name = "PATH or URL")]
    pub source: String,

    /// The name of the task or workflow to validate against.
    ///
    /// If inputs are provided, this will be attempted to be inferred from the
    /// prefixed names of the inputs (e.g, `<name>.<input-name>`).
    ///
    /// If no inputs are provided and this argument is not provided, it will be
    /// assumed you're trying to validate the workflow present in the specified
    /// WDL document.
    #[clap(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// The inputs passed in on the command line.
    pub inputs: Vec<String>,
}

/// The main function for the `validate` command.
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

    match inputs {
        EngineInputs::Task(inputs) => {
            // SAFETY: we wouldn't have a task inputs if a task didn't exist
            // that matched the user's criteria.
            inputs.validate(document, document.task_by_name(&name).unwrap(), None)?
        }
        EngineInputs::Workflow(inputs) => {
            // SAFETY: we wouldn't have a workflow inputs if a workflow didn't
            // exist that matched the user's criteria.
            inputs.validate(document, document.workflow().unwrap(), None)?
        }
    }

    Ok(())
}
