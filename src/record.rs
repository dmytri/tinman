//! Parsing of the `tinman record <program> [args...]` command line into the
//! backend-neutral command specification, and the recording session that
//! collects key presses and screen snapshots into a constrained interaction
//! log.

use crate::bwrap::BubblewrapBackend;
use crate::inference::Settings;
use crate::plan::{Action, Expectation, FlowStep, Plan, TuiProcess};
use crate::pty::{InteractiveCapture, capture_interactive_at};
use crate::sandbox::{CommandSpec, SandboxSpec};
use crate::screen::VirtualScreen;
use crate::tom::{Binding, Model, Region, build, confirm};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// Where a recording's plan is written when the operator names no path.
const DEFAULT_PLAN: &str = "tinman.yaml";

/// The characters that end an operator's input. Ctrl-D reaches a raw terminal
/// as the end-of-transmission character. A Ctrl-D the line discipline already
/// handled, because it was typed before the recording made the terminal raw,
/// reaches it as the disabled character the discipline substitutes for it.
const END_OF_INPUT: [u8; 2] = [0x04, 0x00];

/// How long the recorded program is given to draw its opening screen before
/// that screen is taken as it stands, so a program that draws nothing is
/// recorded rather than waited on.
const DRAW_DEADLINE: Duration = Duration::from_secs(2);

/// Parse a `tinman record <program> [args...]` invocation into the command
/// specification that names the target program and its arguments.
///
/// @planks("the operator runs {string}")
pub fn parse_command_line(line: &str) -> Result<CommandSpec, String> {
    let mut tokens = line.split_whitespace().skip(2).map(str::to_string);
    let program = tokens
        .next()
        .ok_or_else(|| format!("no target program in {line:?}"))?;
    let args: Vec<String> = tokens.collect();
    Ok(CommandSpec { program, args })
}

/// One recorded event, in capture order: either a key press or a screen
/// snapshot. Serialized untagged so a key event is `{key: ...}` and a snapshot
/// event is `{snapshot: ...}`, matching the interaction-log schema's `oneOf`.
#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
enum Event {
    Key { key: String },
    Snapshot { snapshot: Snapshot },
}

/// A captured screen snapshot: the grid dimensions and each row's text.
#[derive(Debug, serde::Serialize)]
struct Snapshot {
    rows: usize,
    cols: usize,
    lines: Vec<String>,
}

/// The serializable interaction log: the launched command and the events in
/// capture order.
#[derive(serde::Serialize)]
struct InteractionLog<'a> {
    command: LogCommand<'a>,
    events: &'a [Event],
}

/// The launched target command, as the interaction log records it.
#[derive(serde::Serialize)]
struct LogCommand<'a> {
    program: &'a str,
    args: &'a [String],
}

/// A recording session: the launched command and the key presses and snapshots
/// captured while driving it, in order.
///
/// @planks("a recording session")
#[derive(Debug)]
pub struct RecordingSession {
    command: CommandSpec,
    events: Vec<Event>,
}

impl RecordingSession {
    /// Start a recording session with no launched command yet.
    ///
    /// @planks("a recording session")
    pub fn new() -> RecordingSession {
        RecordingSession {
            command: CommandSpec {
                program: String::new(),
                args: Vec::new(),
            },
            events: Vec::new(),
        }
    }

    /// Start a recording session for a launched command.
    ///
    /// @planks("a recording session for {string}")
    pub fn for_command(command: CommandSpec) -> RecordingSession {
        RecordingSession {
            command,
            events: Vec::new(),
        }
    }

    /// Record a key press.
    ///
    /// @planks("the operator presses the key {string}")
    pub fn press_key(&mut self, key: &str) {
        self.events.push(Event::Key {
            key: key.to_string(),
        });
    }

    /// Record a screen snapshot.
    ///
    /// @planks("the operator takes a screen snapshot")
    pub fn snapshot(&mut self, screen: &VirtualScreen) {
        let grid = screen.rows();
        let rows = grid.len();
        let cols = grid.first().map(|row| row.len()).unwrap_or(0);
        let lines = grid.iter().map(|row| row.concat()).collect();
        self.events.push(Event::Snapshot {
            snapshot: Snapshot { rows, cols, lines },
        });
    }

    /// The recorded key presses, in order.
    ///
    /// @planks("the session's recorded key events are {string}, {string}, {string} in that order")
    pub fn recorded_keys(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Event::Key { key } => Some(key.clone()),
                Event::Snapshot { .. } => None,
            })
            .collect()
    }

    /// Write the session as a constrained YAML interaction log.
    ///
    /// @planks("the session is written as a YAML interaction log")
    pub fn to_interaction_log(&self) -> String {
        let log = InteractionLog {
            command: LogCommand {
                program: &self.command.program,
                args: &self.command.args,
            },
            events: &self.events,
        };
        serde_yaml::to_string(&log).expect("interaction log serializes to YAML")
    }
}

impl Default for RecordingSession {
    fn default() -> RecordingSession {
        RecordingSession::new()
    }
}

/// Record a live session with `command` and write it into `workspace` as an
/// editable plan, at `output` when the operator names a path and at the default
/// plan file otherwise. The operator drives the program through their own
/// terminal, every key they press is recorded and forwarded, and the session
/// ends when they end their input. A plan already at that path is left as it
/// stands, so a recording never overwrites the operator's own file.
///
/// A recording that cannot replay itself is a draft rather than a recording, so
/// the locators the recording addresses are proved while the operator is still
/// present: every name the opening screen carried must still be on the screen
/// the session ended on. A recording whose regions are renamed between draws is
/// reported instead of written.
///
/// @planks("the operator records the command {string} and presses {string}")
/// @planks("the operator records the command {string}")
/// @planks("the operator records the command {string} with {string}")
/// @planks("the operator records the fixture terminal program")
/// @planks("the operator records that program")
/// @planks("the written plan names the command {string}")
/// @planks("the written plan records a key press {string}")
/// @planks("the written plan carries an expectation on the text {string}")
/// @planks("the plan is written to {string}")
/// @planks("recording fails and reports the file already exists")
/// @planks("the operator records a command that writes to the sentinel path and prints {string}")
/// @planks("the operator records a command that prints {string} and the value of {string}")
/// @planks("the operator records a command that writes {string} into its working directory and prints {string}")
pub fn record(command: &CommandSpec, workspace: &Path, output: Option<&str>) -> Result<(), String> {
    let path = workspace.join(output.unwrap_or(DEFAULT_PLAN));
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    // The recorded program is the operator's target rather than a program this
    // project wrote, so it is launched through the sandbox backend, over the
    // workspace it reads through an overlay whose writes never reach that tree,
    // exactly as replaying the written plan launches it.
    let prepared = BubblewrapBackend::new().prepare_over_tree(
        &SandboxSpec::default_for_record(),
        &CommandSpec {
            program: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), command_line(command)],
        },
        workspace,
        None,
    )?;
    crossterm::terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let mut capture = capture_interactive_at(&prepared, None)?;
    let (opening, opening_view) = opening_screen(&mut capture);
    let mut session = RecordingSession::for_command(command.clone());

    let mut keys = std::io::stdin();
    let mut byte = [0u8; 1];
    loop {
        let read = keys.read(&mut byte).map_err(|e| e.to_string())?;
        if read == 0 || END_OF_INPUT.contains(&byte[0]) {
            break;
        }
        let key = key_name(byte[0]);
        session.press_key(&key);
        if !capture.finished() {
            capture.press_key(&key);
        }
    }
    crossterm::terminal::disable_raw_mode().map_err(|e| e.to_string())?;

    capture.end_session();
    let final_screen = capture.screen();
    let closing = build(&final_screen);
    if let Some(name) = renamed_region(&opening, &closing) {
        return Err(format!(
            "the plan did not replay: the region named {name:?} the recording addresses is not on the screen the session ended on"
        ));
    }

    let mut steps = Vec::new();
    let expectation_text = screen_text(&opening_view);
    if !expectation_text.is_empty() {
        steps.push(Action::Expect(Expectation {
            text: expectation_text,
            within: None,
            locator: None,
        }));
    }
    steps.extend(session.recorded_keys().into_iter().map(Action::Press));

    let plan = Plan {
        sandbox: SandboxSpec::default_for_record(),
        flow: vec![FlowStep::Tui(TuiProcess {
            command: command_line(command),
            steps,
        })],
        sources: Vec::new(),
    };
    let written = serde_yaml::to_string(&plan).map_err(|e| e.to_string())?;
    std::fs::write(&path, written)
        .map_err(|e| format!("the plan {} was not written: {e}", path.display()))
}

/// Record an expectation on the region playing `role` and carrying `name`.
/// The operator is present now and absent at replay, so an expectation whose
/// locator needed a fallback is refused here rather than rebinding silently at
/// replay to a region nobody named.
///
/// @planks("an expectation on that item is recorded")
/// @planks("recording fails and reports the expectation's locator did not bind")
pub fn record_expectation(model: &Model, role: &str, name: &str) -> Result<(), String> {
    plan_expecting(model, role, name).map(|_| ())
}

/// The plan an expectation on the region playing `role` and carrying `name` is
/// written into: one expect step addressing the region by the locator
/// confirmation reached, recording the binding that locator needed.
///
/// @planks("the plan is written")
/// @planks("the plan records the locator's binding as {string}")
pub fn plan_expecting(model: &Model, role: &str, name: &str) -> Result<Plan, String> {
    let confirmed = confirm(model, role, name)
        .filter(|confirmed| confirmed.binding != Binding::Ordinal)
        .ok_or_else(|| format!("the expectation on the {role} named {name:?} did not bind"))?;
    let locator = crate::plan::Locator {
        role: Some(role.to_string()),
        name: name.to_string(),
        within: confirmed.locator.scope().map(str::to_string),
        binding: Some(confirmed.binding.as_str().to_string()),
    };
    Ok(Plan {
        sandbox: SandboxSpec::default(),
        flow: vec![FlowStep::Tui(TuiProcess {
            command: String::new(),
            steps: vec![Action::Expect(Expectation {
                text: name.to_string(),
                within: None,
                locator: Some(locator),
            })],
        })],
        sources: Vec::new(),
    })
}

/// The plan an expectation on the region playing `role` and carrying `name` is
/// written into, for a naming pass over `program`. The pass reads that program's
/// tldr page where the configured source carries one, and a page is somebody
/// else's work, so the plan says whose it read and on what terms. A pass that
/// read no page credits nothing.
///
/// @planks("the plan for {string} is written")
/// @planks("the plan credits the tldr-pages project for the page it read")
/// @planks("the plan names {string} as that page's licence")
/// @planks("the plan credits no page")
pub fn plan_expecting_for(
    model: &Model,
    role: &str,
    name: &str,
    program: &str,
    settings: &Settings,
) -> Result<Plan, String> {
    let mut plan = plan_expecting(model, role, name)?;
    if crate::examples::fetch_page(&settings.tldr_base_url, program).is_some() {
        plan.sources.push(crate::examples::page_credit(program));
    }
    Ok(plan)
}

/// The command line that launches `command`: the program and every argument it
/// takes, as the operator typed them.
///
/// @planks("the operator records the command {string}")
fn command_line(command: &CommandSpec) -> String {
    let mut line = command.program.clone();
    for arg in &command.args {
        line.push(' ');
        line.push_str(arg);
    }
    line
}

/// The key a pressed byte names. Carriage return is the Enter key, as a
/// terminal sends it; any other byte is the key's own text.
///
/// @planks("the operator records the command {string} and presses {string}")
fn key_name(byte: u8) -> String {
    if byte == b'\r' {
        "Enter".to_string()
    } else {
        (byte as char).to_string()
    }
}

/// The model of the screen the recorded program opens on, taken the moment it
/// carries a region or the program finishes, together with the raw screen that
/// model was read from.
///
/// @planks("the operator records the fixture terminal program")
/// @planks("the written plan carries an expectation on the text {string}")
fn opening_screen(capture: &mut InteractiveCapture) -> (Model, VirtualScreen) {
    let deadline = Instant::now() + DRAW_DEADLINE;
    loop {
        let screen = capture.screen();
        let model = build(&screen);
        if !model.root.children.is_empty() || Instant::now() >= deadline {
            return (model, screen);
        }
        if capture.finished() {
            // The program has exited, but the background reader may not yet
            // have drained its last output into the screen: give it one more
            // moment before taking the screen as it stands.
            std::thread::sleep(Duration::from_millis(20));
            let screen = capture.screen();
            let model = build(&screen);
            return (model, screen);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The text of the line the opening screen carried, trimmed, read from the
/// bottom so a screen whose content sits above blank rows still yields it.
/// Recording this line as the plan's first expectation proves the locator
/// while the operator is still present, rather than leaving that proof to
/// whoever replays the plan after they are gone.
///
/// @planks("the written plan carries an expectation on the text {string}")
fn screen_text(screen: &VirtualScreen) -> String {
    screen
        .rows()
        .iter()
        .rev()
        .map(|row| row.concat().trim_end().to_string())
        .find(|line| !line.is_empty())
        .unwrap_or_default()
}

/// The name the opening screen carried that the closing screen no longer
/// carries, so a program that renames a region between draws names the locator
/// its recording cannot replay.
///
/// @planks("the operator records that program")
fn renamed_region(opening: &Model, closing: &Model) -> Option<String> {
    let closing = region_names(closing);
    region_names(opening)
        .into_iter()
        .find(|name| !closing.contains(name))
}

/// Every name a screen's regions carry, which is what a locator addresses.
///
/// @planks("the operator records that program")
fn region_names(model: &Model) -> Vec<String> {
    let mut names = Vec::new();
    collect_region_names(&model.root, &mut names);
    names
}

/// Write the name of `region` and of everything nested inside it into `names`.
///
/// @planks("the operator records that program")
fn collect_region_names(region: &Region, names: &mut Vec<String>) {
    if let Some(name) = &region.name {
        names.push(name.clone());
    }
    for child in &region.children {
        collect_region_names(child, names);
    }
}
