//! Flow execution: one flow drives several processes in order, in one
//! workspace, so a later step sees what an earlier step wrote. The first step
//! that fails stops the flow and names itself.

use crate::backend::resolve;
use crate::driver::{EXPECT_DEADLINE, RESPONSE_DEADLINE, SELECT_KEY};
use crate::plan::{Action, FlowStep, Locator, Plan};
use crate::pty::{InteractiveCapture, capture_interactive_at};
use crate::sandbox::CommandSpec;
use crate::tom::{Resolution, build};
use std::collections::BTreeMap;
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

/// What one locator bound when the flow resolved it: the role and the name
/// carried by the region it found. Both are read off the bound region rather
/// than off the locator that addressed it, so a locator that bound some other
/// region reports that other region.
///
/// @planks("that plan is replayed")
#[derive(Debug, Clone)]
pub struct LocatorBinding {
    pub role: String,
    pub name: Option<String>,
}

/// What a completed flow produced: one outcome per step, in the order written,
/// what every locator the flow resolved bound, in the order resolved, and what
/// every capture step collected, under the name the plan keeps it by.
///
/// @planks("the second step's output is {string}")
/// @planks("that plan is replayed")
/// @planks("the capture carries a {string} named {string}")
#[derive(Debug, Clone)]
pub struct FlowOutcome {
    pub steps: Vec<StepOutcome>,
    pub bindings: Vec<LocatorBinding>,
    pub captures: BTreeMap<String, Vec<String>>,
}

/// Execute a plan's flow in a workspace the caller provisioned and owns, so what
/// the steps write stays there for the caller to read.
///
/// @planks("the flow is executed")
/// @planks("that plan is replayed")
/// @planks("that plan is replayed at {int} columns")
/// @planks("replaying the written plan reproduces the recorded interaction")
pub fn execute(plan: &Plan, workspace: &Path, columns: Option<u16>) -> Result<FlowOutcome, String> {
    run_flow(plan, workspace, columns, false)
}

/// Execute a plan's flow over the operator's own tree: every step reads that
/// directory through an overlay whose writes land in an invisible tmpfs, so a
/// plan written by a stranger leaves the directory the operator is standing in
/// as it found it.
///
/// @planks("the operator runs that plan")
/// @planks("the operator tests that plan")
pub fn execute_over_tree(
    plan: &Plan,
    tree: &Path,
    columns: Option<u16>,
) -> Result<FlowOutcome, String> {
    run_flow(plan, tree, columns, true)
}

/// Execute a plan's flow in the order written, every step in the one workspace
/// directory, stopping at the first step that fails. A step naming its own
/// directory runs in that directory under the workspace, which is created before
/// the step runs. Terminal size is a property of the run rather than of the
/// plan, so the caller states the width every driven program is given, and an
/// unstated width leaves the run on the operator's own terminal size. An
/// overlaid workspace is the operator's own tree, which the steps read and never
/// write through.
///
/// @planks("the flow is executed")
/// @planks("the flow passes")
/// @planks("execution fails and reports the status {int}")
/// @planks("the step's standard output is {string}")
/// @planks("the step's standard error is {string}")
/// @planks("that plan is replayed")
/// @planks("replaying the written plan reproduces the recorded interaction")
/// @planks("that plan is replayed at {int} columns")
/// @planks("the fixture program reports a home directory other than the operator's home")
/// @planks("the step reports a home directory other than the operator's home")
/// @planks("a flow whose only step runs {string} in the directory {string}")
/// @planks("the operator runs that plan")
/// @planks("the operator tests that plan")
/// @planks("the plan is replayed")
/// @planks("the capture carries a {string} named {string}")
/// @planks("the verifier checks the backend construction boundary")
fn run_flow(
    plan: &Plan,
    workspace: &Path,
    columns: Option<u16>,
    overlaid: bool,
) -> Result<FlowOutcome, String> {
    let resolved = resolve(std::env::consts::OS).map_err(|e| format!("{e:?}"))?;
    let backend = resolved.backend();
    let mut steps = Vec::new();
    let mut bindings = Vec::new();
    let mut captures = BTreeMap::new();
    for step in &plan.flow {
        match step {
            FlowStep::Run(run) => {
                if let Some(cwd) = &run.cwd {
                    std::fs::create_dir_all(workspace.join(cwd)).map_err(|e| e.to_string())?;
                }
                let command = CommandSpec {
                    program: "/bin/sh".to_string(),
                    args: vec!["-c".to_string(), run.command.clone()],
                };
                let prepared = if overlaid {
                    backend.prepare_over_tree(
                        &plan.sandbox,
                        &command,
                        workspace,
                        run.cwd.as_deref(),
                    )?
                } else {
                    backend.prepare_with_home(
                        &plan.sandbox,
                        &command,
                        Some(workspace),
                        run.cwd.as_deref(),
                    )?
                };
                let mut child = Command::new(&prepared.program)
                    .args(&prepared.args)
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
                let command = CommandSpec {
                    program: "/bin/sh".to_string(),
                    args: vec!["-c".to_string(), tui.command.clone()],
                };
                let prepared = if overlaid {
                    backend.prepare_over_tree(&plan.sandbox, &command, workspace, None)?
                } else {
                    backend.prepare_with_home(&plan.sandbox, &command, Some(workspace), None)?
                };
                let mut session = capture_interactive_at(&prepared, columns)?;
                for action in &tui.steps {
                    match action {
                        Action::Expect(expectation) => {
                            await_text(&session, &expectation.text)?;
                        }
                        Action::Press(key) => session.press_key(key),
                        Action::Activate(locator) => {
                            bindings.push(activate(&mut session, locator)?)
                        }
                        Action::Fill(fill) => session.press_key(&fill.value),
                        Action::Capture(capture) => {
                            // The capture reads the screen the earlier steps
                            // left, so what it collects is what the flow had
                            // reached at that point.
                            let items = collect(&session, &capture.items);
                            captures.insert(capture.name.clone(), items);
                        }
                    }
                }
                steps.push(StepOutcome {
                    output: session.screen().contents(),
                    error: String::new(),
                });
            }
        }
    }
    Ok(FlowOutcome {
        steps,
        bindings,
        captures,
    })
}

/// How long a capture waits for the driven program to draw the items it
/// collects, so a capture resolves the moment they appear.
const CAPTURE_DEADLINE: Duration = Duration::from_secs(2);

/// Collect the names of the regions playing `role`, reading the live screen in
/// short intervals until it draws one or the deadline passes. An activation
/// resolves the moment the program answers on its status line, which is earlier
/// than the moment the pane that answer opened finishes drawing, so a capture
/// reading the screen once collects whatever had been drawn by then rather than
/// what the step was placed to read. Items are addressed by ordinal among the
/// regions playing the role the plan asks for, which is how the model addresses
/// a region carrying no name of its own, and the names are kept in the order the
/// model lists them.
///
/// @planks("the capture carries a {string} named {string}")
fn collect(session: &InteractiveCapture, role: &str) -> Vec<String> {
    let deadline = Instant::now() + CAPTURE_DEADLINE;
    loop {
        let model = build(&session.screen());
        let mut items: Vec<String> = Vec::new();
        for ordinal in 1.. {
            match crate::tom::Locator::nth(role, ordinal).resolve(&model) {
                Resolution::One(region) => items.extend(region.name),
                _ => break,
            }
        }
        if !items.is_empty() || Instant::now() >= deadline {
            return items;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Activate the region a step's locator addresses: resolve the locator against
/// the model of the current screen, narrowed to the region the locator names
/// under `within` when it names one, a locator matching nothing or several
/// regions failing with what it looked for, then move the selection onto the one
/// region it named and confirm it, the way a person reaches a target with no
/// pointer to click. The step waits for the program's own answer before it ends,
/// so what a later step types lands on the screen the program redrew rather than
/// on the one it is about to erase. The activation reports the region it bound,
/// so a run says which region a locator reached rather than only that it reached
/// one.
///
/// @planks("the plan is replayed")
/// @planks("that plan is replayed")
fn activate(session: &mut InteractiveCapture, locator: &Locator) -> Result<LocatorBinding, String> {
    let name = &locator.name;
    let role = locator
        .role
        .as_deref()
        .unwrap_or_else(|| panic!("the activation of {name:?} names no role"));
    let model = build(&session.screen());
    let target = crate::tom::Locator::new(role, name);
    let target = match locator.within.as_deref() {
        Some(scope) => target.within(scope),
        None => target,
    };
    match target.resolve(&model) {
        Resolution::NoMatch => Err(format!("no {role} named {name:?} was found")),
        Resolution::Ambiguous(count) => {
            Err(format!("{count} matches for the {role} named {name:?}"))
        }
        Resolution::One(region) => {
            let bound = LocatorBinding {
                role: region.role().to_string(),
                name: region.name.clone(),
            };
            let before = session.screen().bottom_line();
            session.press_key(SELECT_KEY);
            session.press_key("Enter");
            let deadline = Instant::now() + RESPONSE_DEADLINE;
            loop {
                if answered(&before, &session.screen().bottom_line()) {
                    return Ok(bound);
                }
                if Instant::now() >= deadline {
                    return Err(format!(
                        "the selection did not reach the {role} named {name:?}"
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

/// Whether the status row now on screen is the program's answer to the keys an
/// activation sent, rather than the terminal's own echo of those keys. The
/// terminal echoes what is typed at it before the program has read a byte, so
/// the row an activation watches carries that echo first: the select key appends
/// to the row the program left, and the confirming carriage return appends a
/// line feed that scrolls that row off the screen and leaves a blank one in its
/// place. A program answers by erasing the row and drawing it again from its
/// start, so an answer is a row that is neither blank nor a continuation of the
/// row the program had left there.
///
/// @planks("the plan is replayed")
fn answered(before: &str, now: &str) -> bool {
    !now.is_empty() && !now.starts_with(before)
}

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
