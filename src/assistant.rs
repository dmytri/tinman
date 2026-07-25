//! The interactive assistant: it answers questions about Tinman and proposes
//! Tinman commands.
//!
//! Its scope is deliberately narrow. A proposal reaches the operating system
//! only through Tinman's own `parse_command_line`, so an inferred string can
//! name a Tinman subcommand and nothing else. There is no path from here to a
//! shell.

use crate::cli::parse_command_line;
use crate::inference::Settings;

/// The marker a proposing reply opens with. A reply carrying it names a Tinman
/// command; any other reply is prose the operator reads.
///
/// @planks("the assistant infers the command {string}")
const PROPOSAL_MARKER: &str = "COMMAND: ";

/// What the assistant made of the operator's question.
///
/// @planks("the operator asks {string}")
#[derive(Debug)]
pub enum Response {
    /// A Tinman command the operator may confirm or decline.
    Proposal(Proposal),
    /// Prose answering the question, naming no command.
    Answer(String),
    /// The reason the assistant offers nothing for what the model named.
    Refusal(String),
}

/// A proposed Tinman command, displayed to the operator and awaiting a decision.
///
/// @planks("the assistant has proposed the command {string}")
#[derive(Debug)]
pub struct Proposal {
    command: String,
}

impl Proposal {
    /// A proposal carrying `command`.
    ///
    /// @planks("the assistant has proposed the command {string}")
    pub fn new(command: &str) -> Proposal {
        Proposal {
            command: command.to_string(),
        }
    }

    /// The command line this proposal displays.
    ///
    /// @planks("the assistant displays the proposed command {string}")
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Decline the proposal, consuming it, so nothing reaches the parser.
    ///
    /// @planks("the operator declines the proposal")
    pub fn decline(self) {}

    /// Confirm the proposal, handing the command line to Tinman's parser and
    /// reporting the arguments the parser took.
    ///
    /// @planks("the operator confirms the proposal")
    pub fn confirm(self) -> Result<Vec<String>, String> {
        parse_command_line(&self.command)
    }
}

/// The model reply that proposes `command`.
///
/// @planks("the assistant infers the command {string}")
pub fn model_reply_proposing(command: &str) -> String {
    format!("{PROPOSAL_MARKER}{command}")
}

/// The model reply that answers with `answer` and names no command.
///
/// @planks("the assistant answers {string}")
pub fn model_reply_answering(answer: &str) -> String {
    answer.to_string()
}

/// Put the operator's question to the configured provider and read its reply: a
/// proposal when the named command line is one Tinman's parser takes, a refusal
/// when it is not, and an answer when the reply names no command.
///
/// @planks("the operator asks {string}")
pub fn ask(settings: &Settings, question: &str) -> Response {
    let reply = crate::inference::assistant_completion(settings, question)
        .expect("the configured provider answers the assistant");
    let reply = reply.trim();
    match reply.strip_prefix(PROPOSAL_MARKER) {
        Some(command) => match parse_command_line(command) {
            Ok(_) => Response::Proposal(Proposal::new(command)),
            Err(refusal) => Response::Refusal(refusal),
        },
        None => Response::Answer(reply.to_string()),
    }
}
