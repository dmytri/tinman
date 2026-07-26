//! Flow execution: one flow drives several processes in order, in one
//! workspace, so a later step sees what an earlier step wrote. The first step
//! that fails stops the flow and names itself.

use crate::plan::{Action, FlowStep, Plan};
use crate::process::PreparedProcess;
use crate::pty::{InteractiveCapture, capture_interactive_at};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What one executed flow step produced: the two output streams a pipe keeps
/// apart.
///
/// @planks("the second step's output is {string}")
/// @planks("the step's standard output is {string}")
/// @planks("the step's standard error is {string}")
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub output: String,
    pub error: String,
}

/// What a completed flow produced: one outcome per step, in the order written.
///
/// @planks("the second step's output is {string}")
#[derive(Debug, Clone)]
pub struct FlowOutcome {
    pub steps: Vec<StepOutcome>,
}

/// Execute a plan's flow in the order written, every step in the one workspace
/// directory, stopping at the first step that fails. Terminal size is a
/// property of the run rather than of the plan, so the caller states the width
/// every driven program is given, and an unstated width leaves the run on the
/// operator's own terminal size.
///
/// @planks("the flow is executed")
/// @planks("the flow passes")
/// @planks("execution fails and reports the status {int}")
/// @planks("the step's standard output is {string}")
/// @planks("the step's standard error is {string}")
/// @planks("that plan is replayed")
/// @planks("replaying the written plan reproduces the recorded interaction")
/// @planks("that plan is replayed at {int} columns")
pub fn execute(plan: &Plan, workspace: &Path, columns: Option<u16>) -> Result<FlowOutcome, String> {
    let mut steps = Vec::new();
    for step in &plan.flow {
        match step {
            FlowStep::Run(run) => {
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg(&run.command)
                    .current_dir(workspace)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| e.to_string())?;
                if let Some(text) = &run.stdin {
                    let mut pipe = child.stdin.take().expect("the step's input is a pipe");
                    pipe.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
                }
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                if output.status.code() != Some(run.status) {
                    return Err(format!(
                        "the step {:?} failed: expected the status {}, got {}",
                        run.command, run.status, output.status
                    ));
                }
                steps.push(StepOutcome {
                    output: String::from_utf8_lossy(&output.stdout).into_owned(),
                    error: String::from_utf8_lossy(&output.stderr).into_owned(),
                });
            }
            FlowStep::Tui(tui) => {
                let mut session = capture_interactive_at(
                    &PreparedProcess {
                        program: "/bin/sh".to_string(),
                        args: vec!["-c".to_string(), tui.command.clone()],
                        env: Vec::new(),
                        cleanup: Vec::new(),
                    },
                    columns,
                )?;
                for action in &tui.steps {
                    match action {
                        Action::Expect(expectation) => {
                            await_text(&session, &expectation.text)?;
                        }
                        Action::Press(key) => session.press_key(key),
                        Action::Activate(_) | Action::Fill(_) => todo!(),
                    }
                }
                steps.push(StepOutcome {
                    output: session.screen().contents(),
                    error: String::new(),
                });
            }
        }
    }
    Ok(FlowOutcome { steps })
}

/// How long a driven program is given to draw the text a step expects.
const EXPECT_DEADLINE: Duration = Duration::from_secs(5);

/// Wait for a driven program to draw the expected text, reading the live screen
/// in short intervals until the deadline, so a step resolves the moment the
/// text appears.
///
/// @planks("that plan is replayed")
fn await_text(session: &InteractiveCapture, text: &str) -> Result<(), String> {
    let deadline = Instant::now() + EXPECT_DEADLINE;
    loop {
        let screen = session.screen();
        if screen.contains(text) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "the step expecting {text:?} found this screen instead:\n{}",
                screen.contents()
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
