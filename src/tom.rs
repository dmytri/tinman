//! The terminal object model: the rendered screen read as nested regions.
//!
//! The model is the terminal's document object model. It is a semantic reading
//! of what the screen shows, not a reconstruction of the program that drew it.
//! Its geometry follows Ratatui: nested rectangles produced by horizontal and
//! vertical splits. Locators bind against the model, so an authored test
//! addresses a role and a name rather than a cell coordinate, and resolution is
//! mechanical: it reads the model and invokes no inference. Inference is a
//! second producer of the same shape, used at capture time only.

use crate::inference::Settings;
use crate::screen::{Colour, Style, VirtualScreen};
use serde::{Deserialize, Serialize};

/// The characters a bordered pane is drawn with. A corner is drawn square or
/// rounded, and a pane is the same region to a reader either way.
const TOP_LEFT: [&str; 2] = ["\u{250c}", "\u{256d}"];
const TOP_RIGHT: [&str; 2] = ["\u{2510}", "\u{256e}"];
const BOTTOM_LEFT: [&str; 2] = ["\u{2514}", "\u{2570}"];
pub(crate) const HORIZONTAL: &str = "\u{2500}";
pub(crate) const VERTICAL: &str = "\u{2502}";

/// A Ratatui-shaped rectangle in screen cells.
///
/// @planks("the first child region covers columns {int} through {int}")
/// @planks("the resolved region lies inside the region named {string}")
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// The semantic role a region plays on screen. An engine's reply is read into
/// this enum, so a reply naming a role the model does not define is a reply
/// Tinman discards.
///
/// @planks("the model contains a region with the role {string}")
/// @planks("the region named {string} has the role {string}")
/// @planks("that region has {int} child regions with the role {string}")
/// @planks("the operator inspects a command printing {string} and {string} on their own lines")
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Application,
    Region,
    Menu,
    Menuitem,
    List,
    Listitem,
    Button,
    Textbox,
    Status,
    Log,
    Article,
    Table,
    Row,
    Columnheader,
    Cell,
}

/// Every role the model defines, so a name maps to its role through one list.
const ROLES: [Role; 15] = [
    Role::Application,
    Role::Region,
    Role::Menu,
    Role::Menuitem,
    Role::List,
    Role::Listitem,
    Role::Button,
    Role::Textbox,
    Role::Status,
    Role::Log,
    Role::Article,
    Role::Table,
    Role::Row,
    Role::Columnheader,
    Role::Cell,
];

impl Role {
    /// The name this role carries in the model.
    ///
    /// @planks("the region named {string} has the role {string}")
    /// @planks("the operator inspects a command printing {string} and {string} on their own lines")
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Application => "application",
            Role::Region => "region",
            Role::Menu => "menu",
            Role::Menuitem => "menuitem",
            Role::List => "list",
            Role::Listitem => "listitem",
            Role::Button => "button",
            Role::Textbox => "textbox",
            Role::Status => "status",
            Role::Log => "log",
            Role::Article => "article",
            Role::Table => "table",
            Role::Row => "row",
            Role::Columnheader => "columnheader",
            Role::Cell => "cell",
        }
    }

    /// Whether `name` is a role this model produces. A locator naming a role no
    /// reading yields matches nothing whatever the screen shows, so a mistyped
    /// role is answered as one rather than as an absent region.
    ///
    /// @planks("the operator executes {string}")
    pub fn is_produced(name: &str) -> bool {
        ROLES.iter().any(|role| role.as_str() == name)
    }

    /// The role `name` addresses.
    ///
    /// @planks("a terminal object model with a {string} containing the menu items {string}, {string}, and {string}")
    pub fn from_name(name: &str) -> Role {
        ROLES
            .iter()
            .copied()
            .find(|role| role.as_str() == name)
            .unwrap_or_else(|| panic!("no region role is named {name:?}"))
    }
}

/// One node of the model. A region owns a rectangle of the screen, carries the
/// role it plays, the accessible name a locator matches and the text it renders,
/// and may split into child regions. A region built from a screen keeps the
/// cells it was built from, so a step reads the region's own grid without
/// addressing the whole screen.
///
/// @planks("the model contains a region named {string}")
/// @planks("the terminal object model is built")
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("the region named {string} carries no style")
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    role: Role,
    pub name: Option<String>,
    pub text: Option<String>,
    pub selected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    style: Option<Style>,
    pub rect: Rect,
    pub children: Vec<Region>,
    #[serde(skip)]
    cells: Vec<Vec<String>>,
}

impl Region {
    /// A region with no children, carrying `role`, `name` and `text`.
    ///
    /// @planks("a terminal object model with a {string} containing the menu items {string}, {string}, and {string}")
    /// @planks("a terminal object model with a {string} named {string} containing {string} and a {string} named {string} containing {string}")
    pub fn leaf<S: AsRef<str>>(role: &str, name: Option<S>, text: Option<S>, rect: Rect) -> Region {
        Region {
            role: Role::from_name(role),
            name: name.map(|name| name.as_ref().to_string()),
            text: text.map(|text| text.as_ref().to_string()),
            selected: false,
            rect,
            children: Vec::new(),
            style: None,
            cells: Vec::new(),
        }
    }

    /// A region carrying `role` and `name` with `children` nested inside it.
    ///
    /// @planks("a terminal object model with a {string} containing the menu items {string}, {string}, and {string}")
    /// @planks("a terminal object model with a {string} named {string} containing {string} and a {string} named {string} containing {string}")
    pub fn parent(role: &str, name: Option<&str>, rect: Rect, children: Vec<Region>) -> Region {
        Region {
            role: Role::from_name(role),
            name: name.map(|name| name.to_string()),
            text: None,
            selected: false,
            rect,
            children,
            style: None,
            cells: Vec::new(),
        }
    }

    /// The name of the role this region plays.
    ///
    /// @planks("the region named {string} has the role {string}")
    /// @planks("that region has {int} child regions with the role {string}")
    pub fn role(&self) -> &str {
        self.role.as_str()
    }

    /// The colour this region is drawn in, where it carries one of its own. A
    /// region drawn in the terminal's own colours carries none, so a listing of
    /// an ordinary screen names no colour anywhere.
    ///
    /// @planks("the operator inspects a command that prints {string} in red")
    pub fn colour(&self) -> Option<&str> {
        match &self.style.as_ref()?.foreground {
            Colour::Name(name) if name != "default" => Some(name),
            _ => None,
        }
    }

    /// The standard presentations this region is drawn with, named as a reader
    /// names them rather than as the wire spells them. A terminal says disabled
    /// with dim, password field with hidden, and done or deleted with strikeout,
    /// so a reader that cannot report them is back to flat text for exactly the
    /// distinctions the model exists to carry.
    ///
    /// @planks("the operator inspects a command that prints {string} {attribute}")
    pub fn presentations(&self) -> Vec<&'static str> {
        let Some(style) = self.style.as_ref() else {
            return Vec::new();
        };
        [
            (style.dim, "dim"),
            (style.italic, "italic"),
            (style.underline, "underlined"),
            (style.hidden, "hidden"),
            (style.strikeout, "struck through"),
            (style.double_underline, "doubly underlined"),
        ]
        .into_iter()
        .filter_map(|(drawn, name)| drawn.then_some(name))
        .collect()
    }

    /// The child region that is the selected one among its siblings.
    ///
    /// @planks("the selected item of the region named {string} is {string}")
    pub fn selected_item(&self) -> Option<&Region> {
        self.children.iter().find(|child| child.selected)
    }

    /// The contents of the cell at the region's own 1-based row and column, as
    /// the screen the region was built from rendered it.
    ///
    /// @planks("the region named {string} reports the screen cell at its own row {int} column {int}")
    pub fn cell(&self, row: u16, col: u16) -> String {
        self.cells
            .get((row - 1) as usize)
            .and_then(|cells| cells.get((col - 1) as usize))
            .cloned()
            .unwrap_or_default()
    }

    /// The first region in this subtree that `matches`, this region included.
    ///
    /// @planks("the model contains a region named {string}")
    fn find(&self, matches: &impl Fn(&Region) -> bool) -> Option<&Region> {
        if matches(self) {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(matches))
    }

    /// Every region in this subtree that `matches`, this region included.
    ///
    /// @planks("the locator for the {string} named {string} is resolved")
    fn collect(&self, matches: &impl Fn(&Region) -> bool, found: &mut Vec<Region>) {
        if matches(self) {
            found.push(self.clone());
        }
        for child in &self.children {
            child.collect(matches, found);
        }
    }
}

/// The model of one rendered screen: the size of the screen it was built from
/// and the root region covering it.
///
/// @planks("the terminal object model is built")
/// @planks("the terminal object model is serialized")
/// @planks("the model's cursor is at row {int} column {int}")
/// @planks("the model carries no cursor")
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub rows: u16,
    pub cols: u16,
    pub root: Region,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// Where the terminal's cursor stands, counted from the top left of the screen.
/// A program that hides the cursor leaves none, which is itself the observable
/// difference between a screen that takes input and one that does not.
///
/// @planks("the model's cursor is at row {int} column {int}")
/// @planks("the model carries no cursor")
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub row: u16,
    pub column: u16,
}

impl Model {
    /// A model of a `rows` by `cols` screen whose root region carries
    /// `children`.
    ///
    /// @planks("a terminal object model with a {string} containing the menu items {string}, {string}, and {string}")
    /// @planks("a terminal object model with a {string} named {string} containing {string} and a {string} named {string} containing {string}")
    pub fn rooted(rows: u16, cols: u16, children: Vec<Region>) -> Model {
        Model {
            rows,
            cols,
            cursor: None,
            root: Region {
                role: Role::Application,
                name: None,
                text: None,
                selected: false,
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: cols,
                    height: rows,
                },
                children,
                style: None,
                cells: Vec::new(),
            },
        }
    }

    /// The first region carrying `name`, searched from the root down. The match
    /// is case-sensitive.
    ///
    /// @planks("the model contains a region named {string}")
    /// @planks("the region named {string} has the role {string}")
    /// @planks("the selected item of the region named {string} is {string}")
    /// @planks("the region named {string} reports the screen cell at its own row {int} column {int}")
    pub fn find_named(&self, name: &str) -> Option<&Region> {
        self.root
            .find(&|region| region.name.as_deref() == Some(name))
    }

    /// The first region playing the role `role`, searched from the root down.
    ///
    /// @planks("the model contains a region with the role {string}")
    pub fn find_role(&self, role: &str) -> Option<&Region> {
        self.root.find(&|region| region.role() == role)
    }
}

/// The name the root region carries where the reading was handed a screen and no
/// program. WAI-ARIA requires a name on `application`, and a screen read on its
/// own says only that it is a terminal screen, so that is what the model says. A
/// reading that knows the program the operator asked about names the root for it
/// instead.
///
/// @planks("the terminal object model is serialized")
const SCREEN_NAME: &str = "terminal";

/// Read `screen` into a terminal object model: the bordered panes it draws with
/// the lines they list, the sibling regions a vertical rule splits it into, the
/// tables its sustained columns are, the menu bar its top line carries or the
/// status region that line reads as instead, the buttons and textboxes it
/// draws, and the status bar its bottom line carries.
///
/// @planks("the terminal object model is built")
/// @planks("the terminal object model is serialized")
/// @planks("the operator inspects {string}")
pub fn build(screen: &VirtualScreen) -> Model {
    let grid = screen.rows();
    let rows = grid.len() as u16;
    let cols = grid[0].len() as u16;
    let mut children = panes(grid, screen);
    if children.is_empty()
        && let Some(column) = divider_column(grid)
    {
        children.push(plain_region(grid, 0, column, rows));
        children.push(plain_region(grid, column + 1, cols - column - 1, rows));
    }
    let summaries = summaries(grid, cols);
    children.extend(tables(grid, cols));
    // The summary reading owns the top line where it claims it, so the line is
    // read once rather than as a summary region and a top status both.
    if !summaries.iter().any(|region| region.rect.y == 0) {
        if let Some(menu) = menu_bar(grid, cols, screen) {
            children.push(menu);
        } else if let Some(status) = top_status(grid, cols) {
            children.push(status);
        }
    }
    children.extend(summaries);
    children.extend(controls(grid));
    if let Some(status) = status_bar(grid, rows, cols) {
        children.push(status);
    }
    let rect = Rect {
        x: 0,
        y: 0,
        width: cols,
        height: rows,
    };
    let mut root = Region {
        role: Role::Application,
        name: Some(SCREEN_NAME.to_string()),
        text: None,
        selected: false,
        rect,
        children,
        style: None,
        cells: cells_of(grid, rect),
    };
    read_style(&mut root, screen);
    Model {
        rows,
        cols,
        root,
        cursor: cursor_of(screen),
    }
}

/// Read everything a finished program wrote into a terminal object model. A
/// program that has exited has finished speaking, so every line it wrote is read
/// rather than the last screenful alone.
///
/// The shape of the stream decides how it reads. Output sustaining columns is a
/// table, cell by cell. Otherwise blank lines separate the output into blocks,
/// and a stream of several blocks one of which runs to more than a line is a
/// log, each block an article. Everything else is a list, one item per line, so
/// a listing of filenames stays a listing whether or not a blank line fell
/// between two of them.
///
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing two two-line blocks separated by a blank line")
/// @planks("the operator inspects a command printing {string}, a blank line and {string}")
/// @planks("the operator inspects a command that prints {string} in red")
/// @planks("the operator inspects a command that writes to the sentinel path and prints {string}")
pub fn read_stream(screen: &VirtualScreen) -> Model {
    let grid = screen.rows();
    let rows = grid.len() as u16;
    let cols = grid[0].len() as u16;
    let rect = Rect {
        x: 0,
        y: 0,
        width: cols,
        height: rows,
    };
    let blocks = blocks_of(grid);
    let children = if blocks.is_empty() {
        Vec::new()
    } else if let Some(table) = table_of(grid, cols, &blocks.concat()) {
        vec![table]
    } else if blocks.len() > 1 && blocks.iter().any(|block| block.len() > 1) {
        vec![stream_log(grid, cols, rect, &blocks)]
    } else {
        vec![stream_list(grid, cols, rect, &blocks.concat())]
    };
    let mut root = Region {
        role: Role::Application,
        name: None,
        text: None,
        selected: false,
        rect,
        children,
        style: None,
        cells: cells_of(grid, rect),
    };
    read_style(&mut root, screen);
    Model {
        rows,
        cols,
        root,
        cursor: cursor_of(screen),
    }
}

/// The runs of drawn lines `grid` carries, each run the rows of one block: a
/// blank line ends the block above it and begins the one below.
///
/// @planks("the operator inspects a command printing two two-line blocks separated by a blank line")
/// @planks("the operator inspects a command printing {string}, a blank line and {string}")
fn blocks_of(grid: &[Vec<String>]) -> Vec<Vec<usize>> {
    let mut blocks = Vec::new();
    let mut block = Vec::new();
    for (row, cells) in grid.iter().enumerate() {
        if cells.concat().trim().is_empty() {
            if !block.is_empty() {
                blocks.push(std::mem::take(&mut block));
            }
            continue;
        }
        block.push(row);
    }
    if !block.is_empty() {
        blocks.push(block);
    }
    blocks
}

/// The stream read as a log: one article per block, carrying the lines that
/// block shows.
///
/// @planks("the operator inspects a command printing two two-line blocks separated by a blank line")
fn stream_log(grid: &[Vec<String>], cols: u16, rect: Rect, blocks: &[Vec<usize>]) -> Region {
    let articles = blocks
        .iter()
        .map(|block| {
            let text = block
                .iter()
                .map(|&row| stream_line(grid, row))
                .collect::<Vec<String>>()
                .join("\n");
            let article = Rect {
                x: 0,
                y: block[0] as u16,
                width: cols,
                height: block.len() as u16,
            };
            stream_region(grid, Role::Article, text, article)
        })
        .collect();
    Region {
        role: Role::Log,
        name: None,
        text: None,
        selected: false,
        rect,
        children: articles,
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The stream read as a list: one item per drawn line.
///
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command that prints {string} in red")
/// @planks("the operator inspects a command that writes to the sentinel path and prints {string}")
fn stream_list(grid: &[Vec<String>], cols: u16, rect: Rect, lines: &[usize]) -> Region {
    let items = lines
        .iter()
        .map(|&row| {
            let item = Rect {
                x: 0,
                y: row as u16,
                width: cols,
                height: 1,
            };
            stream_region(grid, Role::Listitem, stream_line(grid, row), item)
        })
        .collect();
    Region {
        role: Role::List,
        name: None,
        text: None,
        selected: false,
        rect,
        children: items,
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// One entry of a stream: the text it shows is both the text it carries and the
/// name a locator matches it by. A cell of a table and the header labelling its
/// column are read the same way, the field's own text naming it.
///
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing two two-line blocks separated by a blank line")
/// @planks("the operator inspects a command printing {string} and {string} on their own lines")
/// @planks("the operator inspects {string}")
fn stream_region(grid: &[Vec<String>], role: Role, text: String, rect: Rect) -> Region {
    Region {
        role,
        name: Some(text.clone()),
        text: Some(text),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The text row `row` of `grid` shows, with the blank cells beyond it dropped.
///
/// @planks("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")
fn stream_line(grid: &[Vec<String>], row: usize) -> String {
    grid[row].concat().trim_end().to_string()
}

/// The fields the lines `rows` sustain: a column blank on every one of those
/// lines is a gap, and a run of gap columns with drawn cells on both sides ends
/// one field and begins the next. A real listing separates its fields with a
/// single space and varies their widths, so the boundary is the position that
/// stays blank rather than the width of the gap standing there. Prose breaks
/// into words wherever it likes, so the blanks of one line fall where the next
/// line draws, and a paragraph sustains no boundary at all.
///
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing {string} and {string} on their own lines")
/// @planks("the operator inspects {string}")
fn columns_of(grid: &[Vec<String>], rows: &[usize]) -> Vec<(usize, usize)> {
    let width = grid[rows[0]].len();
    let gap: Vec<bool> = (0..width)
        .map(|col| rows.iter().all(|&row| grid[row][col].trim().is_empty()))
        .collect();
    let mut fields = Vec::new();
    let mut start = None;
    let mut col = 0;
    while col < width {
        if !gap[col] {
            start = start.or(Some(col));
            col += 1;
            continue;
        }
        let end = (col..width).find(|&next| !gap[next]).unwrap_or(width);
        if let Some(from) = start.take() {
            fields.push((from, col));
        }
        col = end;
    }
    if let Some(from) = start {
        fields.push((from, width));
    }
    fields
}

/// The lines `rows` read as a table, where they sustain columns across every one
/// of them. A first row that labels the rest is read as the column headers, so a
/// cell is addressed by the column that names it; where nothing labels them the
/// table carries no headers at all and a cell is addressed by its position,
/// which is what WAI-ARIA permits and what the screen shows.
///
/// A row of labels is a row carrying no number where the rows under it do,
/// which is what tells the header of `ps` or `df` from the first line of a bare
/// `ls -la` listing.
///
/// A field every one of the lines draws in is a column; a field some line leaves
/// blank is a coincidence those lines did not sustain. Prose runs to whatever
/// length it likes, so the blank the short line trails is blank on the long line
/// too, and reading that as a boundary would hand back a table whose first row
/// holds an empty cell.
///
/// @planks("the operator inspects a command printing {string}, {string} and {string} on their own lines")
/// @planks("the operator inspects a command printing {string} and {string} on their own lines")
/// @planks("the operator inspects {string}")
fn table_of(grid: &[Vec<String>], cols: u16, rows: &[usize]) -> Option<Region> {
    if rows.len() < 2 {
        return None;
    }
    let fields = columns_of(grid, rows);
    if fields.len() < 2 {
        return None;
    }
    let cells_at = |row: usize| -> Vec<String> {
        fields
            .iter()
            .map(|&(from, to)| grid[row][from..to].concat().trim().to_string())
            .collect()
    };
    if rows
        .iter()
        .any(|&row| cells_at(row).iter().any(|cell| cell.is_empty()))
    {
        return None;
    }
    let first = cells_at(rows[0]);
    let labels = first.iter().all(|cell| cell.parse::<f64>().is_err())
        && rows[1..]
            .iter()
            .any(|&row| cells_at(row).iter().any(|cell| cell.parse::<f64>().is_ok()));
    let mut children = Vec::new();
    if labels {
        children.extend(fields.iter().zip(&first).map(|(&(from, to), label)| {
            let rect = Rect {
                x: from as u16,
                y: rows[0] as u16,
                width: (to - from) as u16,
                height: 1,
            };
            stream_region(grid, Role::Columnheader, label.clone(), rect)
        }));
    }
    let body = if labels { &rows[1..] } else { rows };
    children.extend(body.iter().map(|&row| table_row(grid, cols, &fields, row)));
    let rect = Rect {
        x: 0,
        y: rows[0] as u16,
        width: cols,
        height: rows.len() as u16,
    };
    Some(Region {
        role: Role::Table,
        name: None,
        text: None,
        selected: false,
        rect,
        children,
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// One row of a table: the line it shows names it, and each field the table
/// sustains is a cell inside it.
///
/// @planks("the {string} cell of the row naming {string} is {string}")
/// @planks("the second cell of the row naming {string} is {string}")
fn table_row(grid: &[Vec<String>], cols: u16, fields: &[(usize, usize)], row: usize) -> Region {
    let children = fields
        .iter()
        .map(|&(from, to)| {
            let rect = Rect {
                x: from as u16,
                y: row as u16,
                width: (to - from) as u16,
                height: 1,
            };
            let text = grid[row][from..to].concat().trim().to_string();
            stream_region(grid, Role::Cell, text, rect)
        })
        .collect();
    let rect = Rect {
        x: 0,
        y: row as u16,
        width: cols,
        height: 1,
    };
    let line = stream_line(grid, row);
    Region {
        role: Role::Row,
        name: Some(line.clone()),
        text: Some(line),
        selected: false,
        rect,
        children,
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The tables `grid` draws, one per run of lines that sustains columns. A
/// program still running has drawn its columns onto the screen like anything
/// else it shows, so the same reading serves a table there. A block opening
/// with a summary line reads from the line the columns begin at, because the
/// summary line is not part of the table beneath it.
///
/// @planks("the operator inspects {string}")
/// @planks("the total line is not a row of that table")
fn tables(grid: &[Vec<String>], cols: u16) -> Vec<Region> {
    blocks_of(grid)
        .iter()
        .filter_map(|block| {
            let start = table_start(grid, cols, block)?;
            table_of(grid, cols, &block[start..])
        })
        .collect()
}

/// The row of `block` a table begins at: the first row whose remaining lines
/// sustain columns. A summary line draws in fewer fields than the lines beneath
/// it, so the whole block yields an empty cell and is declined while the lines
/// under that summary line read as the table they are.
///
/// @planks("the total line is not a row of that table")
fn table_start(grid: &[Vec<String>], cols: u16, block: &[usize]) -> Option<usize> {
    (0..block.len()).find(|&start| table_of(grid, cols, &block[start..]).is_some())
}

/// The lines `grid` draws above the first table it draws, each read as a status
/// region. A block of lines above a table is the program reporting on itself,
/// which is what the status role already means at the other edge of the screen,
/// and a leading summary line is no more a row of the table beneath it than the
/// block a process monitor opens above one is.
///
/// @planks("the model contains {int} regions with the role {string}")
/// @planks("each one carries the text of the line it was read from")
fn summaries(grid: &[Vec<String>], cols: u16) -> Vec<Region> {
    let mut above: Vec<usize> = Vec::new();
    for block in blocks_of(grid) {
        let Some(start) = table_start(grid, cols, &block) else {
            above.extend_from_slice(&block);
            continue;
        };
        above.extend_from_slice(&block[..start]);
        return above
            .into_iter()
            .map(|row| summary_region(grid, cols, row))
            .collect();
    }
    Vec::new()
}

/// One summary line read as a status region, carrying the text of the line it
/// was read from across the full width of the screen.
///
/// @planks("each one carries the text of the line it was read from")
fn summary_region(grid: &[Vec<String>], cols: u16, row: usize) -> Region {
    let rect = Rect {
        x: 0,
        y: row as u16,
        width: cols,
        height: 1,
    };
    Region {
        role: Role::Status,
        name: None,
        text: Some(stream_line(grid, row)),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// Read the presentation `region` and everything nested inside it is drawn with.
/// The regions are read after they are found, so a region reports the colours
/// its own rectangle carries whichever pass built it.
///
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("the region named {string} carries no style")
fn read_style(region: &mut Region, screen: &VirtualScreen) {
    region.style = style_of(screen, region.rect);
    for child in &mut region.children {
        read_style(child, screen);
    }
}

/// The presentation every drawn cell of `rect` shares, or none where they
/// disagree. A blank cell shows no colour an operator can read, so the style is
/// the style of the cells carrying ink, and a region whose cells are drawn
/// differently carries no style at all.
///
/// @planks("the region with the role {string} is drawn in the foreground colour {string}")
/// @planks("that region is drawn in the background colour {string}")
/// @planks("the child region showing {string} is drawn in the foreground colour {string}")
/// @planks("the region named {string} carries no style")
fn style_of(screen: &VirtualScreen, rect: Rect) -> Option<Style> {
    let mut drawn = (rect.y..rect.y + rect.height)
        .flat_map(|row| (rect.x..rect.x + rect.width).map(move |col| (row, col)))
        .filter(|&(row, col)| !screen.cell(row + 1, col + 1).trim().is_empty())
        .filter_map(|(row, col)| screen.style(row + 1, col + 1));
    let first = drawn.next()?;
    drawn.all(|style| style == first).then_some(first)
}

/// Where the cursor stands on `screen`, counted from the top left. The screen
/// reports it on the 1-based grid the terminal addresses, and the model counts
/// its rectangles from zero, so the position is converted. A program that has
/// hidden the cursor leaves none.
///
/// @planks("the model's cursor is at row {int} column {int}")
/// @planks("the model carries no cursor")
fn cursor_of(screen: &VirtualScreen) -> Option<Cursor> {
    if !screen.cursor_shown() {
        return None;
    }
    let (row, column) = screen.cursor();
    Some(Cursor {
        row: row - 1,
        column: column - 1,
    })
}

/// Read `screen` into a model, enriched by the configured engine. The
/// deterministic model is the spine: it stands whenever the engine is
/// unavailable, and it stands again whenever the engine answers with something
/// the model's own shape rejects. What the engine contributes is names, region
/// by region, so the model Tinman produces keeps the structure the screen
/// yields whatever the engine sent.
///
/// @planks("the terminal object model is inferred")
/// @planks("the terminal object model is inferred by the configured engine")
pub fn infer(screen: &VirtualScreen, settings: &Settings) -> Model {
    enrich(build(screen), screen, settings)
}

/// `model`, carrying the names the configured engine reads off `screen`. A
/// caller that has already read the screen its own way keeps that reading as
/// the spine, so a stream read whole stays read whole and the engine's
/// contribution stays the names it proposes for what is already there.
///
/// @planks("the operator inspects the fixture terminal program")
pub fn enrich(model: Model, screen: &VirtualScreen, settings: &Settings) -> Model {
    enriched(
        model,
        crate::inference::tom_completion(settings, &screen.contents()),
    )
}

/// `infer`, naming the program the screen belongs to, so the naming pass carries
/// that program's tldr page as the vocabulary it names regions from.
///
/// @planks("the terminal object model of {string} is inferred")
/// @planks("the plan for {string} is written")
pub fn infer_for(screen: &VirtualScreen, settings: &Settings, program: &str) -> Model {
    enriched(
        build(screen),
        crate::inference::tom_completion_for(settings, &screen.contents(), program),
    )
}

/// The model handed in, carrying the names `reply` proposed for its regions. A
/// reply that never arrived, and one nothing of the model's own shape can be
/// read from, both leave the deterministic model standing.
///
/// @planks("the terminal object model is inferred")
/// @planks("the terminal object model is inferred by the configured engine")
/// @planks("the terminal object model of {string} is inferred")
fn enriched(deterministic: Model, reply: Option<String>) -> Model {
    let mut deterministic = deterministic;
    let Some(reply) = reply else {
        return deterministic;
    };
    let Ok(inferred) = serde_yaml::from_str::<Model>(&reply) else {
        return deterministic;
    };
    take_names(&mut deterministic.root, &inferred.root);
    deterministic
}

/// Take onto `region` and the regions beneath it the names `proposed` carries
/// for them, each deterministic region paired with the proposal that plays its
/// role over the same part of the screen. An engine reads the screen into a
/// tree of its own, grouping and ordering it as it sees fit, so a pairing that
/// walked both trees in step would hand a region the name of whatever the
/// engine happened to put in that slot. A region the engine named with nothing,
/// and one no proposal covers, are regions the enrichment is refused for, so the
/// deterministic reading stands there.
///
/// @planks("the terminal object model is inferred")
/// @planks("the terminal object model is inferred by the configured engine")
/// @planks("the operator inspects the fixture terminal program")
fn take_names(region: &mut Region, proposed: &Region) {
    let mut proposals = Vec::new();
    collect(proposed, &mut proposals);
    named_from(region, &proposals);
}

/// Every region `proposed` carries, itself included, so a deterministic region
/// is matched against the whole reading rather than against one branch of it.
///
/// @planks("the terminal object model is inferred")
fn collect<'a>(proposed: &'a Region, into: &mut Vec<&'a Region>) {
    into.push(proposed);
    for child in &proposed.children {
        collect(child, into);
    }
}

/// Take onto `region` and the regions beneath it the name of the proposal that
/// plays the same role over the most of the same screen cells.
///
/// @planks("the terminal object model is inferred")
fn named_from(region: &mut Region, proposals: &[&Region]) {
    let covering = proposals
        .iter()
        .filter(|proposed| proposed.role == region.role && shared(proposed.rect, region.rect) > 0)
        .max_by_key(|proposed| shared(proposed.rect, region.rect));
    if let Some(name) = covering
        .and_then(|proposed| proposed.name.as_deref())
        .filter(|name| !name.trim().is_empty())
    {
        region.name = Some(name.to_string());
    }
    for child in region.children.iter_mut() {
        named_from(child, proposals);
    }
}

/// The screen cells `one` and `other` both cover.
///
/// @planks("the terminal object model is inferred")
fn shared(one: Rect, other: Rect) -> u32 {
    let across = one.x.max(other.x)..(one.x + one.width).min(other.x + other.width);
    let down = one.y.max(other.y)..(one.y + one.height).min(other.y + other.height);
    u32::from(across.len() as u16) * u32::from(down.len() as u16)
}

/// The bordered panes `grid` draws, each read as a list of the lines it shows.
///
/// @planks("the terminal object model is built")
/// @planks("the region titled {string} has the corner glyph {string}")
fn panes(grid: &[Vec<String>], screen: &VirtualScreen) -> Vec<Region> {
    let mut regions = Vec::new();
    for y in 0..grid.len() {
        for x in 0..grid[y].len() {
            if !TOP_LEFT.contains(&grid[y][x].as_str()) {
                continue;
            }
            let Some(right) =
                (x + 1..grid[y].len()).find(|&col| TOP_RIGHT.contains(&grid[y][col].as_str()))
            else {
                continue;
            };
            let Some(bottom) =
                (y + 1..grid.len()).find(|&row| BOTTOM_LEFT.contains(&grid[row][x].as_str()))
            else {
                continue;
            };
            regions.push(pane_region(grid, screen, x, y, right, bottom));
        }
    }
    regions
}

/// One bordered pane read as a list: its title is its name and each line it
/// shows is an item, the reversed line being the selected one. A pane whose
/// lines are separated into several runs by blank lines is a log instead, each
/// run of lines an article. A pane holding the cursor is where typing goes, so
/// it is a textbox carrying the lines it holds as its text.
///
/// @planks("the terminal object model is built")
/// @planks("the region named {string} has the role {string}")
/// @planks("the terminal object model is inferred")
fn pane_region(
    grid: &[Vec<String>],
    screen: &VirtualScreen,
    x: usize,
    y: usize,
    right: usize,
    bottom: usize,
) -> Region {
    let rect = Rect {
        x: x as u16,
        y: y as u16,
        width: (right - x + 1) as u16,
        height: (bottom - y + 1) as u16,
    };
    let title: String = grid[y][x + 1..right]
        .iter()
        .take_while(|cell| cell.as_str() != HORIZONTAL)
        .cloned()
        .collect();
    // The cursor is the signal that tells a field being edited from a panel
    // being displayed. It is reported on the 1-based grid the terminal
    // addresses and the rectangle is 0-based, so the reported cell is converted
    // before it is placed in the pane. A textbox holds text rather than lines,
    // and the blank line an empty field shows is part of that text, so the text
    // is taken from every row inside the border.
    let (cursor_row, cursor_col) = screen.cursor();
    let (cursor_row, cursor_col) = (cursor_row - 1, cursor_col - 1);
    if cursor_row >= rect.y
        && cursor_row < rect.y + rect.height
        && cursor_col >= rect.x
        && cursor_col < rect.x + rect.width
    {
        let text = (y + 1..bottom)
            .map(|row| line_of(grid, row, x, right))
            .collect::<Vec<String>>()
            .join("\n");
        let children = bottom_border_status(grid, x, right, bottom)
            .into_iter()
            .collect();
        return Region {
            role: Role::Textbox,
            name: Some(title),
            text: Some(text),
            selected: false,
            rect,
            children,
            style: None,
            cells: cells_of(grid, rect),
        };
    }
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut run: Vec<usize> = Vec::new();
    for row in y + 1..bottom {
        if line_of(grid, row, x, right).is_empty() {
            if !run.is_empty() {
                runs.push(std::mem::take(&mut run));
            }
            continue;
        }
        run.push(row);
    }
    if !run.is_empty() {
        runs.push(run);
    }
    if runs.len() > 1 {
        let articles = runs
            .iter()
            .map(|rows| article_region(grid, x, right, rows))
            .collect();
        return Region {
            role: Role::Log,
            name: Some(title),
            text: None,
            selected: false,
            rect,
            children: articles,
            style: None,
            cells: cells_of(grid, rect),
        };
    }
    let mut items = Vec::new();
    for row in runs.concat() {
        let text = line_of(grid, row, x, right);
        let item = Rect {
            x: (x + 1) as u16,
            y: row as u16,
            width: (right - x - 1) as u16,
            height: 1,
        };
        let selected = (x + 1..right).any(|col| screen.reverse(row as u16 + 1, col as u16 + 1));
        items.push(Region {
            role: Role::Listitem,
            name: Some(text.clone()),
            text: Some(text),
            selected,
            rect: item,
            children: Vec::new(),
            style: None,
            cells: cells_of(grid, item),
        });
    }
    if let Some(status) = bottom_border_status(grid, x, right, bottom) {
        items.push(status);
    }
    Region {
        role: Role::List,
        name: Some(title),
        text: None,
        selected: false,
        rect,
        children: items,
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The hint text a pane's bottom border carries, read the same way the top
/// border's title is read, as a status region nested inside the pane it
/// belongs to. A bottom border drawn with no hint yields none.
///
/// @planks("the region named {string} contains a region with the role {string}")
fn bottom_border_status(
    grid: &[Vec<String>],
    x: usize,
    right: usize,
    bottom: usize,
) -> Option<Region> {
    let text: String = grid[bottom][x + 1..right]
        .iter()
        .take_while(|cell| cell.as_str() != HORIZONTAL)
        .cloned()
        .collect();
    if text.is_empty() {
        return None;
    }
    let rect = Rect {
        x: (x + 1) as u16,
        y: bottom as u16,
        width: (right - x - 1) as u16,
        height: 1,
    };
    Some(Region {
        role: Role::Status,
        name: None,
        text: Some(text),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// The text row `row` of `grid` shows inside a pane whose borders run down
/// columns `x` and `right`.
///
/// @planks("the terminal object model is built")
fn line_of(grid: &[Vec<String>], row: usize, x: usize, right: usize) -> String {
    grid[row][x + 1..right].concat().trim_end().to_string()
}

/// One entry of a log: the run of lines `rows` covers, read as an article
/// carrying the text those lines show.
///
/// @planks("that region has {int} child regions with the role {string}")
fn article_region(grid: &[Vec<String>], x: usize, right: usize, rows: &[usize]) -> Region {
    let text = rows
        .iter()
        .map(|&row| line_of(grid, row, x, right))
        .collect::<Vec<String>>()
        .join("\n");
    let rect = Rect {
        x: (x + 1) as u16,
        y: rows[0] as u16,
        width: (right - x - 1) as u16,
        height: rows.len() as u16,
    };
    Region {
        role: Role::Article,
        name: Some(text.clone()),
        text: Some(text),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The column a vertical rule runs down, when `grid` carries one.
///
/// @planks("the terminal object model is built")
fn divider_column(grid: &[Vec<String>]) -> Option<u16> {
    (0..grid[0].len())
        .find(|&x| grid.iter().filter(|row| row[x] == VERTICAL).count() * 2 >= grid.len())
        .map(|x| x as u16)
}

/// One side of a split screen: a region carrying no name, covering the full
/// height of the screen from column `x` for `width` columns.
///
/// @planks("the first child region covers columns {int} through {int}")
fn plain_region(grid: &[Vec<String>], x: u16, width: u16, rows: u16) -> Region {
    let rect = Rect {
        x,
        y: 0,
        width,
        height: rows,
    };
    Region {
        role: Role::Region,
        name: None,
        text: None,
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    }
}

/// The bottom line of `grid` read as a status bar, when it carries text.
///
/// @planks("the terminal object model is built")
fn status_bar(grid: &[Vec<String>], rows: u16, cols: u16) -> Option<Region> {
    let text = grid[grid.len() - 1].concat().trim_end().to_string();
    if text.is_empty() {
        return None;
    }
    let rect = Rect {
        x: 0,
        y: rows - 1,
        width: cols,
        height: 1,
    };
    Some(Region {
        role: Role::Status,
        name: None,
        text: Some(text),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// The top line of `grid` read as a menu bar, each label it carries a menu
/// item, the reversed label being the selected one. A line drawing a border is
/// a pane's edge rather than a menu, and a line carrying one label is a
/// heading rather than a bar of items. A menu is the role that carries a
/// selection, so a line no reversed label marks is a sentence with gaps in it
/// rather than a bar of controls to activate.
///
/// @planks("the terminal object model is built")
/// @planks("the second {string} of that region is named {string}")
/// @planks("the menu's selected item is {string}")
fn menu_bar(grid: &[Vec<String>], cols: u16, screen: &VirtualScreen) -> Option<Region> {
    let row = &grid[0];
    if draws_a_border(row) {
        return None;
    }
    let labels = labels_of(row);
    if labels.len() < 2 {
        return None;
    }
    let items: Vec<Region> = labels
        .into_iter()
        .map(|(x, width, text)| {
            let item = Rect {
                x,
                y: 0,
                width,
                height: 1,
            };
            let selected = (x..x + width).any(|col| screen.reverse(1, col + 1));
            Region {
                role: Role::Menuitem,
                name: Some(text.clone()),
                text: Some(text),
                selected,
                rect: item,
                children: Vec::new(),
                style: None,
                cells: cells_of(grid, item),
            }
        })
        .collect();
    if !items.iter().any(|item| item.selected) {
        return None;
    }
    let rect = Rect {
        x: 0,
        y: 0,
        width: cols,
        height: 1,
    };
    Some(Region {
        role: Role::Menu,
        name: None,
        text: None,
        selected: false,
        rect,
        children: items,
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// The top line of `grid` read as a status region, when it carries text no
/// menu reading claimed. What top draws on that line is what an editor draws on
/// its bottom one, the program's own report on itself, so it takes the role the
/// model already reads at the other edge of the screen. A line drawing a border
/// is a pane's edge rather than a report.
///
/// @planks("the terminal object model is built")
/// @planks("that region's text is {string}")
fn top_status(grid: &[Vec<String>], cols: u16) -> Option<Region> {
    let row = &grid[0];
    if draws_a_border(row) {
        return None;
    }
    let text = row.concat().trim_end().to_string();
    if text.is_empty() {
        return None;
    }
    let rect = Rect {
        x: 0,
        y: 0,
        width: cols,
        height: 1,
    };
    Some(Region {
        role: Role::Status,
        name: None,
        text: Some(text),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// Whether `row` draws box-drawing characters, as a pane's border does.
///
/// @planks("the terminal object model is built")
fn draws_a_border(row: &[String]) -> bool {
    row.iter()
        .any(|cell| cell.chars().any(|c| ('\u{2500}'..='\u{257f}').contains(&c)))
}

/// The labels `row` carries: each run of text between blank cells, with the
/// column it starts at and the number of cells it covers.
///
/// @planks("the second {string} of that region is named {string}")
fn labels_of(row: &[String]) -> Vec<(u16, u16, String)> {
    let mut labels = Vec::new();
    let mut start = None;
    for x in 0..=row.len() {
        let blank = x == row.len() || row[x].trim().is_empty();
        match (blank, start) {
            (false, None) => start = Some(x),
            (true, Some(from)) => {
                labels.push((from as u16, (x - from) as u16, row[from..x].concat()));
                start = None;
            }
            _ => {}
        }
    }
    labels
}

/// The controls `grid` draws, read line by line: the buttons its bracketed
/// labels are and the textboxes its labelled input fields are.
///
/// @planks("the terminal object model is built")
fn controls(grid: &[Vec<String>]) -> Vec<Region> {
    let mut regions = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        regions.extend(button(grid, row, y));
        regions.extend(textbox(grid, row, y));
    }
    regions
}

/// The button `row` draws: a label between square brackets, the label naming
/// the button.
///
/// @planks("the model contains a region with the role {string} named {string}")
fn button(grid: &[Vec<String>], row: &[String], y: usize) -> Option<Region> {
    let open = row.iter().position(|cell| cell.as_str() == "[")?;
    let close = (open + 1..row.len()).find(|&x| row[x].as_str() == "]")?;
    let label = row[open + 1..close].concat().trim().to_string();
    let rect = Rect {
        x: open as u16,
        y: y as u16,
        width: (close - open + 1) as u16,
        height: 1,
    };
    Some(Region {
        role: Role::Button,
        name: Some(label.clone()),
        text: Some(label),
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// The textbox `row` draws: a field of underscores, named by the label ending
/// in the colon before it.
///
/// @planks("the model contains a region with the role {string} labelled {string}")
fn textbox(grid: &[Vec<String>], row: &[String], y: usize) -> Option<Region> {
    let start = row.iter().position(|cell| cell.as_str() == "_")?;
    let colon = row[..start].iter().rposition(|cell| cell.as_str() == ":")?;
    let label = row[..colon].concat().trim().to_string();
    let end = (start..row.len())
        .find(|&x| row[x].as_str() != "_")
        .unwrap_or(row.len());
    let rect = Rect {
        x: start as u16,
        y: y as u16,
        width: (end - start) as u16,
        height: 1,
    };
    Some(Region {
        role: Role::Textbox,
        name: Some(label),
        text: None,
        selected: false,
        rect,
        children: Vec::new(),
        style: None,
        cells: cells_of(grid, rect),
    })
}

/// The screen cells `rect` covers, one inner vector per row.
///
/// @planks("the region named {string} reports the screen cell at its own row {int} column {int}")
fn cells_of(grid: &[Vec<String>], rect: Rect) -> Vec<Vec<String>> {
    (rect.y..rect.y + rect.height)
        .map(|y| {
            (rect.x..rect.x + rect.width)
                .map(|x| grid[y as usize][x as usize].clone())
                .collect()
        })
        .collect()
}

/// An address into the model: the role and name a region carries, optionally
/// scoped to the region carrying a given name.
///
/// @planks("the locator for the {string} named {string} is resolved")
/// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
#[derive(Debug, Clone)]
pub struct Locator {
    role: String,
    name: Option<String>,
    scope: Option<String>,
    ordinal: Option<usize>,
}

impl Locator {
    /// A locator for the region playing `role` and carrying `name`.
    ///
    /// @planks("the locator for the {string} named {string} is resolved")
    pub fn new(role: &str, name: &str) -> Locator {
        Locator {
            role: role.to_string(),
            name: Some(name.to_string()),
            scope: None,
            ordinal: None,
        }
    }

    /// A locator for every region playing `role`, carrying no name, so an
    /// address stating a role alone reports what that role reaches on its own.
    ///
    /// @planks("the operator executes {string}")
    pub fn of_role(role: &str) -> Locator {
        Locator {
            role: role.to_string(),
            name: None,
            scope: None,
            ordinal: None,
        }
    }

    /// A locator for the region playing `role` at position `ordinal`, counted
    /// from one among the regions of that role it searches, so a region carrying
    /// no name a locator can trust is still addressed deterministically.
    ///
    /// @planks("the locator addresses the first {string} of the region named {string}")
    pub fn nth(role: &str, ordinal: usize) -> Locator {
        Locator {
            role: role.to_string(),
            name: None,
            scope: None,
            ordinal: Some(ordinal),
        }
    }

    /// The same locator, searching only inside the region named `scope`.
    ///
    /// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
    pub fn within(self, scope: &str) -> Locator {
        Locator {
            scope: Some(scope.to_string()),
            ..self
        }
    }

    /// The name of the region this locator searches inside, when it is scoped.
    ///
    /// @planks("the plan records the locator's binding as {string}")
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    /// Resolve this locator against `model`. Resolution is mechanical: it reads
    /// the model and invokes no inference. A name matches case-sensitively, and
    /// several matches are an ambiguity rather than a choice. A locator carrying
    /// an ordinal addresses the region at that position among the ones it
    /// matches, so it binds one region or none.
    ///
    /// A scope narrows the matches to the regions drawn inside the region it
    /// names. Containment is read off the rectangles rather than off the tree,
    /// because a region drawn inside a pane is not always filed under it: the
    /// model reads controls off the whole screen, so a button drawn inside a
    /// pane is a sibling of that pane. A scope means the part of the screen the
    /// named region occupies, which is what an operator saw when they recorded
    /// it.
    ///
    /// @planks("the locator for the {string} named {string} is resolved")
    /// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
    /// @planks("the locator addresses the first {string} of the region named {string}")
    /// @planks("that plan is replayed")
    pub fn resolve(&self, model: &Model) -> Resolution {
        let scope = self.scope.as_ref().map(|scope| {
            model
                .find_named(scope)
                .unwrap_or_else(|| panic!("the model contains no region named {scope:?}"))
        });
        let mut found = Vec::new();
        model.root.collect(
            &|region| {
                region.role() == self.role
                    && self
                        .name
                        .as_ref()
                        .is_none_or(|name| region.name.as_deref() == Some(name.as_str()))
                    && scope.is_none_or(|scope| drawn_inside(scope.rect, region.rect))
            },
            &mut found,
        );
        if let Some(ordinal) = self.ordinal {
            return match found.into_iter().nth(ordinal - 1) {
                Some(region) => Resolution::One(region),
                None => Resolution::NoMatch,
            };
        }
        match found.len() {
            0 => Resolution::NoMatch,
            1 => Resolution::One(found.remove(0)),
            count => Resolution::Ambiguous(count),
        }
    }
}

/// Whether `inner` is drawn inside `outer`, read off the rectangles the model
/// reports. A region is inside itself, so a scope stays a candidate for its own
/// locator exactly as it was when a scope searched its own subtree.
///
/// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
/// @planks("that plan is replayed")
fn drawn_inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// What resolving a locator against the model found.
///
/// @planks("the resolved region's text is {string}")
/// @planks("resolution fails and reports no match")
/// @planks("resolution fails and reports {int} matches")
#[derive(Debug)]
pub enum Resolution {
    /// The one region the locator addresses.
    One(Region),
    /// The locator addresses no region.
    NoMatch,
    /// The locator addresses several regions, and reports how many.
    Ambiguous(usize),
}

/// The narrowing a proposed locator needed before exactly one region bound.
///
/// @planks("the plan records the locator's binding as {string}")
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Binding {
    /// The proposed role and name address one region on their own.
    Exact,
    /// The proposed name addresses several regions, narrowed by the name of the
    /// region containing the one it binds.
    Scoped,
    /// The proposed name is not on the screen, so the region is addressed by its
    /// position among the regions of its role inside a named region.
    Ordinal,
}

impl Binding {
    /// The name this binding carries in a written plan.
    ///
    /// @planks("the plan records the locator's binding as {string}")
    pub fn as_str(&self) -> &'static str {
        match self {
            Binding::Exact => "exact",
            Binding::Scoped => "scoped",
            Binding::Ordinal => "ordinal",
        }
    }
}

/// A confirmed locator: the address that binds exactly one region, and the
/// narrowing it needed to get there.
///
/// @planks("the inferred locator is round-tripped against the deterministic model")
#[derive(Debug)]
pub struct Confirmation {
    pub locator: Locator,
    pub binding: Binding,
}

/// Confirm a proposed locator against `model`, the deterministic model the
/// screen yields. Confirmation runs at capture time only: it resolves the
/// proposed role and name, narrows an ambiguity to the region containing the
/// first match, and falls back to the ordinal address of the first region of
/// that role inside a named region, the screen's own root excluded, where the
/// proposed name is not on the screen at all. A proposal that nothing
/// deterministic addresses is refused, so a name the engine invented never
/// reaches a plan.
///
/// @planks("the inferred locator is round-tripped against the deterministic model")
/// @planks("the locator is scoped to the region containing that item")
/// @planks("the locator addresses the first {string} of the region named {string}")
pub fn confirm(model: &Model, role: &str, name: &str) -> Option<Confirmation> {
    match Locator::new(role, name).resolve(model) {
        Resolution::One(_) => Some(Confirmation {
            locator: Locator::new(role, name),
            binding: Binding::Exact,
        }),
        Resolution::Ambiguous(_) => {
            let scope = narrowing_scope(&model.root, &|region| {
                region.role() == role && region.name.as_deref() == Some(name)
            })?;
            let locator = Locator::new(role, name).within(&scope);
            match locator.resolve(model) {
                Resolution::One(_) => Some(Confirmation {
                    locator,
                    binding: Binding::Scoped,
                }),
                _ => None,
            }
        }
        Resolution::NoMatch => {
            let scope = narrowing_scope(&model.root, &|region| region.role() == role)?;
            let locator = Locator::nth(role, 1).within(&scope);
            match locator.resolve(model) {
                Resolution::One(_) => Some(Confirmation {
                    locator,
                    binding: Binding::Ordinal,
                }),
                _ => None,
            }
        }
    }
}

/// The scope a locator narrows to for a match somewhere under `root`, where the
/// screen itself is never that scope. The root region covers the whole terminal,
/// so scoping to it selects exactly what an unscoped locator already selects: it
/// is a scope in name only, and confirming through it would report a narrowing
/// that never happened. A proposal the screen carries nowhere but its own root
/// yields none here and is refused, which is what keeps a name the engine
/// invented from binding to whatever region of that role happens to be drawn
/// first.
///
/// @planks("the inferred locator is round-tripped against the deterministic model")
/// @planks("the locator is scoped to the region containing that item")
/// @planks("the locator addresses the first {string} of the region named {string}")
fn narrowing_scope(root: &Region, matches: &impl Fn(&Region) -> bool) -> Option<String> {
    root.children
        .iter()
        .find_map(|child| scope_of(child, matches))
}

/// The name of the region containing the first region under `region` that
/// `matches`, which is the scope a locator narrows to. A match whose containing
/// region carries no name yields none, because a scope a locator cannot address
/// narrows nothing.
///
/// @planks("the locator is scoped to the region containing that item")
fn scope_of(region: &Region, matches: &impl Fn(&Region) -> bool) -> Option<String> {
    for child in &region.children {
        if matches(child) {
            return region.name.clone();
        }
        if let Some(name) = scope_of(child, matches) {
            return Some(name);
        }
    }
    None
}
