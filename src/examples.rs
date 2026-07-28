//! Probing a program's documented examples. A `--help` text and a tldr page
//! both make claims about what a program does, and a claim is the one thing
//! Tinman does not accept, so each documented example is a hypothesis to run
//! rather than a fact to repeat. Every line is tried inside the sandbox, and
//! what the program honoured becomes a replayable plan while what it refused is
//! reported as a finding.

use crate::bwrap::SANDBOX_HOME;
use crate::inspect;
use crate::plan::{Action, Expectation, FlowStep, Plan, Source, TuiProcess};
use crate::sandbox::{CommandSpec, SandboxSpec};
use crate::screen::VirtualScreen;
use std::path::Path;
use std::time::Duration;

/// The project a tldr page belongs to, as a plan crediting it names it.
const TLDR_PROJECT: &str = "tldr-pages";

/// The terms a tldr page comes with.
const TLDR_LICENCE: &str = "CC-BY-4.0";

/// The platforms a page is asked for, in order: the platform page first and the
/// common one where the project carries no platform page. Two attempts, and not
/// a cache.
const PLATFORMS: [&str; 2] = ["linux", "common"];

/// How long a single page request is given before it counts as no page, so a
/// source that accepts the connection and withholds the page does not hold the
/// probe open.
const FETCH_CEILING: Duration = Duration::from_secs(5);

/// What a program is asked to say about itself.
const HELP_FLAG: &str = "--help";

/// The heading the block of examples sits under in a conventional help text.
const EXAMPLES_HEADING: &str = "Examples:";

/// The indent a command line carries inside that block. The convention puts a
/// description above each command line and indents the command line deeper, so
/// the deeper indent is what tells a line to run from a line about it.
const COMMAND_INDENT: &str = "    ";

/// The file Tinman supplies for a documented example whose placeholder names a
/// path to a file, and the text it carries. The example reads it inside the
/// sandbox, where the write lands in the overlay that goes when the sandbox
/// goes, so the operator's own tree gains nothing.
const FIXTURE_FILE: &str = "tinman-fixture.txt";
const FIXTURE_TEXT: &str = "tinman fixture";

/// The modifiers a keypress notation may name in front of a key Tinman can
/// send.
const MODIFIERS: [&str; 3] = ["Ctrl", "Alt", "Shift"];

/// What a probe produced: the listing an operator reads, and the plan the
/// honoured examples make.
///
/// @planks("the operator inspects {string} with its documented examples")
/// @planks("the operator inspects {string} with its documented examples and writes a plan")
#[derive(Debug)]
pub struct Probe {
    pub listing: String,
    pub plan: Plan,
}

/// What one documented example turned out to be.
#[derive(Debug)]
enum Documented {
    /// A command line to run, with its placeholders filled.
    Command(String),
    /// Keys to send to the program the documented command lines drive.
    Keys(Vec<String>),
    /// A line nothing can be made of as written.
    Untestable(String),
}

/// Probe `command`'s documented examples over `workspace`, reading pages from
/// `page_base_url`. The program's own help came out of the binary in hand, so it
/// is ground truth; a page describes some version of some machine's program, so
/// it is a hint. Nothing outranks what the binary says about itself, and both
/// are run rather than believed.
///
/// @planks("the operator inspects {string} with its documented examples")
/// @planks("the operator inspects {string} with its documented examples and writes a plan")
/// @planks("the operator inspects the fixture terminal program with its documented examples")
/// @planks("the operator inspects the fixture terminal program with its documented examples and writes a plan")
/// @planks("the inspect output reports {string} as the ground-truth example")
/// @planks("the inspect output reports {string} as a hint")
/// @planks("the inspect output reports the example {string} was refused")
/// @planks("the inspect output reports {string} as untestable as written")
/// @planks("the inspect output reports no tldr page was read")
/// @planks("the inspect output reports the examples the program's own help carries")
/// @planks("the written plan credits the tldr-pages project for the page it read")
/// @planks("the written plan credits no page")
/// @planks("the written plan names {string} as that page's licence")
/// @planks("the written plan names the command Tinman ran with its substituted path")
/// @planks("the written plan records {string} as the documented example it came from")
/// @planks("the written plan carries a key step sending {string}")
/// @planks("the written plan carries no key step for {string}")
pub fn probe(command: &str, workspace: &Path, page_base_url: &str) -> Result<Probe, String> {
    let spec = probe_sandbox();
    let mut lines = Vec::new();

    let from_binary = examples_in_help(&help_text(command, workspace, &spec)?);
    for example in &from_binary {
        lines.push(format!("ground-truth {example:?}"));
    }

    let name = page_name(command);
    let page = fetch_page(page_base_url, &name);
    let from_page = match &page {
        Some(markdown) => examples_in_page(markdown),
        None => {
            lines.push(format!("no tldr page for {name:?}"));
            Vec::new()
        }
    };
    for example in &from_page {
        lines.push(format!("hint {example:?}"));
    }

    let mut flow: Vec<FlowStep> = Vec::new();
    let mut sources: Vec<Source> = Vec::new();
    // The page is somebody else's work, so the plan says whose it read and on
    // what terms. A plan built from the binary alone read nothing of theirs.
    if page.is_some() {
        sources.push(page_credit(&name));
    }

    for example in from_binary.iter().chain(from_page.iter()) {
        match documented(example) {
            Documented::Untestable(reason) => {
                lines.push(format!("untestable as written {example:?}: {reason}"));
            }
            Documented::Keys(keys) => match flow.last_mut() {
                Some(FlowStep::Tui(driven)) => {
                    driven.steps.extend(keys.into_iter().map(Action::Press))
                }
                _ => lines.push(format!(
                    "untestable as written {example:?}: no documented command line carries it"
                )),
            },
            Documented::Command(line) => {
                let launched = inspect::run(&spec, &line, workspace)?;
                // Only an example the program honoured earns an expectation. A
                // refused one is the finding the operator wants, that the
                // documentation and the binary disagree, so it is reported
                // rather than written into a plan as though it had passed.
                if !launched.honoured {
                    lines.push(format!(
                        "refused {example:?}: the program left an unsuccessful status"
                    ));
                    continue;
                }
                lines.push(line.clone());
                lines.push(inspect::render(&launched.model));
                let mut steps = Vec::new();
                let settled = settled_text(&launched.screen);
                if !settled.is_empty() {
                    steps.push(Action::Expect(Expectation {
                        text: settled,
                        within: None,
                        locator: None,
                    }));
                }
                flow.push(FlowStep::Tui(TuiProcess {
                    command: line,
                    steps,
                }));
                // What Tinman ran and what it read are two facts: the flow
                // carries the line Tinman assembled, and this carries the
                // example the page or the help wrote.
                sources.push(Source {
                    source: format!("documented example: {example}"),
                    licence: None,
                });
            }
        }
    }

    Ok(Probe {
        listing: lines.join("\n"),
        plan: Plan {
            sandbox: spec,
            flow,
            sources,
        },
    })
}

/// Write a probed plan at `path`.
///
/// @planks("the operator inspects {string} with its documented examples and writes a plan")
/// @planks("the operator inspects the fixture terminal program with its documented examples and writes a plan")
pub fn write_plan(plan: &Plan, path: &Path) -> Result<(), String> {
    let written = serde_yaml::to_string(plan).map_err(|e| e.to_string())?;
    std::fs::write(path, written)
        .map_err(|e| format!("the plan {} was not written: {e}", path.display()))
}

/// The sandbox a probe runs in. A documented example names its program the way
/// documentation does, by name rather than by path, so the sandbox home the
/// program under inspection sits in leads the search path: the binary being
/// documented is the one in the operator's own tree, not whichever one of that
/// name the system carries. Nothing else about the isolation moves, and the
/// plan carries this path so replay resolves the same way.
///
/// @planks("the operator inspects the fixture terminal program with its documented examples")
/// @planks("the inspect output reports {string} as the ground-truth example")
fn probe_sandbox() -> SandboxSpec {
    SandboxSpec {
        path: vec![SANDBOX_HOME.to_string(), "/bin".to_string()],
        ..SandboxSpec::default()
    }
}

/// What `command` says about itself when asked for its help. The binary is
/// asked directly rather than through a shell, because a shell answers for a
/// name it carries as a builtin before it looks for the program of that name,
/// and the program in hand is the whole point of ground truth. A program that
/// answers nothing has no help to read, which is a normal answer rather than a
/// failure.
///
/// @planks("the inspect output reports the examples the program's own help carries")
/// @planks("the inspect output reports {string} as the ground-truth example")
fn help_text(command: &str, workspace: &Path, spec: &SandboxSpec) -> Result<String, String> {
    let mut tokens = command.split_whitespace();
    let program = tokens.next().unwrap_or(command).to_string();
    let mut args: Vec<String> = tokens.map(str::to_string).collect();
    args.push(HELP_FLAG.to_string());
    let launched = inspect::run_program(spec, &CommandSpec { program, args }, workspace)?;
    Ok(launched.screen.contents())
}

/// The command lines the examples block of a help text carries.
///
/// @planks("the inspect output reports the examples the program's own help carries")
/// @planks("the inspect output reports {string} as the ground-truth example")
fn examples_in_help(help: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let mut inside = false;
    for line in help.lines() {
        let carried = line.trim();
        if carried == EXAMPLES_HEADING {
            inside = true;
            continue;
        }
        if inside && !carried.is_empty() && line.starts_with(COMMAND_INDENT) {
            examples.push(carried.to_string());
        }
    }
    examples
}

/// The examples a tldr page carries. The style guide writes each one on its own
/// line between backticks, and the page arrives as the raw markdown the guide
/// specifies, so the notation reaches here as written.
///
/// @planks("the inspect output reports {string} as a hint")
fn examples_in_page(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('`')?.strip_suffix('`'))
        .map(str::to_string)
        .collect()
}

/// The page name for `command`: the program it names, without the path an
/// operator reached it by.
///
/// @planks("the inspect output reports no tldr page was read")
fn page_name(command: &str) -> String {
    let program = command.split_whitespace().next().unwrap_or(command);
    program.rsplit('/').next().unwrap_or(program).to_string()
}

/// The tldr page for `name`, read as raw markdown from the configured source. A
/// page the project does not carry, and a source that does not answer, are both
/// no page.
///
/// @planks("the inspect output reports no tldr page was read")
/// @planks("the written plan credits the tldr-pages project for the page it read")
/// @planks("the page is read as raw markdown from the tldr project")
/// @planks("the inference request carries the tldr page for {string}")
/// @planks("the inference request carries no tldr page")
pub fn fetch_page(base_url: &str, name: &str) -> Option<String> {
    for platform in PLATFORMS {
        let url = format!("{base_url}/{platform}/{name}.md");
        let call = ureq::get(&url)
            .config()
            .timeout_global(Some(FETCH_CEILING))
            .build();
        if let Ok(mut response) = call.call()
            && let Ok(markdown) = response.body_mut().read_to_string()
        {
            return Some(markdown);
        }
    }
    None
}

/// The credit a plan gives the tldr page it read for `name`: whose work it was
/// and the terms it came with. The credit rides in the artifact the operator
/// commits rather than on a screen they saw once.
///
/// @planks("the written plan credits the tldr-pages project for the page it read")
/// @planks("the written plan names {string} as that page's licence")
/// @planks("the plan credits the tldr-pages project for the page it read")
/// @planks("the plan names {string} as that page's licence")
pub fn page_credit(name: &str) -> Source {
    Source {
        source: format!("{TLDR_PROJECT}, the page for {name:?}"),
        licence: Some(TLDR_LICENCE.to_string()),
    }
}

/// What `example` turns out to be. A page's keypress notation documents an
/// interaction rather than an invocation, so it becomes keys to send; anything
/// else is a command line whose placeholders are holes a reader is expected to
/// fill, and passing one to the program as written would hand it an argument
/// nobody meant.
///
/// @planks("the inspect output reports {string} as untestable as written")
/// @planks("the written plan carries a key step sending {string}")
fn documented(example: &str) -> Documented {
    match example.trim_start().starts_with('<') {
        true => keypresses(example.trim()),
        false => filled(example.trim()),
    }
}

/// The keys `notation` spells, or why Tinman cannot send them.
///
/// @planks("the written plan carries a key step sending {string}")
/// @planks("the written plan carries no key step for {string}")
fn keypresses(notation: &str) -> Documented {
    let mut keys = Vec::new();
    let mut rest = notation;
    while let Some(open) = rest.strip_prefix('<') {
        let Some((spelled, tail)) = open.split_once('>') else {
            return Documented::Untestable("the keypress notation is never closed".to_string());
        };
        match sendable(spelled) {
            Some(key) => keys.push(key),
            None => {
                return Documented::Untestable(format!("Tinman sends no key spelled {spelled:?}"));
            }
        }
        rest = tail.trim_start();
    }
    match rest.is_empty() && !keys.is_empty() {
        true => Documented::Keys(keys),
        false => Documented::Untestable(
            "the line mixes keypress notation with text Tinman cannot read as either".to_string(),
        ),
    }
}

/// The key `spelled` names, where Tinman can send it: a key on its own, or one
/// modifier in front of a key. A chord of several modifiers is a key no
/// terminal carries one sequence for, so it is reported rather than guessed at.
///
/// @planks("the written plan carries no key step for {string}")
fn sendable(spelled: &str) -> Option<String> {
    let parts: Vec<&str> = spelled.split_whitespace().collect();
    match parts.as_slice() {
        [key] => Some((*key).to_string()),
        [modifier, key] if MODIFIERS.contains(modifier) => Some(format!("{modifier} {key}")),
        _ => None,
    }
}

/// `example` with every placeholder filled, or the placeholder that leaves it
/// untestable as written.
///
/// @planks("the inspect output reports {string} as untestable as written")
/// @planks("the written plan names the command Tinman ran with its substituted path")
fn filled(example: &str) -> Documented {
    let mut line = String::new();
    let mut rest = example;
    let mut seeded = false;
    while let Some(open) = rest.find("{{") {
        let Some(close) = rest[open..].find("}}") else {
            return Documented::Untestable("the placeholder is never closed".to_string());
        };
        let name = &rest[open + 2..open + close];
        let Some(value) = value_for(name) else {
            return Documented::Untestable(format!(
                "Tinman has no value for the placeholder {name:?}"
            ));
        };
        line.push_str(&rest[..open]);
        line.push_str(&value);
        seeded = true;
        rest = &rest[open + close + 2..];
    }
    line.push_str(rest);
    // The fixture the filled path names is made by the line that reads it, so
    // it exists inside the sandbox and nowhere else.
    match seeded {
        true => Documented::Command(format!(
            "printf '{FIXTURE_TEXT}\\n' > {FIXTURE_FILE}; {line}"
        )),
        false => Documented::Command(line),
    }
}

/// The value Tinman fills the placeholder `name` with. A placeholder naming a
/// path to a file gets the fixture file Tinman supplies; a placeholder naming
/// something only the reader knows gets none.
///
/// @planks("the inspect output reports {string} as untestable as written")
fn value_for(name: &str) -> Option<String> {
    match name.rsplit('/').next() {
        Some("file") => Some(FIXTURE_FILE.to_string()),
        _ => None,
    }
}

/// The text the program settled on: the lowest row it left something on. An
/// expectation on that row proves the example ran while the evidence is still
/// in hand, rather than leaving that proof to whoever replays the plan.
///
/// @planks("the written plan carries an expectation on the text {string}")
fn settled_text(screen: &VirtualScreen) -> String {
    screen
        .rows()
        .iter()
        .rev()
        .map(|row| row.concat().trim_end().to_string())
        .find(|line| !line.is_empty())
        .unwrap_or_default()
}
