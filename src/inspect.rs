//! The inspect command: read the terminal object model of a running program and
//! render it for an operator discovering the roles and names to address.

use crate::backend::resolve;
use crate::pty::capture_interactive;
use crate::sandbox::{CommandSpec, SandboxSpec};
use crate::screen::VirtualScreen;
use crate::tom::{Model, Region, build, read_stream};
use std::path::Path;
use std::time::{Duration, Instant};

/// How long a program is given to finish before what it has drawn is read as
/// the screen it is showing, so a program that keeps running is reported rather
/// than waited on for ever.
pub(crate) const EXIT_DEADLINE: Duration = Duration::from_secs(2);

/// What one sandboxed launch produced: the model of what the program drew, the
/// screen that model was read from, and whether the program honoured the line
/// it was given. A program still running has not refused anything yet, so it
/// counts as honouring the line; one that exited unsuccessfully refused it.
///
/// @planks("the operator inspects {string} with its documented examples")
/// @planks("the inspect output reports the example {string} was refused")
#[derive(Debug)]
pub struct Run {
    pub model: Model,
    pub screen: VirtualScreen,
    pub honoured: bool,
}

/// Launch `command` under `spec` through the sandbox backend on a PTY and read
/// what it shows. A program that exits on its own has finished speaking, so its
/// whole output is read; a program still running has not, so what it has drawn
/// is read as a screen. Exiting is observable, so the reading follows the
/// program rather than a flag. The launched command is an unfamiliar program,
/// so it runs isolated over `workspace`, reading that tree through an overlay
/// whose writes never reach it, and the backend is the only thing that prepares
/// it.
///
/// @planks("the operator inspects {string} with its documented examples")
/// @planks("the operator inspects {string} with its documented examples and writes a plan")
/// @planks("the operator inspects the fixture terminal program with its documented examples")
/// @planks("the operator inspects the fixture terminal program with its documented examples and writes a plan")
pub fn run(spec: &SandboxSpec, command: &str, workspace: &Path) -> Result<Run, String> {
    run_program(
        spec,
        &CommandSpec {
            program: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), command.to_string()],
        },
        workspace,
    )
}

/// The same launch, given the program to run and its arguments rather than a
/// command line for a shell to read. A shell answers for a name it carries as a
/// builtin before it looks for the program of that name, so asking a binary
/// what it says about itself means running that binary rather than a line about
/// it.
///
/// @planks("the inspect output reports {string} as the ground-truth example")
/// @planks("the inspect output reports the examples the program's own help carries")
/// @planks("the verifier checks the backend construction boundary")
pub fn run_program(
    spec: &SandboxSpec,
    command: &CommandSpec,
    workspace: &Path,
) -> Result<Run, String> {
    let resolved = resolve(std::env::consts::OS).map_err(|e| format!("{e:?}"))?;
    let prepared = resolved
        .backend()
        .prepare_over_tree(spec, command, workspace, None)?;
    let mut session = capture_interactive(&prepared)?;
    let deadline = Instant::now() + EXIT_DEADLINE;
    while !session.finished() {
        if Instant::now() >= deadline {
            let screen = session.screen();
            return Ok(Run {
                model: build(&screen),
                screen,
                honoured: true,
            });
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let honoured = session.exit_status().success();
    let screen = session.stream();
    Ok(Run {
        model: read_stream(&screen),
        screen,
        honoured,
    })
}

/// The model of what `command` shows, read through the default sandbox.
///
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the fixture terminal program as JSON")
/// @planks("the operator inspects the command {string}")
/// @planks("the operator inspects a command that writes to the sentinel path and prints {string}")
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing two two-line blocks separated by a blank line")
/// @planks("the operator inspects a command printing {string}, a blank line and {string}")
/// @planks("the operator inspects a command that prints {string} in red")
/// @planks("the operator inspects a command that writes {string} into its working directory and prints {string}")
/// @planks("the operator inspects a command that prints the contents of {string}")
/// @planks("the operator inspects {string}")
/// @planks("the inspect output names the root region {string}")
pub fn model(command: &str, workspace: &Path) -> Result<Model, String> {
    let launched = run(&SandboxSpec::default(), command, workspace)?;
    // A program that exited unsuccessfully before drawing anything never
    // started, which is a different finding from one that ran and drew an
    // empty screen. Reporting both as "no regions on screen" would send an
    // operator looking for a drawing bug in a program that never launched.
    if !launched.honoured {
        return Err(format!("the program {command:?} did not start"));
    }
    Ok(named(launched.model, command))
}

/// Name the root region for the program the operator asked about. The root
/// carries the `application` role, and a listener told the role and not the
/// subject has been told half of it, so the model says which application this
/// is.
///
/// @planks("the inspect output names the root region {string}")
fn named(mut model: Model, command: &str) -> Model {
    model.root.name = Some(command.to_string());
    model
}

/// Render a model as the listing an operator reads: one line per region, nested
/// under the region containing it, naming the role it plays and the name a
/// locator matches. A model carrying no region reports that instead.
///
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the command {string}")
pub fn render(model: &Model) -> String {
    if model.root.children.is_empty() {
        return "no regions on screen".to_string();
    }
    let mut lines = Vec::new();
    describe(&model.root, 0, &mut lines);
    lines.join("\n")
}

/// Write `region` and everything nested inside it into `lines`, one line each,
/// indented by the depth the region sits at. A region drawn in a colour of its
/// own is named with that colour, because presentation is half of what a test
/// author decides whether to assert on; a region drawn in the terminal's own
/// colours is named without one, which is what keeps an ordinary listing short.
///
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects a command that prints {string} in red")
/// @planks("the operator inspects a command that prints {string} {attribute}")
fn describe(region: &Region, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let role = region.role();
    let name = match &region.name {
        Some(name) => format!(" {name:?}"),
        None => String::new(),
    };
    let colour = match region.colour() {
        Some(colour) => format!(" in {colour}"),
        None => String::new(),
    };
    let presentations = region
        .presentations()
        .iter()
        .map(|presentation| format!(" {presentation}"))
        .collect::<String>();
    lines.push(format!("{indent}{role}{name}{colour}{presentations}"));
    for child in &region.children {
        describe(child, depth + 1, lines);
    }
}
