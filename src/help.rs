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

/// The help an operator on a terminal sees: the bundled help with whatever the
/// model made of the skill on the tagline line, or the unavailable notice when
/// the model gave no `expansion`. The assistant draws its own box beneath this
/// output, so the help itself carries no prompt.
///
/// @planks("the operator runs {string} in an interactive terminal")
pub fn interactive(expansion: Option<&str>) -> String {
    let tagline = expansion.unwrap_or_else(|| UNAVAILABLE.trim());
    HELP.lines()
        .map(|line| {
            if line.contains(TAGLINE_PLACEHOLDER) {
                tagline
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
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
