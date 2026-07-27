//! The interactive assistant: it answers questions about Tinman and proposes
//! Tinman commands.
//!
//! Its scope is deliberately narrow. A proposal reaches the operating system
//! only through Tinman's own `parse_command_line`, so an inferred string can
//! name a Tinman subcommand and nothing else. There is no path from here to a
//! shell.

use crate::cli::parse_command_line;
use crate::inference::{Exchange, Settings};

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
/// programs it drives is drawn with. The corners are the rounded ones.
const TOP_LEFT: &str = "\u{256d}";
const TOP_RIGHT: &str = "\u{256e}";
const BOTTOM_LEFT: &str = "\u{2570}";
const BOTTOM_RIGHT: &str = "\u{256f}";
const HORIZONTAL: &str = "\u{2500}";
const VERTICAL: &str = "\u{2502}";

/// The widest the box is drawn, however wide the terminal is.
const MAX_COLUMNS: usize = 80;

/// The escapes that draw the box in a colour other than the terminal's default
/// foreground, and put the default foreground back at the end of each line.
const COLOUR: &str = "\u{1b}[36m";
const DEFAULT_COLOUR: &str = "\u{1b}[39m";

/// The colours the two halves of an exchange are written above the box in. The
/// question carries the box's own colour and the answer the terminal's default
/// foreground, so the operator reads whose line each is without re-reading it,
/// and the answer they came for is drawn plainly.
const QUESTION_COLOUR: &str = COLOUR;
const ANSWER_COLOUR: &str = DEFAULT_COLOUR;

/// The bytes the terminal delivers when the operator ends the input. Ending the
/// input before the program takes raw control leaves the line discipline's own
/// end-of-file marker in the queue, which a raw read takes as the disabled
/// character, so both stand for the same operator action.
const END_OF_INPUT: [u8; 2] = [0x04, 0x00];

/// The byte the escape key sends.
const ESCAPE: u8 = 0x1b;

/// The byte the backspace key puts in the terminal's input queue.
const BACKSPACE: u8 = 0x7f;

/// The keys the box names while a call is pending. Escape abandons that call
/// rather than leaving the session, so the hint names what the key does now.
///
/// @planks("the assistant prompt names {string} as the key that cancels")
const PENDING_KEYS: &str = "esc to cancel";

/// How often the box is drawn again while a call is pending, which is how often
/// the seconds it reports can advance.
const TICK: std::time::Duration = std::time::Duration::from_millis(200);

/// A model call the box is waiting on: the moment it went, and where its reply
/// will arrive. The call runs on a thread of its own, so the box keeps drawing
/// while the call is pending and the operator can abandon it.
///
/// @planks("the region titled {string} shows the elapsed seconds of the pending call")
/// @planks("the reported elapsed seconds advance while the call is pending")
struct Pending {
    started: std::time::Instant,
    replies: std::sync::mpsc::Receiver<Response>,
    /// The question this call put to the model, held so the exchange can join
    /// the session's history once the reply arrives.
    asked: String,
}

/// The row the box reports a pending call on: the seconds already spent, which
/// advance while the call is pending, so an operator deciding whether to keep
/// waiting reads how long it has been rather than only that the program lives.
///
/// @planks("the region titled {string} shows the elapsed seconds of the pending call")
/// @planks("the reported elapsed seconds advance while the call is pending")
fn waiting_line(seconds: u64) -> String {
    format!("waiting for an answer, {seconds}s")
}

/// Put `question` to the model on a thread of its own, carrying `history`
/// ahead of it, and report where its reply will arrive, so the box keeps
/// drawing while the call is pending. A reply the operator has abandoned is
/// dropped where it lands, the receiver having gone with the abandoned call.
///
/// @planks("the operator types {string} at the assistant prompt")
/// @planks("the operator presses {string} at the assistant prompt")
fn call(settings: &Settings, history: Vec<Exchange>, question: &str) -> Pending {
    let (replied, replies) = std::sync::mpsc::channel();
    let settings = settings.clone();
    let asked = question.to_string();
    let thread_question = asked.clone();
    std::thread::spawn(move || {
        let _ = replied.send(ask_with_history(&settings, &history, &thread_question));
    });
    Pending {
        started: std::time::Instant::now(),
        replies,
        asked,
    }
}

/// Put the operator's questions to the model in a box drawn beneath the help,
/// until they end the input or press escape. The box claims only the rows
/// beneath the help, so the output the operator asked for stays where it is.
/// Each half of an exchange is written above the box as it happens, the question
/// as it was sent and then the answer, so the box holds only what is being
/// typed and the exchange accumulates in the terminal's own scrollback. While a
/// call is pending the box reports the seconds already spent and names escape as
/// the key that abandons it, so a wait no operator wants ends when they say so
/// rather than on the call's ceiling.
///
/// @planks("the operator types {string} at the assistant prompt")
/// @planks("{string} appears above the region titled {string}")
/// @planks("the region titled {string} does not show {string}")
/// @planks("the region titled {string} is the lowest region on the screen")
/// @planks("{string} is drawn in a different colour from {string}")
/// @planks("the operator presses {string} at the assistant prompt")
/// @planks("the operator ends the input")
/// @planks("a bordered region titled {string} is drawn")
/// @planks("the region titled {string} shows {string}")
/// @planks("the assistant prompt names {string} as the key that sends")
/// @planks("the assistant prompt names {string} as the key that leaves")
/// @planks("the region titled {string} is at most {int} columns wide")
/// @planks("the region titled {string} is drawn in a colour other than the default foreground")
/// @planks("no cell is drawn in a colour other than the default foreground")
/// @planks("the region titled {string} has the corner glyph {string}")
/// @planks("the cursor is inside the region titled {string}")
/// @planks("the cursor is one column past the {string} it shows")
/// @planks("the region titled {string} shows the elapsed seconds of the pending call")
/// @planks("the reported elapsed seconds advance while the call is pending")
/// @planks("the assistant prompt names {string} as the key that cancels")
/// @planks("the region titled {string} is drawn")
/// @planks("the command has not exited")
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
    writeln!(out).expect("the terminal takes the assistant box");
    crossterm::terminal::enable_raw_mode().expect("the terminal enters raw mode");
    draw(
        &mut out,
        &box_lines(width, title, &question, keys, coloured),
        true,
        cursor_column(&question),
    );
    // The keystrokes are read on a thread of their own and delivered one byte at
    // a time, so the loop below wakes on a tick while a call is pending and
    // still takes a keystroke the moment it lands. The end of the input is
    // delivered as the character a raw read reports it with, so it reaches the
    // loop the way an operator ending the input does.
    let (typed, keystrokes) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut input = std::io::stdin();
        let mut byte = [0u8; 1];
        loop {
            let read = input
                .read(&mut byte)
                .expect("the terminal input is readable");
            let delivered = if read == 0 { 0u8 } else { byte[0] };
            if typed.send(delivered).is_err() || read == 0 {
                return;
            }
        }
    });
    // A character outside ASCII arrives as several bytes, so bytes are held here
    // until they spell one, and the character is shown as the operator typed it.
    let mut partial: Vec<u8> = Vec::new();
    let mut pending: Option<Pending> = None;
    let mut history: Vec<Exchange> = Vec::new();
    loop {
        // Idle, the loop waits on the operator. With a call pending it wakes on
        // a tick as well, so the seconds the box reports advance while nothing
        // is typed.
        let keystroke = match &pending {
            Some(_) => match keystrokes.recv_timeout(TICK) {
                Ok(byte) => Some(byte),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            },
            None => match keystrokes.recv() {
                Ok(byte) => Some(byte),
                Err(_) => break,
            },
        };
        if let Some(byte) = keystroke {
            if byte == ESCAPE {
                // Escape abandons a pending call and keeps the session: the box
                // takes the next question where it left off. With nothing
                // pending it leaves.
                if pending.take().is_none() {
                    break;
                }
                draw(
                    &mut out,
                    &box_lines(width, title, &question, keys, coloured),
                    false,
                    cursor_column(&question),
                );
                continue;
            }
            if END_OF_INPUT.contains(&byte) {
                break;
            }
            if pending.is_none() {
                if byte == b'\r' {
                    // The question leaves the field the moment it is sent: it is
                    // written above the box as it was sent, the box is left
                    // empty for the next one, and the answer follows it above
                    // the box when the model replies.
                    let asked = std::mem::take(&mut question);
                    let empty = box_lines(width, title, &question, keys, coloured);
                    transcribe(
                        &mut out,
                        &asked,
                        QUESTION_COLOUR,
                        coloured,
                        &empty,
                        cursor_column(&question),
                    );
                    pending = Some(call(settings, history.clone(), &asked));
                } else {
                    if byte == BACKSPACE {
                        question.pop();
                    } else {
                        partial.push(byte);
                        if let Ok(typed) = std::str::from_utf8(&partial) {
                            question.push_str(typed);
                            partial.clear();
                        }
                    }
                    draw(
                        &mut out,
                        &box_lines(width, title, &question, keys, coloured),
                        false,
                        cursor_column(&question),
                    );
                }
            }
        }
        if let Some(waiting) = &pending {
            match waiting.replies.try_recv() {
                Ok(Response::Answer(replied)) => {
                    transcribe(
                        &mut out,
                        &replied,
                        ANSWER_COLOUR,
                        coloured,
                        &box_lines(width, title, &question, keys, coloured),
                        cursor_column(&question),
                    );
                    history.push(Exchange::new(&waiting.asked, &replied));
                    pending = None;
                }
                Ok(_) => {
                    draw(
                        &mut out,
                        &box_lines(width, title, &question, keys, coloured),
                        false,
                        cursor_column(&question),
                    );
                    pending = None;
                }
                Err(_) => {
                    let waited = waiting_line(waiting.started.elapsed().as_secs());
                    draw(
                        &mut out,
                        &box_lines(width, title, &waited, PENDING_KEYS, coloured),
                        false,
                        cursor_column(&question),
                    );
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode().expect("the terminal leaves raw mode");
    println!();
}

/// Write `text` above the box in `colour` and draw the box again beneath it, so
/// the line joins the terminal's own scrollback and the box stays the lowest
/// thing on the screen. The cursor rests on the input line, so the box is taken
/// back to its own top row and erased from there down; the line is written where
/// the box stood and the box follows it, which pushes the earlier exchange up
/// the way ordinary terminal output scrolls.
///
/// @planks("{string} appears above the region titled {string}")
/// @planks("the region titled {string} does not show {string}")
/// @planks("the region titled {string} is the lowest region on the screen")
/// @planks("{string} is drawn in a different colour from {string}")
fn transcribe(
    out: &mut impl std::io::Write,
    text: &str,
    colour: &str,
    coloured: bool,
    lines: &[String],
    column: usize,
) {
    let mut frame = format!("\r\u{1b}[{INPUT_LINE}A\u{1b}[J");
    for line in text.lines() {
        if coloured {
            frame.push_str(colour);
            frame.push_str(line);
            frame.push_str(DEFAULT_COLOUR);
        } else {
            frame.push_str(line);
        }
        frame.push_str("\r\n");
    }
    write!(out, "{frame}").expect("the terminal takes the assistant transcript");
    out.flush()
        .expect("the terminal takes the assistant transcript");
    draw(out, lines, true, column);
}

/// The lines the box is drawn from, each `width` cells wide: a top border
/// carrying the title, one body row carrying `content`, and a bottom border
/// carrying `hint`, read the same way the top border's title is read, as the
/// standing terminal idiom for a pane's key hints. A row wider than the box is
/// cut to it, so the border stands whatever the operator types. A coloured box
/// carries the colour escapes inside each line, so the cells the box claims are
/// the only ones drawn in it.
///
/// @planks("a bordered region titled {string} is drawn")
/// @planks("the region titled {string} shows {string}")
/// @planks("the region titled {string} has the corner glyph {string}")
/// @planks("the region titled {string} is drawn in a colour other than the default foreground")
/// @planks("no cell is drawn in a colour other than the default foreground")
fn box_lines(width: usize, title: &str, content: &str, hint: &str, coloured: bool) -> Vec<String> {
    let inner = width - 2;
    let mut lines = Vec::with_capacity(3);
    let named = cut(title, inner);
    let rule = HORIZONTAL.repeat(inner - named.chars().count());
    lines.push(format!("{TOP_LEFT}{named}{rule}{TOP_RIGHT}"));
    let shown = cut(content, inner);
    let padding = " ".repeat(inner - shown.chars().count());
    lines.push(format!("{VERTICAL}{shown}{padding}{VERTICAL}"));
    let shown_hint = cut(hint, inner);
    let rule = HORIZONTAL.repeat(inner - shown_hint.chars().count());
    lines.push(format!("{BOTTOM_LEFT}{shown_hint}{rule}{BOTTOM_RIGHT}"));
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
/// @planks("a bordered region titled {string} is drawn")
fn cut(text: &str, cells: usize) -> String {
    text.chars().take(cells).collect()
}

/// The body line the operator's question is drawn on, counted from the top
/// border, which is where the cursor is left.
const INPUT_LINE: usize = 1;

/// How far right of the box's left border the cursor sits while `question` is
/// being typed: past the border and past every character shown, which is where
/// the next one lands.
///
/// @planks("the cursor is inside the region titled {string}")
/// @planks("the cursor is one column past the {string} it shows")
fn cursor_column(question: &str) -> usize {
    1 + question.chars().count()
}

/// Draw `lines` where the cursor stands, or over the box already drawn there,
/// and leave the cursor at `column` of the input line, where the next character
/// the operator types will land. The cursor rests on the input line, so a redraw
/// takes it back up to the first line and each line is erased before it is
/// written again. The box is drawn in one write, so the terminal never shows
/// half a frame.
///
/// @planks("the region titled {string} shows {string}")
/// @planks("the cursor is inside the region titled {string}")
/// @planks("the cursor is one column past the {string} it shows")
fn draw(out: &mut impl std::io::Write, lines: &[String], first: bool, column: usize) {
    let mut frame = String::new();
    if !first {
        frame.push_str(&format!("\r\u{1b}[{INPUT_LINE}A"));
    }
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            frame.push_str("\r\n");
        }
        frame.push_str("\u{1b}[2K");
        frame.push_str(line);
    }
    frame.push_str(&format!(
        "\r\u{1b}[{}A\u{1b}[{column}C",
        lines.len() - 1 - INPUT_LINE
    ));
    write!(out, "{frame}").expect("the terminal takes the assistant box");
    out.flush().expect("the terminal takes the assistant box");
}

/// Put the operator's question to the configured provider and read its reply: a
/// proposal when the named command line is one Tinman's parser takes, a refusal
/// when it is not, and an answer when the reply names no command.
///
/// @planks("the operator asks {string}")
pub fn ask(settings: &Settings, question: &str) -> Response {
    ask_with_history(settings, &[], question)
}

/// `ask`, carrying `history`'s compacted transcript ahead of `question`.
///
/// @planks("the operator asks {string}")
fn ask_with_history(settings: &Settings, history: &[Exchange], question: &str) -> Response {
    let reply = crate::inference::assistant_completion_for_turn(settings, history, question)
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
