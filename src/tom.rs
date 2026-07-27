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
use crate::screen::VirtualScreen;
use serde::{Deserialize, Serialize};

/// The characters a bordered pane is drawn with. A corner is drawn square or
/// rounded, and a pane is the same region to a reader either way.
const TOP_LEFT: [&str; 2] = ["\u{250c}", "\u{256d}"];
const TOP_RIGHT: [&str; 2] = ["\u{2510}", "\u{256e}"];
const BOTTOM_LEFT: [&str; 2] = ["\u{2514}", "\u{2570}"];
const HORIZONTAL: &str = "\u{2500}";
const VERTICAL: &str = "\u{2502}";

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
}

/// Every role the model defines, so a name maps to its role through one list.
const ROLES: [Role; 11] = [
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
];

impl Role {
    /// The name this role carries in the model.
    ///
    /// @planks("the region named {string} has the role {string}")
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
        }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    role: Role,
    pub name: Option<String>,
    pub text: Option<String>,
    pub selected: bool,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub rows: u16,
    pub cols: u16,
    pub root: Region,
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

/// Read `screen` into a terminal object model: the bordered panes it draws with
/// the lines they list, the sibling regions a vertical rule splits it into, the
/// menu bar its top line carries, the buttons and textboxes it draws, and the
/// status bar its bottom line carries.
///
/// @planks("the terminal object model is built")
/// @planks("the terminal object model is serialized")
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
    if let Some(menu) = menu_bar(grid, cols, screen) {
        children.push(menu);
    }
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
    Model {
        rows,
        cols,
        root: Region {
            role: Role::Application,
            name: None,
            text: None,
            selected: false,
            rect,
            children,
            cells: cells_of(grid, rect),
        },
    }
}

/// Read `screen` into a model, enriched by the configured engine. The
/// deterministic model is the spine: it stands whenever the engine is
/// unavailable, and it stands again whenever the engine answers with something
/// the model's own shape rejects.
///
/// @planks("the terminal object model is inferred")
pub fn infer(screen: &VirtualScreen, settings: &Settings) -> Model {
    let deterministic = build(screen);
    let Some(reply) = crate::inference::tom_completion(settings, &screen.contents()) else {
        return deterministic;
    };
    match serde_yaml::from_str::<Model>(&reply) {
        Ok(inferred) => inferred,
        Err(_) => deterministic,
    }
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
        cells: cells_of(grid, rect),
    })
}

/// The top line of `grid` read as a menu bar, each label it carries a menu
/// item, the reversed label being the selected one. A line drawing a border is
/// a pane's edge rather than a menu, and a line carrying one label is a
/// heading rather than a bar of items.
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
    let items = labels
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
                cells: cells_of(grid, item),
            }
        })
        .collect();
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
    /// @planks("the locator for the {string} named {string} is resolved")
    /// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
    /// @planks("the locator addresses the first {string} of the region named {string}")
    pub fn resolve(&self, model: &Model) -> Resolution {
        let root = match &self.scope {
            Some(scope) => model
                .find_named(scope)
                .unwrap_or_else(|| panic!("the model contains no region named {scope:?}")),
            None => &model.root,
        };
        let mut found = Vec::new();
        root.collect(
            &|region| {
                region.role() == self.role
                    && self
                        .name
                        .as_ref()
                        .is_none_or(|name| region.name.as_deref() == Some(name.as_str()))
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
/// that role inside a named region where the proposed name is not on the screen
/// at all. A proposal that nothing deterministic addresses is refused, so a name
/// the engine invented never reaches a plan.
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
            let scope = scope_of(&model.root, &|region| {
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
            let scope = scope_of(&model.root, &|region| region.role() == role)?;
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
