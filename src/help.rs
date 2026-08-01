//! The conventional help text: a human-owned asset inlined at build time.

/// The tagline placeholder line, which inference fills and which is removed
/// when nothing fills it.
///
/// @planks("its tagline placeholders are counted")
pub const TAGLINE_PLACEHOLDER: &str = "{{tagline}}";

/// The help asset, inlined at build time.
const HELP: &str = include_str!("../assets/help/tinman.txt");

/// The notice that fills the tagline when inference is unavailable, inlined at
/// build time.
const UNAVAILABLE: &str = include_str!("../assets/help/inference-unavailable.txt");

/// The escapes that draw a line in a colour other than the terminal's default
/// foreground, and put the default foreground back at the end of it. The
/// assistant's own box reuses these, since a box drawn in one colour beside a
/// line drawn in another would give one screen two conventions.
pub(crate) const COLOUR: &str = "\u{1b}[36m";
pub(crate) const DEFAULT_COLOUR: &str = "\u{1b}[39m";

/// The marker a fenced block opens and closes with in the text the model
/// returns. It is markdown notation rather than the expansion's wording, so it
/// is left off the tagline and the words it delimited are kept. The assistant
/// reuses this marker for the same reason: one model, one fence convention.
pub(crate) const FENCE: &str = "```";

/// The escape that moves the cursor down a row, scrolling at the foot of the
/// screen as a line feed does. The help drawn ahead of the tagline moves the
/// cursor rather than ending lines, so the drawing is one pass of the terminal
/// and the filled help written over it is the help the operator keeps.
const INDEX: &str = "\u{1b}D";

/// The help an operator on a terminal sees: the bundled help with whatever the
/// model made of the skill on the tagline line, or the unavailable notice when
/// the model gave no `expansion`. The assistant draws its own box beneath this
/// output, so the help itself carries no prompt.
///
/// The name and the tagline are the program saying what it is, and the asset's
/// first line is the name, so they are drawn in a colour of their own and the
/// reference material beneath them is left in the terminal's own foreground.
/// NO_COLOR is honoured, since it is the convention every other terminal program
/// already answers to.
///
/// @planks("the operator runs {string} in an interactive terminal")
/// @planks("the name {string} is drawn in a colour other than the default foreground")
/// @planks("the tagline is drawn in a colour other than the default foreground")
/// @planks("the Commands block is drawn in the default foreground")
/// @planks("no line on the screen carries a fence marker")
/// @planks("no placeholder delimiter is on the screen")
pub fn interactive(expansion: Option<&str>) -> String {
    let coloured = std::env::var_os("NO_COLOR").is_none();
    let paint = |line: &str| match coloured {
        true => format!("{COLOUR}{line}{DEFAULT_COLOUR}"),
        false => line.to_string(),
    };
    let tagline = tagline_index();
    rendered(expansion)
        .into_iter()
        .enumerate()
        .map(|(index, line)| match index == 0 || index == tagline {
            true => paint(&line),
            false => line,
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// The help an operator sees while the tagline is still being generated: the
/// same help with its tagline line left empty, and the cursor left where the
/// drawing began so the filled help is written over this one. The reference
/// material is on the screen at once, whatever the provider is doing, and the
/// tagline arrives into its own line rather than holding the rest back.
///
/// @planks("the operator runs {string} in an interactive terminal")
pub fn drawn_ahead(columns: u16) -> String {
    let rows: usize = rendered(Some(""))
        .iter()
        .map(|line| rows_of(line, columns.max(1) as usize))
        .sum();
    let mut drawn = String::new();
    for line in interactive(Some("")).lines() {
        drawn.push_str(line);
        drawn.push('\r');
        drawn.push_str(INDEX);
    }
    drawn.push_str(&format!("\u{1b}[{rows}A\r"));
    drawn
}

/// The rows `line` occupies on a terminal `columns` wide: a line wider than the
/// terminal wraps onto rows of its own, and an empty line still occupies one.
fn rows_of(line: &str, columns: usize) -> usize {
    line.chars().count().div_ceil(columns).max(1)
}

/// The line of the help the tagline placeholder reserves.
fn tagline_index() -> usize {
    HELP.lines()
        .position(|line| line.contains(TAGLINE_PLACEHOLDER))
        .expect("the help asset carries a tagline line")
}

/// The help's lines with `expansion` on the tagline line and every example
/// placeholder expanded, before any styling: the unavailable notice fills the
/// tagline where the model gave no expansion.
fn rendered(expansion: Option<&str>) -> Vec<String> {
    let tagline = expansion.unwrap_or_else(|| UNAVAILABLE.trim());
    let tagline = tagline
        .lines()
        .filter(|line| !line.trim_start().starts_with(FENCE))
        .collect::<Vec<&str>>()
        .join("\n");
    let values = example_values();
    HELP.lines()
        .map(|line| match line.contains(TAGLINE_PLACEHOLDER) {
            true => tagline.clone(),
            false => expand_placeholders(line, &values),
        })
        .collect()
}

/// The values that fill the tldr-pages placeholders the Examples block
/// carries, read from the asset's own "Example values:" section: each
/// indented "name: value" line beneath that heading.
fn example_values() -> Vec<(&'static str, &'static str)> {
    let mut values = Vec::new();
    let mut in_block = false;
    for line in HELP.lines() {
        if line.trim_end() == "Example values:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let Some((name, value)) = line.trim().split_once(':') else {
            continue;
        };
        values.push((name.trim(), value.trim()));
    }
    values
}

/// A line with every tldr-pages `{{name}}` placeholder it carries replaced by
/// its matching example value.
fn expand_placeholders(line: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = line.to_string();
    for (name, value) in values {
        let placeholder = format!("{{{{{name}}}}}");
        rendered = rendered.replace(&placeholder, value);
    }
    rendered
}

/// The conventional help with nothing filling the tagline: every asset line but
/// the placeholder line.
///
/// @planks("the operator runs {string} with stdout redirected to a file")
pub fn conventional() -> String {
    HELP.lines()
        .filter(|line| !line.contains(TAGLINE_PLACEHOLDER))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Tinman said in one line: the first line the asset carries beneath the one the
/// tagline placeholder reserves. The name and the tagline are the program saying
/// what it is, and the line beneath them is the program in one sentence.
///
/// @planks("it carries the one-line description from the asset at {string}")
pub fn one_line_description() -> String {
    HELP.lines()
        .skip_while(|line| !line.contains(TAGLINE_PLACEHOLDER))
        .skip(1)
        .find(|line| !line.trim().is_empty())
        .expect("the help asset carries a one-line description")
        .trim()
        .to_string()
}

/// Tinman said in the operator's terms: the asset's closing paragraph, its last
/// run of unindented lines, joined into one paragraph. The indented blocks are
/// reference material the page renders for itself.
///
/// @planks("the DESCRIPTION section carries the closing paragraph of the asset at {string}")
pub fn closing_paragraph() -> String {
    let mut paragraph: Vec<&str> = Vec::new();
    for line in HELP.lines() {
        if line.trim().is_empty() || line.starts_with(' ') {
            paragraph.clear();
        } else {
            paragraph.push(line.trim());
        }
    }
    paragraph.join(" ")
}
