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

/// The characters a bordered pane is drawn with.
const TOP_LEFT: &str = "\u{250c}";
const TOP_RIGHT: &str = "\u{2510}";
const BOTTOM_LEFT: &str = "\u{2514}";
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
    Screen,
    Region,
    Menu,
    Menuitem,
    List,
    Listitem,
    Table,
    Row,
    Column,
    Dialog,
    Button,
    Textbox,
    Statusbar,
    #[serde(rename = "message-pane")]
    MessagePane,
    Message,
    Tree,
    Treeitem,
}

/// Every role the model defines, so a name maps to its role through one list.
const ROLES: [Role; 17] = [
    Role::Screen,
    Role::Region,
    Role::Menu,
    Role::Menuitem,
    Role::List,
    Role::Listitem,
    Role::Table,
    Role::Row,
    Role::Column,
    Role::Dialog,
    Role::Button,
    Role::Textbox,
    Role::Statusbar,
    Role::MessagePane,
    Role::Message,
    Role::Tree,
    Role::Treeitem,
];

impl Role {
    /// The name this role carries in the model.
    ///
    /// @planks("the region named {string} has the role {string}")
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Screen => "screen",
            Role::Region => "region",
            Role::Menu => "menu",
            Role::Menuitem => "menuitem",
            Role::List => "list",
            Role::Listitem => "listitem",
            Role::Table => "table",
            Role::Row => "row",
            Role::Column => "column",
            Role::Dialog => "dialog",
            Role::Button => "button",
            Role::Textbox => "textbox",
            Role::Statusbar => "statusbar",
            Role::MessagePane => "message-pane",
            Role::Message => "message",
            Role::Tree => "tree",
            Role::Treeitem => "treeitem",
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
                role: Role::Screen,
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
/// the lines they list, the sibling regions a vertical rule splits it into, and
/// the status bar its bottom line carries.
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
            role: Role::Screen,
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
fn panes(grid: &[Vec<String>], screen: &VirtualScreen) -> Vec<Region> {
    let mut regions = Vec::new();
    for y in 0..grid.len() {
        for x in 0..grid[y].len() {
            if grid[y][x] != TOP_LEFT {
                continue;
            }
            let Some(right) = (x + 1..grid[y].len()).find(|&col| grid[y][col] == TOP_RIGHT) else {
                continue;
            };
            let Some(bottom) = (y + 1..grid.len()).find(|&row| grid[row][x] == BOTTOM_LEFT) else {
                continue;
            };
            regions.push(pane_region(grid, screen, x, y, right, bottom));
        }
    }
    regions
}

/// One bordered pane read as a list: its title is its name and each line it
/// shows is an item, the reversed line being the selected one.
///
/// @planks("the terminal object model is built")
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
    let mut items = Vec::new();
    for row in y + 1..bottom {
        let text = grid[row][x + 1..right].concat().trim_end().to_string();
        if text.is_empty() {
            continue;
        }
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
        role: Role::Statusbar,
        name: None,
        text: Some(text),
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
    name: String,
    scope: Option<String>,
}

impl Locator {
    /// A locator for the region playing `role` and carrying `name`.
    ///
    /// @planks("the locator for the {string} named {string} is resolved")
    pub fn new(role: &str, name: &str) -> Locator {
        Locator {
            role: role.to_string(),
            name: name.to_string(),
            scope: None,
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

    /// Resolve this locator against `model`. Resolution is mechanical: it reads
    /// the model and invokes no inference. A name matches case-sensitively, and
    /// several matches are an ambiguity rather than a choice.
    ///
    /// @planks("the locator for the {string} named {string} is resolved")
    /// @planks("the locator for the {string} named {string} is resolved within the region named {string}")
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
                region.role() == self.role && region.name.as_deref() == Some(self.name.as_str())
            },
            &mut found,
        );
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
