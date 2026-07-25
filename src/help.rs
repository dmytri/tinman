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

/// The prompt inviting the operator to put a question to the assistant, inlined
/// at build time.
const ASSISTANT_PROMPT: &str = include_str!("../assets/help/assistant-prompt.txt");

/// The help an operator on a terminal sees: the bundled help with whatever the
/// model made of the skill on the tagline line, or the unavailable notice. The
/// assistant prompt closes the help wherever inference answers.
///
/// @planks("the operator runs {string} in an interactive terminal")
pub fn interactive(settings: &crate::inference::Settings) -> String {
    let expansion = crate::inference::expansion(settings);
    let tagline = expansion
        .as_deref()
        .unwrap_or_else(|| UNAVAILABLE.trim())
        .to_string();
    let filled = HELP
        .lines()
        .map(|line| {
            if line.contains(TAGLINE_PLACEHOLDER) {
                tagline.as_str()
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
    match expansion {
        Some(_) => format!("{}\n\n{}", filled.trim_end(), ASSISTANT_PROMPT.trim()),
        None => filled,
    }
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
