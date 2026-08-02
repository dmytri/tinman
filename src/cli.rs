//! The Tinman command line: the one parser every invocation goes through.

use clap::{CommandFactory, Parser, Subcommand};

/// The Tinman command line. The help flag and the help subcommand both render
/// the bundled help asset, so the parser's own help output is disabled for
/// both. The version flag is clap's own.
///
/// @planks("each is passed to the command parser")
/// @planks("the operator runs {string} with stdout redirected to a file")
#[derive(Debug, Parser)]
#[command(
    name = "tinman",
    version,
    disable_help_flag = true,
    disable_help_subcommand = true
)]
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
/// @planks("the operator runs {string} with stdout redirected to a file")
/// @planks("the operator executes {string}")
/// @planks("a session named {string} running {string}")
/// @planks("the operator launches a session against a target that prints whether it read {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator has expected {string} and pressed {string} in that session")
/// @planks("each is requested from {string}")
/// @planks("each is passed to the command parser")
/// @planks("{string} is passed to the command parser")
/// @planks("the accepted command set is read")
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
    /// Run a plan and report whether it passed
    Test {
        /// The plan file to run
        plan: std::path::PathBuf,
    },
    /// Start a named session against a program
    Launch {
        /// The name the session takes
        #[arg(long)]
        session: String,
        /// The terminal program the session runs, and the arguments it takes
        #[arg(required = true)]
        command: Vec<String>,
    },
    /// Send one key to the program
    Press {
        /// The session the key is sent to
        #[arg(long)]
        session: String,
        /// The key the program is sent
        key: String,
    },
    /// State one expectation against the screen
    Expect {
        /// The role the expected region plays
        #[arg(long)]
        role: Option<String>,
        /// The name the expected region carries
        #[arg(long)]
        name: Option<String>,
        /// The session the expectation is stated against
        #[arg(long)]
        session: Option<String>,
        /// Where the plan is written
        #[arg(long)]
        output: Option<String>,
        /// The plan whose steps reach the screen the expectation is read against
        #[arg(long)]
        after: Option<String>,
        /// The text expected where no role is named, and the terminal program
        /// the expectation is stated against
        #[arg(required = true)]
        expectation: Vec<String>,
    },
    /// End a named session
    Close {
        /// The name of the session to end
        #[arg(long)]
        session: String,
        /// Where the plan the session performed is written
        #[arg(long)]
        output: Option<String>,
    },
    /// Print the terminal object model of a running program
    Inspect {
        /// The terminal command to inspect
        command: String,
        /// Write the model as JSON
        #[arg(long)]
        json: bool,
        /// Run the examples the program's own help and its tldr page document
        #[arg(long)]
        examples: bool,
        /// Where the plan is written
        #[arg(long)]
        output: Option<String>,
    },
    /// Speak the JSON driver protocol on stdin and stdout
    Driver,
    /// Show this help
    Help,
    /// Write Tinman's manual page as roff on stdout
    Man {
        /// The command whose page is written, in place of Tinman's own
        command: Option<String>,
    },
    /// Write a completion script for the named shell on stdout
    Completions {
        /// The shell the script is written for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// The arguments Tinman's parser takes from a command line: every token after
/// the program name. A line naming a program other than `tinman`, or naming a
/// command the parser does not accept, is refused. A line naming no command is
/// the bare invocation, which the parser takes.
///
/// @planks("the operator asks {string}")
/// @planks("the operator confirms the proposal")
/// @planks("each is passed to the command parser")
pub fn parse_command_line(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = line.split_whitespace().map(str::to_string);
    let program = tokens
        .next()
        .ok_or_else(|| format!("{line:?} names no program"))?;
    if program != "tinman" {
        return Err(format!("{program:?} is not a Tinman command line"));
    }
    let arguments: Vec<String> = tokens.collect();
    if let Some(command) = arguments.first()
        && !accepted_commands().contains(command)
    {
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
