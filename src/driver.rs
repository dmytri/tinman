//! The driver: the JSON-RPC 2.0 protocol a test runner in any language speaks
//! to Tinman, one message per line on stdin and stdout. A protocol fault is an
//! error object carrying a reserved code. A failed expectation is a result
//! whose `ok` is false, because the call succeeded and the product disagreed.

use crate::bwrap::BubblewrapBackend;
use crate::pty::{InteractiveCapture, capture_interactive};
use crate::sandbox::{CommandSpec, SandboxSpec};
use crate::tom::build;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The reserved JSON-RPC code for a method the driver does not answer.
const METHOD_NOT_FOUND: i64 = -32601;

/// How long a driven program is given to draw the text an expectation names, so
/// an expectation resolves the moment the text appears.
const EXPECT_DEADLINE: Duration = Duration::from_secs(5);

/// One live session: the program running on its PTY, and the temporary
/// directory the sandbox uses as its home, which closing the session reclaims.
struct Session {
    capture: InteractiveCapture,
    home: PathBuf,
}

/// The sessions a running driver holds, addressed by the identifiers it issued.
#[derive(Default)]
struct Sessions {
    open: HashMap<String, Session>,
    launched: u64,
}

/// Speak the driver protocol on stdin and stdout until the client closes its
/// end: read one request per line, answer each with one reply line, flushed so
/// the client reads it without waiting for the next.
///
/// @planks("the Tinman driver is running")
/// @planks("the test runner sends the request:")
/// @planks("every exchanged message conforms to the {string} schema in {string}")
pub fn serve() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut sessions = Sessions::default();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("the request line is not JSON: {e}\n{line}"));
        let reply = answer(&request, &mut sessions);
        writeln!(stdout, "{reply}").expect("the reply reaches the client");
        stdout.flush().expect("the reply is flushed");
    }
}

/// Answer one request. A method outside the protocol is a fault, so it is
/// answered with the reserved code and the name it called.
///
/// @planks("the test runner sends the request:")
/// @planks("the driver replies to request {int} with the error code {int}")
/// @planks("the error data names the method {string}")
fn answer(request: &Value, sessions: &mut Sessions) -> Value {
    let id = request["id"].clone();
    let method = request["method"]
        .as_str()
        .unwrap_or_else(|| panic!("the request names no method: {request}"))
        .to_string();
    let params = &request["params"];
    match method.as_str() {
        "launch" => launch(id, params, sessions),
        "expect" => expect(id, params, sessions),
        "screen" => screen(id, params, sessions),
        "tom" => tom(id, params, sessions),
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

/// The session a call addresses.
fn addressed<'a>(params: &Value, sessions: &'a mut Sessions) -> &'a mut Session {
    let name = params["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the call addresses no session: {params}"));
    sessions
        .open
        .get_mut(name)
        .unwrap_or_else(|| panic!("the driver holds no session {name:?}"))
}

/// Launch the named command in a sandbox of its own and keep it running. The
/// session gets a temporary directory as its sandbox home, named after the
/// identifier the reply carries, so the client can see what the session owns.
///
/// @planks("the Tinman driver has a session running {string}")
/// @planks("the Tinman driver has a session running the fixture terminal program")
/// @planks("the driver replies to request {int} with a session identifier")
fn launch(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    sessions.launched += 1;
    let name = format!("sess-{}-{}", std::process::id(), sessions.launched);
    let home = std::env::temp_dir().join(format!("tinman-{name}"));
    std::fs::create_dir_all(&home)
        .unwrap_or_else(|e| panic!("the sandbox home {} was not created: {e}", home.display()));
    let command = params["command"]
        .as_str()
        .unwrap_or_else(|| panic!("the launch call names no command: {params}"));
    let prepared = BubblewrapBackend::new()
        .prepare_with_home(
            &SandboxSpec::default_for_record(),
            &CommandSpec {
                program: "/bin/sh".to_string(),
                args: vec!["-c".to_string(), command.to_string()],
            },
            Some(&home),
        )
        .unwrap_or_else(|e| panic!("the session's process was not prepared: {e}"));
    let capture = capture_interactive(&prepared)
        .unwrap_or_else(|e| panic!("the session's process was not launched: {e}"));
    sessions
        .open
        .insert(name.clone(), Session { capture, home });
    reply(id, json!({"ok": true, "session": name}))
}

/// Answer whether the session's program has drawn the named text, waiting for
/// it until the deadline. Absent text is a result rather than a fault: the call
/// succeeded, so the reply reports what the screen held instead.
///
/// @planks("the test runner requests the text {string} is present")
/// @planks("the driver replies with a result whose {string} is false")
/// @planks("the reply carries no error object")
/// @planks("the driver replies with a failed result")
fn expect(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let text = params["text"]
        .as_str()
        .unwrap_or_else(|| panic!("the expectation names no text: {params}"))
        .to_string();
    let session = addressed(params, sessions);
    let deadline = Instant::now() + EXPECT_DEADLINE;
    loop {
        let screen = session.capture.screen();
        if screen.contains(&text) {
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

/// Answer with the session's current screen.
///
/// @planks("the driver answers a later screen request for the same session")
fn screen(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let session = addressed(params, sessions);
    reply(
        id,
        json!({"ok": true, "screen": session.capture.screen().contents()}),
    )
}

/// Answer with the terminal object model of the session's current screen.
///
/// @planks("the test runner requests the terminal object model")
fn tom(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let session = addressed(params, sessions);
    let model = build(&session.capture.screen());
    reply(
        id,
        json!({
            "ok": true,
            "tom": serde_json::to_value(model).expect("the model is written as JSON"),
        }),
    )
}

/// Close a session and reclaim what it owns: the running program ends with the
/// session, and its sandbox home directory is removed.
///
/// @planks("the test runner closes the session")
/// @planks("the session's temporary sandbox directories no longer exist")
fn close(id: Value, params: &Value, sessions: &mut Sessions) -> Value {
    let name = params["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the close call addresses no session: {params}"));
    let mut session = sessions
        .open
        .remove(name)
        .unwrap_or_else(|| panic!("the driver holds no session {name:?}"));
    session.capture.end_session();
    std::fs::remove_dir_all(&session.home).unwrap_or_else(|e| {
        panic!(
            "the sandbox home {} was not reclaimed: {e}",
            session.home.display()
        )
    });
    reply(id, json!({"ok": true}))
}
