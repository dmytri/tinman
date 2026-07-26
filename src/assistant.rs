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

/// The prompt the box carries, inlined at build time: its first line titles the
/// box and its second names the keys that work it.
const PROMPT: &str = include_str!("../assets/help/assistant-prompt.txt");

/// The characters the box is drawn with, the same a pane Tinman reads in the
/// programs it drives is drawn with.
const TOP_LEFT: &str = "\u{250c}";
const TOP_RIGHT: &str = "\u{2510}";
const BOTTOM_LEFT: &str = "\u{2514}";
const BOTTOM_RIGHT: &str = "\u{2518}";
const HORIZONTAL: &str = "\u{2500}";
const VERTICAL: &str = "\u{2502}";

/// The widest the box is drawn, however wide the terminal is.
const MAX_COLUMNS: usize = 80;

/// The escapes that draw the box in a colour other than the terminal's default
/// foreground, and put the default foreground back at the end of each line.
const COLOUR: &str = "\u{1b}[36m";
const DEFAULT_COLOUR: &str = "\u{1b}[39m";

/// The bytes the terminal delivers when the operator ends the input. Ending the
/// input before the program takes raw control leaves the line discipline's own
/// end-of-file marker in the queue, which a raw read takes as the disabled
/// character, so both stand for the same operator action.
const END_OF_INPUT: [u8; 2] = [0x04, 0x00];

/// The byte the escape key sends.
const ESCAPE: u8 = 0x1b;

/// The byte the backspace key puts in the terminal's input queue.
const BACKSPACE: u8 = 0x7f;

/// Put the operator's questions to the model in a box drawn beneath the help,
/// until they end the input or press escape. The box claims only the rows
/// beneath the help, so the output the operator asked for stays where it is.
///
/// @planks("the operator types {string} at the assistant prompt")
/// @planks("the operator presses {string} at the assistant prompt")
/// @planks("the operator ends the input")
/// @planks("a bordered region titled {string} is drawn beneath it")
/// @planks("the region titled {string} shows {string}")
/// @planks("the assistant prompt names {string} as the key that sends")
/// @planks("the assistant prompt names {string} as the key that leaves")
/// @planks("the region titled {string} is {int} columns wide")
/// @planks("the region titled {string} is at most {int} columns wide")
/// @planks("the region titled {string} is drawn in a colour other than the default foreground")
/// @planks("no cell is drawn in a colour other than the default foreground")
pub fn converse(settings: &Settings) {
    use std::io::{Read, Write};
    let mut prompt = PROMPT.trim().lines();
    let title = prompt
        .next()
        .expect("the assistant prompt asset titles the box");
    let keys = prompt
        .next()
        .expect("the assistant prompt asset names the keys");
    let (columns, _) = crossterm::terminal::size().expect("the terminal reports its size");
    let width = (columns as usize).min(MAX_COLUMNS);
    let coloured = std::env::var_os("NO_COLOR").is_none();
    let mut out = std::io::stdout();
    let mut question = String::new();
    let mut answer = String::new();
    writeln!(out).expect("the terminal takes the assistant box");
    crossterm::terminal::enable_raw_mode().expect("the terminal enters raw mode");
    draw(
        &mut out,
        &box_lines(width, title, &[&question, &answer, keys], coloured),
        true,
    );
    let mut input = std::io::stdin();
    let mut byte = [0u8; 1];
    // A character outside ASCII arrives as several bytes, so bytes are held here
    // until they spell one, and the character is shown as the operator typed it.
    let mut partial: Vec<u8> = Vec::new();
    loop {
        let read = input
            .read(&mut byte)
            .expect("the terminal input is readable");
        if read == 0 || END_OF_INPUT.contains(&byte[0]) || byte[0] == ESCAPE {
            break;
        }
        if byte[0] == b'\r' {
            if let Response::Answer(replied) = ask(settings, &question) {
                answer = replied;
            }
        } else if byte[0] == BACKSPACE {
            question.pop();
        } else {
            partial.push(byte[0]);
            if let Ok(typed) = std::str::from_utf8(&partial) {
                question.push_str(typed);
                partial.clear();
            }
        }
        draw(
            &mut out,
            &box_lines(width, title, &[&question, &answer, keys], coloured),
            false,
        );
    }
    crossterm::terminal::disable_raw_mode().expect("the terminal leaves raw mode");
    println!();
}

/// The lines the box is drawn from, each `width` cells wide: a top border
/// carrying the title, one line for each body row, and a bottom border. A row
/// wider than the box is cut to it, so the border stands whatever the operator
/// types. A coloured box carries the colour escapes inside each line, so the
/// cells the box claims are the only ones drawn in it.
///
/// @planks("a bordered region titled {string} is drawn beneath it")
/// @planks("the region titled {string} shows {string}")
/// @planks("the region titled {string} is drawn in a colour other than the default foreground")
/// @planks("no cell is drawn in a colour other than the default foreground")
fn box_lines(width: usize, title: &str, body: &[&str], coloured: bool) -> Vec<String> {
    let inner = width - 2;
    let mut lines = Vec::with_capacity(body.len() + 2);
    let named = cut(title, inner);
    let rule = HORIZONTAL.repeat(inner - named.chars().count());
    lines.push(format!("{TOP_LEFT}{named}{rule}{TOP_RIGHT}"));
    for row in body {
        let shown = cut(row, inner);
        let padding = " ".repeat(inner - shown.chars().count());
        lines.push(format!("{VERTICAL}{shown}{padding}{VERTICAL}"));
    }
    lines.push(format!(
        "{BOTTOM_LEFT}{}{BOTTOM_RIGHT}",
        HORIZONTAL.repeat(inner)
    ));
    if coloured {
        lines = lines
            .iter()
            .map(|line| format!("{COLOUR}{line}{DEFAULT_COLOUR}"))
            .collect();
    }
    lines
}

/// `text` cut to the first `cells` characters.
///
/// @planks("a bordered region titled {string} is drawn beneath it")
fn cut(text: &str, cells: usize) -> String {
    text.chars().take(cells).collect()
}

/// Draw `lines` where the cursor stands, or over the box already drawn there.
/// The cursor rests at the end of the last line, so a redraw takes it back up to
/// the first and each line is erased before it is written again. The box is
/// drawn in one write, so the terminal never shows half a frame.
///
/// @planks("the region titled {string} shows {string}")
fn draw(out: &mut impl std::io::Write, lines: &[String], first: bool) {
    let mut frame = String::new();
    if !first {
        frame.push_str(&format!("\r\u{1b}[{}A", lines.len() - 1));
    }
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            frame.push_str("\r\n");
        }
        frame.push_str("\u{1b}[2K");
        frame.push_str(line);
    }
    write!(out, "{frame}").expect("the terminal takes the assistant box");
    out.flush().expect("the terminal takes the assistant box");
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
