//! Parsing of the `tinman record <program> [args...]` command line into the
//! backend-neutral command specification, and the recording session that
//! collects key presses and screen snapshots into a constrained interaction
//! log.

use crate::sandbox::CommandSpec;
use crate::screen::VirtualScreen;

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
