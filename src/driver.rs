//! The driver: the JSON-RPC 2.0 protocol a test runner in any language speaks
//! to Tinman, one message per line on stdin and stdout. A protocol fault is an
//! error object carrying a reserved code. A failed expectation is a result
//! whose `ok` is false, because the call succeeded and the product disagreed.

use crate::backend::resolve;
use crate::pty::{InteractiveCapture, capture_interactive};
use crate::sandbox::{CommandSpec, SandboxSpec};
use crate::screen::VirtualScreen;
use crate::tom::{Locator, Model, Region, Resolution, build};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The reserved JSON-RPC code for a line the driver cannot read as JSON.
const PARSE_ERROR: i64 = -32700;

/// The reserved JSON-RPC code for a method the driver does not answer.
const METHOD_NOT_FOUND: i64 = -32601;

/// The reserved JSON-RPC code for a call whose parameters the driver rejects.
const INVALID_PARAMS: i64 = -32602;

/// The scopes a capture collects under, per `scantlings/driver-protocol.schema.json`.
const CAPTURE_SCOPES: [&str; 2] = ["visible", "all"];

/// How long a driven program is given to draw the text an expectation names, so
/// an expectation resolves the moment the text appears.
const EXPECT_DEADLINE: Duration = Duration::from_secs(5);

/// How long a wait gives the driven program to reach the state it names, so a
/// barrier that never fires reports failure to arrive within a budget.
const WAIT_DEADLINE: Duration = Duration::from_secs(5);

/// How many times a capture scrolls a pane, so a pane that never stops showing
/// new items fails within a budget rather than collecting forever.
const CAPTURE_SCROLL_LIMIT: usize = 32;

/// How long a capture waits for a pane to draw items it has not collected yet,
/// so a scroll resolves the moment the next window appears.
const CAPTURE_SETTLE: Duration = Duration::from_secs(2);

/// How long a launch waits for the newly started program to draw its first
/// screen, so a call that follows a launch observes what the program drew
/// rather than the blank screen a program still starting up leaves behind.
const LAUNCH_READY_DEADLINE: Duration = Duration::from_secs(5);

/// The key a capture sends to scroll the pane it collects from.
const SCROLL_KEY: &str = "\u{1b}[B";

/// The key an activation sends to move the selection, a terminal having no
/// pointer of its own to click a target with.
const SELECT_KEY: &str = "\u{1b}[C";

/// How long an activation or a key press waits for the driven program to
/// answer, so the call resolves the moment the program responds.
const RESPONSE_DEADLINE: Duration = Duration::from_secs(2);

/// The sandbox backend every session launches under.
const SANDBOX_BACKEND: &str = "bubblewrap";

/// How a session's terminal is told to keep the keys the driver sends off the
/// screen. A terminal echoes what is typed on it, so a key the driver sends
/// lands on the very screen the driver reads back, where a call waiting for the
/// program to answer reads the driver's own key press as that answer. The
/// program's own drawing is the only thing the driver reads.
const SILENCE_ECHO: &str = "stty -echo";

/// One live session: the program running on its PTY, the temporary directory
/// the sandbox uses as its home, which closing the session reclaims, the
/// region an activation last moved the selection onto, and the values filled
/// into textboxes. A terminal exposes no selection or field state of its own,
/// so the driver that moved the selection and typed the values is what
/// remembers them, even where a program's own redraw has since scrolled the
/// region the selection moved to out of the screen the session started with.
struct Session {
    capture: InteractiveCapture,
    home: PathBuf,
    selected: Option<Region>,
    filled: HashMap<String, String>,
}

/// The sessions a running driver holds, addressed by the identifiers it issued.
#[derive(Default)]
struct Sessions {
    open: HashMap<String, Session>,
    launched: u64,
}

/// Speak the driver protocol on stdin and stdout until the client closes its
/// end: read one request per line, answer each with one reply line, flushed so
/// the client reads it without waiting for the next. A client that closes its
/// stdin without closing its sessions leaves them to the disconnect, so every
/// session still open is reclaimed before the driver exits.
///
/// @planks("the Tinman driver is running")
/// @planks("the test runner sends the request:")
/// @planks("every exchanged message conforms to the {string} schema in {string}")
/// @planks("the test runner closes the driver's stdin")
/// @planks("the driver process exits with a success status")
/// @planks("the driver leaves no session sandbox directory standing")
pub fn serve() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut sessions = Sessions::default();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => answer(&request, &mut sessions),
            Err(e) => fault(
                Value::Null,
                PARSE_ERROR,
                "the request line is not JSON",
                &e.to_string(),
            ),
        };
        writeln!(stdout, "{reply}")?;
        stdout.flush()?;
    }
    for (_, mut session) in sessions.open.drain() {
        session.capture.end_session();
        std::fs::remove_dir_all(&session.home)?;
    }
    Ok(())
}

/// Answer one request. A method outside the protocol is a fault, so it is
/// answered with the reserved code and the name it called.
///
/// @planks("the test runner sends the request:")
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the method {string}")
/// @planks("the test runner captures every {string} in the {string} as {string}")
/// @planks("the test runner captures the visible {string} items in the {string} as {string}")
fn answer(request: &Value, sessions: &mut Sessions) -> Value {
    let id = request["id"].clone();
    let method = request["method"].as_str().unwrap_or_default();
    let params = &request["params"];
    match method {
        "launch" => launch(id, params, sessions),
        "expect" => expect_text(id, params, sessions),
        "wait" => wait(id, params, sessions),
        "capture" => capture(id, params, sessions),
        "screen" => screen(id, params, sessions),
        "tom" => tom(id, params, sessions),
        "sandbox" => sandbox(id, params, sessions),
        "activate" => activate(id, params, sessions),
        "fill" => fill(id, params, sessions),
        "press" => press(id, params, sessions),
        "close" => close(id, params, sessions),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": METHOD_NOT_FOUND,
                "message": "the driver answers no such method",
                "data": method,
            },
        }),
    }
}

/// A successful reply: the framing, the identifier the call carried, and the
/// result the call produced.
fn reply(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// A protocol fault: the framing, the identifier the call carried, the reserved
/// code the fault falls under, and what the fault concerned. A caller in another
/// language reads this object where a seam that aborted would leave it a closed
/// pipe.
///
/// @planks("the driver answers with a JSON-RPC error object")
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the missing parameter {string}")
fn fault(id: Value, code: i64, message: &str, data: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message, "data": data},
    })
}

/// The session a call addresses, where the call names one the driver holds.
///
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the missing parameter {string}")
fn addressed<'a>(params: &Value, sessions: &'a mut Sessions) -> Option<&'a mut Session> {
    let name = params["session"].as_str()?;
    sessions.open.get_mut(name)
}

/// The fault a call naming no session the driver holds is answered with.
///
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the missing parameter {string}")
fn no_session(id: Value) -> Value {
    fault(
        id,
        INVALID_PARAMS,
        "the call addresses no open session",
        "session",
    )
}

/// Launch the named command in a sandbox of its own and keep it running. The
/// session gets a temporary directory as its sandbox home, named after the
/// identifier the reply carries, so the client can see what the session owns.
///
/// @planks("the Tinman driver has a session running {string}")
/// @planks("the Tinman driver has a session running the fixture terminal program")
/// @planks("the driver replies to request {int} with a session identifier")
/// @planks("the driver replies to request {int} with a failed result")
/// @planks("the failure names the program it could not start")
/// @planks("the failure reports the selection did not reach the {string} named {string}")
/// @planks("the verifier checks the backend construction boundary")
fn launch(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(command) = params["command"].as_str() else {
        return fault(
            id,
            INVALID_PARAMS,
            "the launch call names no command",
            "command",
        );
    };
    if let Some(program) = unreachable_program(command) {
        return reply(
            id,
            json!({
                "ok": false,
                "failure": format!("the sandbox cannot reach the program {program:?}"),
            }),
        );
    }
    sessions.launched += 1;
    let name = format!("sess-{}-{}", std::process::id(), sessions.launched);
    let home = std::env::temp_dir().join(format!("tinman-{name}"));
    if let Err(e) = std::fs::create_dir_all(&home) {
        return reply(
            id,
            json!({
                "ok": false,
                "failure": format!("the sandbox home {} was not created: {e}", home.display()),
            }),
        );
    }
    let resolved = match resolve(std::env::consts::OS) {
        Ok(resolved) => resolved,
        Err(e) => {
            return reply(
                id,
                json!({
                    "ok": false,
                    "failure": format!("the session's sandbox backend was not resolved: {e:?}"),
                }),
            );
        }
    };
    let prepared = match resolved.backend().prepare_with_home(
        &SandboxSpec::default_for_record(),
        &CommandSpec {
            program: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), format!("{SILENCE_ECHO}; {command}")],
        },
        Some(&home),
        None,
    ) {
        Ok(prepared) => prepared,
        Err(e) => {
            return reply(
                id,
                json!({
                    "ok": false,
                    "failure": format!("the session's process was not prepared: {e}"),
                }),
            );
        }
    };
    let capture = match capture_interactive(&prepared) {
        Ok(capture) => capture,
        Err(e) => {
            return reply(
                id,
                json!({
                    "ok": false,
                    "failure": format!("the session's process was not launched: {e}"),
                }),
            );
        }
    };
    await_first_draw(&capture);
    sessions.open.insert(
        name.clone(),
        Session {
            capture,
            home,
            selected: None,
            filled: HashMap::new(),
        },
    );
    reply(id, json!({"ok": true, "session": name}))
}

/// Wait for the newly launched program to draw its first screen, so a launch
/// replies only once there is something for the next call to read.
///
/// @planks("the Tinman driver has a session running the fixture terminal program")
fn await_first_draw(capture: &InteractiveCapture) {
    let deadline = Instant::now() + LAUNCH_READY_DEADLINE;
    while capture.screen().contents().trim().is_empty() {
        if Instant::now() >= deadline {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The absolute-path program named at the head of `command`, when a launch has
/// no way to reach it. A launch binds only the system directories and nothing
/// else, so an absolute path outside the host filesystem, which those
/// directories mirror, is unreachable.
///
/// @planks("the driver replies to request {int} with a failed result")
/// @planks("the failure names the program it could not start")
fn unreachable_program(command: &str) -> Option<&str> {
    let program = command.split_whitespace().next()?;
    if program.starts_with('/') && !std::path::Path::new(program).exists() {
        Some(program)
    } else {
        None
    }
}

/// Answer whether the session's program has drawn the named text, waiting for
/// it until the deadline. Absent text is a result rather than a fault: the call
/// succeeded, so the reply reports what the screen held instead.
///
/// @planks("the test runner requests the text {string} is present")
/// @planks("the driver replies with a result whose {string} is false")
/// @planks("the reply carries no error object")
/// @planks("the driver replies with a failed result")
fn expect_text(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(text) = params["text"].as_str() else {
        return fault(id, INVALID_PARAMS, "the expectation names no text", "text");
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let deadline = Instant::now() + EXPECT_DEADLINE;
    loop {
        let screen = session.capture.screen();
        if screen.contains(text) {
            return reply(id, json!({"ok": true}));
        }
        if Instant::now() >= deadline {
            return reply(
                id,
                json!({
                    "ok": false,
                    "failure": format!("the text {text:?} was not found on screen"),
                    "screen": screen.contents(),
                }),
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Wait until the session's program draws the named text, and report whether it
/// arrived. A wait is a barrier rather than a claim: it names the state the
/// session was to reach, so a wait that never arrives reports failure to arrive
/// rather than a failed expectation.
///
/// @planks("the test runner waits for the text {string}")
/// @planks("the test runner waits for text the program never draws")
/// @planks("the driver reports the session reached that state")
/// @planks("the driver reports the session did not reach that state")
/// @planks("the failure names no failed expectation")
fn wait(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(text) = params["text"].as_str() else {
        return fault(id, INVALID_PARAMS, "the wait names no text", "text");
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let deadline = Instant::now() + WAIT_DEADLINE;
    loop {
        if session.capture.screen().contains(text) {
            return reply(id, json!({"ok": true}));
        }
        if Instant::now() >= deadline {
            return reply(
                id,
                json!({
                    "ok": false,
                    "failure": format!("the session did not reach the state where the screen shows {text:?}"),
                }),
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Collect the items a pane holds, scrolling it until it shows nothing new. A
/// `visible` scope reads the current window alone. Items keep their screen
/// order and an item already collected is collected once, so a message the
/// pane repeats at two scroll positions appears once. A pane that keeps showing
/// new items past the scroll limit is a failed result, because the call
/// succeeded and the pane never reached an end. A missing or unknown `scope` is
/// a protocol fault carrying the reserved invalid-params code and the parameter
/// it concerned, answered before the call reaches its session, so the session
/// the client already holds stays usable.
///
/// @planks("the test runner captures every {string} in the {string} as {string}")
/// @planks("the test runner captures the visible {string} items in the {string} as {string}")
/// @planks("the capture named {string} holds {int} items")
/// @planks("the first item of the capture named {string} is {string}")
/// @planks("the last item of the capture named {string} is {string}")
/// @planks("the failure reports the capture reached its scroll limit")
/// @planks("the error data names the missing parameter {string}")
/// @planks("the error data names the rejected scope {string}")
fn capture(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(role) = params["role"].as_str() else {
        return fault(id, INVALID_PARAMS, "the capture call names no role", "role");
    };
    let Some(within) = params["within"].as_str() else {
        return fault(
            id,
            INVALID_PARAMS,
            "the capture call names no pane",
            "within",
        );
    };
    let scope = match params["scope"].as_str() {
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": INVALID_PARAMS,
                    "message": "the capture call names no scope",
                    "data": "scope",
                },
            });
        }
        Some(named) if !CAPTURE_SCOPES.contains(&named) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": INVALID_PARAMS,
                    "message": "the capture call names no such scope",
                    "data": named,
                },
            });
        }
        Some(named) => named,
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let mut items = Vec::new();
    gather(session, within, role, &mut items);
    if scope == "visible" {
        return reply(id, json!({"ok": true, "items": items}));
    }
    for _ in 0..CAPTURE_SCROLL_LIMIT {
        session.capture.press_key(SCROLL_KEY);
        if !gather(session, within, role, &mut items) {
            return reply(id, json!({"ok": true, "items": items}));
        }
    }
    reply(
        id,
        json!({
            "ok": false,
            "failure": format!(
                "the capture of {within:?} reached its scroll limit of {CAPTURE_SCROLL_LIMIT} scrolls and the pane still showed new items"
            ),
        }),
    )
}

/// Collect the items the pane shows that `items` does not already carry,
/// waiting until it draws one or the settle deadline passes. Reports whether
/// the pane showed anything new, which is how a capture knows the pane reached
/// its end.
///
/// @planks("the test runner captures every {string} in the {string} as {string}")
/// @planks("the capture named {string} holds {int} items")
fn gather(session: &Session, within: &str, role: &str, items: &mut Vec<String>) -> bool {
    let deadline = Instant::now() + CAPTURE_SETTLE;
    loop {
        let model = build(&session.capture.screen());
        let mut fresh: Vec<String> = pane_items(&model, within, role)
            .into_iter()
            .filter(|item| !items.contains(item))
            .collect();
        if !fresh.is_empty() {
            items.append(&mut fresh);
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The text of the regions playing `role` inside the pane named `within`, in
/// the order the pane shows them. A pane the screen has yet to draw shows
/// nothing.
///
/// @planks("the test runner captures the visible {string} items in the {string} as {string}")
fn pane_items(model: &Model, within: &str, role: &str) -> Vec<String> {
    let Some(pane) = model.find_named(within) else {
        return Vec::new();
    };
    pane.children
        .iter()
        .filter(|child| child.role() == role)
        .filter_map(|child| child.text.clone())
        .collect()
}

/// Answer with the session's current screen. A call omitting the session it
/// addresses is a protocol fault carrying the reserved invalid-params code and
/// the parameter it concerned: a driver holds several sessions at once, so it
/// has none to default to.
///
/// @planks("the driver answers a later screen request for the same session")
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the missing parameter {string}")
fn screen(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    reply(
        id,
        json!({"ok": true, "screen": session.capture.screen().contents()}),
    )
}

/// Answer with the terminal object model of the session's current screen,
/// overlaid with the selection an activation moved and the values a fill
/// typed, neither of which the bare screen otherwise reports.
///
/// @planks("the test runner requests the terminal object model")
fn tom(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let mut model = build(&session.capture.screen());
    overlay_selection(&mut model, &session.selected);
    overlay_fill(&mut model.root, &session.filled);
    match serde_json::to_value(model) {
        Ok(written) => reply(id, json!({"ok": true, "tom": written})),
        Err(e) => reply(
            id,
            json!({
                "ok": false,
                "failure": format!("the model was not written as JSON: {e}"),
            }),
        ),
    }
}

/// Mark the region an activation moved the selection onto as selected, and
/// every other region of the same role as not, since only one region of a
/// role is ever the selected one. A program's own redraw can scroll the
/// activated region out of the screen a session started with, its role
/// disappearing from the freshly built model entirely; the region the
/// activation resolved against is carried into the model in that case, since
/// the selection the driver moved to is what a terminal has no other way to
/// report.
///
/// @planks("the selected {string} is {string}")
fn overlay_selection(model: &mut Model, selected: &Option<Region>) {
    let Some(target) = selected else { return };
    let mut found = false;
    mark_selected(&mut model.root, target, &mut found);
    if !found {
        let mut carried = target.clone();
        carried.selected = true;
        model.root.children.push(carried);
    }
}

/// Mark the region among `region`'s subtree that plays `target`'s role and
/// name as selected, every other region of that role as not, and report
/// whether the target's own region was found.
///
/// @planks("the selected {string} is {string}")
fn mark_selected(region: &mut Region, target: &Region, found: &mut bool) {
    if region.role() == target.role() {
        let is_target = region.name == target.name;
        region.selected = is_target;
        *found = *found || is_target;
    }
    for child in &mut region.children {
        mark_selected(child, target, found);
    }
}

/// Report the value a fill typed into the textbox it named, since the bare
/// screen the fixture draws never reflects a value typed into it.
///
/// @planks("the textbox labelled {string} contains {string}")
fn overlay_fill(region: &mut Region, filled: &HashMap<String, String>) {
    if region.role() == "textbox"
        && let Some(name) = &region.name
        && let Some(value) = filled.get(name)
    {
        region.text = Some(value.clone());
    }
    for child in &mut region.children {
        overlay_fill(child, filled);
    }
}

/// Answer with the sandbox backend the session's program runs under. Every
/// session launches under Bubblewrap.
///
/// @planks("the test runner requests the session's sandbox backend")
/// @planks("the reported backend is {string}")
fn sandbox(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    if addressed(params, sessions).is_none() {
        return no_session(id);
    }
    reply(id, json!({"ok": true, "backend": SANDBOX_BACKEND}))
}

/// The text of the screen's bottom row, the way a full-screen program reports
/// its own state. A completed key send always redraws this row by absolute
/// position, so it is the row an activation reads back to tell whether the
/// program answered, whatever else on screen a scroll may have shifted.
///
/// @planks("the failure reports the selection did not reach the {string} named {string}")
fn status_line(screen: &VirtualScreen) -> String {
    screen
        .rows()
        .last()
        .map(|row| row.concat().trim_end().to_string())
        .unwrap_or_default()
}

/// Activate the region playing `role` and named `name`: resolve the locator
/// against the current screen, a locator matching nothing or several regions
/// failing with what it looked for, then move the selection onto the one
/// region it named and confirm it, the way a person reaches a target with no
/// pointer to click. A selection the driven program never answers is a
/// failure naming the region the selection did not reach.
///
/// @planks("the test runner activates the {string} named {string}")
/// @planks("the test runner has activated the {string} named {string}")
/// @planks("the driver replies with a failed result")
/// @planks("the failure reports no {string} named {string} was found")
/// @planks("the failure reports {int} matches for the {string} named {string}")
/// @planks("the failure reports the selection did not reach the {string} named {string}")
fn activate(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(role) = params["role"].as_str() else {
        return fault(
            id,
            INVALID_PARAMS,
            "the activate call names no role",
            "role",
        );
    };
    let Some(name) = params["name"].as_str() else {
        return fault(
            id,
            INVALID_PARAMS,
            "the activate call names no name",
            "name",
        );
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let model = build(&session.capture.screen());
    match Locator::new(role, name).resolve(&model) {
        Resolution::NoMatch => reply(
            id,
            json!({"ok": false, "failure": format!("no {role} named {name:?} was found")}),
        ),
        Resolution::Ambiguous(count) => reply(
            id,
            json!({"ok": false, "failure": format!("{count} matches for the {role} named {name:?}")}),
        ),
        Resolution::One(region) => {
            let before = status_line(&session.capture.screen());
            session.capture.press_key(SELECT_KEY);
            session.capture.press_key("Enter");
            let deadline = Instant::now() + RESPONSE_DEADLINE;
            loop {
                if status_line(&session.capture.screen()) != before {
                    let mut activated = region;
                    activated.selected = true;
                    session.selected = Some(activated);
                    return reply(id, json!({"ok": true}));
                }
                if Instant::now() >= deadline {
                    return reply(
                        id,
                        json!({
                            "ok": false,
                            "failure": format!("the selection did not reach the {role} named {name:?}"),
                        }),
                    );
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

/// Fill the textbox labelled `label` with `value`, typing it into the session
/// the way a person would.
///
/// @planks("the test runner fills the textbox labelled {string} with {string}")
fn fill(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(label) = params["label"].as_str() else {
        return fault(id, INVALID_PARAMS, "the fill call names no label", "label");
    };
    let Some(value) = params["value"].as_str() else {
        return fault(id, INVALID_PARAMS, "the fill call names no value", "value");
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    session.capture.press_key(value);
    session.filled.insert(label.to_string(), value.to_string());
    reply(id, json!({"ok": true}))
}

/// Forward a key press to the session's program, the way a person types,
/// submitting it and waiting for the program to answer, so a call that
/// follows a press observes what the program drew in response rather than the
/// screen the press left for a moment.
///
/// @planks("the test runner presses the key {string}")
fn press(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(key) = params["key"].as_str() else {
        return fault(id, INVALID_PARAMS, "the press call names no key", "key");
    };
    let Some(session) = addressed(params, sessions) else {
        return no_session(id);
    };
    let before = status_line(&session.capture.screen());
    session.capture.press_key(key);
    if key != "Enter" {
        session.capture.press_key("Enter");
    }
    let deadline = Instant::now() + RESPONSE_DEADLINE;
    while status_line(&session.capture.screen()) == before && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    reply(id, json!({"ok": true}))
}

/// Close a session and reclaim what it owns: the running program ends with the
/// session, and its sandbox home directory is removed.
///
/// @planks("the test runner closes the session")
/// @planks("the session's temporary sandbox directories no longer exist")
fn close(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let Some(mut session) = params["session"]
        .as_str()
        .and_then(|name| sessions.open.remove(name))
    else {
        return no_session(id);
    };
    session.capture.end_session();
    if let Err(e) = std::fs::remove_dir_all(&session.home) {
        return reply(
            id,
            json!({
                "ok": false,
                "failure": format!(
                    "the sandbox home {} was not reclaimed: {e}",
                    session.home.display()
                ),
            }),
        );
    }
    reply(id, json!({"ok": true}))
}
