//! The virtual screen: a terminal grid parsed from program output.
//!
//! A launched program's bytes are parsed by a real terminal emulator into a
//! fixed grid of cells. Steps assert what the program displayed by text or by
//! addressed cell, so the grid preserves cursor-positioned output exactly as a
//! terminal would render it. The emulator honours the whole family of cursor
//! addressing sequences a full-screen program emits, not the CUP form alone.

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Processor, StdSyncHandler};
use serde::{Deserialize, Serialize};

/// The number of rows in the virtual terminal grid, and the number of columns a
/// screen takes when the caller states no width. Matches the PTY size the
/// capture path opens, so a program's cursor addressing lands on the same cell
/// the screen records.
const ROWS: u16 = 24;
const COLS: u16 = 80;

/// The position and length of each erase-to-end-of-line sequence in `bytes`:
/// CSI K, with the parameter left out or written as zero.
///
/// @planks("the process is captured through a PTY")
fn erases_to_end_of_line(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut index = 0;
    while index + 2 < bytes.len() {
        if bytes[index..index + 2] == [0x1b, b'['] {
            let mut end = index + 2;
            while bytes.get(end) == Some(&b'0') {
                end += 1;
            }
            if bytes.get(end) == Some(&b'K') {
                found.push((index, end + 1 - index));
                index = end + 1;
                continue;
            }
        }
        index += 1;
    }
    found
}

/// The grid the emulator is built at: the virtual screen's own size, with no
/// scrollback, so the lines the emulator holds are the lines the screen shows.
/// The width is the width of the run, so a program driven on a wide terminal is
/// parsed at the width it drew on. The height is the height of the reading: a
/// screen is read at the terminal's own height, and a finished program's whole
/// output is read at a height that holds every line it wrote.
///
/// @planks("the process is captured through a PTY")
/// @planks("that plan is replayed at {int} columns")
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
struct ScreenSize {
    columns: u16,
    rows: u16,
}

impl Dimensions for ScreenSize {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }

    fn screen_lines(&self) -> usize {
        self.rows as usize
    }

    fn columns(&self) -> usize {
        self.columns as usize
    }
}

/// How many rows a grid needs to hold everything `bytes` carries without
/// scrolling any of it away: one row for each line the program wrote, and the
/// rows a line longer than the terminal is wide wraps onto. Control sequences
/// count as the characters they are written with, so the height is generous
/// rather than short, and the extra rows are blank ones nothing reads.
///
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
fn stream_rows(bytes: &[u8], columns: u16) -> u16 {
    bytes
        .split(|&byte| byte == b'\n')
        .map(|line| line.len().div_ceil(columns as usize).max(1))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16
}

/// One cell colour. `"default"` is the terminal's own foreground or background,
/// which is what a program that sets no colour leaves behind, and it is distinct
/// from a named colour that happens to render the same way. A colour the program
/// set by palette index or by channel is reported as it set it, because the
/// screen reports what was drawn rather than guessing which name was meant.
///
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("that region is drawn in the background colour {string}")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Colour {
    /// The terminal's own colour, or one of the sixteen the palette names.
    Name(String),
    /// A colour the program set by palette index.
    Indexed { index: u8 },
    /// A colour the program set by channel.
    Channels { red: u8, green: u8, blue: u8 },
}

/// The presentation one cell is drawn with. Colour is meaning on a terminal
/// screen, so the attributes the emulator parsed are carried beside the
/// character rather than dropped.
///
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("that region is drawn in the background colour {string}")
/// @planks("the child region showing {string} is drawn in the foreground colour {string}")
/// @planks("the operator inspects a command that prints {string} {attribute}")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    pub foreground: Colour,
    pub background: Colour,
    pub bold: bool,
    pub reverse: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub hidden: bool,
    pub strikeout: bool,
    #[serde(rename = "doubleUnderline")]
    pub double_underline: bool,
}

/// The colour a parsed cell carries, named as the palette names it. The
/// emulator's foreground and background are the terminal's own colours, which is
/// what a program that sets no colour leaves behind; its remaining names are
/// render-time variants a parsed cell never carries.
///
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("that region is drawn in the background colour {string}")
fn colour_of(colour: Color) -> Colour {
    let named = match colour {
        Color::Indexed(index) => return Colour::Indexed { index },
        Color::Spec(rgb) => {
            return Colour::Channels {
                red: rgb.r,
                green: rgb.g,
                blue: rgb.b,
            };
        }
        Color::Named(named) => named,
    };
    Colour::Name(
        match named {
            NamedColor::Black => "black",
            NamedColor::Red => "red",
            NamedColor::Green => "green",
            NamedColor::Yellow => "yellow",
            NamedColor::Blue => "blue",
            NamedColor::Magenta => "magenta",
            NamedColor::Cyan => "cyan",
            NamedColor::White => "white",
            NamedColor::BrightBlack => "brightBlack",
            NamedColor::BrightRed => "brightRed",
            NamedColor::BrightGreen => "brightGreen",
            NamedColor::BrightYellow => "brightYellow",
            NamedColor::BrightBlue => "brightBlue",
            NamedColor::BrightMagenta => "brightMagenta",
            NamedColor::BrightCyan => "brightCyan",
            NamedColor::BrightWhite => "brightWhite",
            _ => "default",
        }
        .to_string(),
    )
}

/// A parsed terminal screen: rows of cell contents, with the reversed video a
/// program highlights a line with, the presentation each cell is drawn with, the
/// columns a wide character continues into, and the cell the cursor rests on,
/// which a program that hides the cursor leaves unshown. Row and column
/// addressing is 1-based, matching ANSI cursor addressing.
///
/// @planks("a virtual screen that shows {string}")
/// @planks("the process is captured through a PTY")
/// @planks("the line {string} is rendered with reversed video")
/// @planks("the virtual screen row {int} reads {string}")
/// @planks("the virtual screen cell at row {int} column {int} continues the character at column {int}")
/// @planks("every cell of row {int} from column {int} through column {int} is rendered with reversed video")
/// @planks("the terminal object model is inferred")
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("the model's cursor is at row {int} column {int}")
/// @planks("the model carries no cursor")
#[derive(Debug, Clone)]
pub struct VirtualScreen {
    rows: Vec<Vec<String>>,
    reversed: Vec<Vec<bool>>,
    styles: Vec<Vec<Style>>,
    continuations: Vec<Vec<bool>>,
    cursor: (u16, u16),
    cursor_shown: bool,
}

impl VirtualScreen {
    /// Parse text into a virtual screen, as if the text were written to a fresh
    /// terminal.
    ///
    /// @planks("a virtual screen that shows {string}")
    pub fn from_text(text: &str) -> VirtualScreen {
        Self::parse(text.as_bytes(), COLS, ROWS)
    }

    /// Parse raw PTY output bytes into a virtual screen. Control sequences such
    /// as ANSI cursor positioning are honoured by the terminal emulator.
    ///
    /// @planks("the process is captured through a PTY")
    pub fn from_pty_output(bytes: &[u8]) -> VirtualScreen {
        Self::parse(bytes, COLS, ROWS)
    }

    /// Parse raw PTY output bytes into a virtual screen `columns` wide, the
    /// width the run gave the terminal the program drew on.
    ///
    /// @planks("that plan is replayed at {int} columns")
    pub fn from_pty_output_at(bytes: &[u8], columns: u16) -> VirtualScreen {
        Self::parse(bytes, columns, ROWS)
    }

    /// Parse everything a finished program wrote, on a terminal `columns` wide
    /// and tall enough to hold every line of it. A program that has exited has
    /// finished speaking, so its output is read whole; a grid the height of the
    /// operator's terminal would scroll all but the last screenful away.
    ///
    /// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
    pub fn from_pty_stream(bytes: &[u8], columns: u16) -> VirtualScreen {
        Self::parse(bytes, columns, stream_rows(bytes, columns))
    }

    /// @planks("the process is captured through a PTY")
    /// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
    /// @planks("the operator inspects a command that prints {string} {attribute}")
    fn parse(bytes: &[u8], columns: u16, rows: u16) -> VirtualScreen {
        let config = Config {
            scrolling_history: 0,
            ..Config::default()
        };
        let mut term = Term::new(config, &ScreenSize { columns, rows }, VoidListener);
        let mut processor = Processor::<StdSyncHandler>::new();
        // Erasing to the end of the line clears the erased cells to the
        // background colour alone, dropping the reversed video the pen carries.
        // A full-screen program highlights a row by turning reversed video on
        // and erasing the rest of the line, so the erase is fed on its own and
        // the cells it cleared take back the reversed video it discarded.
        let mut fed = 0;
        for (start, length) in erases_to_end_of_line(bytes) {
            processor.advance(&mut term, &bytes[fed..start]);
            let pen_reversed = term.grid().cursor.template.flags.contains(Flags::INVERSE);
            let point = term.grid().cursor.point;
            fed = start + length;
            processor.advance(&mut term, &bytes[start..fed]);
            if pen_reversed {
                let row = &mut term.grid_mut()[point.line];
                for col in point.column.0..columns as usize {
                    row[Column(col)].flags.insert(Flags::INVERSE);
                }
            }
        }
        processor.advance(&mut term, &bytes[fed..]);
        let grid = term.grid();
        // Where the cursor rests is the cell the next character the program
        // writes will land in. It is reported on the emulator's own 0-based
        // grid, and a program that scrolled its output reports a row above the
        // first, so the row is taken from the top of the screen.
        let point = grid.cursor.point;
        let cursor = (
            u16::try_from(point.line.0.max(0)).expect("the cursor row fits a terminal") + 1,
            u16::try_from(point.column.0).expect("the cursor column fits a terminal") + 1,
        );
        // A program that hides the cursor has said something the operator can
        // see, so the mode the emulator holds is read beside the position.
        let cursor_shown = term.mode().contains(TermMode::SHOW_CURSOR);
        let mut cell_rows = Vec::with_capacity(rows as usize);
        let mut reversed = Vec::with_capacity(rows as usize);
        let mut styles = Vec::with_capacity(rows as usize);
        let mut continuations = Vec::with_capacity(rows as usize);
        for row in 0..rows {
            let mut cells = Vec::with_capacity(columns as usize);
            let mut inverses: Vec<bool> = Vec::with_capacity(columns as usize);
            let mut drawn = Vec::with_capacity(columns as usize);
            let mut continued = Vec::with_capacity(columns as usize);
            for col in 0..columns {
                let cell = &grid[Line(row as i32)][Column(col as usize)];
                let continuation = cell.flags.contains(Flags::WIDE_CHAR_SPACER);
                // The second column of a wide character carries no character of
                // its own: the character it continues covers both columns.
                cells.push(if continuation {
                    String::new()
                } else {
                    cell.c.to_string()
                });
                // The emulator clears the second column of a wide character to
                // the default attributes, so that column takes the reversed
                // video of the character it continues.
                inverses.push(if continuation {
                    inverses.last().copied().unwrap_or_default()
                } else {
                    cell.flags.contains(Flags::INVERSE)
                });
                drawn.push(Style {
                    foreground: colour_of(cell.fg),
                    background: colour_of(cell.bg),
                    bold: cell.flags.contains(Flags::BOLD),
                    reverse: inverses.last().copied().unwrap_or_default(),
                    dim: cell.flags.contains(Flags::DIM),
                    italic: cell.flags.contains(Flags::ITALIC),
                    underline: cell.flags.contains(Flags::UNDERLINE),
                    hidden: cell.flags.contains(Flags::HIDDEN),
                    strikeout: cell.flags.contains(Flags::STRIKEOUT),
                    double_underline: cell.flags.contains(Flags::DOUBLE_UNDERLINE),
                });
                continued.push(continuation);
            }
            cell_rows.push(cells);
            reversed.push(inverses);
            styles.push(drawn);
            continuations.push(continued);
        }
        VirtualScreen {
            rows: cell_rows,
            reversed,
            styles,
            continuations,
            cursor,
            cursor_shown,
        }
    }

    /// The 1-based row and column the cursor rests on.
    ///
    /// @planks("the terminal object model is inferred")
    pub fn cursor(&self) -> (u16, u16) {
        self.cursor
    }

    /// Whether the program leaves the cursor showing, which is what tells an
    /// operator the program is listening and where the next character lands.
    ///
    /// @planks("the model's cursor is at row {int} column {int}")
    /// @planks("the model carries no cursor")
    pub fn cursor_shown(&self) -> bool {
        self.cursor_shown
    }

    /// The presentation the cell at the given 1-based row and column is drawn
    /// with.
    ///
    /// @planks("the region with the role {string} is drawn in the foreground colour {string}")
    /// @planks("that region is drawn in the background colour {string}")
    /// @planks("the child region showing {string} is drawn in the foreground colour {string}")
    pub fn style(&self, row: u16, col: u16) -> Option<Style> {
        self.styles
            .get((row - 1) as usize)
            .and_then(|cells| cells.get((col - 1) as usize))
            .cloned()
    }

    /// The 1-based column of the wide character that the cell at the given
    /// 1-based row and column continues, when the cell is the second column of
    /// a wide character.
    ///
    /// @planks("the virtual screen cell at row {int} column {int} continues the character at column {int}")
    pub fn continuation_start(&self, row: u16, col: u16) -> Option<u16> {
        let continuations = self.continuations.get((row - 1) as usize)?;
        if !continuations
            .get((col - 1) as usize)
            .copied()
            .unwrap_or_default()
        {
            return None;
        }
        (1..col)
            .rev()
            .find(|start| !continuations[(start - 1) as usize])
    }

    /// Whether the screen displays the given text on any single row.
    ///
    /// @planks("the virtual screen contains the text {string}")
    pub fn contains(&self, text: &str) -> bool {
        self.rows.iter().any(|row| row.concat().contains(text))
    }

    /// The contents of the cell at the given 1-based row and column.
    ///
    /// @planks("the virtual screen cell at row {int} column {int} shows {string}")
    pub fn cell(&self, row: u16, col: u16) -> String {
        self.rows
            .get((row - 1) as usize)
            .and_then(|cells| cells.get((col - 1) as usize))
            .cloned()
            .unwrap_or_default()
    }

    /// Whether the cell at the given 1-based row and column is drawn with
    /// reversed video, as a highlighted line is.
    ///
    /// @planks("the line {string} is rendered with reversed video")
    pub fn reverse(&self, row: u16, col: u16) -> bool {
        self.reversed
            .get((row - 1) as usize)
            .and_then(|cells| cells.get((col - 1) as usize))
            .copied()
            .unwrap_or_default()
    }

    /// The full screen contents, rows joined by newlines. A cell the program
    /// left blank reads as a space, so text placed by cursor addressing keeps
    /// the column spacing the terminal displays. The second column of a wide
    /// character reads as nothing, the character itself covering both columns.
    ///
    /// @planks("the virtual screen contains the text {string}")
    /// @planks("the virtual screen row {int} reads {string}")
    pub fn contents(&self) -> String {
        self.rows
            .iter()
            .enumerate()
            .map(|(row, cells)| {
                cells
                    .iter()
                    .enumerate()
                    .map(|(col, text)| {
                        if self.continuations[row][col] {
                            ""
                        } else if text.is_empty() {
                            " "
                        } else {
                            text.as_str()
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The cell grid, one inner vector per row, for a renderer to draw.
    ///
    /// @planks("the capture view is rendered to a {int} by {int} test terminal")
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }
}
