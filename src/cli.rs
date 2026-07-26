//! The Tinman command line: the one parser every invocation goes through.

use clap::{CommandFactory, Parser, Subcommand};

/// The Tinman command line. The help flag renders the bundled help asset, so
/// the parser's own help output is disabled. The version flag is clap's own.
///
/// @planks("each is passed to the command parser")
#[derive(Debug, Parser)]
#[command(name = "tinman", version, disable_help_flag = true)]
pub struct Cli {
    /// Show this help
    #[arg(short = 'h', long = "help")]
    pub help: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The commands Tinman accepts.
///
/// @planks("the operator records the command {string}")
/// @planks("the operator records the command {string} with {string}")
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Capture a live session into an editable plan
    Record {
        /// Where the plan is written
        #[arg(long)]
        output: Option<String>,
        /// The terminal program to record, and the arguments it takes
        #[arg(required = true)]
        command: Vec<String>,
    },
    /// Replay a recorded plan exactly, with no inference
    Replay,
    /// Run a plan and report whether it passed
    Test {
        /// The plan file to run
        plan: std::path::PathBuf,
    },
    /// Print the terminal object model of a running program
    Inspect {
        /// The terminal command to inspect
        command: String,
        /// Write the model as JSON
        #[arg(long)]
        json: bool,
    },
    /// Speak the JSON driver protocol on stdin and stdout
    Driver,
}

/// The arguments Tinman's parser takes from a command line: every token after
/// the program name. A line naming a program other than `tinman`, or naming a
/// command the parser does not accept, is refused.
///
/// @planks("the operator asks {string}")
/// @planks("the operator confirms the proposal")
pub fn parse_command_line(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = line.split_whitespace().map(str::to_string);
    let program = tokens
        .next()
        .ok_or_else(|| format!("{line:?} names no program"))?;
    if program != "tinman" {
        return Err(format!("{program:?} is not a Tinman command line"));
    }
    let arguments: Vec<String> = tokens.collect();
    let command = arguments
        .first()
        .ok_or_else(|| format!("{line:?} names no command"))?;
    if !accepted_commands().contains(command) {
        return Err(format!("tinman has no {command:?} command"));
    }
    Ok(arguments)
}

/// The commands the parser accepts, named as the operator types them.
///
/// @planks("the commands the parser accepts")
pub fn accepted_commands() -> Vec<String> {
    let mut command = Cli::command();
    command.build();
    command
        .get_subcommands()
        .map(|sub| sub.get_name().to_string())
        .collect()
}
