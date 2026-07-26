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
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

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
/// parsed at the width it drew on.
///
/// @planks("the process is captured through a PTY")
/// @planks("that plan is replayed at {int} columns")
struct ScreenSize {
    columns: u16,
}

impl Dimensions for ScreenSize {
    fn total_lines(&self) -> usize {
        ROWS as usize
    }

    fn screen_lines(&self) -> usize {
        ROWS as usize
    }

    fn columns(&self) -> usize {
        self.columns as usize
    }
}

/// A parsed terminal screen: rows of cell contents, with the reversed video a
/// program highlights a line with and the columns a wide character continues
/// into. Row and column addressing is 1-based, matching ANSI cursor addressing.
///
/// @planks("a virtual screen that shows {string}")
/// @planks("the process is captured through a PTY")
/// @planks("the line {string} is rendered with reversed video")
/// @planks("the virtual screen row {int} reads {string}")
/// @planks("the virtual screen cell at row {int} column {int} continues the character at column {int}")
/// @planks("every cell of row {int} from column {int} through column {int} is rendered with reversed video")
#[derive(Debug, Clone)]
pub struct VirtualScreen {
    rows: Vec<Vec<String>>,
    reversed: Vec<Vec<bool>>,
    continuations: Vec<Vec<bool>>,
}

impl VirtualScreen {
    /// Parse text into a virtual screen, as if the text were written to a fresh
    /// terminal.
    ///
    /// @planks("a virtual screen that shows {string}")
    pub fn from_text(text: &str) -> VirtualScreen {
        Self::parse(text.as_bytes(), COLS)
    }

    /// Parse raw PTY output bytes into a virtual screen. Control sequences such
    /// as ANSI cursor positioning are honoured by the terminal emulator.
    ///
    /// @planks("the process is captured through a PTY")
    pub fn from_pty_output(bytes: &[u8]) -> VirtualScreen {
        Self::parse(bytes, COLS)
    }

    /// Parse raw PTY output bytes into a virtual screen `columns` wide, the
    /// width the run gave the terminal the program drew on.
    ///
    /// @planks("that plan is replayed at {int} columns")
    pub fn from_pty_output_at(bytes: &[u8], columns: u16) -> VirtualScreen {
        Self::parse(bytes, columns)
    }

    fn parse(bytes: &[u8], columns: u16) -> VirtualScreen {
        let config = Config {
            scrolling_history: 0,
            ..Config::default()
        };
        let mut term = Term::new(config, &ScreenSize { columns }, VoidListener);
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
        let mut rows = Vec::with_capacity(ROWS as usize);
        let mut reversed = Vec::with_capacity(ROWS as usize);
        let mut continuations = Vec::with_capacity(ROWS as usize);
        for row in 0..ROWS {
            let mut cells = Vec::with_capacity(columns as usize);
            let mut inverses: Vec<bool> = Vec::with_capacity(columns as usize);
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
                continued.push(continuation);
            }
            rows.push(cells);
            reversed.push(inverses);
            continuations.push(continued);
        }
        VirtualScreen {
            rows,
            reversed,
            continuations,
        }
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
