//! The expect command: state one expectation against the screen a program
//! draws, and write the plan an honoured expectation proved. The expectation is
//! either the text wanted on the screen or the locator wanted to resolve, which
//! are the two forms a plan already carries, so the command line coins no third
//! vocabulary.

use crate::driver::EXPECT_DEADLINE;
use crate::inspect;
use crate::plan::{Action, Expectation, FlowStep, Locator, Plan, TuiProcess};
use crate::pty::InteractiveCapture;
use crate::sandbox::SandboxSpec;
use crate::tom::{Resolution, Role, build};
use std::path::Path;
use std::time::{Duration, Instant};

/// State one expectation against `args`, launching the terminal program they
/// name inside the sandbox over `workspace`. A stated role makes the expectation
/// a locator, so every argument names the program; otherwise the first argument
/// is the text expected and the rest name the program. A failure carries the
/// screen the program drew, because the operator's next act is to correct the
/// expectation and only the drawn screen tells them how. An honoured expectation
/// writes the plan it proved at `output`, so what survives the moment is a plan
/// the operator commits and re-runs. A stated `after` names the plan whose steps
/// reach the screen the expectation is read against, so the command line grows
/// no driving verbs of its own.
///
/// @planks("the operator executes {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
pub fn state(
    args: &[String],
    role: Option<&str>,
    name: Option<&str>,
    output: Option<&str>,
    after: Option<&str>,
    workspace: &Path,
) -> Result<(), String> {
    let (text, program) = match role {
        Some(_) => (None, args),
        None => {
            let (text, program) = args
                .split_first()
                .ok_or_else(|| "the expectation names nothing".to_string())?;
            (Some(text.as_str()), program)
        }
    };
    let command = program.join(" ");
    let proved = match after {
        Some(after) => drawn(after, role, name, text, workspace)?,
        None => {
            let launched = inspect::run(&SandboxSpec::default(), &command, workspace)?;
            prove(&launched, role, name, text)?
        }
    };
    let Some(output) = output else {
        return Ok(());
    };
    let plan = Plan {
        sandbox: SandboxSpec::default(),
        flow: vec![FlowStep::Tui(TuiProcess {
            command,
            steps: vec![Action::Expect(proved)],
        })],
        sources: Vec::new(),
    };
    crate::examples::write_plan(&plan, &workspace.join(output))
}

/// Run the plan at `after` over the operator's own tree, so its steps reach the
/// screen the expectation is stated against. A plan that failed on the way never
/// reached that screen, so its own failing step is the answer and is reported as
/// the plan's. The screen the plan carries with its failure stays with the plan:
/// it is a screen the expectation was never read against, and printing it here
/// would report the expectation this command was asked to state as though
/// something had been read.
///
/// @planks("the operator executes {string}")
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
fn reached(after: &str, workspace: &Path) -> Result<InteractiveCapture, String> {
    let path = workspace.join(after);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("the plan {} was not read: {e}", path.display()))?;
    let plan = crate::plan::parse(&source).map_err(|e| format!("the plan did not parse: {e}"))?;
    let session = crate::flow::reach_over_tree(&plan, workspace).map_err(|failure| {
        let step = failure.lines().next().unwrap_or(failure.as_str());
        format!("the plan {after:?} failed on its way to the screen:\n  {step}")
    })?;
    session.ok_or_else(|| format!("the plan {after:?} drives no terminal program"))
}

/// The expectation proved against the screen the plan at `after` reached, read
/// off the session that plan is still running in rather than off a fresh launch,
/// which would show the opening screen the plan's steps were run to leave. A key
/// reaches the program before the program has answered it, so the reading is
/// given the same deadline a plan's own expectation step has, and the screen the
/// last reading found is the evidence a failure carries.
///
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
fn drawn(
    after: &str,
    role: Option<&str>,
    name: Option<&str>,
    text: Option<&str>,
    workspace: &Path,
) -> Result<Expectation, String> {
    let session = reached(after, workspace)?;
    let deadline = Instant::now() + EXPECT_DEADLINE;
    loop {
        let screen = session.screen();
        let launched = inspect::Run {
            model: build(&screen),
            screen,
            honoured: true,
        };
        match prove(&launched, role, name, text) {
            Ok(proved) => return Ok(proved),
            Err(report) if Instant::now() >= deadline => return Err(report),
            Err(_) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

/// The expectation a launched screen proved, in whichever of the two forms the
/// command line stated: a role makes it a locator to resolve, and its absence
/// leaves the text to find.
///
/// @planks("the operator executes {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
fn prove(
    launched: &inspect::Run,
    role: Option<&str>,
    name: Option<&str>,
    text: Option<&str>,
) -> Result<Expectation, String> {
    match role {
        Some(role) => resolved(launched, role, name),
        None => held(launched, text.expect("a text expectation carries its text")),
    }
}

/// The expectation a text form proved: the text is on the screen the program
/// drew. A screen without it carries the reading itself, which is the evidence
/// the operator corrects the expectation from.
///
/// @planks("the operator executes {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
fn held(launched: &inspect::Run, text: &str) -> Result<Expectation, String> {
    if !launched.screen.contains(text) {
        return Err(format!(
            "the expectation of {text:?} found this screen instead:\n{}",
            launched.screen.contents()
        ));
    }
    Ok(Expectation {
        text: text.to_string(),
        within: None,
        locator: None,
    })
}

/// The expectation a locator form proved: the role and name address exactly one
/// region of the model the program drew. A role the model never produces is
/// reported as that rather than as an absent region, and several matches are an
/// ambiguity rather than a choice among them.
///
/// @planks("the operator executes {string}")
fn resolved(
    launched: &inspect::Run,
    role: &str,
    name: Option<&str>,
) -> Result<Expectation, String> {
    if !Role::is_produced(role) {
        return Err(format!("{role:?} is not a role the model produces"));
    }
    let target = match name {
        Some(name) => crate::tom::Locator::new(role, name),
        None => crate::tom::Locator::of_role(role),
    };
    match target.resolve(&launched.model) {
        Resolution::Ambiguous(count) => Err(format!(
            "the locator for the {role} named {name:?} matches {count} regions"
        )),
        Resolution::NoMatch => Err(format!(
            "the locator for the {role} named {name:?} found this screen instead:\n{}",
            launched.screen.contents()
        )),
        Resolution::One(_) => {
            let name = name.unwrap_or_default().to_string();
            Ok(Expectation {
                // A region's name is its content in this model, so the name the
                // locator resolved is the value the expectation states.
                text: name.clone(),
                within: None,
                locator: Some(Locator {
                    role: Some(role.to_string()),
                    name,
                    within: None,
                    binding: None,
                }),
            })
        }
    }
}
