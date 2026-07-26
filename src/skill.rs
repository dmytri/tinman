//! The one skills.sh-compatible skill Tinman ships.
//!
//! The skill is the single source of truth for what Tinman is and how it is
//! used. It has three consumers: an external coding agent reads the whole file,
//! the assistant answers from the whole body, and the acronym generator reads
//! the name and description.

use serde::{Deserialize, Serialize};

/// The shipped skill, inlined at build time, so a running Tinman reads the same
/// bytes an installing coding agent reads.
const BUNDLED: &str = include_str!("../assets/skill/SKILL.md");

/// The instruction the acronym generator is given before the skill's name and
/// description, inlined at build time.
const ACRONYM_PROMPT: &str = include_str!("../assets/help/acronym-prompt.txt");

/// The instruction the assistant is given before the skill, inlined at build
/// time.
const ASSISTANT_INSTRUCTION: &str = include_str!("../assets/help/assistant-instruction.txt");

/// The front matter of a skills.sh skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatter {
    /// The skill's identifier, in kebab-case.
    pub name: String,
    /// One line stating what the skill covers and when to use it.
    pub description: String,
}

/// A skill: its front matter and the Markdown body beneath it.
#[derive(Debug, Clone)]
pub struct Skill {
    pub front_matter: FrontMatter,
    pub body: String,
}

/// Parse skill text: YAML front matter fenced by `---` lines, then the body.
///
/// @planks("the skill front matter is parsed")
pub fn parse(text: &str) -> Result<Skill, String> {
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| "the skill does not open with a --- front matter fence".to_string())?;
    let (front, body) = rest
        .split_once("\n---\n")
        .ok_or_else(|| "the skill front matter is not closed by a --- fence".to_string())?;
    let front_matter: FrontMatter = serde_yaml::from_str(front)
        .map_err(|e| format!("the skill front matter did not parse: {e}"))?;
    Ok(Skill {
        front_matter,
        body: body.to_string(),
    })
}

/// The skill Tinman ships.
///
/// @planks("Tinman loads its bundled skill")
pub fn bundled() -> Skill {
    parse(BUNDLED).expect("the bundled skill parses")
}

/// The context the acronym generator is given: the instruction the asset
/// carries, then the skill's name and description.
///
/// @planks("the acronym context is built")
pub fn acronym_context() -> String {
    let skill = bundled();
    format!(
        "{}\n\n{}\n{}",
        ACRONYM_PROMPT.trim(),
        skill.front_matter.name,
        skill.front_matter.description
    )
}

/// The context the assistant answers from: the instruction the asset carries,
/// then the skill's name and description, then the whole skill body.
///
/// @planks("the assistant context is built")
pub fn assistant_context() -> String {
    let skill = bundled();
    format!(
        "{}\n\n{}\n{}\n{}",
        ASSISTANT_INSTRUCTION.trim(),
        skill.front_matter.name,
        skill.front_matter.description,
        skill.body
    )
}
