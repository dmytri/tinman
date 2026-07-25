//! The virtual screen: a terminal grid parsed from program output.
//!
//! A launched program's bytes are parsed by a real VT100 terminal emulator into
//! a fixed grid of cells. Steps assert what the program displayed by text or by
//! addressed cell, so the grid preserves cursor-positioned output exactly as a
//! terminal would render it.

/// The number of rows and columns in the virtual terminal grid. Matches the
/// PTY size the capture path opens, so a program's cursor addressing lands on
/// the same cell the screen records.
const ROWS: u16 = 24;
const COLS: u16 = 80;

/// A parsed terminal screen: rows of cell contents. Row and column addressing
/// is 1-based, matching ANSI cursor addressing.
///
/// @planks("a virtual screen that shows {string}")
/// @planks("the process is captured through a PTY")
#[derive(Debug, Clone)]
pub struct VirtualScreen {
    rows: Vec<Vec<String>>,
}

impl VirtualScreen {
    /// Parse text into a virtual screen, as if the text were written to a fresh
    /// terminal.
    ///
    /// @planks("a virtual screen that shows {string}")
    pub fn from_text(text: &str) -> VirtualScreen {
        Self::parse(text.as_bytes())
    }

    /// Parse raw PTY output bytes into a virtual screen. Control sequences such
    /// as ANSI cursor positioning are honoured by the terminal emulator.
    ///
    /// @planks("the process is captured through a PTY")
    pub fn from_pty_output(bytes: &[u8]) -> VirtualScreen {
        Self::parse(bytes)
    }

    fn parse(bytes: &[u8]) -> VirtualScreen {
        let mut parser = vt100::Parser::new(ROWS, COLS, 0);
        parser.process(bytes);
        let screen = parser.screen();
        let mut rows = Vec::with_capacity(ROWS as usize);
        for row in 0..ROWS {
            let mut cells = Vec::with_capacity(COLS as usize);
            for col in 0..COLS {
                let contents = screen
                    .cell(row, col)
                    .map(|cell| cell.contents().to_string())
                    .unwrap_or_default();
                cells.push(contents);
            }
            rows.push(cells);
        }
        VirtualScreen { rows }
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

    /// The full screen contents, rows joined by newlines. Used in assertion
    /// messages.
    ///
    /// @planks("the virtual screen contains the text {string}")
    pub fn contents(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.concat())
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
