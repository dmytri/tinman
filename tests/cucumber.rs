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
    // methodology conformance
    conformance_scope: Option<String>,
    conformance_matches: Option<Vec<support::ConformanceMatch>>,
    // bundled skill
    skill_path: Option<String>,
    loaded_skill: Option<tinman::skill::Skill>,
    skill_context: Option<String>,
    // help and command line
    asset_text: Option<String>,
    accepted_commands: Option<Vec<String>>,
    placeholder_count: Option<usize>,
    // running the real binary
    scratch: Option<support::ScratchDir>,
    provider: Option<support::LocalProvider>,
    run_stdout: Option<String>,
    run_status: Option<i32>,
    // flow orchestration
    flow_outcome: Option<tinman::flow::FlowOutcome>,
    flow_error: Option<String>,
    // driver protocol
    driver: Option<support::DriverProcess>,
    reply: Option<serde_json::Value>,
    session_id: Option<String>,
    session_dirs: Vec<std::path::PathBuf>,
    next_request_id: u64,
    // harness plans
    plan_sources: Vec<String>,
    parsed_plans: Vec<tinman::plan::Plan>,
    parsed_sandbox: Option<tinman::sandbox::SandboxSpec>,
    parse_error: Option<String>,
    // terminal object model
    pane: Option<(String, Vec<String>)>,
    tom: Option<tinman::tom::Model>,
    found_region: Option<tinman::tom::Region>,
    tom_resolution: Option<tinman::tom::Resolution>,
    // interactive assistant
    response: Option<tinman::assistant::Response>,
    parser_arguments: Option<Vec<String>>,
    // inference configuration and requests
    env_vars: std::collections::BTreeMap<String, String>,
    settings: Option<tinman::inference::Settings>,
    built_requests: Vec<tinman::inference::Request>,
    inference_available: Option<bool>,
}

/// The scenario's working directory: the one a dotenv file is staged in and the
/// one a launched Tinman runs in. Created on first use, so a scenario that never
/// names a working directory still runs in a directory holding no `.env`.
fn working_dir(world: &mut TinmanWorld) -> std::path::PathBuf {
    if world.scratch.is_none() {
        world.scratch = Some(support::ScratchDir::new("workdir"));
    }
    world
        .scratch
        .as_ref()
        .expect("a working directory")
        .path()
        .to_path_buf()
}

/// The environment the scenario configured, as the pairs a launched process is
/// given.
fn configured_env(world: &TinmanWorld) -> Vec<(String, String)> {
    world
        .env_vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Resolve Tinman's inference settings from the environment and working
/// directory the scenario configured.
fn resolved_settings(world: &mut TinmanWorld) -> tinman::inference::Settings {
    let dir = working_dir(world);
    tinman::inference::Settings::resolve(&world.env_vars, &dir)
}

/// Point the scenario's configuration at a local provider.
fn use_provider(world: &mut TinmanWorld, provider: support::LocalProvider) {
    world.env_vars.insert(
        "TINMAN_API_KEY".to_string(),
        "sk-local-provider".to_string(),
    );
    world.env_vars.insert(
        "TINMAN_BASE_URL".to_string(),
        provider.base_url().to_string(),
    );
    world.provider = Some(provider);
}

/// The body of an asset: its content with surrounding blank space removed.
fn asset_body(path: &str) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("asset {path} unreadable: {e}"))
        .trim()
        .to_string()
}

/// Split an operator command line such as `tinman --help` into the arguments
/// the binary receives, dropping the program name itself.
fn tinman_args(line: &str) -> Vec<String> {
    line.split_whitespace()
        .skip(1)
        .map(str::to_string)
        .collect()
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
    world.boundary_counterexamples = Some(support::check_boundary(
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

// ---------------------------------------------------------------------------
// bundled skill: the one skill Tinman ships, read by every consumer
// ---------------------------------------------------------------------------

#[given(expr = "the bundled skill at {string}")]
async fn the_bundled_skill_at(world: &mut TinmanWorld, path: String) {
    world.skill_path = Some(path);
}

fn skill_from_file(world: &TinmanWorld) -> tinman::skill::Skill {
    let path = world.skill_path.as_ref().expect("a bundled skill path");
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("bundled skill {path} unreadable: {e}"));
    tinman::skill::parse(&text)
        .unwrap_or_else(|e| panic!("bundled skill {path} did not parse: {e}"))
}

#[when("the skill front matter is parsed")]
async fn skill_front_matter_is_parsed(world: &mut TinmanWorld) {
    let skill = skill_from_file(world);
    world.serialized = Some(to_json(&skill.front_matter));
}

#[when("Tinman loads its bundled skill")]
async fn tinman_loads_its_bundled_skill(world: &mut TinmanWorld) {
    world.loaded_skill = Some(tinman::skill::bundled());
}

#[then("the loaded skill body is identical to the file's body")]
async fn loaded_skill_body_identical(world: &mut TinmanWorld) {
    let loaded = world.loaded_skill.as_ref().expect("Tinman loaded a skill");
    let on_disk = skill_from_file(world);
    assert_eq!(
        loaded.body, on_disk.body,
        "the loaded skill body differs from the shipped file's body"
    );
}

#[when("the acronym context is built")]
async fn acronym_context_is_built(world: &mut TinmanWorld) {
    world.skill_context = Some(tinman::skill::acronym_context());
}

#[then(expr = "the context is the skill's {string} and {string} fields")]
async fn context_is_the_skill_fields(world: &mut TinmanWorld, first: String, second: String) {
    assert_eq!(
        (first.as_str(), second.as_str()),
        ("name", "description"),
        "the scenario names fields this step does not read"
    );
    let context = world.skill_context.as_ref().expect("a context was built");
    let skill = skill_from_file(world);
    assert!(
        context.contains(&skill.front_matter.name),
        "the context omits the skill name {:?}; context: {context:?}",
        skill.front_matter.name
    );
    assert!(
        context.contains(&skill.front_matter.description),
        "the context omits the skill description; context: {context:?}"
    );
    assert!(
        !context.contains(skill.body.trim()),
        "the context carries the skill body beyond its name and description"
    );
}

#[when("the assistant context is built")]
async fn assistant_context_is_built(world: &mut TinmanWorld) {
    world.skill_context = Some(tinman::skill::assistant_context());
}

#[then("the context contains the skill body")]
async fn context_contains_the_skill_body(world: &mut TinmanWorld) {
    let context = world.skill_context.as_ref().expect("a context was built");
    let skill = skill_from_file(world);
    assert!(
        context.contains(skill.body.trim()),
        "the assistant context omits the skill body"
    );
}

// ---------------------------------------------------------------------------
// conventional help: the bundled help asset, rendered with no inference
// ---------------------------------------------------------------------------

#[given("inference is available")]
async fn inference_is_available(world: &mut TinmanWorld) {
    let provider =
        support::LocalProvider::returning("Tinman Inspects Numerous Machine Agent Nodes");
    use_provider(world, provider);
}

#[when(expr = "the operator runs {string} with stdout redirected to a file")]
async fn operator_runs_redirected(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let out = dir.join("stdout.txt");
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let outcome = support::run_tinman(&dir, &argv, &env, Some(&out))
        .unwrap_or_else(|e| panic!("running {line:?} failed: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
}

#[then(expr = "the help output is the asset at {string} with the tagline line removed")]
async fn help_output_is_asset_without_tagline(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    let expected: Vec<&str> = asset
        .lines()
        .filter(|line| !line.contains(tinman::help::TAGLINE_PLACEHOLDER))
        .collect();
    let actual = world.run_stdout.as_ref().expect("the help output was read");
    assert_eq!(
        actual.trim_end(),
        expected.join("\n").trim_end(),
        "the help output is not the asset with its tagline line removed"
    );
}

#[given("the commands the parser accepts")]
async fn the_commands_the_parser_accepts(world: &mut TinmanWorld) {
    world.accepted_commands = Some(tinman::cli::accepted_commands());
}

#[when(expr = "each is looked for in the asset at {string}")]
async fn each_is_looked_for_in_the_asset(world: &mut TinmanWorld, path: String) {
    world.asset_text = Some(
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}")),
    );
}

#[then("every accepted command appears in the help text")]
async fn every_accepted_command_appears(world: &mut TinmanWorld) {
    let commands = world
        .accepted_commands
        .as_ref()
        .expect("the parser reported its commands");
    let text = world.asset_text.as_ref().expect("the help asset was read");
    assert!(
        !commands.is_empty(),
        "the parser reported no commands, so this scenario would assert nothing"
    );
    let missing: Vec<&String> = commands
        .iter()
        .filter(|command| !text.contains(command.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "commands the parser accepts but the help text omits: {missing:?}"
    );
}

#[given(expr = "the asset at {string}")]
async fn the_asset_at(world: &mut TinmanWorld, path: String) {
    world.asset_text = Some(
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}")),
    );
}

#[when("its tagline placeholders are counted")]
async fn its_tagline_placeholders_are_counted(world: &mut TinmanWorld) {
    let text = world.asset_text.as_ref().expect("the asset was read");
    world.placeholder_count = Some(text.matches(tinman::help::TAGLINE_PLACEHOLDER).count());
}

#[then(expr = "the count is {int}")]
async fn the_count_is(world: &mut TinmanWorld, expected: usize) {
    let count = world
        .placeholder_count
        .as_ref()
        .expect("placeholders were counted");
    assert_eq!(*count, expected, "tagline placeholder count");
}

#[then(expr = "the command exits with status {int}")]
async fn the_command_exits_with_status(world: &mut TinmanWorld, expected: i32) {
    let status = world.run_status.expect("a command was run");
    assert_eq!(status, expected, "exit status");
}

// ---------------------------------------------------------------------------
// inference configuration: credential, endpoint and model
// ---------------------------------------------------------------------------

#[given(expr = "a working directory holding a {string} file that sets {string} to {string}")]
async fn a_working_directory_holding_dotenv(
    world: &mut TinmanWorld,
    file: String,
    key: String,
    value: String,
) {
    let dir = working_dir(world);
    std::fs::write(dir.join(&file), format!("{key}={value}\n"))
        .unwrap_or_else(|e| panic!("{file} not written in {}: {e}", dir.display()));
}

#[given(expr = "the environment does not set {string}")]
async fn the_environment_does_not_set(world: &mut TinmanWorld, key: String) {
    world.env_vars.remove(&key);
}

#[given(expr = "the environment sets {string} to {string}")]
async fn the_environment_sets(world: &mut TinmanWorld, key: String, value: String) {
    world.env_vars.insert(key, value);
}

#[given(expr = "neither the environment nor a dotenv file sets {string}")]
async fn neither_environment_nor_dotenv_sets(world: &mut TinmanWorld, key: String) {
    world.env_vars.remove(&key);
    let dir = working_dir(world);
    let dotenv = dir.join(".env");
    assert!(
        !dotenv.exists(),
        "a dotenv file is staged at {}, so this scenario's precondition does not hold",
        dotenv.display()
    );
}

#[given(expr = "the configured model is {string}")]
async fn the_configured_model_is(world: &mut TinmanWorld, model: String) {
    world.env_vars.insert("TINMAN_MODEL".to_string(), model);
}

#[given("the inference credential is configured")]
async fn the_inference_credential_is_configured(world: &mut TinmanWorld) {
    world
        .env_vars
        .insert("TINMAN_API_KEY".to_string(), "sk-configured".to_string());
}

#[given("the inference provider endpoint is unreachable")]
async fn the_provider_endpoint_is_unreachable(world: &mut TinmanWorld) {
    world.env_vars.insert(
        "TINMAN_BASE_URL".to_string(),
        support::unreachable_base_url(),
    );
}

#[given("the inference provider rejects the configured credential")]
async fn the_provider_rejects_the_credential(world: &mut TinmanWorld) {
    let provider = support::LocalProvider::rejecting();
    use_provider(world, provider);
}

#[given("inference is unavailable")]
async fn inference_is_unavailable(world: &mut TinmanWorld) {
    world.env_vars.remove("TINMAN_API_KEY");
    world.env_vars.remove("TINMAN_BASE_URL");
    let dir = working_dir(world);
    assert!(
        !dir.join(".env").exists(),
        "a dotenv file is staged, so inference would not be unavailable"
    );
}

#[given(expr = "a provider that returns {string}")]
async fn a_provider_that_returns(world: &mut TinmanWorld, content: String) {
    let provider = support::LocalProvider::returning(&content);
    use_provider(world, provider);
}

#[given("a provider that returns an empty response")]
async fn a_provider_that_returns_empty(world: &mut TinmanWorld) {
    let provider = support::LocalProvider::returning("");
    use_provider(world, provider);
}

#[when("Tinman resolves its inference credential")]
async fn tinman_resolves_its_credential(world: &mut TinmanWorld) {
    world.settings = Some(resolved_settings(world));
}

#[then(expr = "the resolved credential is {string}")]
async fn the_resolved_credential_is(world: &mut TinmanWorld, expected: String) {
    let settings = world.settings.as_ref().expect("settings were resolved");
    assert_eq!(
        settings.api_key.as_deref(),
        Some(expected.as_str()),
        "resolved credential"
    );
}

#[when("Tinman checks whether inference is available")]
async fn tinman_checks_availability(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.inference_available = Some(tinman::inference::is_available(&settings));
}

#[then("inference is reported unavailable")]
async fn inference_is_reported_unavailable(world: &mut TinmanWorld) {
    let available = world.inference_available.expect("availability was checked");
    assert!(!available, "inference was reported available");
}

// ---------------------------------------------------------------------------
// inference requests: what Tinman sends, and with which sampling profile
// ---------------------------------------------------------------------------

#[when("an inference request is built")]
async fn an_inference_request_is_built(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.built_requests = vec![
        tinman::inference::acronym_request(&settings),
        tinman::inference::assistant_request(&settings, "what does replay do"),
    ];
}

#[when("the acronym request is built")]
async fn the_acronym_request_is_built(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.built_requests = vec![tinman::inference::acronym_request(&settings)];
}

#[when("the assistant request is built")]
async fn the_assistant_request_is_built(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.built_requests = vec![tinman::inference::assistant_request(
        &settings,
        "what does replay do",
    )];
}

#[when("the acronym request and the assistant request are built")]
async fn both_requests_are_built(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.built_requests = vec![
        tinman::inference::acronym_request(&settings),
        tinman::inference::assistant_request(&settings, "what does replay do"),
    ];
}

fn built_requests(world: &TinmanWorld) -> &[tinman::inference::Request] {
    assert!(
        !world.built_requests.is_empty(),
        "no inference request was built, so this scenario would assert nothing"
    );
    &world.built_requests
}

#[then(expr = "the request addresses {string}")]
async fn the_request_addresses(world: &mut TinmanWorld, expected: String) {
    for request in built_requests(world) {
        assert_eq!(request.address(), expected, "request endpoint");
    }
}

#[then(expr = "the request names the model {string}")]
async fn the_request_names_the_model(world: &mut TinmanWorld, expected: String) {
    for request in built_requests(world) {
        assert_eq!(request.model(), expected, "request model");
    }
}

#[then(expr = "both requests name the model {string}")]
async fn both_requests_name_the_model(world: &mut TinmanWorld, expected: String) {
    let requests = built_requests(world);
    assert_eq!(requests.len(), 2, "two requests were built");
    for request in requests {
        assert_eq!(request.model(), expected, "request model");
    }
}

#[then(expr = "the request temperature is {float}")]
async fn the_request_temperature_is(world: &mut TinmanWorld, expected: f64) {
    for request in built_requests(world) {
        assert!(
            (request.temperature() - expected).abs() < 1e-9,
            "request temperature is {}, expected {expected}",
            request.temperature()
        );
    }
}

#[then(expr = "the request carries the authorization header {string}")]
async fn the_request_carries_authorization(world: &mut TinmanWorld, expected: String) {
    for request in built_requests(world) {
        assert_eq!(
            request.authorization(),
            Some(expected.as_str()),
            "authorization header"
        );
    }
}

// ---------------------------------------------------------------------------
// help on a terminal: the tagline and the assistant prompt
// ---------------------------------------------------------------------------

#[when(expr = "the operator runs {string} in an interactive terminal")]
async fn operator_runs_interactive(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let outcome = support::run_tinman_on_a_terminal(&dir, &argv, &env)
        .unwrap_or_else(|e| panic!("running {line:?} on a terminal failed: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
}

/// The line of the run's output that the help asset reserves for the tagline.
fn tagline_line(world: &TinmanWorld) -> String {
    let asset = std::fs::read_to_string("assets/help/tinman.txt")
        .expect("the help asset assets/help/tinman.txt is readable");
    let index = support::tagline_line_index(&asset, tinman::help::TAGLINE_PLACEHOLDER);
    let output = world.run_stdout.as_ref().expect("the help output was read");
    output
        .lines()
        .nth(index)
        .unwrap_or_else(|| panic!("the help output has no line {index}; output:\n{output}"))
        .trim()
        .to_string()
}

#[then(expr = "the tagline line is {string}")]
async fn the_tagline_line_is(world: &mut TinmanWorld, expected: String) {
    assert_eq!(tagline_line(world), expected, "tagline line");
}

#[then(expr = "the tagline line is the body of the asset at {string}")]
async fn the_tagline_line_is_the_asset_body(world: &mut TinmanWorld, path: String) {
    assert_eq!(tagline_line(world), asset_body(&path), "tagline line");
}

#[then(expr = "the help output omits the body of the asset at {string}")]
async fn the_help_output_omits_asset_body(world: &mut TinmanWorld, path: String) {
    let body = asset_body(&path);
    let output = world.run_stdout.as_ref().expect("the help output was read");
    assert!(
        !output.contains(&body),
        "the help output carries {body:?}; output:\n{output}"
    );
}

#[then(expr = "the help output ends with the body of the asset at {string}")]
async fn the_help_output_ends_with_asset_body(world: &mut TinmanWorld, path: String) {
    let body = asset_body(&path);
    let output = world.run_stdout.as_ref().expect("the help output was read");
    assert!(
        output.trim_end().ends_with(&body),
        "the help output does not end with {body:?}; output:\n{output}"
    );
}

// ---------------------------------------------------------------------------
// flow orchestration: one flow driving several processes in order
// ---------------------------------------------------------------------------

#[given(expr = "a flow that runs {string}, then runs {string}")]
async fn a_flow_that_runs_then_runs(world: &mut TinmanWorld, first: String, second: String) {
    world
        .plan_sources
        .push(format!("flow:\n  - run: {first}\n  - run: {second}\n"));
}

#[when("the flow is executed")]
async fn the_flow_is_executed(world: &mut TinmanWorld) {
    let source = world
        .plan_sources
        .first()
        .expect("a flow was given")
        .clone();
    let plan =
        tinman::plan::parse(&source).unwrap_or_else(|e| panic!("the flow did not parse: {e}"));
    let workspace = working_dir(world);
    match tinman::flow::execute(&plan, &workspace) {
        Ok(outcome) => world.flow_outcome = Some(outcome),
        Err(e) => world.flow_error = Some(e.to_string()),
    }
}

#[then(expr = "the file {string} contains {string}")]
async fn the_file_contains(world: &mut TinmanWorld, name: String, expected: String) {
    let path = working_dir(world).join(&name);
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} unreadable: {e}", path.display()));
    assert_eq!(contents.trim_end(), expected, "contents of {name}");
}

#[then(expr = "the file {string} does not exist")]
async fn the_file_does_not_exist(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    assert!(
        !path.exists(),
        "{} exists, so the flow did not stop",
        path.display()
    );
}

#[then("execution fails and reports the step that failed")]
async fn execution_reports_the_failed_step(world: &mut TinmanWorld) {
    let message = world
        .flow_error
        .as_deref()
        .expect("the flow failed and reported why");
    let source = world.plan_sources.first().expect("a flow was given");
    let failing = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("- run: "))
        .expect("the flow names a run step");
    assert!(
        message.contains(failing),
        "the failure does not name the step that failed {failing:?}: {message}"
    );
}

#[then(expr = "the second step's output is {string}")]
async fn the_second_steps_output_is(world: &mut TinmanWorld, expected: String) {
    let outcome = world
        .flow_outcome
        .as_ref()
        .expect("the flow ran to completion");
    let step = outcome
        .steps
        .get(1)
        .unwrap_or_else(|| panic!("the flow ran {} steps", outcome.steps.len()));
    assert_eq!(step.output.trim_end(), expected, "second step output");
}

// ---------------------------------------------------------------------------
// driver protocol: one JSON message per line on stdin and stdout
// ---------------------------------------------------------------------------

/// The next client-assigned request identifier for this scenario.
fn next_id(world: &mut TinmanWorld) -> u64 {
    world.next_request_id += 1;
    world.next_request_id
}

fn driver(world: &mut TinmanWorld) -> &mut support::DriverProcess {
    world.driver.as_mut().expect("the Tinman driver is running")
}

fn session(world: &TinmanWorld) -> String {
    world
        .session_id
        .clone()
        .expect("the driver opened a session")
}

#[given("the Tinman driver is running")]
async fn the_tinman_driver_is_running(world: &mut TinmanWorld) {
    world.driver = Some(support::DriverProcess::start());
}

#[given(expr = "the Tinman driver has a session running {string}")]
async fn the_driver_has_a_session_running(world: &mut TinmanWorld, command: String) {
    world.driver = Some(support::DriverProcess::start());
    let id = next_id(world);
    let reply = driver(world).request(serde_json::json!({
        "id": id,
        "op": "launch",
        "command": command,
    }));
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(true),
        "the launch request failed: {reply}"
    );
    let identifier = reply["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the launch reply carries no session identifier: {reply}"))
        .to_string();
    world.session_id = Some(identifier);
    world.reply = Some(reply);
}

#[when("the test runner sends the request:")]
async fn the_test_runner_sends_the_request(
    world: &mut TinmanWorld,
    step: &cucumber::gherkin::Step,
) {
    let line = step
        .docstring()
        .expect("the step carries a request doc string")
        .trim()
        .to_string();
    let reply = driver(world).send_line(&line);
    world.reply = Some(reply);
}

#[when(expr = "the test runner requests the text {string} is present")]
async fn the_runner_requests_the_text_is_present(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(serde_json::json!({
        "id": id,
        "op": "expect",
        "session": session,
        "text": text,
    }));
    world.reply = Some(reply);
}

#[when("the test runner requests the terminal object model")]
async fn the_runner_requests_the_model(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(serde_json::json!({
        "id": id,
        "op": "tom",
        "session": session,
    }));
    world.reply = Some(reply);
}

#[when("the test runner closes the session")]
async fn the_test_runner_closes_the_session(world: &mut TinmanWorld) {
    let session = session(world);
    world.session_dirs = support::session_sandbox_dirs(&session);
    assert!(
        !world.session_dirs.is_empty(),
        "the session created no temporary sandbox directory, so this scenario would assert nothing"
    );
    let id = next_id(world);
    let reply = driver(world).request(serde_json::json!({
        "id": id,
        "op": "close",
        "session": session,
    }));
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(true),
        "the close request failed: {reply}"
    );
    world.reply = Some(reply);
}

fn reply(world: &TinmanWorld) -> &serde_json::Value {
    world.reply.as_ref().expect("the driver replied")
}

#[then(expr = "the driver replies to request {int} with a session identifier")]
async fn the_driver_replies_with_a_session(world: &mut TinmanWorld, id: u64) {
    let reply = reply(world);
    assert_eq!(reply["id"], serde_json::json!(id), "replied request id");
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(true),
        "the reply is not a success: {reply}"
    );
    let identifier = reply["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no session identifier: {reply}"));
    assert!(
        !identifier.is_empty(),
        "the reply carries an empty session identifier"
    );
}

#[then(expr = "the driver replies to request {int} with the error {string}")]
async fn the_driver_replies_with_the_error(world: &mut TinmanWorld, id: u64, message: String) {
    let reply = reply(world);
    assert_eq!(reply["id"], serde_json::json!(id), "replied request id");
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(false),
        "the reply is not a failure: {reply}"
    );
    assert_eq!(
        reply["error"].as_str(),
        Some(message.as_str()),
        "reported error"
    );
}

#[then("the driver replies with a failed result")]
async fn the_driver_replies_with_a_failed_result(world: &mut TinmanWorld) {
    let reply = reply(world);
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(false),
        "the reply is not a failure: {reply}"
    );
}

#[then("the driver answers a later screen request for the same session")]
async fn the_driver_answers_a_later_screen_request(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(serde_json::json!({
        "id": id,
        "op": "screen",
        "session": session,
    }));
    assert_eq!(
        reply["ok"],
        serde_json::Value::Bool(true),
        "the session did not survive the failed action: {reply}"
    );
}

#[then("the session's temporary sandbox directories no longer exist")]
async fn the_session_sandbox_directories_are_gone(world: &mut TinmanWorld) {
    let standing: Vec<&std::path::PathBuf> = world
        .session_dirs
        .iter()
        .filter(|path| path.exists())
        .collect();
    assert!(
        standing.is_empty(),
        "the session's sandbox directories still exist: {standing:?}"
    );
}

#[then(expr = "every exchanged message conforms to the {string} schema in {string}")]
async fn every_exchanged_message_conforms(
    world: &mut TinmanWorld,
    _schema_id: String,
    path: String,
) {
    let messages: Vec<serde_json::Value> = driver(world).exchanged().to_vec();
    assert!(
        !messages.is_empty(),
        "no message was exchanged, so this scenario would assert nothing"
    );
    let mut bad = Vec::new();
    for message in &messages {
        for violation in support::schema_counterexamples(&path, message) {
            bad.push(format!("{message}: {violation}"));
        }
    }
    assert!(bad.is_empty(), "schema violations: {bad:#?}");
}

// ---------------------------------------------------------------------------
// harness plan: the canonical YAML representation of a flow
// ---------------------------------------------------------------------------

/// A minimal shorthand plan carrying `step` as its only step, so a scenario can
/// name one step's shorthand without restating a whole plan.
fn plan_with_only_step(step: &str) -> String {
    format!("tui: printf READY\nsteps:\n  - {step}\n")
}

#[given(expr = "the harness plan at {string}")]
async fn the_harness_plan_at(world: &mut TinmanWorld, path: String) {
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("harness plan {path} unreadable: {e}"));
    world.plan_sources.push(text);
}

#[given("the harness plan:")]
async fn the_harness_plan_docstring(world: &mut TinmanWorld, step: &cucumber::gherkin::Step) {
    let text = step
        .docstring()
        .expect("the step carries a harness plan doc string")
        .clone();
    world.plan_sources.push(text);
}

#[given(expr = "a harness plan whose first step uses the keyword {string}")]
async fn a_plan_whose_first_step_uses_keyword(world: &mut TinmanWorld, keyword: String) {
    world
        .plan_sources
        .push(plan_with_only_step(&format!("{keyword}: Settings")));
}

#[given("a harness plan that defines no flow")]
async fn a_plan_that_defines_no_flow(world: &mut TinmanWorld) {
    world
        .plan_sources
        .push("sandbox:\n  network: deny\n".to_string());
}

#[given(expr = "a harness plan whose only step is {string}")]
async fn a_plan_whose_only_step_is(world: &mut TinmanWorld, shorthand: String) {
    world.plan_sources.push(plan_with_only_step(&shorthand));
}

#[when("the plan is parsed")]
async fn the_plan_is_parsed(world: &mut TinmanWorld) {
    let source = world
        .plan_sources
        .first()
        .expect("a harness plan was given")
        .clone();
    match tinman::plan::parse(&source) {
        Ok(plan) => {
            world.serialized = Some(to_json(&plan));
            world.parsed_plans = vec![plan];
        }
        Err(e) => world.parse_error = Some(e.to_string()),
    }
}

#[when("both plans are parsed")]
async fn both_plans_are_parsed(world: &mut TinmanWorld) {
    assert_eq!(
        world.plan_sources.len(),
        2,
        "two harness plans were given to parse"
    );
    world.parsed_plans = world
        .plan_sources
        .iter()
        .map(|source| {
            tinman::plan::parse(source)
                .unwrap_or_else(|e| panic!("harness plan did not parse: {e}"))
        })
        .collect();
}

fn parse_failure(world: &TinmanWorld) -> &str {
    world
        .parse_error
        .as_deref()
        .expect("parsing failed and reported why")
}

#[then(expr = "parsing fails and reports the unknown step keyword {string}")]
async fn parsing_reports_unknown_keyword(world: &mut TinmanWorld, keyword: String) {
    let message = parse_failure(world);
    assert!(
        message.contains(&keyword),
        "the parse failure does not name {keyword:?}: {message}"
    );
    assert!(
        message.to_lowercase().contains("unknown"),
        "the parse failure does not report the keyword as unknown: {message}"
    );
}

#[then("parsing fails and reports a missing flow")]
async fn parsing_reports_missing_flow(world: &mut TinmanWorld) {
    let message = parse_failure(world);
    assert!(
        message.to_lowercase().contains("flow"),
        "the parse failure does not report a missing flow: {message}"
    );
}

fn parsed_plan(world: &TinmanWorld) -> &tinman::plan::Plan {
    world
        .parsed_plans
        .first()
        .expect("a harness plan was parsed")
}

/// The sandbox specification the scenario put under assertion: the one a plan
/// parsed into, or the one a bare sandbox section parsed into.
fn parsed_sandbox(world: &TinmanWorld) -> &tinman::sandbox::SandboxSpec {
    if let Some(spec) = world.parsed_sandbox.as_ref() {
        return spec;
    }
    &parsed_plan(world).sandbox
}

#[then("the parsed sandbox specification names no Bubblewrap flag")]
async fn sandbox_names_no_bubblewrap_flag(world: &mut TinmanWorld) {
    let text =
        serde_yaml::to_string(parsed_sandbox(world)).expect("the sandbox specification serializes");
    let flags: Vec<&str> = text
        .split_whitespace()
        .filter(|token| token.starts_with("--"))
        .collect();
    assert!(
        flags.is_empty(),
        "the parsed sandbox specification names the flags {flags:?}"
    );
    assert!(
        !text.contains("bwrap"),
        "the parsed sandbox specification names bwrap: {text}"
    );
}

#[then("the parsed sandbox specification denies network access")]
async fn sandbox_denies_network(world: &mut TinmanWorld) {
    assert_eq!(
        parsed_sandbox(world).network,
        tinman::sandbox::Network::Deny,
        "sandbox network policy"
    );
}

#[then("the parsed sandbox specification provisions an empty home")]
async fn sandbox_provisions_empty_home(world: &mut TinmanWorld) {
    assert_eq!(
        parsed_sandbox(world).home,
        tinman::sandbox::Home::Empty,
        "sandbox home provisioning"
    );
}

#[then(expr = "the plan's flow holds {int} step")]
async fn the_plans_flow_holds_steps(world: &mut TinmanWorld, expected: usize) {
    assert_eq!(parsed_plan(world).flow.len(), expected, "flow steps");
}

#[then(expr = "the flow's first step drives the command {string}")]
async fn the_first_step_drives_the_command(world: &mut TinmanWorld, expected: String) {
    match parsed_plan(world)
        .flow
        .first()
        .expect("the flow has a step")
    {
        tinman::plan::FlowStep::Tui(tui) => {
            assert_eq!(tui.command, expected, "driven command");
        }
        other => panic!("the flow's first step drives no terminal program: {other:?}"),
    }
}

/// The only action of the only flow step, for the shorthand scenarios that name
/// a plan's single step.
fn only_action(world: &TinmanWorld) -> &tinman::plan::Action {
    match parsed_plan(world)
        .flow
        .first()
        .expect("the flow has a step")
    {
        tinman::plan::FlowStep::Tui(tui) => tui.steps.first().expect("the step drives one action"),
        other => panic!("the flow's first step drives no terminal program: {other:?}"),
    }
}

#[then(expr = "the step expects the text {string}")]
async fn the_step_expects_the_text(world: &mut TinmanWorld, expected: String) {
    match only_action(world) {
        tinman::plan::Action::Expect(expectation) => {
            assert_eq!(expectation.text, expected, "expected text");
        }
        other => panic!("the step is not an expectation: {other:?}"),
    }
}

#[then(expr = "the step's locator name is {string}")]
async fn the_steps_locator_name_is(world: &mut TinmanWorld, expected: String) {
    match only_action(world) {
        tinman::plan::Action::Activate(locator) => {
            assert_eq!(locator.name, expected, "locator name");
        }
        other => panic!("the step activates nothing: {other:?}"),
    }
}

#[then("the step's locator names no role")]
async fn the_steps_locator_names_no_role(world: &mut TinmanWorld) {
    match only_action(world) {
        tinman::plan::Action::Activate(locator) => {
            assert_eq!(locator.role, None, "locator role");
        }
        other => panic!("the step activates nothing: {other:?}"),
    }
}

#[then("the two parsed plans are identical")]
async fn the_two_parsed_plans_are_identical(world: &mut TinmanWorld) {
    assert_eq!(world.parsed_plans.len(), 2, "two plans were parsed");
    let first = to_json(&world.parsed_plans[0]);
    let second = to_json(&world.parsed_plans[1]);
    assert_eq!(
        first, second,
        "the full form and the shorthand form parsed to different plans"
    );
}

#[given(expr = "a plan sandbox section mounting {string} at {string} with no mode")]
async fn a_sandbox_section_mounting_without_mode(
    world: &mut TinmanWorld,
    source: String,
    target: String,
) {
    world.plan_sources.push(format!(
        "mounts:\n  - source: {source}\n    target: {target}\n"
    ));
}

#[when("the sandbox specification is parsed")]
async fn the_sandbox_specification_is_parsed(world: &mut TinmanWorld) {
    let source = world
        .plan_sources
        .first()
        .expect("a sandbox section was given")
        .clone();
    match tinman::plan::parse_sandbox(&source) {
        Ok(spec) => world.parsed_sandbox = Some(spec),
        Err(e) => world.parse_error = Some(e.to_string()),
    }
}

#[then(expr = "the mount's mode is {string}")]
async fn the_mounts_mode_is(world: &mut TinmanWorld, expected: String) {
    let spec = parsed_sandbox(world);
    let mount = spec.mounts.first().expect("the sandbox names a mount");
    assert_eq!(mount.mode.as_name(), expected, "mount mode");
}

// ---------------------------------------------------------------------------
// terminal object model: the screen read as nested regions with roles
// ---------------------------------------------------------------------------

#[given(expr = "a virtual screen {int} columns wide split vertically at column {int}")]
async fn a_screen_split_vertically(world: &mut TinmanWorld, cols: u16, column: u16) {
    world.screen = Some(support::vertical_split_screen(cols, column));
}

#[given(expr = "a virtual screen showing a bordered pane titled {string}")]
async fn a_screen_with_a_bordered_pane(world: &mut TinmanWorld, title: String) {
    world.screen = Some(support::bordered_pane_screen(&title, &[], None));
    world.pane = Some((title, Vec::new()));
}

#[given(
    expr = "a virtual screen showing a bordered pane titled {string} listing {string} and {string}"
)]
async fn a_screen_with_a_pane_listing_two(
    world: &mut TinmanWorld,
    title: String,
    first: String,
    second: String,
) {
    let items = vec![first, second];
    world.screen = Some(support::bordered_pane_screen(&title, &items, None));
    world.pane = Some((title, items));
}

#[given(
    expr = "a virtual screen showing a bordered pane titled {string} listing {string}, {string}, and {string}"
)]
async fn a_screen_with_a_pane_listing_three(
    world: &mut TinmanWorld,
    title: String,
    first: String,
    second: String,
    third: String,
) {
    let items = vec![first, second, third];
    world.screen = Some(support::bordered_pane_screen(&title, &items, None));
    world.pane = Some((title, items));
}

#[given(expr = "the line {string} is rendered with reversed video")]
async fn the_line_is_reversed(world: &mut TinmanWorld, line: String) {
    let (title, items) = world
        .pane
        .as_ref()
        .expect("a bordered pane was drawn")
        .clone();
    assert!(
        items.contains(&line),
        "the pane lists {items:?}, so it has no line {line:?} to reverse"
    );
    world.screen = Some(support::bordered_pane_screen(&title, &items, Some(&line)));
}

#[given(expr = "a virtual screen whose bottom line reads {string}")]
async fn a_screen_whose_bottom_line_reads(world: &mut TinmanWorld, text: String) {
    world.screen = Some(support::bottom_line_screen(&text));
}

#[given(expr = "a virtual screen whose top line reads {string}")]
async fn a_screen_whose_top_line_reads(world: &mut TinmanWorld, text: String) {
    world.screen = Some(support::top_line_screen(&text));
}

#[when("the terminal object model is built")]
async fn the_model_is_built(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a virtual screen");
    world.tom = Some(tinman::tom::build(screen));
}

#[when("the terminal object model is serialized")]
async fn the_model_is_serialized(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a virtual screen");
    world.serialized = Some(to_json(&tinman::tom::build(screen)));
}

fn model(world: &TinmanWorld) -> &tinman::tom::Model {
    world.tom.as_ref().expect("a terminal object model")
}

#[then(expr = "the model's root has {int} child regions")]
async fn the_root_has_child_regions(world: &mut TinmanWorld, expected: usize) {
    let root_children = model(world).root.children.len();
    assert_eq!(root_children, expected, "root child regions");
}

#[then(expr = "the first child region covers columns {int} through {int}")]
async fn the_first_child_covers_columns(world: &mut TinmanWorld, first: u16, last: u16) {
    let child = model(world)
        .root
        .children
        .first()
        .expect("the root has a child region");
    assert_eq!(
        child.rect.x, first,
        "first column of the first child region"
    );
    assert_eq!(
        child.rect.x + child.rect.width - 1,
        last,
        "last column of the first child region"
    );
}

#[then(expr = "the model contains a region named {string}")]
async fn the_model_contains_a_region_named(world: &mut TinmanWorld, name: String) {
    let region = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"))
        .clone();
    world.found_region = Some(region);
}

#[then(expr = "the model contains a region with the role {string}")]
async fn the_model_contains_a_region_with_role(world: &mut TinmanWorld, role: String) {
    let region = model(world)
        .find_role(&role)
        .unwrap_or_else(|| panic!("the model contains no region with the role {role:?}"))
        .clone();
    world.found_region = Some(region);
}

#[then(expr = "the region named {string} has the role {string}")]
async fn the_region_named_has_the_role(world: &mut TinmanWorld, name: String, role: String) {
    let region = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"))
        .clone();
    assert_eq!(region.role(), role, "role of the region named {name:?}");
    world.found_region = Some(region);
}

#[then(expr = "that region has {int} child regions with the role {string}")]
async fn that_region_has_child_regions_with_role(
    world: &mut TinmanWorld,
    expected: usize,
    role: String,
) {
    let region = world.found_region.as_ref().expect("a region was found");
    let matching = region
        .children
        .iter()
        .filter(|child| child.role() == role)
        .count();
    assert_eq!(matching, expected, "children with the role {role:?}");
}

#[then(expr = "that region's text is {string}")]
async fn that_regions_text_is(world: &mut TinmanWorld, expected: String) {
    let region = world.found_region.as_ref().expect("a region was found");
    assert_eq!(
        region.text.as_deref(),
        Some(expected.as_str()),
        "region text"
    );
}

#[then(expr = "the selected item of the region named {string} is {string}")]
async fn the_selected_item_is(world: &mut TinmanWorld, name: String, expected: String) {
    let region = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"));
    let selected = region
        .selected_item()
        .unwrap_or_else(|| panic!("the region named {name:?} has no selected item"));
    assert_eq!(
        selected.text.as_deref(),
        Some(expected.as_str()),
        "selected item"
    );
}

#[then(
    expr = "the region named {string} reports the screen cell at its own row {int} column {int}"
)]
async fn the_region_reports_its_own_cell(
    world: &mut TinmanWorld,
    name: String,
    row: u16,
    col: u16,
) {
    let screen = world.screen.as_ref().expect("a virtual screen");
    let region = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"));
    let expected = screen.cell(region.rect.y + row, region.rect.x + col);
    assert_eq!(
        region.cell(row, col),
        expected,
        "the region's own cell at row {row} column {col} differs from the screen cell it was built from"
    );
}

// ---------------------------------------------------------------------------
// terminal object model locators: mechanical resolution by role and name
// ---------------------------------------------------------------------------

#[given(
    expr = "a terminal object model with a {string} containing the menu items {string}, {string}, and {string}"
)]
async fn a_model_with_a_menu(
    world: &mut TinmanWorld,
    container_role: String,
    first: String,
    second: String,
    third: String,
) {
    let items: Vec<tinman::tom::Region> = [first, second, third]
        .iter()
        .enumerate()
        .map(|(index, label)| {
            tinman::tom::Region::leaf(
                "menuitem",
                Some(label),
                Some(label),
                tinman::tom::Rect {
                    x: index as u16 * 10,
                    y: 0,
                    width: 10,
                    height: 1,
                },
            )
        })
        .collect();
    let menu = tinman::tom::Region::parent(
        &container_role,
        None,
        tinman::tom::Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        },
        items,
    );
    world.tom = Some(tinman::tom::Model::rooted(24, 80, vec![menu]));
}

#[given(
    expr = "a terminal object model with a {string} named {string} containing {string} and a {string} named {string} containing {string}"
)]
async fn a_model_with_two_lists(
    world: &mut TinmanWorld,
    left_role: String,
    left_name: String,
    left_item: String,
    right_role: String,
    right_name: String,
    right_item: String,
) {
    let list = |role: &str, name: &str, item: &str, x: u16| {
        tinman::tom::Region::parent(
            role,
            Some(name),
            tinman::tom::Rect {
                x,
                y: 0,
                width: 40,
                height: 24,
            },
            vec![tinman::tom::Region::leaf(
                "listitem",
                Some(item),
                Some(item),
                tinman::tom::Rect {
                    x,
                    y: 1,
                    width: 40,
                    height: 1,
                },
            )],
        )
    };
    world.tom = Some(tinman::tom::Model::rooted(
        24,
        80,
        vec![
            list(&left_role, &left_name, &left_item, 0),
            list(&right_role, &right_name, &right_item, 40),
        ],
    ));
}

#[when(expr = "the locator for the {string} named {string} is resolved")]
async fn the_locator_is_resolved(world: &mut TinmanWorld, role: String, name: String) {
    let locator = tinman::tom::Locator::new(&role, &name);
    world.tom_resolution = Some(locator.resolve(model(world)));
}

#[when(
    expr = "the locator for the {string} named {string} is resolved within the region named {string}"
)]
async fn the_locator_is_resolved_within(
    world: &mut TinmanWorld,
    role: String,
    name: String,
    scope: String,
) {
    let locator = tinman::tom::Locator::new(&role, &name).within(&scope);
    world.tom_resolution = Some(locator.resolve(model(world)));
}

#[then(expr = "the resolved region's text is {string}")]
async fn the_resolved_regions_text_is(world: &mut TinmanWorld, expected: String) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::One(region) => {
            assert_eq!(
                region.text.as_deref(),
                Some(expected.as_str()),
                "region text"
            );
        }
        other => panic!("the locator did not resolve to one region: {other:?}"),
    }
}

#[then("resolution fails and reports no match")]
async fn resolution_reports_no_match(world: &mut TinmanWorld) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::NoMatch => {}
        other => panic!("the locator did not report no match: {other:?}"),
    }
}

#[then(expr = "resolution fails and reports {int} matches")]
async fn resolution_reports_matches(world: &mut TinmanWorld, expected: usize) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::Ambiguous(count) => {
            assert_eq!(*count, expected, "ambiguous match count");
        }
        other => panic!("the locator did not report an ambiguity: {other:?}"),
    }
}

#[then(expr = "the resolved region lies inside the region named {string}")]
async fn the_resolved_region_lies_inside(world: &mut TinmanWorld, name: String) {
    let scope = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"))
        .rect;
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::One(region) => {
            let r = region.rect;
            assert!(
                r.x >= scope.x
                    && r.y >= scope.y
                    && r.x + r.width <= scope.x + scope.width
                    && r.y + r.height <= scope.y + scope.height,
                "the resolved region at {r:?} lies outside the region named {name:?} at {scope:?}"
            );
        }
        other => panic!("the locator did not resolve to one region: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// terminal object model inference: a second producer of the same shape
// ---------------------------------------------------------------------------

#[given(expr = "an engine that labels that line a {string}")]
async fn an_engine_that_labels_that_line(world: &mut TinmanWorld, role: String) {
    let reply = serde_json::json!({
        "rows": 24,
        "cols": 80,
        "root": {
            "role": "screen",
            "name": null,
            "text": null,
            "selected": false,
            "rect": {"x": 0, "y": 0, "width": 80, "height": 24},
            "children": [{
                "role": role,
                "name": null,
                "text": null,
                "selected": false,
                "rect": {"x": 0, "y": 0, "width": 80, "height": 1},
                "children": []
            }]
        }
    });
    let provider = support::LocalProvider::returning(&reply.to_string());
    use_provider(world, provider);
}

#[given(expr = "an engine that returns a region with the role {string}")]
async fn an_engine_that_returns_role(world: &mut TinmanWorld, role: String) {
    let reply = serde_json::json!({
        "rows": 24,
        "cols": 80,
        "root": {
            "role": role,
            "name": null,
            "text": null,
            "selected": false,
            "rect": {"x": 0, "y": 0, "width": 80, "height": 24},
            "children": []
        }
    });
    let provider = support::LocalProvider::returning(&reply.to_string());
    use_provider(world, provider);
}

#[when("the terminal object model is inferred")]
async fn the_model_is_inferred(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    let screen = world.screen.as_ref().expect("a virtual screen").clone();
    world.tom = Some(tinman::tom::infer(&screen, &settings));
}

// ---------------------------------------------------------------------------
// interactive assistant: proposals reach the operating system only through the
// command parser
// ---------------------------------------------------------------------------

#[given(expr = "the assistant infers the command {string}")]
async fn the_assistant_infers_the_command(world: &mut TinmanWorld, command: String) {
    let reply = tinman::assistant::model_reply_proposing(&command);
    let provider = support::LocalProvider::returning(&reply);
    use_provider(world, provider);
}

#[given(expr = "the assistant answers {string}")]
async fn the_assistant_answers(world: &mut TinmanWorld, answer: String) {
    let reply = tinman::assistant::model_reply_answering(&answer);
    let provider = support::LocalProvider::returning(&reply);
    use_provider(world, provider);
}

#[when(expr = "the operator asks {string}")]
async fn the_operator_asks(world: &mut TinmanWorld, question: String) {
    let settings = resolved_settings(world);
    world.response = Some(tinman::assistant::ask(&settings, &question));
}

#[given(expr = "the assistant has proposed the command {string}")]
async fn the_assistant_has_proposed(world: &mut TinmanWorld, command: String) {
    world.response = Some(tinman::assistant::Response::Proposal(
        tinman::assistant::Proposal::new(&command),
    ));
}

#[when("the operator declines the proposal")]
async fn the_operator_declines_the_proposal(world: &mut TinmanWorld) {
    match world.response.take().expect("the assistant responded") {
        tinman::assistant::Response::Proposal(proposal) => proposal.decline(),
        other => panic!("there is no proposal to decline: {other:?}"),
    }
}

#[when("the operator confirms the proposal")]
async fn the_operator_confirms_the_proposal(world: &mut TinmanWorld) {
    match world.response.take().expect("the assistant responded") {
        tinman::assistant::Response::Proposal(proposal) => {
            world.parser_arguments = Some(
                proposal
                    .confirm()
                    .expect("the command parser accepts the confirmed proposal"),
            );
        }
        other => panic!("there is no proposal to confirm: {other:?}"),
    }
}

#[then(expr = "the assistant displays the proposed command {string}")]
async fn the_assistant_displays_the_proposed_command(world: &mut TinmanWorld, expected: String) {
    match world.response.as_ref().expect("the assistant responded") {
        tinman::assistant::Response::Proposal(proposal) => {
            assert_eq!(proposal.command(), expected, "proposed command");
        }
        other => panic!("the assistant proposed no command: {other:?}"),
    }
}

#[then("the proposed command has not run")]
async fn the_proposed_command_has_not_run(world: &mut TinmanWorld) {
    assert!(
        world.parser_arguments.is_none(),
        "the command parser received {:?}",
        world.parser_arguments
    );
}

#[then(expr = "the command parser receives the arguments {string} and {string}")]
async fn the_command_parser_receives_the_arguments(
    world: &mut TinmanWorld,
    first: String,
    second: String,
) {
    let arguments = world
        .parser_arguments
        .as_ref()
        .expect("a proposal was confirmed");
    assert_eq!(arguments, &vec![first, second], "parser arguments");
}

#[then("the assistant refuses the proposal")]
async fn the_assistant_refuses_the_proposal(world: &mut TinmanWorld) {
    match world.response.as_ref().expect("the assistant responded") {
        tinman::assistant::Response::Refusal(_) => {}
        other => panic!("the assistant did not refuse: {other:?}"),
    }
}

#[then("no command is offered to the operator")]
async fn no_command_is_offered(world: &mut TinmanWorld) {
    if let Some(tinman::assistant::Response::Proposal(proposal)) = world.response.as_ref() {
        panic!("the assistant offered the command {:?}", proposal.command());
    }
    assert!(
        world.parser_arguments.is_none(),
        "the command parser received {:?}",
        world.parser_arguments
    );
}

#[then(expr = "the assistant displays the answer {string}")]
async fn the_assistant_displays_the_answer(world: &mut TinmanWorld, expected: String) {
    match world.response.as_ref().expect("the assistant responded") {
        tinman::assistant::Response::Answer(text) => {
            assert_eq!(text, &expected, "displayed answer");
        }
        other => panic!("the assistant gave no answer: {other:?}"),
    }
}

#[given("the interactive assistant source")]
async fn the_interactive_assistant_source(_world: &mut TinmanWorld) {
    // The boundary policy names the module path; nothing to stage here.
}

#[when("the verifier checks the assistant command boundary")]
async fn the_verifier_checks_the_assistant_boundary(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_boundary(
        "scantlings/assistant-command-boundary.json",
    ));
}

// ---------------------------------------------------------------------------
// methodology conformance: the derived rule set run as verification
// ---------------------------------------------------------------------------

#[given("the implementation sources")]
async fn implementation_sources(world: &mut TinmanWorld) {
    world.conformance_scope = Some("src".to_string());
}

#[given("the verification support sources")]
async fn verification_support_sources(world: &mut TinmanWorld) {
    world.conformance_scope = Some("tests".to_string());
}

#[when("the verification-conformance rule set is run")]
async fn conformance_rule_set_is_run(world: &mut TinmanWorld) {
    let scope = world
        .conformance_scope
        .as_ref()
        .expect("a source scope was named");
    world.conformance_matches = Some(support::run_conformance_scan(scope));
}

#[then(expr = "the {string} rule reports no match")]
async fn rule_reports_no_match(world: &mut TinmanWorld, rule_id: String) {
    let matches = world
        .conformance_matches
        .as_ref()
        .expect("the conformance rule set ran");
    let reported: Vec<String> = matches
        .iter()
        .filter(|m| m.rule_id == rule_id)
        .map(|m| m.to_string())
        .collect();
    assert!(
        reported.is_empty(),
        "the {rule_id} rule reported {} match(es):\n{}",
        reported.len(),
        reported.join("\n")
    );
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
