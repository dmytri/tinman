//! The interactive assistant: it answers questions about Tinman and proposes
//! Tinman commands.
//!
//! Its scope is deliberately narrow. A proposal reaches the operating system
//! only through Tinman's own `parse_command_line`, so an inferred string can
//! name a Tinman subcommand and nothing else. There is no path from here to a
//! shell.

use crate::cli::parse_command_line;
use crate::inference::{Exchange, Settings};
use ratatui::backend::IntoCrossterm;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget, Wrap};

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
    answer: Option<String>,
}

impl Proposal {
    /// A proposal carrying `command` and no answer beside it.
    ///
    /// @planks("the assistant has proposed the command {string}")
    pub fn new(command: &str) -> Proposal {
        Proposal {
            command: command.to_string(),
            answer: None,
        }
    }

    /// The command line this proposal displays.
    ///
    /// @planks("the assistant displays the proposed command {string}")
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The prose the reply carried above the marked line, which the assistant
    /// displays beside the proposed command.
    ///
    /// @planks("the assistant displays the answer {string}")
    pub fn answer(&self) -> Option<&str> {
        self.answer.as_deref()
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

/// The widest the box is drawn, and the measure the transcript is wrapped to,
/// however wide the terminal is. One measure for the whole screen: a transcript
/// wrapped at the terminal beside a box capped somewhere else would give one
/// screen two measures.
///
/// @planks("the region titled {string} is at most {int} columns wide")
/// @planks("no line of the answer is wider than {int} columns")
/// @planks("the terminal object model of the screen conforms to the {string} schema in {string}")
const MAX_COLUMNS: usize = 72;

/// The escapes that put the terminal on the alternate screen and take it back
/// off again, DEC private mode 1049. The session redraws itself constantly, so
/// it draws on a screen of its own and the operator's scrollback is theirs until
/// the session leaves.
const ALTERNATE_SCREEN: &str = "\u{1b}[?1049h";
const MAIN_SCREEN: &str = "\u{1b}[?1049l";

/// The escapes that draw the box in a colour other than the terminal's default
/// foreground, and put the default foreground back at the end of each line.
const COLOUR: &str = "\u{1b}[36m";
const DEFAULT_COLOUR: &str = "\u{1b}[39m";

/// The background the operator's own turn is drawn as a block of, running the
/// full measure of the transcript, so the operator reads at a glance where their
/// turn started. Colour laid on the words themselves would read as a highlighter
/// smear over them; a block is what marks authorship.
const QUESTION_BACKGROUND: Color = Color::Blue;

/// The marker the operator's turn carries where colour is off. A background
/// block degrades to no structure at all, so the boundary is drawn with a
/// character instead of with colour.
const QUESTION_MARKER: &str = "> ";

/// The marker a fenced block opens and closes with in the markdown the model
/// writes. The block it delimits is drawn as a block, so the marker itself is
/// notation the operator reads past rather than content they came for.
const FENCE: &str = "```";

/// The bytes the terminal delivers when the operator ends the input. Ending the
/// input before the program takes raw control leaves the line discipline's own
/// end-of-file marker in the queue, which a raw read takes as the disabled
/// character, so both stand for the same operator action.
const END_OF_INPUT: [u8; 2] = [0x04, 0x00];

/// The byte the escape key sends.
const ESCAPE: u8 = 0x1b;

/// The byte the backspace key puts in the terminal's input queue.
const BACKSPACE: u8 = 0x7f;

/// The signal an interrupt at the terminal delivers.
const INTERRUPT: i32 = 2;

/// The status a program interrupted at the terminal exits with: the standing
/// convention of 128 above the signal it took.
const INTERRUPTED_STATUS: i32 = 130;

unsafe extern "C" {
    /// The C library call that says what happens when a signal arrives. It is
    /// declared here rather than reached through a crate: the C library is
    /// already linked into the program, and one disposition is the whole of
    /// what handing the terminal back needs.
    fn signal(signal: i32, disposition: extern "C" fn(i32)) -> usize;
}

/// Hand the terminal back and go. An interrupt takes the program where it
/// stands, so the restoration at the end of the session never runs: the operator
/// gets their terminal back here instead, before the process goes. Both halves
/// of what the session took are given back, the raw mode and the screen it drew
/// on, so the operator meets their own screen rather than the box's remains.
///
/// @planks("the operator interrupts the session")
/// @planks("the terminal is out of raw mode")
/// @planks("the alternate screen has been left")
extern "C" fn hand_the_terminal_back(_signal: i32) {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut out = std::io::stdout();
    let _ = std::io::Write::write_all(&mut out, MAIN_SCREEN.as_bytes());
    let _ = std::io::Write::flush(&mut out);
    std::process::exit(INTERRUPTED_STATUS);
}

/// The title, the sending and leaving key hints, and the pending-call hint the
/// box draws, read from the first three lines of the prompt asset. The box reads
/// its copy from that asset each time it draws, so an edited asset is a live
/// reading rather than a hand-kept copy.
///
/// @planks("the assistant prompt names {string} as the key that cancels")
fn prompt_copy() -> (&'static str, &'static str, &'static str) {
    let mut lines = PROMPT.trim().lines();
    let title = lines.next().unwrap_or_default();
    let keys = lines.next().unwrap_or_default();
    let pending_keys = lines.next().unwrap_or_default();
    (title, keys, pending_keys)
}

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

/// Put the operator's questions to the model in a box, until they end the input
/// or press escape. The session runs on the alternate screen, so every redraw it
/// makes lands there and the operator's own scrollback is theirs until the
/// session leaves. Each half of an exchange is written above the box as it
/// happens, the question as it was sent and then the answer, so the box holds
/// only what is being typed and the exchange accumulates above it. While a
/// call is pending the box reports the seconds already spent and names escape as
/// the key that abandons it, so a wait no operator wants ends when they say so
/// rather than on the call's ceiling.
///
/// @planks("the terminal is on the alternate screen")
/// @planks("{string} is not on the screen")
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
/// @planks("the operator interrupts the session")
pub fn converse(settings: &Settings) -> std::io::Result<()> {
    use std::io::{Read, Write};
    let (title, keys, pending_keys) = prompt_copy();
    let columns = crossterm::terminal::size()?.0 as usize;
    let width = columns.min(MAX_COLUMNS);
    let coloured = std::env::var_os("NO_COLOR").is_none();
    let mut out = std::io::stdout();
    let mut question = String::new();
    crossterm::terminal::enable_raw_mode()?;
    write!(out, "{ALTERNATE_SCREEN}")?;
    writeln!(out)?;
    // The session owns the terminal however it leaves, and an interrupt is the
    // way out that runs none of the code below.
    unsafe { signal(INTERRUPT, hand_the_terminal_back) };
    draw(
        &mut out,
        &box_lines(width, title, &question, keys, coloured),
        true,
        cursor_column(&question),
    )?;
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
            let Ok(read) = input.read(&mut byte) else {
                return;
            };
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
                )?;
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
                        &question_lines(&asked, width, coloured),
                        &empty,
                        cursor_column(&question),
                    )?;
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
                    )?;
                }
            }
        }
        if let Some(waiting) = &pending {
            match waiting.replies.try_recv() {
                Ok(Response::Answer(replied)) => {
                    transcribe(
                        &mut out,
                        &answer_lines(&replied, width, coloured),
                        &box_lines(width, title, &question, keys, coloured),
                        cursor_column(&question),
                    )?;
                    history.push(Exchange::new(&waiting.asked, &replied));
                    pending = None;
                }
                Ok(_) => {
                    draw(
                        &mut out,
                        &box_lines(width, title, &question, keys, coloured),
                        false,
                        cursor_column(&question),
                    )?;
                    pending = None;
                }
                Err(_) => {
                    let waited = waiting_line(waiting.started.elapsed().as_secs());
                    draw(
                        &mut out,
                        &box_lines(width, title, &waited, pending_keys, coloured),
                        false,
                        cursor_column(&question),
                    )?;
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    write!(out, "{MAIN_SCREEN}")?;
    out.flush()?;
    println!();
    Ok(())
}

/// Write `written` above the box and draw the box again beneath it, so the lines
/// join the terminal's own scrollback and the box stays the lowest thing on the
/// screen. The cursor rests on the input line, so the box is taken back to its
/// own top row and erased from there down; the lines are written where the box
/// stood and the box follows them, which pushes the earlier exchange up the way
/// ordinary terminal output scrolls.
///
/// @planks("{string} appears above the region titled {string}")
/// @planks("the region titled {string} does not show {string}")
/// @planks("the region titled {string} is the lowest region on the screen")
/// @planks("{string} is drawn in a different colour from {string}")
fn transcribe(
    out: &mut impl std::io::Write,
    written: &[String],
    lines: &[String],
    column: usize,
) -> std::io::Result<()> {
    let mut frame = format!("\r\u{1b}[{INPUT_LINE}A\u{1b}[J");
    for line in written {
        frame.push_str(line);
        frame.push_str("\r\n");
    }
    write!(out, "{frame}")?;
    out.flush()?;
    draw(out, lines, true, column)
}

/// The lines the operator's own turn is written above the box as: the question
/// laid out to `measure` and drawn as a block of background colour running the
/// full measure. Where colour is off the block would leave no structure behind,
/// so the marker leads each line instead.
///
/// @planks("{string} is drawn on a background other than the default background")
/// @planks("that background runs the full measure of the transcript")
/// @planks("the question is drawn with the leading marker {string}")
/// @planks("no cell is drawn on a background other than the default background")
fn question_lines(question: &str, measure: usize, coloured: bool) -> Vec<String> {
    let text = match coloured {
        true => Text::raw(question.to_string()),
        false => Text::raw(format!("{QUESTION_MARKER}{question}")),
    };
    let mut laid_out = laid_out(text, measure);
    if coloured {
        laid_out.set_style(laid_out.area, Style::new().bg(QUESTION_BACKGROUND));
    }
    painted(&laid_out, coloured)
}

/// The lines the model's answer is written above the box as: the reply read as
/// the markdown the model writes, laid out to `measure`, so a fenced block is
/// drawn as the block it is rather than printed with its fence, and each line of
/// prose is drawn plainly on the terminal's own background.
///
/// @planks("no line of the answer is wider than {int} columns")
/// @planks("{string} is drawn in a different colour from {string}")
/// @planks("{string} is drawn on the default background")
/// @planks("no line on the screen carries a fence marker")
fn answer_lines(answer: &str, measure: usize, coloured: bool) -> Vec<String> {
    let mut text = tui_markdown::from_str(answer);
    text.lines.retain(|line| {
        !line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
            .trim_start()
            .starts_with(FENCE)
    });
    painted(&laid_out(text, measure), coloured)
}

/// `text` laid out `measure` columns wide, wrapped at its words, in a buffer of
/// exactly the rows it took. The rows a transcript takes are what the layout
/// decides, so they are read back off the laid-out buffer rather than counted
/// ahead of it.
///
/// @planks("the answer is drawn on more than one line")
/// @planks("no line of the answer is wider than {int} columns")
fn laid_out(text: Text<'_>, measure: usize) -> Buffer {
    let width = u16::try_from(measure).unwrap_or(u16::MAX);
    // A wrapped line takes at most one row per column it holds, which bounds the
    // buffer the layout is drawn into before the layout is known.
    let rows: usize = text.lines.iter().map(|line| line.width().max(1)).sum();
    let rows = u16::try_from(rows).unwrap_or(u16::MAX);
    let area = Rect::new(0, 0, width, rows);
    let mut buffer = Buffer::empty(area);
    Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .render(area, &mut buffer);
    let drawn = (0..rows)
        .rev()
        .find(|row| {
            (0..width).any(|column| {
                buffer
                    .cell((column, *row))
                    .is_some_and(|cell| cell.symbol() != " ")
            })
        })
        .map(|row| row + 1)
        .unwrap_or(1);
    buffer.resize(Rect::new(0, 0, width, drawn));
    buffer
}

/// The terminal lines `buffer` was laid out as, each carrying the colours its
/// cells were drawn in, or the characters alone where colour is off.
///
/// @planks("{string} is drawn on a background other than the default background")
/// @planks("{string} is drawn in a different colour from {string}")
/// @planks("no cell is drawn on a background other than the default background")
fn painted(buffer: &Buffer, coloured: bool) -> Vec<String> {
    use crossterm::style::{ResetColor, SetBackgroundColor, SetForegroundColor};

    let mut lines = Vec::with_capacity(buffer.area.height as usize);
    for row in 0..buffer.area.height {
        let mut line = String::new();
        let mut carried: Option<(Color, Color)> = None;
        for column in 0..buffer.area.width {
            let Some(cell) = buffer.cell((column, row)) else {
                continue;
            };
            if coloured && carried != Some((cell.fg, cell.bg)) {
                line.push_str(&SetForegroundColor(cell.fg.into_crossterm()).to_string());
                line.push_str(&SetBackgroundColor(cell.bg.into_crossterm()).to_string());
                carried = Some((cell.fg, cell.bg));
            }
            line.push_str(cell.symbol());
        }
        if coloured {
            line.push_str(&ResetColor.to_string());
        }
        lines.push(line);
    }
    lines
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
fn draw(
    out: &mut impl std::io::Write,
    lines: &[String],
    first: bool,
    column: usize,
) -> std::io::Result<()> {
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
    write!(out, "{frame}")?;
    out.flush()
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
/// @planks("the assistant displays the answer {string}")
fn ask_with_history(settings: &Settings, history: &[Exchange], question: &str) -> Response {
    let Some(reply) = crate::inference::assistant_completion_for_turn(settings, history, question)
    else {
        return Response::Refusal("the configured provider answered nothing".to_string());
    };
    let reply = reply.trim();
    let marked = reply
        .lines()
        .position(|line| line.starts_with(PROPOSAL_MARKER));
    match marked {
        Some(index) => {
            let command = reply
                .lines()
                .nth(index)
                .and_then(|line| line.strip_prefix(PROPOSAL_MARKER))
                .unwrap_or_default()
                .trim();
            let answer = reply
                .lines()
                .take(index)
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string();
            match parse_command_line(command) {
                Ok(_) => Response::Proposal(Proposal {
                    command: command.to_string(),
                    answer: (!answer.is_empty()).then_some(answer),
                }),
                Err(refusal) => Response::Refusal(refusal),
            }
        }
        None => Response::Answer(reply.to_string()),
    }
}
