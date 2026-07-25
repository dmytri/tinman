//! Flow execution: one flow drives several processes in order, in one
//! workspace, so a later step sees what an earlier step wrote. The first step
//! that fails stops the flow and names itself.

use crate::plan::{FlowStep, Plan};
use std::path::Path;
use std::process::Command;

/// What one executed flow step produced.
///
/// @planks("the second step's output is {string}")
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub output: String,
}

/// What a completed flow produced: one outcome per step, in the order written.
///
/// @planks("the second step's output is {string}")
#[derive(Debug, Clone)]
pub struct FlowOutcome {
    pub steps: Vec<StepOutcome>,
}

/// Execute a plan's flow in the order written, every step in the one workspace
/// directory, stopping at the first step that fails.
///
/// @planks("the flow is executed")
pub fn execute(plan: &Plan, workspace: &Path) -> Result<FlowOutcome, String> {
    let mut steps = Vec::new();
    for step in &plan.flow {
        match step {
            FlowStep::Run(command) => {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .current_dir(workspace)
                    .output()
                    .map_err(|e| e.to_string())?;
                if !output.status.success() {
                    return Err(format!("the step {command:?} failed: {}", output.status));
                }
                steps.push(StepOutcome {
                    output: String::from_utf8_lossy(&output.stdout).into_owned(),
                });
            }
            FlowStep::Tui(_) => todo!(),
        }
    }
    Ok(FlowOutcome { steps })
}
