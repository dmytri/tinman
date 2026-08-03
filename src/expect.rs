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
/// no driving verbs of its own. A stated `conforms` names the layout scantling
/// the whole model is attested against, which is a constraint on every region
/// rather than a spot check, so every argument names the program.
///
/// @planks("the operator executes {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator expects {string} after {string} against the fixture terminal program")
/// @planks("the operator attests {string} after {string} against the fixture terminal program")
pub fn state(
    args: &[String],
    role: Option<&str>,
    name: Option<&str>,
    output: Option<&str>,
    after: Option<&str>,
    conforms: Option<&str>,
    workspace: &Path,
) -> Result<(), String> {
    if let Some(conforms) = conforms {
        return attested(&args.join(" "), conforms, output, after, workspace);
    }
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

/// Attest the model `command` draws against the layout the scantling at
/// `conforms` states. A stated `after` names the plan whose steps reach the
/// screen the model is read off, so the layout is attested against what the
/// plan left rather than the opening screen. An attestation that held writes
/// the plan it proved at `output`, which carries the attestation as its own
/// step, so the operator commits and re-runs what the probe proved.
///
/// @planks("the operator executes {string}")
/// @planks("the operator attests {string} after {string} against the fixture terminal program")
fn attested(
    command: &str,
    conforms: &str,
    output: Option<&str>,
    after: Option<&str>,
    workspace: &Path,
) -> Result<(), String> {
    let scantling = layout(conforms, workspace)?;
    let validator = compiled(&scantling, conforms)?;
    match after {
        Some(after) => {
            let session = reached(after, workspace)?;
            settled(&validator, &scantling, &session, command, conforms)?;
        }
        None => {
            let launched = inspect::launched(command, workspace)?;
            let drawn =
                serde_json::to_value(&launched.model).expect("the model is written as JSON");
            conformed(
                &validator,
                &scantling,
                &drawn,
                command,
                conforms,
                &launched.screen.contents(),
            )?;
        }
    }
    let Some(output) = output else {
        return Ok(());
    };
    let plan = Plan {
        sandbox: SandboxSpec::default(),
        flow: vec![FlowStep::Tui(TuiProcess {
            command: command.to_string(),
            steps: vec![Action::Conforms(conforms.to_string())],
        })],
        sources: Vec::new(),
    };
    crate::examples::write_plan(&plan, &workspace.join(output))
}

/// Attest the model `session` is drawing against the layout the scantling at
/// `conforms` states, which is the form a plan's own attestation step takes:
/// the program is already running and the screen to read is the one its steps
/// reached.
///
/// @planks("the operator attests {string} after {string} against the fixture terminal program")
/// @planks("that plan is replayed")
pub(crate) fn attest_session(
    session: &InteractiveCapture,
    command: &str,
    conforms: &str,
    workspace: &Path,
) -> Result<(), String> {
    let scantling = layout(conforms, workspace)?;
    let validator = compiled(&scantling, conforms)?;
    settled(&validator, &scantling, session, command, conforms)
}

/// The attestation read off the screen `session` is drawing. A key reaches the
/// program before the program has answered it, so the reading is given the same
/// deadline a plan's own expectation step has, and the departures the last
/// reading found are the evidence a failure carries.
///
/// @planks("the operator attests {string} after {string} against the fixture terminal program")
fn settled(
    validator: &jsonschema::Validator,
    scantling: &serde_json::Value,
    session: &InteractiveCapture,
    command: &str,
    conforms: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + EXPECT_DEADLINE;
    loop {
        let screen = session.screen();
        let drawn = serde_json::to_value(build(&screen)).expect("the model is written as JSON");
        match conformed(
            validator,
            scantling,
            &drawn,
            command,
            conforms,
            &screen.contents(),
        ) {
            Ok(()) => return Ok(()),
            Err(report) if Instant::now() >= deadline => return Err(report),
            Err(_) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

/// The layout the scantling at `conforms` states. The scantling is judged
/// before the program is launched, because a document that is not a schema, and
/// one carrying no keyword any model can fail, both hold against everything:
/// reporting either as an attestation that passed would report a check that
/// cannot fail as one that did.
///
/// @planks("the operator executes {string}")
fn layout(conforms: &str, workspace: &Path) -> Result<serde_json::Value, String> {
    let path = workspace.join(conforms);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("the layout scantling {} was not read: {e}", path.display()))?;
    let scantling: serde_json::Value = serde_json::from_str(&source)
        .map_err(|e| format!("the layout scantling {conforms:?} is not JSON: {e}"))?;
    jsonschema::draft202012::meta::validate(&scantling).map_err(|e| {
        format!(
            "the layout scantling {conforms:?} is not a schema: {e} at {}",
            e.instance_path()
        )
    })?;
    // The dialect a scantling declares says which schema language it is written
    // in, and nothing about the model, so a document carrying that and nothing
    // else constrains nothing.
    if scantling
        .as_object()
        .is_some_and(|keywords| keywords.keys().all(|keyword| keyword == "$schema"))
    {
        return Err(format!(
            "the layout scantling {conforms:?} constrains nothing, so every model holds against \
             it and the attestation cannot fail"
        ));
    }
    Ok(scantling)
}

/// The validator the layout compiles to.
///
/// @planks("the operator executes {string}")
fn compiled(
    scantling: &serde_json::Value,
    conforms: &str,
) -> Result<jsonschema::Validator, String> {
    jsonschema::validator_for(scantling)
        .map_err(|e| format!("the layout scantling {conforms:?} is not a schema: {e}"))
}

/// The attestation of one model against the compiled layout. A model that
/// departs from the layout carries every departure, since the operator's next
/// act is to correct the program or the layout and one departure at a time
/// tells them only where to start. The evidence a failure carries is `screen`,
/// the screen the program drew, which is the form the operator reads and acts
/// on; the model itself stays out of the report, because a screen's model runs
/// to tens of thousands of characters on one line and buries the departure
/// inside the document that carries it.
///
/// @planks("the operator executes {string}")
fn conformed(
    validator: &jsonschema::Validator,
    scantling: &serde_json::Value,
    drawn: &serde_json::Value,
    command: &str,
    conforms: &str,
    screen: &str,
) -> Result<(), String> {
    let departures: Vec<String> = validator
        .iter_errors(drawn)
        .map(|failure| departure(&failure, scantling))
        .collect();
    if departures.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the model {command:?} drew does not conform to the layout {conforms:?}:\n{}\n  on this \
         screen:\n{screen}",
        departures.join("\n")
    ))
}

/// One departure from the layout, told so the operator can act on it: what the
/// model carries, where in the model it sits, and the demand it failed. The
/// demand is read back out of the scantling at the failing location, because
/// the validator names the keyword that failed and the value the model drew,
/// and never the region or the position the layout asked for.
///
/// @planks("the operator executes {string}")
fn departure(failure: &jsonschema::ValidationError<'_>, scantling: &serde_json::Value) -> String {
    let location = failure.schema_path().as_str();
    let demand = scantling
        .pointer(location)
        .map_or_else(String::new, |demand| format!(", which demands {demand}"));
    format!(
        "  {}\n    at {} of the model, against the layout at {location}{demand}",
        told(failure),
        failure.instance_path()
    )
}

/// What the validator found, told without the document it found it in. The
/// validator writes the value that failed into its own account of a departure,
/// which is what an operator wants where that value is a number or a text, and
/// is the whole serialized model where it is a region or the list of them. The
/// screen the program drew is carried with the failure, so restating the
/// document adds nothing the reader can act on; the location and the demand
/// already say which constraint went.
///
/// @planks("the operator executes {string}")
fn told(failure: &jsonschema::ValidationError<'_>) -> String {
    match failure.instance().as_ref() {
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            "the model departs from the layout here".to_string()
        }
        _ => failure.to_string(),
    }
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
/// @planks("that expectation carries no text")
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
    // The reader of this failure is an operator or an agent, so a carried name
    // is told as the name rather than as the optional the command line parsed
    // it into, and a locator that carries none reports no name at all.
    let named = match name {
        Some(name) => format!(" named {name:?}"),
        None => String::new(),
    };
    match target.resolve(&launched.model) {
        Resolution::Ambiguous(count) => Err(format!(
            "the locator for the {role}{named} matches {count} regions"
        )),
        Resolution::NoMatch => Err(format!(
            "the locator for the {role}{named} found this screen instead:\n{}",
            launched.screen.contents()
        )),
        Resolution::One(_) => Ok(Expectation {
            // A region's name is its content in this model, so the locator
            // states the whole of what was proved and no text is stated beside
            // it. A text repeating the name would be a second copy of one fact
            // in the plan the operator commits.
            text: String::new(),
            within: None,
            locator: Some(Locator {
                role: Some(role.to_string()),
                name: name.unwrap_or_default().to_string(),
                within: None,
                binding: None,
            }),
        }),
    }
}
