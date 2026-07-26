//! The inspect command: read the terminal object model of a running program and
//! render it for an operator discovering the roles and names to address.

use crate::process::PreparedProcess;
use crate::pty::capture_interactive;
use crate::tom::{Model, Region, build};
use std::time::{Duration, Instant};

/// How long a program is given to draw a region before its screen is read as it
/// stands, so a program that draws nothing is reported rather than waited on.
const DRAW_DEADLINE: Duration = Duration::from_secs(2);

/// Launch `command` on a PTY and read the model of the screen it draws, taking
/// the screen the moment it carries a region.
///
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the fixture terminal program as JSON")
/// @planks("the operator inspects the command {string}")
pub fn model(command: &str) -> Result<Model, String> {
    let session = capture_interactive(&PreparedProcess {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), command.to_string()],
        env: Vec::new(),
        cwd: None,
        cleanup: Vec::new(),
    })?;
    let deadline = Instant::now() + DRAW_DEADLINE;
    loop {
        let model = build(&session.screen());
        if !model.root.children.is_empty() || Instant::now() >= deadline {
            return Ok(model);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Render a model as the listing an operator reads: one line per region, nested
/// under the region containing it, naming the role it plays and the name a
/// locator matches. A model carrying no region reports that instead.
///
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the command {string}")
pub fn render(model: &Model) -> String {
    if model.root.children.is_empty() {
        return "no regions on screen".to_string();
    }
    let mut lines = Vec::new();
    describe(&model.root, 0, &mut lines);
    lines.join("\n")
}

/// Write `region` and everything nested inside it into `lines`, one line each,
/// indented by the depth the region sits at.
///
/// @planks("the operator inspects the fixture terminal program")
fn describe(region: &Region, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let role = region.role();
    lines.push(match &region.name {
        Some(name) => format!("{indent}{role} {name:?}"),
        None => format!("{indent}{role}"),
    });
    for child in &region.children {
        describe(child, depth + 1, lines);
    }
}
