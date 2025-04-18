use std::sync::Mutex;

use anyhow::Result;
use anyhow::anyhow;
use colored::Colorize as _;
use indexmap::IndexSet;
use tracing_indicatif::span_ext::IndicatifSpanExt as _;
use wdl_analysis::document::Document;
use wdl_engine::Inputs;

pub struct Evaluator<'a> {
    /// The document that contains the task or workflow to run.
    document: &'a Document,

    /// The name of the task or workflow to run.
    name: &'a str,

    /// The inputs to the task or workflow.
    inputs: &'a Inputs,
}

impl<'a> Evaluator<'a> {
    /// Creates a new task or workflow evaluator.
    pub fn new(document: &'a Document, name: &'a str, inputs: &'a Inputs) -> Self {
        Self {
            document,
            name,
            inputs,
        }
    }

    // pub async fn run(self) -> Result<()> {
    //     let result = match self.inputs {
    //         &Inputs::Task(mut inputs) => {
    //             // Make any paths specified in the inputs absolute.
    //             let task = self.document.task_by_name(self.name).ok_or_else(|| {
    //                 anyhow!(
    //                     "document does not contain a task named `{name}`",
    //                     name = self.name
    //                 )
    //             })?;

    //             // Ensure all the paths specified in the inputs file are
    //             // relative to the file's directory.
    //             if let Some(path) = path.as_ref().and_then(|p| p.parent()) {
    //                 inputs.join_paths(task, path)?;
    //             }

    //             let evaluator = TaskEvaluator::new(config, token).await?;
    //             evaluator
    //                 .evaluate(document, task, &inputs, output_dir, move |kind| {
    //                     progress(kind, &pb, &state);
    //                     async {}
    //                 })
    //                 .await
    //                 .and_then(EvaluatedTask::into_result)
    //         }
    //         &Inputs::Workflow(mut inputs) => {
    //             let workflow = document
    //                 .workflow()
    //                 .ok_or_else(|| anyhow!("document does not contain a workflow"))?;
    //             if workflow.name() != name {
    //                 bail!("document does not contain a workflow named `{name}`");
    //             }

    //             // Ensure all the paths specified in the inputs file are relative to the file's
    //             // directory
    //             if let Some(path) = path.as_ref().and_then(|p| p.parent()) {
    //                 inputs.join_paths(workflow, path)?;
    //             }

    //             let evaluator = WorkflowEvaluator::new(config, token).await?;
    //             evaluator
    //                 .evaluate(document, inputs, output_dir, move |kind| {
    //                     progress(kind, &pb, &state);
    //                     async {}
    //                 })
    //                 .await
    //         }
    //     };

    //     match result {
    //         Ok(outputs) => {
    //             let s = to_string_pretty(&outputs)?;
    //             println!("{s}");
    //             Ok(None)
    //         }
    //         Err(e) => match e {
    //             EvaluationError::Source(diagnostic) => Ok(Some(diagnostic)),
    //             EvaluationError::Other(e) => Err(e),
    //         },
    //     }
    // }
}
