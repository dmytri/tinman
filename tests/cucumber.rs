//! Cucumber-rs runner for Tinman's dev rigging.
//!
//! Shipshape dev-rigging note: our own verification is real-by-default. Steps
//! drive real Tinman seams (real command parse, real backend resolution, real
//! Bubblewrap argument generation, real source inspection). This is distinct
//! from Tinman's product mandate, which is itself to drive *real* TUIs (real
//! coding agents) with no mocks. Both layers exercise real behaviour.

use cucumber::{World, given, then, when};

#[path = "cucumber/support.rs"]
mod support;

use tinman::backend::{ResolveError, ResolvedBackend, resolve};
use tinman::bwrap::BubblewrapBackend;
use tinman::process::PreparedProcess;
use tinman::sandbox::{Backend, CommandSpec, Network, SandboxSpec};

/// Shared scenario state. Fields are populated by the step definitions that
/// need them. `Option` fields stay `None` until a `Given`/`When` sets them.
#[derive(Debug, Default, World)]
struct TinmanWorld {
    // command invocation
    command: Option<CommandSpec>,
    // backend selection
    requested_backend: Option<Backend>,
    resolution: Option<Result<ResolvedBackend, ResolveError>>,
    // bwrap availability + launch
    backend: Option<BubblewrapBackend>,
    spec: Option<SandboxSpec>,
    launch_error: Option<String>,
    sentinel: Option<std::path::PathBuf>,
    // bwrap argument generation
    generated_args: Option<Vec<String>>,
    network_denied: bool,
    // pty boundary
    boundary_counterexamples: Option<Vec<String>>,
    // schema-conformance: the serialized artifact under test, as JSON
    serialized: Option<serde_json::Value>,
    // capture pipeline
    prepared: Option<PreparedProcess>,
    screen: Option<tinman::screen::VirtualScreen>,
    rendered_frame: Option<String>,
    // sandboxed launch
    secret_name: Option<String>,
    secret_value: Option<String>,
    // key recording and interaction log
    session: Option<tinman::record::RecordingSession>,
    interactive: Option<tinman::pty::InteractiveCapture>,
    log_yaml: Option<String>,
}

/// A prepared process that runs a shell command line locally, with no sandbox
/// backend. Used by the default-tier capture scenarios, which exercise the real
/// PTY and virtual-screen path against a real local subprocess.
fn shell_process(command_line: &str) -> PreparedProcess {
    PreparedProcess {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), command_line.to_string()],
        env: Vec::new(),
        cleanup: Vec::new(),
    }
}

/// Serialize any `serde::Serialize` production artifact to a JSON value through
/// the real serialization path. Production owns the shape via its serde
/// derive; the suite reads that shape back for schema validation. YAML is the
/// wire form, and JSON is a subset of YAML, so the parse is loss-free.
fn to_json<T: serde::Serialize>(value: &T) -> serde_json::Value {
    let yaml = serde_yaml::to_string(value).expect("artifact serializes to YAML");
    serde_yaml::from_str(&yaml).expect("serialized artifact parses as a value")
}

// ---------------------------------------------------------------------------
// command invocation
// ---------------------------------------------------------------------------

#[when(expr = "the operator runs {string}")]
async fn operator_runs(world: &mut TinmanWorld, line: String) {
    let command = tinman::record::parse_command_line(&line).expect("record command line parses");
    world.command = Some(command);
}

#[then(expr = "the capture target program is {string}")]
async fn target_program_is(world: &mut TinmanWorld, program: String) {
    let command = world.command.as_ref().expect("a command was parsed");
    assert_eq!(command.program, program);
}

#[then(expr = "the capture target arguments are {string} and {string}")]
async fn target_arguments_are(world: &mut TinmanWorld, first: String, second: String) {
    let command = world.command.as_ref().expect("a command was parsed");
    assert_eq!(command.args, vec![first, second]);
}

// ---------------------------------------------------------------------------
// backend selection
// ---------------------------------------------------------------------------

#[given(expr = "the requested backend is {string}")]
async fn requested_backend(world: &mut TinmanWorld, name: String) {
    world.requested_backend = Some(Backend::from_name(&name).expect("known backend name"));
}

#[when("the backend is resolved on Linux")]
async fn resolved_on_linux(world: &mut TinmanWorld) {
    // The suite runs on Linux, so real resolution exercises the Linux path.
    let requested = world.requested_backend.expect("a backend was requested");
    world.resolution = Some(resolve(requested, false));
}

#[when("the backend is resolved")]
async fn resolved(world: &mut TinmanWorld) {
    let requested = world.requested_backend.expect("a backend was requested");
    world.resolution = Some(resolve(requested, false));
}

#[when("the backend is resolved without the unsafe option")]
async fn resolved_without_unsafe(world: &mut TinmanWorld) {
    let requested = world.requested_backend.expect("a backend was requested");
    world.resolution = Some(resolve(requested, false));
}

#[then(expr = "the resolved backend is {string}")]
async fn resolved_backend_is(world: &mut TinmanWorld, name: String) {
    let resolved = world
        .resolution
        .as_ref()
        .expect("resolution ran")
        .as_ref()
        .expect("resolution succeeded");
    assert_eq!(resolved.name(), name);
}

#[then("resolution fails with an unsupported-backend error")]
async fn fails_unsupported(world: &mut TinmanWorld) {
    let err = world
        .resolution
        .as_ref()
        .expect("resolution ran")
        .as_ref()
        .expect_err("resolution failed");
    assert!(
        matches!(err, ResolveError::UnsupportedBackend { .. }),
        "expected unsupported-backend error, got {err:?}"
    );
}

#[then("resolution fails and reports that the unsafe option is required")]
async fn fails_unsafe_required(world: &mut TinmanWorld) {
    let err = world
        .resolution
        .as_ref()
        .expect("resolution ran")
        .as_ref()
        .expect_err("resolution failed");
    assert!(
        matches!(err, ResolveError::UnsafeRequired),
        "expected unsafe-required error, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// unavailable Bubblewrap is a hard failure
// ---------------------------------------------------------------------------

#[given("the Bubblewrap executable is absent")]
async fn bwrap_absent(world: &mut TinmanWorld) {
    // Point the backend at an executable name guaranteed to be off PATH, so
    // the real availability check fails without touching the host's real bwrap.
    let missing = format!("bwrap-absent-{}", std::process::id());
    world.backend = Some(BubblewrapBackend::with_executable(missing));
}

#[when("a process is prepared and launched")]
async fn prepared_and_launched(world: &mut TinmanWorld) {
    // A command whose only effect would be to create a sentinel file. If any
    // unsandboxed process runs, the sentinel appears; the assertion below then
    // proves nothing ran.
    let sentinel = std::env::temp_dir().join(format!(
        "tinman-sentinel-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&sentinel);
    world.sentinel = Some(sentinel.clone());

    let command = CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), format!("touch {}", sentinel.display())],
    };
    let spec = SandboxSpec::default_for_record();
    let backend = world.backend.as_ref().expect("a backend was configured");

    match backend.prepare(&spec, &command) {
        Ok(prepared) => match tinman::pty::launch(&prepared) {
            Ok(()) => {}
            Err(e) => world.launch_error = Some(e.to_string()),
        },
        Err(e) => world.launch_error = Some(e.to_string()),
    }
}

#[then("launching fails and reports Bubblewrap is unavailable")]
async fn launching_fails_unavailable(world: &mut TinmanWorld) {
    let message = world.launch_error.as_ref().expect("launching failed");
    assert!(
        message.to_lowercase().contains("bubblewrap")
            && message.to_lowercase().contains("unavailable"),
        "expected a Bubblewrap-unavailable message, got {message:?}"
    );
}

#[then("no unsandboxed process is started")]
async fn no_unsandboxed_process(world: &mut TinmanWorld) {
    let sentinel = world.sentinel.as_ref().expect("a sentinel path was set");
    assert!(
        !sentinel.exists(),
        "sentinel {} exists: an unsandboxed process ran",
        sentinel.display()
    );
}

// ---------------------------------------------------------------------------
// bubblewrap argument generation
// ---------------------------------------------------------------------------

#[given(expr = "a command specification for {string}")]
async fn command_specification(world: &mut TinmanWorld, program: String) {
    world.command = Some(CommandSpec {
        program,
        args: Vec::new(),
    });
}

#[given("a sandbox specification that denies network access")]
async fn spec_denies_network(world: &mut TinmanWorld) {
    let mut spec = SandboxSpec::default_for_record();
    spec.network = Network::Deny;
    world.network_denied = true;
    world.spec = Some(spec);
}

#[when("the Bubblewrap backend generates its arguments")]
async fn backend_generates_arguments(world: &mut TinmanWorld) {
    let spec = world.spec.as_ref().expect("a sandbox specification");
    let command = world.command.as_ref().expect("a command specification");
    let backend = BubblewrapBackend::new();
    world.generated_args = Some(backend.generate_args(spec, command));
}

#[then(expr = "the arguments satisfy the isolation policy in {string}")]
async fn arguments_satisfy_policy(world: &mut TinmanWorld, policy_path: String) {
    let argv = world
        .generated_args
        .as_ref()
        .expect("arguments were generated");
    let operator_home = std::env::var("HOME").unwrap_or_default();
    let counterexamples =
        support::check_bwrap_policy(&policy_path, argv, world.network_denied, &operator_home);
    assert!(
        counterexamples.is_empty(),
        "isolation policy violated: {counterexamples:?}"
    );
}

// ---------------------------------------------------------------------------
// pty sandbox boundary
// ---------------------------------------------------------------------------

#[given("the PTY runner source")]
async fn pty_runner_source(_world: &mut TinmanWorld) {
    // The boundary policy names the module path; nothing to stage here.
}

#[when("the verifier checks the PTY sandbox boundary")]
async fn verifier_checks_boundary(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_pty_boundary(
        "scantlings/pty-sandbox-boundary.json",
    ));
}

#[then("no counterexample is found")]
async fn no_counterexample(world: &mut TinmanWorld) {
    let counterexamples = world
        .boundary_counterexamples
        .as_ref()
        .expect("the boundary verifier ran");
    assert!(
        counterexamples.is_empty(),
        "boundary counterexamples: {counterexamples:?}"
    );
}

// ---------------------------------------------------------------------------
// sandbox specification schema conformance
// ---------------------------------------------------------------------------

#[given(expr = "the default sandbox specification for {string}")]
async fn default_sandbox_spec(world: &mut TinmanWorld, _mode: String) {
    world.spec = Some(SandboxSpec::default_for_record());
}

#[when("the specification is serialized")]
async fn specification_is_serialized(world: &mut TinmanWorld) {
    let spec = world.spec.as_ref().expect("a sandbox specification");
    world.serialized = Some(to_json(spec));
}

#[then(expr = "it conforms to the {string} schema in {string}")]
async fn it_conforms_to_schema(world: &mut TinmanWorld, _schema_id: String, path: String) {
    let instance = world
        .serialized
        .as_ref()
        .expect("an artifact was serialized");
    let bad = support::schema_counterexamples(&path, instance);
    assert!(bad.is_empty(), "schema violations: {bad:?}");
}

// ---------------------------------------------------------------------------
// prepared process schema conformance
// ---------------------------------------------------------------------------

#[when("the Bubblewrap backend prepares the process")]
async fn bwrap_prepares_process(world: &mut TinmanWorld) {
    let command = world.command.as_ref().expect("a command specification");
    let spec = SandboxSpec::default_for_record();
    let backend = BubblewrapBackend::new();
    let prepared = backend
        .prepare(&spec, command)
        .expect("the Bubblewrap backend prepares the process");
    world.serialized = Some(to_json(&prepared));
}

#[then(expr = "the prepared process conforms to the {string} schema in {string}")]
async fn prepared_conforms_to_schema(world: &mut TinmanWorld, _schema_id: String, path: String) {
    let instance = world
        .serialized
        .as_ref()
        .expect("an artifact was serialized");
    let bad = support::schema_counterexamples(&path, instance);
    assert!(bad.is_empty(), "schema violations: {bad:?}");
}

// ---------------------------------------------------------------------------
// virtual screen: PTY capture into a parsed screen
// ---------------------------------------------------------------------------

#[given(expr = "a prepared process that runs {string}")]
async fn prepared_that_runs(world: &mut TinmanWorld, command_line: String) {
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} using ANSI positioning"
)]
async fn prepared_ansi_write(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    // ANSI cursor addressing is 1-based row;col. printf's octal escape emits ESC.
    let command_line = format!("printf '\\033[{row};{col}H{text}'");
    world.prepared = Some(shell_process(&command_line));
}

#[when("the process is captured through a PTY")]
async fn captured_through_pty(world: &mut TinmanWorld) {
    let prepared = world.prepared.as_ref().expect("a prepared process");
    world.screen = Some(tinman::pty::capture(prepared).expect("the process is captured"));
}

#[then(expr = "the virtual screen contains the text {string}")]
async fn virtual_screen_contains(world: &mut TinmanWorld, text: String) {
    // A live interactive capture echoes forwarded input asynchronously, so poll
    // its screen until the text appears, bounded by a deadline.
    if let Some(capture) = world.interactive.as_mut() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let screen = capture.screen();
            if screen.contains(&text) {
                return;
            }
            if std::time::Instant::now() >= deadline {
                panic!(
                    "virtual screen never showed {text:?}; contents:\n{}",
                    screen.contents()
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    assert!(
        screen.contains(&text),
        "virtual screen does not contain {text:?}; contents: {:?}",
        screen.contents()
    );
}

#[then(expr = "the virtual screen cell at row {int} column {int} shows {string}")]
async fn virtual_screen_cell_shows(world: &mut TinmanWorld, row: u16, col: u16, text: String) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    assert_eq!(
        screen.cell(row, col),
        text,
        "cell at row {row} column {col}"
    );
}

// ---------------------------------------------------------------------------
// terminal view: render the virtual screen to a test terminal
// ---------------------------------------------------------------------------

#[given(expr = "a virtual screen that shows {string}")]
async fn a_virtual_screen_showing(world: &mut TinmanWorld, text: String) {
    world.screen = Some(tinman::screen::VirtualScreen::from_text(&text));
}

#[when(expr = "the capture view is rendered to a {int} by {int} test terminal")]
async fn capture_view_rendered(world: &mut TinmanWorld, width: u16, height: u16) {
    let screen = world.screen.as_ref().expect("a virtual screen");
    world.rendered_frame = Some(support::render_capture_view(screen, width, height));
}

#[then(expr = "the rendered frame contains {string}")]
async fn rendered_frame_contains(world: &mut TinmanWorld, text: String) {
    let frame = world.rendered_frame.as_ref().expect("a rendered frame");
    assert!(
        frame.contains(&text),
        "rendered frame does not contain {text:?}; frame:\n{frame}"
    );
}

// ---------------------------------------------------------------------------
// sandboxed launch: a Bubblewrap-prepared process, captured for real
// ---------------------------------------------------------------------------

#[given(expr = "the operator's environment defines the secret {string} as {string}")]
async fn env_defines_secret(world: &mut TinmanWorld, name: String, value: String) {
    // Set the secret in the test process's own environment. Bubblewrap inherits
    // this environment and must clear it, so a leak into the sandbox is a real
    // leak of the operator's secret.
    unsafe {
        std::env::set_var(&name, &value);
    }
    world.secret_name = Some(name);
    world.secret_value = Some(value);
}

#[given(
    expr = "a Bubblewrap-prepared process that prints its home directory and the value of {string}"
)]
async fn bwrap_prints_home_and_secret(world: &mut TinmanWorld, secret: String) {
    let script = format!("printf 'HOME=%s\\n' \"$HOME\"; printf 'SECRET=[%s]\\n' \"${secret}\"");
    let command = CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), script],
    };
    let backend = BubblewrapBackend::new();
    let prepared = backend
        .prepare(&SandboxSpec::default_for_record(), &command)
        .expect("the Bubblewrap backend prepares the process");
    world.prepared = Some(prepared);
}

#[given("a Bubblewrap-prepared process that probes for a network route")]
async fn bwrap_network_probe(world: &mut TinmanWorld) {
    // Read the sandbox's own routing table. An unshared network namespace has
    // no route, so a denied-network sandbox reports none. The probe fails loudly
    // when procfs is absent, so it can never report "unreachable" vacuously: the
    // routing table must be genuinely observable and genuinely empty.
    let script = "if [ ! -e /proc/net/route ]; then echo NETWORK_PROBE_NO_PROCFS; exit 0; fi; \
        routes=$(tail -n +2 /proc/net/route | wc -l); \
        if [ \"$routes\" = \"0\" ]; then echo NETWORK_UNREACHABLE; else echo NETWORK_REACHABLE; fi";
    let command = CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
    };
    let backend = BubblewrapBackend::new();
    let prepared = backend
        .prepare(&SandboxSpec::default_for_record(), &command)
        .expect("the Bubblewrap backend prepares the process");
    world.prepared = Some(prepared);
}

#[then("the virtual screen shows the secret value is absent")]
async fn secret_value_absent(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let value = world.secret_value.as_ref().expect("a secret value was set");
    assert!(
        !screen.contains(value),
        "secret value {value:?} leaked into the sandbox; screen:\n{}",
        screen.contents()
    );
}

#[then("the virtual screen shows a home directory other than the operator's home")]
async fn home_other_than_operator(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let operator_home = std::env::var("HOME").expect("operator HOME is set");
    let contents = screen.contents();
    let home = contents
        .lines()
        .find_map(|line| line.strip_prefix("HOME="))
        .map(|home| home.trim_end())
        .unwrap_or_else(|| {
            panic!("the sandboxed program printed no home directory; screen:\n{contents}")
        });
    assert!(!home.is_empty(), "the sandbox home directory is empty");
    assert_ne!(
        home, operator_home,
        "the sandbox home equals the operator's home {operator_home}"
    );
}

#[then("the virtual screen shows the network probe found no route")]
async fn network_probe_no_route(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    assert!(
        screen.contains("NETWORK_UNREACHABLE"),
        "the network probe did not report an unreachable network; screen:\n{}",
        screen.contents()
    );
}

// ---------------------------------------------------------------------------
// key recording: a recording session records key events in order
// ---------------------------------------------------------------------------

#[given("a recording session")]
async fn a_recording_session(world: &mut TinmanWorld) {
    world.session = Some(tinman::record::RecordingSession::new());
}

#[given(expr = "a recording session for {string}")]
async fn a_recording_session_for(world: &mut TinmanWorld, program: String) {
    let command = CommandSpec {
        program,
        args: Vec::new(),
    };
    world.session = Some(tinman::record::RecordingSession::for_command(command));
}

#[given(expr = "a recording session for {string} with one key press and one snapshot")]
async fn a_recording_session_with_one_each(world: &mut TinmanWorld, program: String) {
    let command = CommandSpec {
        program,
        args: Vec::new(),
    };
    let mut session = tinman::record::RecordingSession::for_command(command);
    session.press_key("h");
    session.snapshot(&tinman::screen::VirtualScreen::from_text("PROMPT>"));
    world.session = Some(session);
}

// A live interactive capture of a running program, so forwarded keys reach it.
#[given("the process is captured through a PTY")]
async fn given_captured_interactive(world: &mut TinmanWorld) {
    let prepared = world.prepared.as_ref().expect("a prepared process");
    world.interactive = Some(
        tinman::pty::capture_interactive(prepared).expect("the process is captured interactively"),
    );
}

#[when(expr = "the operator presses the keys {string}, {string}, and Enter")]
async fn presses_keys(world: &mut TinmanWorld, first: String, second: String) {
    let keys = [first.as_str(), second.as_str(), "Enter"];
    if let Some(capture) = world.interactive.as_mut() {
        for key in keys {
            capture.press_key(key);
        }
    } else if let Some(session) = world.session.as_mut() {
        for key in keys {
            session.press_key(key);
        }
    } else {
        panic!("no recording session or interactive capture in scope");
    }
}

#[given(expr = "the operator presses the key {string}")]
async fn presses_single_key(world: &mut TinmanWorld, key: String) {
    let session = world.session.as_mut().expect("a recording session");
    session.press_key(&key);
}

#[given("the operator takes a screen snapshot")]
async fn takes_snapshot(world: &mut TinmanWorld) {
    let session = world.session.as_mut().expect("a recording session");
    session.snapshot(&tinman::screen::VirtualScreen::from_text("PROMPT>"));
}

#[then(expr = "the session's recorded key events are {string}, {string}, {string} in that order")]
async fn recorded_key_events(
    world: &mut TinmanWorld,
    first: String,
    second: String,
    third: String,
) {
    let session = world.session.as_ref().expect("a recording session");
    assert_eq!(session.recorded_keys(), vec![first, second, third]);
}

// ---------------------------------------------------------------------------
// interaction log: a recording session written as constrained YAML
// ---------------------------------------------------------------------------

#[when("the session is written as a YAML interaction log")]
async fn session_written_as_log(world: &mut TinmanWorld) {
    let session = world.session.as_ref().expect("a recording session");
    world.log_yaml = Some(session.to_interaction_log());
}

#[then(expr = "the log conforms to the {string} schema in {string}")]
async fn log_conforms_to_schema(world: &mut TinmanWorld, _schema_id: String, path: String) {
    let yaml = world.log_yaml.as_ref().expect("a written interaction log");
    let instance: serde_json::Value =
        serde_yaml::from_str(yaml).expect("the interaction log parses as a value");
    let bad = support::schema_counterexamples(&path, &instance);
    assert!(bad.is_empty(), "schema violations: {bad:?}");
}

#[then(expr = "the log lists a key press {string}")]
async fn log_lists_key_press(world: &mut TinmanWorld, key: String) {
    let yaml = world.log_yaml.as_ref().expect("a written interaction log");
    let value: serde_json::Value =
        serde_yaml::from_str(yaml).expect("the interaction log parses as a value");
    let events = value
        .get("events")
        .and_then(|events| events.as_array())
        .expect("the log has an events array");
    let found = events
        .iter()
        .any(|event| event.get("key").and_then(|k| k.as_str()) == Some(key.as_str()));
    assert!(found, "log does not list a key press {key:?}; log:\n{yaml}");
}

#[then("the log lists one snapshot")]
async fn log_lists_one_snapshot(world: &mut TinmanWorld) {
    let yaml = world.log_yaml.as_ref().expect("a written interaction log");
    let value: serde_json::Value =
        serde_yaml::from_str(yaml).expect("the interaction log parses as a value");
    let events = value
        .get("events")
        .and_then(|events| events.as_array())
        .expect("the log has an events array");
    let snapshots = events
        .iter()
        .filter(|event| event.get("snapshot").is_some())
        .count();
    assert_eq!(snapshots, 1, "expected exactly one snapshot; log:\n{yaml}");
}

// Keep the prepared-process type referenced so the launch seam signature stays
// pinned by the suite even before watch2 exercises capture.
#[allow(dead_code)]
fn _pin_prepared_process(p: &PreparedProcess) -> &str {
    &p.program
}

#[tokio::main]
async fn main() {
    // `fail_on_skipped` makes undefined or unimplemented steps redden, so a
    // missing step definition is a failing verification target the QM can see.
    TinmanWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("features")
        .await;
}
