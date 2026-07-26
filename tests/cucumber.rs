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
    conformance_scopes: Option<Vec<String>>,
    conformance_matches: Option<Vec<support::ConformanceMatch>>,
    // the plank inventory, the step-definition pattern set, the scenarios the
    // specs declare, and each join's counterexamples
    planks: Option<Vec<support::Plank>>,
    step_patterns: Option<Vec<support::StepPattern>>,
    spec_scenarios: Option<Vec<support::SpecScenario>>,
    stale_planks: Option<Vec<String>>,
    unbound_patterns: Option<Vec<String>>,
    metacharacter_names: Option<Vec<String>>,
    // the provisional planks the implementation carries, and the spent ones,
    // naming a scenario Captain has already disposed of
    provisional_planks: Option<support::ProvisionalInventory>,
    spent_provisional_planks: Option<Vec<String>>,
    // the tier ceilings the rigging declares, the sweeps the weather record
    // carries, and the tiers whose most recent sweep outran their ceiling
    rigging_path: Option<String>,
    tier_budgets: Option<Vec<support::TierBudget>>,
    recorded_sweeps: Option<Vec<support::RecordedSweep>>,
    over_budget_sweeps: Option<Vec<String>>,
    // published scantling contracts: the dialect-declaring scantlings, what the
    // meta-schema said of each, the packaged version, and the URIs consumers
    // fetch
    dialect_scantlings: Option<Vec<(String, serde_json::Value)>>,
    meta_schema_results: Option<Vec<(String, Option<String>)>>,
    package_version: Option<String>,
    published_uris: Option<Vec<(String, String)>>,
    // the proof contracts and the meta-schema their shape is checked against
    proof_contracts: Option<Vec<(String, serde_json::Value)>>,
    meta_schema_path: Option<String>,
    // scantling enumerations joined to the production enumerations they
    // constrain, and each direction's counterexamples
    enumeration_pairs: Option<Vec<support::EnumerationPair>>,
    rejected_values: Option<Vec<String>>,
    undeclared_variants: Option<Vec<String>>,
    // bundled skill
    skill_path: Option<String>,
    loaded_skill: Option<tinman::skill::Skill>,
    skill_context: Option<String>,
    // help and command line
    asset_text: Option<String>,
    accepted_commands: Option<Vec<String>>,
    read_command_set: Option<Vec<String>>,
    listed_commands: Option<Vec<String>>,
    advertised_options: Option<Vec<String>>,
    option_rejections: Option<Vec<String>>,
    // the Tinman command lines an asset names, and the parser's refusals
    named_command_lines: Option<Vec<String>>,
    command_line_rejections: Option<Vec<String>>,
    placeholder_count: Option<usize>,
    // running the real binary
    scratch: Option<support::ScratchDir>,
    terminal_session: Option<support::TerminalSession>,
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
    // the command a launch call named, so a failed launch can be checked to
    // name the program it could not start
    launched_command: Option<String>,
    // a staged fixture file and the contents it was staged with, so a mount
    // scenario can tell a changed source from an untouched one
    fixture_file: Option<(std::path::PathBuf, String)>,
    // semantic capture
    captures: std::collections::BTreeMap<String, Vec<String>>,
    log_title: Option<String>,
    log_messages: Option<usize>,
    log_window: Option<usize>,
    // harness plans
    plan_sources: Vec<String>,
    parsed_plans: Vec<tinman::plan::Plan>,
    replay_plan: Option<tinman::plan::Plan>,
    replay_plan_source: Option<String>,
    replay_columns: Option<u16>,
    record_program: Option<String>,
    proposed_name: Option<String>,
    parsed_sandbox: Option<tinman::sandbox::SandboxSpec>,
    parse_error: Option<String>,
    // terminal object model
    pane: Option<(String, Vec<String>)>,
    tom: Option<tinman::tom::Model>,
    found_region: Option<tinman::tom::Region>,
    tom_resolution: Option<tinman::tom::Resolution>,
    tom_binding: Option<String>,
    record_error: Option<String>,
    written_plan_source: Option<String>,
    // interactive assistant
    response: Option<tinman::assistant::Response>,
    parser_arguments: Option<Vec<String>>,
    // inference configuration and requests
    env_vars: std::collections::BTreeMap<String, String>,
    settings: Option<tinman::inference::Settings>,
    built_requests: Vec<tinman::inference::Request>,
    inference_available: Option<bool>,
    // how long the availability check took, so a report claimed to be bounded is
    // judged on the clock rather than on its own word
    availability_elapsed: Option<std::time::Duration>,
    // what the configured provider generated, on the @inference tier
    provider_reply: Option<String>,
    // how long the configured provider took to generate it, so a step that got
    // nothing reports the clock it waited rather than only the absence
    provider_elapsed: Option<std::time::Duration>,
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
        cwd: None,
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
    // The sandbox-configuration scenarios give a sandbox section and assert on
    // what the launched process could see, so it reports what it was granted.
    if world.backend.is_none() {
        let spec = sandbox_section(world);
        let name = world
            .secret_name
            .clone()
            .unwrap_or_else(|| "PATH".to_string());
        let prepared = BubblewrapBackend::new()
            .prepare(&spec, &reporting_process(&name))
            .unwrap_or_else(|e| panic!("the Bubblewrap backend did not prepare the process: {e}"));
        world.screen = Some(tinman::pty::capture(&prepared).expect("the process is captured"));
        return;
    }
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

#[given(expr = "a prepared process that writes {string} at row {int} column {int}")]
async fn prepared_writes_at(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    let command_line = format!("printf '\\033[{row};{col}H{text}'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} and {string} at row {int} column {int}"
)]
async fn prepared_writes_at_two_cells(
    world: &mut TinmanWorld,
    first: String,
    first_row: u16,
    first_col: u16,
    second: String,
    second_row: u16,
    second_col: u16,
) {
    let command_line = format!(
        "printf '\\033[{first_row};{first_col}H{first}\\033[{second_row};{second_col}H{second}'"
    );
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} in reversed video"
)]
async fn prepared_writes_reversed(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    // SGR 7 turns on reversed video; SGR 0 restores the default attributes.
    let command_line = format!("printf '\\033[{row};{col}H\\033[7m{text}\\033[0m'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} in reversed video and erases to the end of the line"
)]
async fn prepared_writes_reversed_and_erases(
    world: &mut TinmanWorld,
    text: String,
    row: u16,
    col: u16,
) {
    // ESC[K erases from the cursor to the end of the line. A full-screen program
    // highlights a whole row this way: the erased cells carry the reversed video
    // that is in force when the erase runs.
    let command_line = format!("printf '\\033[{row};{col}H\\033[7m{text}\\033[K\\033[0m'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} using the HVP sequence"
)]
async fn prepared_writes_hvp(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    // HVP is ESC[row;colf. It addresses the same cell CUP does, through the
    // final byte `f` rather than `H`.
    let command_line = format!("printf '\\033[{row};{col}f{text}'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} using the HPA sequence"
)]
async fn prepared_writes_hpa(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    // HPA is ESC[col` and sets the column alone, so the row is addressed first
    // with the CUP form the suite already covers. Only the horizontal move is
    // under test.
    let command_line = format!("printf '\\033[{row}H\\033[{col}`{text}'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that writes {string} at row {int} column {int} and repeats it {int} times with the REP sequence"
)]
async fn prepared_writes_rep(
    world: &mut TinmanWorld,
    text: String,
    row: u16,
    col: u16,
    times: u16,
) {
    // REP is ESC[Nb and redraws the preceding character N further times, so the
    // row holds the written character plus N copies of it.
    let command_line = format!("printf '\\033[{row};{col}H{text}\\033[{times}b'");
    world.prepared = Some(shell_process(&command_line));
}

#[given(
    expr = "a prepared process that disables autowrap and writes {int} characters ending in {string} at row {int} column {int}"
)]
async fn prepared_writes_without_autowrap(
    world: &mut TinmanWorld,
    count: usize,
    last: String,
    row: u16,
    col: u16,
) {
    // ESC[?7l turns DECAWM off. With autowrap off a terminal holds the cursor in
    // the last column, so every character written past the screen width
    // overwrites that column and the final character is the one left standing.
    let filler = "a".repeat(count - last.chars().count());
    let command_line = format!("printf '\\033[?7l\\033[{row};{col}H{filler}{last}'");
    world.prepared = Some(shell_process(&command_line));
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

#[then(expr = "the virtual screen row {int} reads {string}")]
async fn virtual_screen_row_reads(world: &mut TinmanWorld, row: usize, text: String) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let contents = screen.contents();
    let actual = contents
        .lines()
        .nth(row - 1)
        .unwrap_or_default()
        .trim_end_matches(' ');
    assert_eq!(actual, text, "row {row}");
}

#[then(
    expr = "the virtual screen cell at row {int} column {int} continues the character at column {int}"
)]
async fn virtual_screen_cell_continues(world: &mut TinmanWorld, row: u16, col: u16, start: u16) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    assert_eq!(
        screen.continuation_start(row, col),
        Some(start),
        "cell at row {row} column {col} continues the character starting at column {start}"
    );
}

#[then(
    expr = "every cell of row {int} from column {int} through column {int} is rendered with reversed video"
)]
async fn every_cell_reversed(world: &mut TinmanWorld, row: u16, from: u16, through: u16) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let plain: Vec<u16> = (from..=through)
        .filter(|col| !screen.reverse(row, *col))
        .collect();
    assert!(
        plain.is_empty(),
        "row {row} columns {plain:?} are not reversed; contents:\n{}",
        screen.contents()
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

#[given(expr = "the command lines in the asset at {string}")]
async fn the_command_lines_in_the_asset(world: &mut TinmanWorld, path: String) {
    let asset =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}"));
    world.named_command_lines = Some(support::named_command_lines(&asset));
}

#[when("each named command is passed to the command parser")]
async fn each_named_command_is_passed_to_the_parser(world: &mut TinmanWorld) {
    let lines = world
        .named_command_lines
        .as_ref()
        .expect("the asset's command lines were read");
    let mut rejections = Vec::new();
    for line in lines {
        if let Err(e) = tinman::cli::parse_command_line(line) {
            rejections.push(format!("{line:?} refused: {e}"));
        }
    }
    world.command_line_rejections = Some(rejections);
}

#[then("the parser accepts every command the skill names")]
async fn the_parser_accepts_every_command_the_skill_names(world: &mut TinmanWorld) {
    let lines = world
        .named_command_lines
        .as_ref()
        .expect("the asset's command lines were read");
    assert!(
        !lines.is_empty(),
        "the skill names no command line, so this scenario would assert nothing"
    );
    let rejected = world
        .command_line_rejections
        .as_ref()
        .expect("the command lines were passed to the parser");
    assert!(
        rejected.is_empty(),
        "command lines the skill names but the parser refuses: {rejected:?}"
    );
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

#[then(expr = "the context begins with the body of the asset at {string}")]
async fn the_context_begins_with_the_asset_body(world: &mut TinmanWorld, path: String) {
    let body = asset_body(&path);
    let context = world.skill_context.as_ref().expect("a context was built");
    assert!(
        !body.is_empty(),
        "the asset at {path} is empty, so this scenario would assert nothing"
    );
    assert!(
        context.trim_start().starts_with(&body),
        "the context does not begin with the body of {path}; context: {context:?}"
    );
}

#[then(expr = "the context carries the skill's {string} and {string} fields")]
async fn the_context_carries_the_skill_fields(
    world: &mut TinmanWorld,
    first: String,
    second: String,
) {
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

#[when(expr = "the operator executes {string}")]
async fn operator_executes(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let outcome = support::run_tinman(&dir, &argv, &env, None)
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

#[when("the accepted command set is read")]
async fn the_accepted_command_set_is_read(world: &mut TinmanWorld) {
    let mut commands = world
        .accepted_commands
        .as_ref()
        .expect("the parser reported its commands")
        .clone();
    commands.sort();
    commands.dedup();
    world.read_command_set = Some(commands);
}

#[then(expr = "it is exactly {string}, {string}, {string}, {string} and {string}")]
async fn the_command_set_is_exactly(
    world: &mut TinmanWorld,
    first: String,
    second: String,
    third: String,
    fourth: String,
    fifth: String,
) {
    let read = world
        .read_command_set
        .as_ref()
        .expect("the accepted command set was read");
    let mut expected = vec![first, second, third, fourth, fifth];
    expected.sort();
    assert_eq!(
        read, &expected,
        "the parser accepts {read:?}, the scenario names {expected:?}"
    );
}

#[given("the command dispatch source")]
async fn the_command_dispatch_source(_world: &mut TinmanWorld) {
    // The completeness contract names the module path; nothing to stage here.
}

#[when("the verifier checks the command dispatch completeness")]
async fn the_verifier_checks_command_dispatch_completeness(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_boundary(
        "scantlings/command-dispatch-completeness.json",
    ));
}

#[when(expr = "each is looked for in the Commands block of the asset at {string}")]
async fn each_is_looked_for_in_the_commands_block(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    world.listed_commands = Some(support::listed_commands(&asset));
}

#[then("every accepted command is listed in the Commands block")]
async fn every_accepted_command_is_listed(world: &mut TinmanWorld) {
    let commands = world
        .accepted_commands
        .as_ref()
        .expect("the parser reported its commands");
    let listed = world
        .listed_commands
        .as_ref()
        .expect("the Commands block was read");
    assert!(
        !commands.is_empty(),
        "the parser reported no commands, so this scenario would assert nothing"
    );
    assert!(
        !listed.is_empty(),
        "the Commands block lists nothing, so this scenario would assert nothing"
    );
    let missing: Vec<&String> = commands
        .iter()
        .filter(|command| !listed.contains(command))
        .collect();
    assert!(
        missing.is_empty(),
        "commands the parser accepts but the Commands block omits: {missing:?}"
    );
}

#[given(expr = "the options the asset at {string} advertises")]
async fn the_options_the_asset_advertises(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    world.advertised_options = Some(support::advertised_options(&asset));
}

#[when("each is passed to the command parser")]
async fn each_is_passed_to_the_command_parser(world: &mut TinmanWorld) {
    use clap::Parser;
    let options = world
        .advertised_options
        .as_ref()
        .expect("the help asset's options were read");
    let mut rejected = Vec::new();
    for option in options {
        if let Err(e) = tinman::cli::Cli::try_parse_from(["tinman", option]) {
            // Clap reports a help or version flag as a display request rather
            // than a parse failure: the parser took the option and asked to
            // print. Every other error is the parser refusing the option.
            if !matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                rejected.push(format!("{option} refused as {:?}", e.kind()));
            }
        }
    }
    world.option_rejections = Some(rejected);
}

#[then("the parser accepts every advertised option")]
async fn the_parser_accepts_every_advertised_option(world: &mut TinmanWorld) {
    let options = world
        .advertised_options
        .as_ref()
        .expect("the help asset's options were read");
    assert!(
        !options.is_empty(),
        "the help text advertises no options, so this scenario would assert nothing"
    );
    let rejected = world
        .option_rejections
        .as_ref()
        .expect("the options were passed to the parser");
    assert!(
        rejected.is_empty(),
        "options the help text advertises but the parser refuses: {rejected:?}"
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

#[given(expr = "the environment sets {string} to {string} and {string} to {string}")]
async fn the_environment_sets_two(
    world: &mut TinmanWorld,
    first_key: String,
    first_value: String,
    second_key: String,
    second_value: String,
) {
    world.env_vars.insert(first_key, first_value);
    world.env_vars.insert(second_key, second_value);
}

#[given(expr = "neither the environment nor a dotenv file sets {string} or {string}")]
async fn neither_environment_nor_dotenv_sets_either(
    world: &mut TinmanWorld,
    first_key: String,
    second_key: String,
) {
    world.env_vars.remove(&first_key);
    world.env_vars.remove(&second_key);
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

#[given("the inference provider endpoint accepts the connection and never answers")]
async fn the_provider_accepts_and_never_answers(world: &mut TinmanWorld) {
    let provider = support::LocalProvider::stalling();
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
    let started = std::time::Instant::now();
    // A provider that accepts the connection and never answers returns only once
    // a ceiling ends the call, so this real-service step carries a failure
    // ceiling of its own. It sits above any ceiling a scenario asserts: what
    // fails here is a call carrying no ceiling at all, and a call whose ceiling a
    // scenario judges is left for that scenario to judge.
    let available = support::within_budget(
        "the configured inference provider",
        std::time::Duration::from_secs(45),
        move || tinman::inference::is_available(&settings),
    );
    world.availability_elapsed = Some(started.elapsed());
    world.inference_available = Some(available);
}

#[then("inference is reported unavailable")]
async fn inference_is_reported_unavailable(world: &mut TinmanWorld) {
    let available = world.inference_available.expect("availability was checked");
    assert!(!available, "inference was reported available");
}

#[then("the stalled endpoint received the request")]
async fn the_stalled_endpoint_received_the_request(world: &mut TinmanWorld) {
    let provider = world
        .provider
        .as_ref()
        .expect("a local provider is running");
    assert!(
        provider.received_request(),
        "the stalled endpoint received no request, so the report came from \
         something other than a real call left unanswered"
    );
}

#[then(expr = "inference is reported unavailable within {int} seconds")]
async fn inference_is_reported_unavailable_within(world: &mut TinmanWorld, seconds: u64) {
    let available = world.inference_available.expect("availability was checked");
    let elapsed = world
        .availability_elapsed
        .expect("availability was checked");
    assert!(!available, "inference was reported available");
    assert!(
        elapsed <= std::time::Duration::from_secs(seconds),
        "inference was reported unavailable only after {:.1}s, over the {seconds}s \
         the caller allows",
        elapsed.as_secs_f64()
    );
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
    // A built request is also the artifact the provider contract governs, so
    // keep its serialized wire form for the conformance assertion.
    world.serialized = Some(to_json(&world.built_requests[0]));
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

#[then(expr = "the request addresses {string} with the model {string}")]
async fn the_request_addresses_with_model(
    world: &mut TinmanWorld,
    endpoint: String,
    model: String,
) {
    for request in built_requests(world) {
        assert_eq!(request.address(), endpoint, "request endpoint");
        assert_eq!(request.model(), model, "request model");
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

#[then("the request carries no authorization header")]
async fn the_request_carries_no_authorization(world: &mut TinmanWorld) {
    for request in built_requests(world) {
        assert_eq!(request.authorization(), None, "authorization header");
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

#[given(expr = "the operator runs {string} in an interactive terminal")]
async fn given_operator_runs_interactive(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let session = support::TerminalSession::start(&dir, &argv, &env)
        .unwrap_or_else(|e| panic!("starting {line:?} on a terminal failed: {e}"));
    // The session is up once the prompt inviting a question stands on the
    // terminal, so nothing is typed before the program can read it.
    session.await_output(
        &asset_body("assets/help/assistant-prompt.txt"),
        std::time::Duration::from_secs(10),
    );
    world.terminal_session = Some(session);
}

#[when(expr = "the operator types {string} at the assistant prompt")]
async fn the_operator_types_at_the_assistant_prompt(world: &mut TinmanWorld, question: String) {
    world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session")
        .type_line(&question);
}

#[then(expr = "the output displays the answer {string}")]
async fn the_output_displays_the_answer(world: &mut TinmanWorld, expected: String) {
    world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .await_output_after_mark(&expected, std::time::Duration::from_secs(20));
}

#[when("the operator ends the input")]
async fn the_operator_ends_the_input(world: &mut TinmanWorld) {
    let status = {
        let session = world
            .terminal_session
            .as_mut()
            .expect("an interactive terminal session");
        session.end_input();
        session.wait(std::time::Duration::from_secs(20))
    };
    world.run_status = Some(status);
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
    match tinman::flow::execute(&plan, &workspace, None) {
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

#[given(expr = "a flow that runs {string}")]
async fn a_flow_that_runs(world: &mut TinmanWorld, command: String) {
    world
        .plan_sources
        .push(format!("flow:\n  - run: {command:?}\n"));
}

#[given(expr = "a flow whose only step runs {string} and expects the status {int}")]
async fn a_flow_whose_only_step_expects_the_status(
    world: &mut TinmanWorld,
    command: String,
    status: i32,
) {
    world.plan_sources.push(format!(
        "flow:\n  - run:\n      command: {command:?}\n      status: {status}\n"
    ));
}

#[given(expr = "a flow whose only step runs {string} in the directory {string}")]
async fn a_flow_whose_only_step_runs_in_the_directory(
    world: &mut TinmanWorld,
    command: String,
    directory: String,
) {
    world.plan_sources.push(format!(
        "flow:\n  - run:\n      command: {command:?}\n      cwd: {directory:?}\n"
    ));
}

#[given(expr = "a flow whose only step runs {string} with the input {string}")]
async fn a_flow_whose_only_step_runs_with_the_input(
    world: &mut TinmanWorld,
    command: String,
    input: String,
) {
    world.plan_sources.push(format!(
        "flow:\n  - run:\n      command: {command:?}\n      stdin: {input:?}\n"
    ));
}

#[given("a flow whose only step drives the fixture terminal program")]
async fn a_flow_whose_only_step_drives_the_fixture(world: &mut TinmanWorld) {
    let source = support::fixture_terminal_source();
    // The fixture draws its home directory before it draws READY, so expecting
    // READY gates the step on the program having drawn, rather than reading the
    // screen while it is still blank.
    world.plan_sources.push(format!(
        "flow:\n  - tui:\n      command: {source:?}\n      steps:\n        - expect: READY\n"
    ));
}

#[given(expr = "a flow whose only step runs {string}")]
async fn a_flow_whose_only_step_runs(world: &mut TinmanWorld, command: String) {
    world
        .plan_sources
        .push(format!("flow:\n  - run: {command:?}\n"));
}

#[then("the fixture program reports a home directory other than the operator's home")]
async fn the_fixture_reports_another_home(world: &mut TinmanWorld) {
    let screen = only_step(world).output.clone();
    let operator_home = std::env::var("HOME").expect("operator HOME is set");
    let home = screen
        .lines()
        .find_map(|line| line.trim_end().strip_prefix("HOME:"))
        .map(str::trim_end)
        .unwrap_or_else(|| {
            panic!("the fixture program printed no home directory; screen:\n{screen}")
        })
        .to_string();
    assert!(!home.is_empty(), "the sandbox home directory is empty");
    assert_ne!(
        home, operator_home,
        "the fixture program ran with the operator's home {operator_home}"
    );
}

#[then("the step reports a home directory other than the operator's home")]
async fn the_step_reports_another_home(world: &mut TinmanWorld) {
    let reported = only_step(world).output.trim().to_string();
    let operator_home = std::env::var("HOME").expect("operator HOME is set");
    assert!(!reported.is_empty(), "the step reported no home directory");
    assert_ne!(
        reported, operator_home,
        "the step ran with the operator's home {operator_home}"
    );
}

/// The one outcome a single-step flow produced.
fn only_step(world: &TinmanWorld) -> &tinman::flow::StepOutcome {
    let outcome = world.flow_outcome.as_ref().unwrap_or_else(|| {
        panic!(
            "the flow did not run to completion: {}",
            world.flow_error.as_deref().unwrap_or("no error reported")
        )
    });
    outcome
        .steps
        .last()
        .unwrap_or_else(|| panic!("the flow ran no steps"))
}

#[then(expr = "the step's standard output is {string}")]
async fn the_steps_standard_output_is(world: &mut TinmanWorld, expected: String) {
    assert_eq!(only_step(world).output, expected, "step standard output");
}

#[then(expr = "the step's standard error is {string}")]
async fn the_steps_standard_error_is(world: &mut TinmanWorld, expected: String) {
    assert_eq!(only_step(world).error, expected, "step standard error");
}

#[then("the flow passes")]
async fn the_flow_passes(world: &mut TinmanWorld) {
    assert!(
        world.flow_error.is_none(),
        "the flow failed: {}",
        world.flow_error.as_deref().unwrap_or_default()
    );
    assert!(world.flow_outcome.is_some(), "the flow produced no outcome");
}

#[then(expr = "execution fails and reports the status {int}")]
async fn execution_fails_and_reports_the_status(world: &mut TinmanWorld, status: i32) {
    let message = world
        .flow_error
        .as_deref()
        .expect("the flow failed and reported why");
    assert!(
        message.contains(&status.to_string()),
        "the failure does not report the status {status}: {message}"
    );
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

/// One JSON-RPC 2.0 request, the framing the driver protocol speaks. A call
/// carrying no arguments omits `params`, which the protocol schema allows.
fn rpc(id: u64, method: &str, params: serde_json::Value) -> serde_json::Value {
    let mut request = serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method});
    if params.as_object().is_some_and(|params| !params.is_empty()) {
        request["params"] = params;
    }
    request
}

/// The `result` object of a reply, failing loudly when the driver answered
/// with an error instead.
fn result(reply: &serde_json::Value) -> &serde_json::Value {
    reply
        .get("result")
        .unwrap_or_else(|| panic!("the reply carries no result object: {reply}"))
}

#[given("the Tinman driver is running")]
async fn the_tinman_driver_is_running(world: &mut TinmanWorld) {
    world.driver = Some(support::DriverProcess::start());
}

#[given(expr = "the Tinman driver has a session running {string}")]
async fn the_driver_has_a_session_running(world: &mut TinmanWorld, command: String) {
    launch_driver_session(world, &command).await;
}

#[given("the Tinman driver has a session running the fixture terminal program")]
async fn the_driver_has_a_session_running_the_fixture(world: &mut TinmanWorld) {
    launch_driver_session(world, support::fixture_terminal_source()).await;
}

/// Start the driver and launch `command`, keeping the session it opened, so
/// every step that needs a running session opens it the one way.
///
/// A launch reply says the program started, not that it has drawn. A program
/// that draws `READY` draws it last, after the menu line closes its
/// reverse-video run, so such a session is gated on that observed signal before
/// any step reads the screen. Without the gate a step can read a half-drawn
/// line whose reverse video has not been reset yet, which reads back as a menu
/// with every item selected.
///
/// A program that draws nothing until it is typed at announces no such signal,
/// so gating it on `READY` would wait out the driver's expectation deadline and
/// then fail on a signal the program never promised. Its own assertion step
/// carries the wait instead, on the output the program answers with. The PTY
/// buffers input the program has not read yet, so keys typed before it reaches
/// its read still reach it.
async fn launch_driver_session(world: &mut TinmanWorld, command: &str) {
    world.driver = Some(support::DriverProcess::start());
    let id = next_id(world);
    let reply = driver(world).request(rpc(id, "launch", serde_json::json!({"command": command})));
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "the launch request failed: {reply}"
    );
    let identifier = result(&reply)["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the launch reply carries no session identifier: {reply}"))
        .to_string();
    if command.contains("READY") {
        let gate_id = next_id(world);
        let drawn = driver(world).request(rpc(
            gate_id,
            "expect",
            serde_json::json!({"session": identifier, "text": "READY"}),
        ));
        assert_eq!(
            result(&drawn)["ok"],
            serde_json::Value::Bool(true),
            "the launched program never drew READY: {drawn}"
        );
    }
    world.session_id = Some(identifier);
    world.reply = Some(reply);
}

#[when(expr = "the test runner types {string}")]
async fn the_test_runner_types(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "press",
        serde_json::json!({"session": session, "key": text}),
    ));
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "typing {text:?} failed: {reply}"
    );
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
    // Keep the command the call named, so a step asserting on a failed launch
    // reads the program from the request rather than restating it.
    let request: serde_json::Value =
        serde_json::from_str(&line).expect("the request doc string is JSON");
    if let Some(command) = request["params"]["command"].as_str() {
        world.launched_command = Some(command.to_string());
    }
    let reply = driver(world).send_line(&line);
    world.reply = Some(reply);
}

#[when(expr = "the test runner requests the text {string} is present")]
async fn the_runner_requests_the_text_is_present(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "expect",
        serde_json::json!({"session": session, "text": text}),
    ));
    world.reply = Some(reply);
}

#[when("the test runner requests the terminal object model")]
async fn the_runner_requests_the_model(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "tom", serde_json::json!({"session": session})));
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
    let reply = driver(world).request(rpc(id, "close", serde_json::json!({"session": session})));
    assert_eq!(
        result(&reply)["ok"],
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
    assert_eq!(reply["jsonrpc"], serde_json::json!("2.0"), "reply framing");
    assert_eq!(reply["id"], serde_json::json!(id), "replied request id");
    assert_eq!(
        result(reply)["ok"],
        serde_json::Value::Bool(true),
        "the reply is not a success: {reply}"
    );
    let identifier = result(reply)["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no session identifier: {reply}"));
    assert!(
        !identifier.is_empty(),
        "the reply carries an empty session identifier"
    );
}

// ---------------------------------------------------------------------------
// driver session: the semantic verbs a test runner drives a terminal with
// ---------------------------------------------------------------------------

/// The `failure` a failed result carries, which names what was looked for.
fn failure_message(world: &TinmanWorld) -> String {
    let reply = reply(world);
    result(reply)["failure"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no failure message: {reply}"))
        .to_string()
}

/// The session's current screen, as the driver reports it.
fn session_screen(world: &mut TinmanWorld) -> String {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "screen", serde_json::json!({"session": session})));
    result(&reply)["screen"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no screen: {reply}"))
        .to_string()
}

#[then(expr = "the screen carries {string}")]
async fn the_screen_carries(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    // The driver's own expectation waits for the text toward a deadline, so this
    // ends on the program's output rather than on a screen read at a guessed
    // moment.
    let reply = driver(world).request(rpc(
        id,
        "expect",
        serde_json::json!({"session": session, "text": text}),
    ));
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "the screen never carried {text:?}: {reply}"
    );
}

#[then(expr = "the screen does not carry {string}")]
async fn the_screen_does_not_carry(world: &mut TinmanWorld, text: String) {
    let screen = session_screen(world);
    assert!(
        !screen.contains(&text),
        "the screen carries {text:?}, so what the driver typed is on it beside \
         what the program drew:\n{screen}"
    );
}

/// The root region of the terminal object model of the session's current
/// screen. The model carries its screen size beside a single root region, so
/// the walk starts at that root rather than at the model itself.
fn session_model(world: &mut TinmanWorld) -> serde_json::Value {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "tom", serde_json::json!({"session": session})));
    let model = result(&reply)["tom"].clone();
    let root = model["root"].clone();
    assert!(!root.is_null(), "the model carries no root region: {model}");
    root
}

/// The regions playing `role` anywhere in the model, in the order the model
/// carries them.
fn regions_playing(model: &serde_json::Value, role: &str, found: &mut Vec<serde_json::Value>) {
    if model["role"].as_str() == Some(role) {
        found.push(model.clone());
    }
    if let Some(children) = model["children"].as_array() {
        for child in children {
            regions_playing(child, role, found);
        }
    }
}

/// Ask the driver to activate the region playing `role` and named `name`.
fn activate_region(world: &mut TinmanWorld, role: &str, name: &str) -> serde_json::Value {
    let id = next_id(world);
    let session = session(world);
    driver(world).request(rpc(
        id,
        "activate",
        serde_json::json!({"session": session, "role": role, "name": name}),
    ))
}

#[when("the test runner requests the session's sandbox backend")]
async fn the_runner_requests_the_sandbox_backend(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "sandbox", serde_json::json!({"session": session})));
    world.reply = Some(reply);
}

#[then(expr = "the reported backend is {string}")]
async fn the_reported_backend_is(world: &mut TinmanWorld, expected: String) {
    let reply = reply(world);
    let backend = result(reply)["backend"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no backend: {reply}"));
    assert_eq!(
        backend, expected,
        "the sandbox backend the session runs under"
    );
}

#[when(expr = "the test runner activates the {string} named {string}")]
async fn the_runner_activates(world: &mut TinmanWorld, role: String, name: String) {
    let reply = activate_region(world, &role, &name);
    world.reply = Some(reply);
}

#[given(expr = "the test runner has activated the {string} named {string}")]
async fn the_runner_has_activated(world: &mut TinmanWorld, role: String, name: String) {
    let reply = activate_region(world, &role, &name);
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "the activation this scenario builds on failed: {reply}"
    );
    world.reply = Some(reply);
}

#[when(expr = "the test runner fills the textbox labelled {string} with {string}")]
async fn the_runner_fills_the_textbox(world: &mut TinmanWorld, label: String, value: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "fill",
        serde_json::json!({"session": session, "label": label, "value": value}),
    ));
    world.reply = Some(reply);
}

#[when(expr = "the test runner presses the key {string}")]
async fn the_runner_presses_the_key(world: &mut TinmanWorld, key: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "press",
        serde_json::json!({"session": session, "key": key}),
    ));
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "the key press failed: {reply}"
    );
    world.reply = Some(reply);
}

#[then(expr = "the screen contains the text {string}")]
async fn the_screen_contains_the_text(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "screen", serde_json::json!({"session": session})));
    let screen = result(&reply)["screen"]
        .as_str()
        .unwrap_or_else(|| panic!("the reply carries no screen: {reply}"))
        .to_string();
    assert!(
        screen.contains(&text),
        "the screen does not contain {text:?}; screen:\n{screen}"
    );
}

#[then(expr = "the screen shows a {string} named {string}")]
async fn the_screen_shows_a_region(world: &mut TinmanWorld, role: String, name: String) {
    let model = session_model(world);
    let mut regions = Vec::new();
    regions_playing(&model, &role, &mut regions);
    let shown = regions
        .iter()
        .any(|region| region["name"].as_str() == Some(name.as_str()));
    assert!(
        shown,
        "the screen shows no {role:?} named {name:?}; model:\n{model:#}"
    );
}

#[given(expr = "the menu's selected item is {string}")]
async fn the_menus_selected_item_is(world: &mut TinmanWorld, expected: String) {
    let model = session_model(world);
    let mut items = Vec::new();
    regions_playing(&model, "menuitem", &mut items);
    assert!(
        !items.is_empty(),
        "the program draws no menu items, so this scenario's precondition does not hold"
    );
    let selected: Vec<&str> = items
        .iter()
        .filter(|item| item["selected"] == serde_json::Value::Bool(true))
        .filter_map(|item| item["name"].as_str())
        .collect();
    assert_eq!(
        selected,
        vec![expected.as_str()],
        "the menu items the model reports as selected"
    );
}

#[then(expr = "the selected {string} is {string}")]
async fn the_selected_region_is(world: &mut TinmanWorld, role: String, expected: String) {
    let model = session_model(world);
    let mut items = Vec::new();
    regions_playing(&model, &role, &mut items);
    let selected: Vec<&str> = items
        .iter()
        .filter(|item| item["selected"] == serde_json::Value::Bool(true))
        .filter_map(|item| item["name"].as_str())
        .collect();
    assert_eq!(
        selected,
        vec![expected.as_str()],
        "the {role} regions the model reports as selected"
    );
}

#[then(expr = "the textbox labelled {string} contains {string}")]
async fn the_textbox_labelled_contains(world: &mut TinmanWorld, label: String, value: String) {
    let model = session_model(world);
    let mut boxes = Vec::new();
    regions_playing(&model, "textbox", &mut boxes);
    let found = boxes
        .iter()
        .find(|region| region["name"].as_str() == Some(label.as_str()))
        .unwrap_or_else(|| panic!("the model carries no textbox labelled {label:?}: {model}"));
    let text = found["text"].as_str().unwrap_or_default();
    assert!(
        text.contains(&value),
        "the textbox labelled {label:?} reads {text:?}, so it does not contain {value:?}"
    );
}

#[then("the failure reports the text was not found on screen")]
async fn the_failure_reports_the_text_not_found(world: &mut TinmanWorld) {
    let message = failure_message(world);
    assert!(
        message.contains("not found"),
        "the failure does not report the text was not found: {message}"
    );
    let reply = reply(world);
    let screen = result(reply)["screen"]
        .as_str()
        .unwrap_or_else(|| panic!("the failed reply carries no screen: {reply}"));
    assert!(
        !screen.trim().is_empty(),
        "the failed reply carries an empty screen, so it does not show what the step saw"
    );
}

#[then(expr = "the failure reports no {string} named {string} was found")]
async fn the_failure_reports_none_found(world: &mut TinmanWorld, role: String, name: String) {
    let message = failure_message(world);
    assert!(
        message.contains(&role) && message.contains(&name),
        "the failure does not name the {role} {name:?} it looked for: {message}"
    );
}

#[then(expr = "the failure reports {int} matches for the {string} named {string}")]
async fn the_failure_reports_matches(
    world: &mut TinmanWorld,
    count: usize,
    role: String,
    name: String,
) {
    let message = failure_message(world);
    assert!(
        message.contains(&count.to_string()) && message.contains(&role) && message.contains(&name),
        "the failure does not report {count} matches for the {role} {name:?}: {message}"
    );
}

#[then(expr = "the failure reports the selection did not reach the {string} named {string}")]
async fn the_failure_reports_selection_did_not_reach(
    world: &mut TinmanWorld,
    role: String,
    name: String,
) {
    let message = failure_message(world);
    assert!(
        message.contains(&role) && message.contains(&name),
        "the failure does not name the {role} {name:?} the selection did not reach: {message}"
    );
}

#[given("the fixture program ignores directional keys")]
async fn the_fixture_ignores_directional_keys(world: &mut TinmanWorld) {
    launch_driver_session(world, support::fixture_ignoring_directional_keys_source()).await;
}

#[given(expr = "the fixture program shows two buttons named {string}")]
async fn the_fixture_shows_two_buttons(world: &mut TinmanWorld, name: String) {
    launch_driver_session(world, &support::fixture_with_two_buttons_source(&name)).await;
}

#[then(expr = "the driver replies to request {int} with a failed result")]
async fn the_driver_replies_to_request_with_a_failed_result(world: &mut TinmanWorld, id: u64) {
    let reply = reply(world);
    assert_eq!(reply["jsonrpc"], serde_json::json!("2.0"), "reply framing");
    assert_eq!(reply["id"], serde_json::json!(id), "replied request id");
    assert_eq!(
        result(reply)["ok"],
        serde_json::Value::Bool(false),
        "the reply is not a failed result: {reply}"
    );
}

#[then("the failure names the program it could not start")]
async fn the_failure_names_the_program(world: &mut TinmanWorld) {
    let command = world
        .launched_command
        .clone()
        .expect("a launch call named a command");
    let reply = reply(world);
    let failure = result(reply)["failure"]
        .as_str()
        .unwrap_or_else(|| panic!("the failed reply carries no failure message: {reply}"));
    assert!(
        failure.contains(&command),
        "the failure does not name the program {command:?}: {failure}"
    );
}

#[then(expr = "the driver replies to request {int} with the error code {int}")]
async fn the_driver_replies_with_the_error_code(world: &mut TinmanWorld, id: u64, code: i64) {
    let reply = reply(world);
    assert_eq!(reply["jsonrpc"], serde_json::json!("2.0"), "reply framing");
    assert_eq!(reply["id"], serde_json::json!(id), "replied request id");
    let error = reply
        .get("error")
        .unwrap_or_else(|| panic!("the reply carries no error object: {reply}"));
    assert_eq!(
        error["code"],
        serde_json::json!(code),
        "reserved error code"
    );
}

#[then(expr = "the error data names the method {string}")]
async fn the_error_data_names_the_method(world: &mut TinmanWorld, method: String) {
    let reply = reply(world);
    let error = reply
        .get("error")
        .unwrap_or_else(|| panic!("the reply carries no error object: {reply}"));
    let data = error
        .get("data")
        .unwrap_or_else(|| panic!("the error object carries no data: {reply}"));
    let rendered = data
        .as_str()
        .map(str::to_string)
        .unwrap_or(data.to_string());
    assert!(
        rendered.contains(&method),
        "the error data {rendered:?} does not name the method {method:?}"
    );
}

#[then(expr = "the error data names the missing parameter {string}")]
async fn the_error_data_names_the_missing_parameter(world: &mut TinmanWorld, parameter: String) {
    let reply = reply(world);
    let error = reply
        .get("error")
        .unwrap_or_else(|| panic!("the reply carries no error object: {reply}"));
    let data = error
        .get("data")
        .unwrap_or_else(|| panic!("the error object carries no data: {reply}"));
    let rendered = data
        .as_str()
        .map(str::to_string)
        .unwrap_or(data.to_string());
    assert!(
        rendered.contains(&parameter),
        "the error data {rendered:?} does not name the missing parameter {parameter:?}"
    );
}

#[then(expr = "the error data names the rejected scope {string}")]
async fn the_error_data_names_the_rejected_scope(world: &mut TinmanWorld, scope: String) {
    let reply = reply(world);
    let error = reply
        .get("error")
        .unwrap_or_else(|| panic!("the reply carries no error object: {reply}"));
    let data = error
        .get("data")
        .unwrap_or_else(|| panic!("the error object carries no data: {reply}"));
    let rendered = data
        .as_str()
        .map(str::to_string)
        .unwrap_or(data.to_string());
    assert!(
        rendered.contains(&scope),
        "the error data {rendered:?} does not name the rejected scope {scope:?}"
    );
}

#[when("the test runner closes the driver's stdin")]
async fn the_test_runner_closes_the_drivers_stdin(world: &mut TinmanWorld) {
    driver(world).close_stdin();
}

#[then("the driver process exits with a success status")]
async fn the_driver_process_exits_with_a_success_status(world: &mut TinmanWorld) {
    let status = driver(world).wait_for_exit();
    assert!(
        status.success(),
        "the driver exited with {status} rather than successfully"
    );
}

#[then("the driver leaves no session sandbox directory standing")]
async fn the_driver_leaves_no_session_sandbox_directory(world: &mut TinmanWorld) {
    let pid = driver(world).pid();
    let standing = support::standing_session_dirs(pid);
    assert!(
        standing.is_empty(),
        "the driver left {} session sandbox directory/directories standing: {standing:?}",
        standing.len()
    );
}

#[then(expr = "the driver replies with a result whose {string} is false")]
async fn the_driver_replies_with_a_result_field_false(world: &mut TinmanWorld, field: String) {
    let reply = reply(world);
    assert_eq!(
        result(reply)[&field],
        serde_json::Value::Bool(false),
        "the result's {field:?} is not false: {reply}"
    );
}

#[then("the reply carries no error object")]
async fn the_reply_carries_no_error_object(world: &mut TinmanWorld) {
    let reply = reply(world);
    assert!(
        reply.get("error").is_none(),
        "the reply carries an error object: {reply}"
    );
}

#[then("the driver replies with a failed result")]
async fn the_driver_replies_with_a_failed_result(world: &mut TinmanWorld) {
    let reply = reply(world);
    assert_eq!(
        result(reply)["ok"],
        serde_json::Value::Bool(false),
        "the reply is not a failure: {reply}"
    );
}

#[then("the driver answers a later screen request for the same session")]
async fn the_driver_answers_a_later_screen_request(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "screen", serde_json::json!({"session": session})));
    assert_eq!(
        result(&reply)["ok"],
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
// semantic capture: collecting the items of a scrolling pane
// ---------------------------------------------------------------------------

/// Replace the session's program with `program`, keeping the driver already
/// running and closing the session it holds, so the sandbox home that session
/// owns is reclaimed rather than left standing.
async fn relaunch_driver_session(world: &mut TinmanWorld, program: &str) {
    if let Some(name) = world.session_id.take() {
        let id = next_id(world);
        let reply = driver(world).request(rpc(id, "close", serde_json::json!({"session": name})));
        assert_eq!(
            result(&reply)["ok"],
            serde_json::Value::Bool(true),
            "the session the fixture replaces did not close: {reply}"
        );
    }
    let id = next_id(world);
    let reply = driver(world).request(rpc(id, "launch", serde_json::json!({"command": program})));
    assert_eq!(
        result(&reply)["ok"],
        serde_json::Value::Bool(true),
        "the fixture program did not launch: {reply}"
    );
    let identifier = result(&reply)["session"]
        .as_str()
        .unwrap_or_else(|| panic!("the launch reply carries no session identifier: {reply}"))
        .to_string();
    world.session_id = Some(identifier);
    world.reply = Some(reply);
}

/// The log pane an earlier step of the scenario established: its title, how
/// many messages it holds, and how many lines its window shows.
fn log_shape(world: &TinmanWorld) -> (String, usize, usize) {
    (
        world
            .log_title
            .clone()
            .expect("an earlier step showed a log"),
        world.log_messages.expect("the log holds messages"),
        world.log_window.expect("the log has a window"),
    )
}

#[given(
    expr = "the fixture program shows a {string} holding {int} messages in a {int} line window"
)]
async fn the_fixture_shows_a_log(
    world: &mut TinmanWorld,
    title: String,
    count: usize,
    window: usize,
) {
    let program = support::log_fixture_program(&title, count, window);
    world.log_title = Some(title);
    world.log_messages = Some(count);
    world.log_window = Some(window);
    relaunch_driver_session(world, &program).await;
}

#[given(expr = "the fixture program repeats its last {int} messages at each scroll position")]
async fn the_fixture_repeats_its_last_messages(world: &mut TinmanWorld, repeat: usize) {
    let (title, count, window) = log_shape(world);
    let program = support::repeating_log_fixture_program(&title, count, window, repeat);
    relaunch_driver_session(world, &program).await;
}

#[given(expr = "the fixture program scrolls its {string} without ever reaching an end")]
async fn the_fixture_scrolls_without_end(world: &mut TinmanWorld, title: String) {
    let (_, _, window) = log_shape(world);
    let program = support::endless_log_fixture_program(&title, window);
    relaunch_driver_session(world, &program).await;
}

/// Ask the driver to capture the items a locator names, binding the items the
/// reply carries to `name`, which is what the call's `as` argument binds them
/// to for a client. A call that captured nothing binds nothing, so a failed
/// capture is reported by the step that reads the reply.
async fn capture_items(world: &mut TinmanWorld, role: &str, within: &str, scope: &str, name: &str) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "capture",
        serde_json::json!({
            "session": session,
            "role": role,
            "within": within,
            "scope": scope,
            "as": name,
        }),
    ));
    if let Some(items) = reply
        .get("result")
        .and_then(|result| result.get("items"))
        .and_then(serde_json::Value::as_array)
    {
        let items = items
            .iter()
            .map(|item| {
                item.as_str()
                    .unwrap_or_else(|| panic!("the captured item is not a string: {item}"))
                    .to_string()
            })
            .collect();
        world.captures.insert(name.to_string(), items);
    }
    world.reply = Some(reply);
}

#[when(expr = "the test runner captures every {string} in the {string} as {string}")]
async fn the_runner_captures_every(
    world: &mut TinmanWorld,
    role: String,
    within: String,
    name: String,
) {
    capture_items(world, &role, &within, "all", &name).await;
}

#[when(expr = "the test runner captures the visible {string} items in the {string} as {string}")]
async fn the_runner_captures_the_visible(
    world: &mut TinmanWorld,
    role: String,
    within: String,
    name: String,
) {
    capture_items(world, &role, &within, "visible", &name).await;
}

/// The items bound to `name`, reported with the driver's own reply when the
/// capture bound nothing, so a call that failed says what it answered.
fn captured<'a>(world: &'a TinmanWorld, name: &str) -> &'a [String] {
    world
        .captures
        .get(name)
        .map(Vec::as_slice)
        .unwrap_or_else(|| {
            panic!(
                "no capture is bound to {name:?}; the driver replied {}",
                world
                    .reply
                    .as_ref()
                    .map_or_else(|| "nothing".to_string(), ToString::to_string)
            )
        })
}

#[then(expr = "the capture named {string} holds {int} items")]
async fn the_capture_holds_items(world: &mut TinmanWorld, name: String, count: usize) {
    let items = captured(world, &name);
    assert_eq!(items.len(), count, "the capture {name:?} holds {items:#?}");
}

#[then(expr = "the first item of the capture named {string} is {string}")]
async fn the_first_item_of_the_capture_is(world: &mut TinmanWorld, name: String, expected: String) {
    let items = captured(world, &name);
    let first = items
        .first()
        .unwrap_or_else(|| panic!("the capture {name:?} is empty"));
    assert_eq!(first, &expected, "the capture {name:?} holds {items:#?}");
}

#[then(expr = "the last item of the capture named {string} is {string}")]
async fn the_last_item_of_the_capture_is(world: &mut TinmanWorld, name: String, expected: String) {
    let items = captured(world, &name);
    let last = items
        .last()
        .unwrap_or_else(|| panic!("the capture {name:?} is empty"));
    assert_eq!(last, &expected, "the capture {name:?} holds {items:#?}");
}

#[then("the failure reports the capture reached its scroll limit")]
async fn the_failure_reports_the_scroll_limit(world: &mut TinmanWorld) {
    let reply = reply(world);
    let failure = result(reply)["failure"]
        .as_str()
        .unwrap_or_else(|| panic!("the failed reply reports no failure: {reply}"));
    assert!(
        failure.to_lowercase().contains("scroll limit"),
        "the failure does not report the capture reached its scroll limit: {failure:?}"
    );
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

#[given(expr = "a plan sandbox section mounting {string} at {string} with mode {string}")]
async fn a_sandbox_section_mounting_with_mode(
    world: &mut TinmanWorld,
    source: String,
    target: String,
    mode: String,
) {
    world.plan_sources.push(format!(
        "mounts:\n  - source: {source}\n    target: {target}\n    mode: {mode}\n"
    ));
}

#[given(expr = "a plan sandbox section that names no environment variables")]
async fn a_sandbox_section_naming_no_environment(world: &mut TinmanWorld) {
    world
        .plan_sources
        .push("backend: auto\nhome: empty\nnetwork: deny\n".to_string());
}

#[given(expr = "a plan sandbox section that injects {string} from the host")]
async fn a_sandbox_section_injecting_from_the_host(world: &mut TinmanWorld, name: String) {
    world.plan_sources.push(format!(
        "backend: auto\nhome: empty\nnetwork: deny\nenv:\n  {name}:\n    from: host\n"
    ));
}

#[given(expr = "a plan sandbox section whose path lists {string}")]
async fn a_sandbox_section_whose_path_lists(world: &mut TinmanWorld, entry: String) {
    world.plan_sources.push(format!(
        "backend: auto\nhome: empty\nnetwork: deny\npath:\n  - {entry}\n"
    ));
}

#[given(expr = "the operator's environment defines {string} as {string}")]
async fn the_operator_environment_defines(world: &mut TinmanWorld, name: String, value: String) {
    // Set it in the test process's own environment, which Bubblewrap inherits
    // and must clear unless the plan names it, so a leak is a real leak.
    unsafe {
        std::env::set_var(&name, &value);
    }
    world.secret_name = Some(name);
    world.secret_value = Some(value);
}

#[given(expr = "the fixture directory {string} contains the file {string}")]
async fn the_fixture_directory_contains_the_file(
    world: &mut TinmanWorld,
    directory: String,
    name: String,
) {
    let dir = working_dir(world).join(directory.trim_start_matches("./"));
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
        panic!(
            "the fixture directory {} was not created: {e}",
            dir.display()
        )
    });
    let path = dir.join(&name);
    let contents = "the original fixture contents\n";
    std::fs::write(&path, contents)
        .unwrap_or_else(|e| panic!("the fixture file {} was not written: {e}", path.display()));
    world.fixture_file = Some((path, contents.to_string()));
}

/// The sandbox specification the scenario's given section describes, parsed by
/// the production parser so a section production cannot read fails here.
fn sandbox_section(world: &TinmanWorld) -> tinman::sandbox::SandboxSpec {
    let source = world
        .plan_sources
        .first()
        .expect("a plan sandbox section was given");
    tinman::plan::parse_sandbox(source)
        .unwrap_or_else(|e| panic!("the sandbox section did not parse: {e}\n{source}"))
}

/// A process that reports what the sandbox granted it: the value of the named
/// environment variable, and the PATH it was given.
fn reporting_process(name: &str) -> CommandSpec {
    let script =
        format!("printf 'ENVVAL=[%s]\\n' \"${{{name}}}\"; printf 'PATHVAL=[%s]\\n' \"$PATH\"");
    CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), script],
    }
}

/// The bracketed value the reporting process printed for `field`.
fn reported_field(world: &TinmanWorld, field: &str) -> String {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let contents = screen.contents();
    contents
        .lines()
        .find_map(|line| line.trim_end().strip_prefix(&format!("{field}=[")))
        .and_then(|rest| rest.strip_suffix(']'))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("the launched process reported no {field}; screen:\n{contents}"))
}

#[when(expr = "the fixture terminal program writes {string} into {string}")]
async fn the_fixture_writes_into(world: &mut TinmanWorld, text: String, target: String) {
    let spec = sandbox_section(world);
    let command = CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec![
            "-c".to_string(),
            // Only report the write when the redirect actually succeeded, so a
            // sandbox that never mounted the target cannot pass the scenario by
            // leaving the source untouched for the wrong reason.
            format!("printf '{text}' > {target} && printf 'WROTE\\n'"),
        ],
    };
    let prepared = BubblewrapBackend::new()
        .prepare(&spec, &command)
        .unwrap_or_else(|e| panic!("the Bubblewrap backend did not prepare the process: {e}"));
    let screen = tinman::pty::capture(&prepared).expect("the process is captured");
    assert!(
        screen.contains("WROTE"),
        "the sandboxed process did not write into {target}; screen:\n{}",
        screen.contents()
    );
    world.screen = Some(screen);
}

#[then(expr = "the file {string} is unchanged")]
async fn the_file_is_unchanged(world: &mut TinmanWorld, _path: String) {
    let (path, original) = world
        .fixture_file
        .clone()
        .expect("a fixture file was staged");
    let now = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the fixture file {} unreadable: {e}", path.display()));
    assert_eq!(
        now,
        original,
        "the fixture file {} was changed through the copy mount",
        path.display()
    );
}

#[then(expr = "the launched process reports {string} is unset")]
async fn the_launched_process_reports_unset(world: &mut TinmanWorld, _name: String) {
    let value = reported_field(world, "ENVVAL");
    assert!(
        value.is_empty(),
        "the sandboxed process saw the value {value:?}, so the variable reached it"
    );
}

#[then(expr = "the launched process reports {string} is {string}")]
async fn the_launched_process_reports_value(
    world: &mut TinmanWorld,
    _name: String,
    expected: String,
) {
    let value = reported_field(world, "ENVVAL");
    assert_eq!(value, expected, "the value the sandboxed process saw");
}

#[then(expr = "the launched process reports its PATH is {string}")]
async fn the_launched_process_reports_its_path(world: &mut TinmanWorld, expected: String) {
    let value = reported_field(world, "PATHVAL");
    assert_eq!(value, expected, "the PATH the sandboxed process saw");
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
// replay: a written plan driven against a real terminal program
// ---------------------------------------------------------------------------

/// A plan driving the fixture terminal program, expecting the status bar it
/// draws, and any further text the scenario names as the final step.
fn fixture_plan_source(final_expect: Option<&str>) -> String {
    // A flow step runs inside a sandbox binding the system directories and the
    // workspace, so a program written to a temporary directory is not there to
    // run. The step's shell reads the program on its command line instead, as a
    // driver session does.
    let program = support::fixture_terminal_source();
    let mut source = format!("tui: {program:?}\nsteps:\n  - expect: READY\n");
    if let Some(text) = final_expect {
        source.push_str(&format!("  - expect: {text}\n"));
    }
    source
}

/// Give the scenario a plan driving the fixture program, parsed through the
/// real plan seam and kept in its written form for the commands that take a
/// plan file.
fn give_fixture_plan(world: &mut TinmanWorld, final_expect: Option<&str>) {
    let source = fixture_plan_source(final_expect);
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the fixture plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
}

#[given("a harness plan driving the fixture terminal program")]
async fn a_plan_driving_the_fixture(world: &mut TinmanWorld) {
    give_fixture_plan(world, None);
}

#[given(
    expr = "a harness plan driving the fixture terminal program whose final step expects the text {string}"
)]
async fn a_plan_driving_the_fixture_expecting(world: &mut TinmanWorld, text: String) {
    give_fixture_plan(world, Some(&text));
}

#[when("the operator tests that plan")]
async fn the_operator_tests_that_plan(world: &mut TinmanWorld) {
    let source = world
        .replay_plan_source
        .clone()
        .expect("a harness plan was given");
    let dir = working_dir(world);
    let path = dir.join("plan.yaml");
    std::fs::write(&path, &source)
        .unwrap_or_else(|e| panic!("the plan {} was not written: {e}", path.display()));
    run_tinman_command(world, &["test", &path.to_string_lossy()]);
}

/// Run the real `tinman` binary in the scenario's working directory, keeping
/// what it wrote and the status it left.
fn run_tinman_command(world: &mut TinmanWorld, args: &[&str]) {
    let dir = working_dir(world);
    let env = configured_env(world);
    let outcome = support::run_tinman(&dir, args, &env, None)
        .unwrap_or_else(|e| panic!("the tinman binary did not run: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
}

/// What the last run of the real binary wrote.
fn run_output(world: &TinmanWorld) -> &str {
    world
        .run_stdout
        .as_deref()
        .expect("the tinman binary was run")
}

#[when("the operator inspects the fixture terminal program")]
async fn the_operator_inspects_the_fixture(world: &mut TinmanWorld) {
    let program = support::fixture_terminal_program();
    run_tinman_command(world, &["inspect", &program.to_string_lossy()]);
}

#[when("the operator inspects the fixture terminal program as JSON")]
async fn the_operator_inspects_the_fixture_as_json(world: &mut TinmanWorld) {
    let program = support::fixture_terminal_program();
    run_tinman_command(world, &["inspect", &program.to_string_lossy(), "--json"]);
}

#[when(expr = "the operator inspects the command {string}")]
async fn the_operator_inspects_the_command(world: &mut TinmanWorld, command: String) {
    run_tinman_command(world, &["inspect", &command]);
}

#[then(expr = "the inspect output lists a {string} named {string}")]
async fn the_inspect_output_lists(world: &mut TinmanWorld, role: String, name: String) {
    let output = run_output(world);
    let listed = output
        .lines()
        .any(|line| line.contains(&role) && line.contains(&name));
    assert!(
        listed,
        "no line of the inspect output lists a {role:?} named {name:?}:\n{output}"
    );
}

#[then(expr = "the inspect output conforms to the {string} schema in {string}")]
async fn the_inspect_output_conforms(world: &mut TinmanWorld, schema: String, path: String) {
    let output = run_output(world);
    let instance: serde_json::Value = serde_json::from_str(output)
        .unwrap_or_else(|e| panic!("the inspect output is not JSON: {e}\n{output}"));
    let bad = support::schema_counterexamples(&path, &instance);
    assert!(
        bad.is_empty(),
        "the inspect output violates the {schema:?} schema: {bad:?}"
    );
}

#[then(expr = "the inspect output reports {string}")]
async fn the_inspect_output_reports(world: &mut TinmanWorld, expected: String) {
    let output = run_output(world);
    assert!(
        output.contains(&expected),
        "the inspect output does not report {expected:?}:\n{output}"
    );
}

#[then(expr = "the output reports the step expecting {string}")]
async fn the_output_reports_the_step_expecting(world: &mut TinmanWorld, text: String) {
    let output = run_output(world);
    let reported = output
        .lines()
        .any(|line| line.contains("expect") && line.contains(&text));
    assert!(
        reported,
        "no line of the output names the step expecting {text:?}:\n{output}"
    );
}

#[then(expr = "the output contains the text {string}")]
async fn the_output_contains_the_text(world: &mut TinmanWorld, text: String) {
    let output = run_output(world);
    assert!(
        output.contains(&text),
        "the output does not contain {text:?}:\n{output}"
    );
}

/// Replay the plan the scenario was given, keeping what the replay produced.
fn replay_current_plan(world: &mut TinmanWorld) {
    let plan = world.replay_plan.clone().expect("a harness plan was given");
    let workspace = working_dir(world);
    // Terminal size is a property of the run, so the caller supplies it and an
    // unstated width leaves the run on the operator's own terminal.
    let columns = world.replay_columns;
    match tinman::flow::execute(&plan, &workspace, columns) {
        Ok(outcome) => world.flow_outcome = Some(outcome),
        Err(e) => world.flow_error = Some(e),
    }
}

#[when("that plan is replayed")]
async fn that_plan_is_replayed(world: &mut TinmanWorld) {
    replay_current_plan(world);
}

#[when(expr = "that plan is replayed at {int} columns")]
async fn that_plan_is_replayed_at_columns(world: &mut TinmanWorld, cols: u16) {
    world.replay_columns = Some(cols);
    replay_current_plan(world);
}

#[then("the replay passes")]
async fn the_replay_passes(world: &mut TinmanWorld) {
    assert!(
        world.flow_error.is_none(),
        "the replay failed: {}",
        world.flow_error.as_deref().unwrap_or_default()
    );
    let outcome = world
        .flow_outcome
        .as_ref()
        .expect("the replay produced no outcome");
    // The driven program reports the width of the terminal it was given, so a
    // replay at a named width is proved by the program's own report rather than
    // by the harness restating what it asked for.
    if let Some(cols) = world.replay_columns {
        let marker = format!("WIDTH:{cols}");
        let seen: Vec<&str> = outcome.steps.iter().map(|s| s.output.as_str()).collect();
        let seen = seen.join("\n");
        assert!(
            seen.contains(&marker),
            "the driven program reported no {marker:?}, so the replay did not run at {cols} columns:\n{seen}"
        );
    }
}

#[given(expr = "a harness plan captured from the fixture terminal program at {int} columns")]
async fn a_plan_captured_at_columns(world: &mut TinmanWorld, _cols: u16) {
    give_fixture_plan(world, None);
}

#[given(
    expr = "a harness plan whose step expects the status bar to contain {string}, captured at {int} columns"
)]
async fn a_plan_expecting_the_status_bar(world: &mut TinmanWorld, text: String, _cols: u16) {
    // A flow step runs sandboxed, so the step's shell reads the program on its
    // command line rather than from a temporary directory the sandbox does not
    // bind.
    let program = support::fixture_terminal_source();
    // `within` is the scoping key the harness-plan scantling defines for an
    // expectation, so the plan is written in the durable form rather than one
    // the suite invents.
    let source = format!(
        "tui: {program:?}\nsteps:\n  - expect:\n      text: {text}\n      within: status\n"
    );
    let plan = tinman::plan::parse(&source)
        .unwrap_or_else(|e| panic!("the fixture plan did not parse: {e}"));
    let written = serde_yaml::to_string(&plan).expect("the parsed plan serializes");
    assert!(
        written.contains("within: status"),
        "the parsed plan dropped the status-bar scope of its expectation, so the step is not bound to the status bar:\n{written}"
    );
    world.replay_plan = Some(plan);
    world.replay_plan_source = Some(source);
}

#[then(expr = "the replay fails and reports the step expecting {string}")]
async fn the_replay_fails_reporting_the_step(world: &mut TinmanWorld, text: String) {
    let reported = world
        .flow_error
        .as_deref()
        .expect("the replay was expected to fail, and it passed");
    assert!(
        reported.contains("expect") && reported.contains(&text),
        "the failure does not name the step expecting {text:?}:\n{reported}"
    );
}

#[then(expr = "the failure report contains the text {string}")]
async fn the_failure_report_contains(world: &mut TinmanWorld, text: String) {
    let reported = world
        .flow_error
        .as_deref()
        .expect("the replay was expected to fail, and it passed");
    assert!(
        reported.contains(&text),
        "the failure report does not contain {text:?}:\n{reported}"
    );
}

// ---------------------------------------------------------------------------
// record command: capturing a live session to an editable file
// ---------------------------------------------------------------------------

/// Run `tinman record` against `command` on a real terminal, pressing `keys`
/// while it runs, and keep what it wrote and the status it left.
fn run_record(world: &mut TinmanWorld, command: &str, keys: &[&str]) {
    run_record_with_options(world, command, "", keys);
}

/// Run a recording as `run_record` does, with `options` passed to `tinman
/// record` ahead of the recorded command, as the operator types them.
fn run_record_with_options(world: &mut TinmanWorld, command: &str, options: &str, keys: &[&str]) {
    let dir = working_dir(world);
    let env = configured_env(world);
    let mut owned = vec!["record".to_string()];
    owned.extend(options.split_whitespace().map(str::to_string));
    owned.extend(command.split_whitespace().map(str::to_string));
    let args: Vec<&str> = owned.iter().map(String::as_str).collect();
    let mut session = support::TerminalSession::start(&dir, &args, &env)
        .unwrap_or_else(|e| panic!("the tinman binary did not start: {e}"));
    for key in keys {
        session.press(key);
    }
    session.end_input();
    let outcome = session.finish(std::time::Duration::from_secs(30));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
}

/// The YAML text a record run wrote, read from the working directory.
fn written_plan_text(world: &mut TinmanWorld, name: &str) -> String {
    let path = working_dir(world).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("no plan at {}: {e}", path.display()))
}

/// The plan a record run wrote, parsed by the production plan parser, so a file
/// that is not a plan fails here rather than reading as one.
fn written_plan(world: &mut TinmanWorld, name: &str) -> tinman::plan::Plan {
    let text = written_plan_text(world, name);
    tinman::plan::parse(&text)
        .unwrap_or_else(|e| panic!("the written plan did not parse: {e}\nit reads:\n{text}"))
}

/// Whether the written plan records a press of `key` anywhere in its flow.
fn records_press(value: &serde_yaml::Value, key: &str) -> bool {
    match value {
        serde_yaml::Value::Mapping(map) => map.iter().any(|(name, nested)| {
            (name.as_str() == Some("press") && nested.as_str() == Some(key))
                || records_press(nested, key)
        }),
        serde_yaml::Value::Sequence(items) => items.iter().any(|item| records_press(item, key)),
        _ => false,
    }
}

#[given(expr = "the file {string} already exists")]
async fn the_file_already_exists(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    std::fs::write(&path, "already here\n")
        .unwrap_or_else(|e| panic!("the file {} was not created: {e}", path.display()));
}

#[when(expr = "the operator records the command {string} and presses {string}")]
async fn the_operator_records_and_presses(world: &mut TinmanWorld, command: String, key: String) {
    run_record(world, &command, &[&key]);
}

#[when(expr = "the operator records the command {string}")]
async fn the_operator_records_the_command(world: &mut TinmanWorld, command: String) {
    run_record(world, &command, &[]);
}

#[when("the operator records the fixture terminal program")]
async fn the_operator_records_the_fixture(world: &mut TinmanWorld) {
    // A recorded plan replays inside a sandbox binding the workspace, so the
    // program is staged in the working directory the recording runs in and
    // named relatively, as an operator's own project program is.
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    run_record(world, &program, &["q"]);
}

#[given("a fixture terminal program whose pane titles change between draws")]
async fn a_fixture_whose_titles_change(world: &mut TinmanWorld) {
    world.record_program = Some(
        support::unstable_fixture_terminal_program()
            .to_string_lossy()
            .into_owned(),
    );
}

#[when("the operator records that program")]
async fn the_operator_records_that_program(world: &mut TinmanWorld) {
    let program = world
        .record_program
        .clone()
        .expect("a fixture terminal program was given");
    run_record(world, &program, &["q"]);
}

#[when(expr = "the operator records the command {string} with {string}")]
async fn the_operator_records_the_command_with(
    world: &mut TinmanWorld,
    command: String,
    options: String,
) {
    run_record_with_options(world, &command, &options, &[]);
}

#[then(expr = "the written plan names the command {string}")]
async fn the_plan_names_the_command(world: &mut TinmanWorld, expected: String) {
    let plan = written_plan(world, "tinman.yaml");
    let named: Vec<String> = plan
        .flow
        .iter()
        .map(|step| match step {
            tinman::plan::FlowStep::Run(run) => run.command.clone(),
            tinman::plan::FlowStep::Tui(tui) => tui.command.clone(),
        })
        .collect();
    assert!(
        named.iter().any(|command| command == &expected),
        "the written plan names {named:?}, so it does not name the command {expected:?}"
    );
}

#[then(expr = "the written plan records a key press {string}")]
async fn the_plan_records_a_key_press(world: &mut TinmanWorld, key: String) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    let written: serde_yaml::Value =
        serde_yaml::from_str(&text).expect("the written plan parses as YAML");
    assert!(
        records_press(&written, &key),
        "the written plan records no key press {key:?}; it reads:\n{text}"
    );
}

#[then(expr = "the written plan carries an expectation on the text {string}")]
async fn the_plan_carries_an_expectation(world: &mut TinmanWorld, expected: String) {
    let text = written_plan_text(world, "tinman.yaml");
    let plan = written_plan(world, "tinman.yaml");
    let expectations: Vec<String> = plan
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.clone(),
            tinman::plan::FlowStep::Run(_) => Vec::new(),
        })
        .filter_map(|action| match action {
            tinman::plan::Action::Expect(expectation) => Some(expectation.text),
            _ => None,
        })
        .collect();
    assert!(
        expectations.iter().any(|found| found == &expected),
        "the written plan carries the expectations {expectations:?}, \
         so none is on the text {expected:?}; it reads:\n{text}"
    );
}

#[then("the recorded snapshots show the secret value is absent")]
async fn the_recorded_snapshots_show_the_secret_absent(world: &mut TinmanWorld) {
    let value = world
        .secret_value
        .clone()
        .expect("a secret value was set")
        .clone();
    let text = written_plan_text(world, "tinman.yaml");
    let plan = written_plan(world, "tinman.yaml");
    let snapshots: Vec<String> = plan
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.clone(),
            tinman::plan::FlowStep::Run(_) => Vec::new(),
        })
        .filter_map(|action| match action {
            tinman::plan::Action::Expect(expectation) => Some(expectation.text),
            _ => None,
        })
        .collect();
    // A recording that captured no screen at all would carry no secret either,
    // and would pass this step exactly as a clean recording does.
    assert!(
        !snapshots.is_empty(),
        "the recording captured no screen, so this scenario would assert nothing; it reads:\n{text}"
    );
    let leaked: Vec<&String> = snapshots
        .iter()
        .filter(|snapshot| snapshot.contains(&value))
        .collect();
    assert!(
        leaked.is_empty(),
        "the secret value {value:?} leaked into the recorded snapshots {leaked:?}"
    );
}

#[then(expr = "the plan is written to {string}")]
async fn the_plan_is_written_to(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    assert!(path.exists(), "no plan at {}", path.display());
}

#[then("no plan is written")]
async fn no_plan_is_written(world: &mut TinmanWorld) {
    let dir = working_dir(world);
    let written: Vec<String> = std::fs::read_dir(&dir)
        .expect("the working directory is readable")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".yaml"))
        .collect();
    assert!(written.is_empty(), "recording wrote {written:?}");
}

#[then("recording fails and reports the file already exists")]
async fn recording_reports_the_file_already_exists(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a command was run");
    let output = run_output(world);
    assert_ne!(status, 0, "recording exited successfully:\n{output}");
    assert!(
        output.contains("already exists"),
        "recording does not report the file already exists:\n{output}"
    );
}

#[then("recording fails and reports the plan did not replay")]
async fn recording_reports_the_plan_did_not_replay(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a command was run");
    let output = run_output(world);
    assert_ne!(status, 0, "recording exited successfully:\n{output}");
    assert!(
        output.contains("replay"),
        "recording does not report the plan failed its own replay:\n{output}"
    );
}

#[then("replaying the written plan reproduces the recorded interaction")]
async fn replaying_the_written_plan(world: &mut TinmanWorld) {
    let dir = working_dir(world);
    let path = dir.join("tinman.yaml");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("no written plan at {}: {e}", path.display()));
    let plan = tinman::plan::parse(&source)
        .unwrap_or_else(|e| panic!("the written plan did not parse: {e}"));
    tinman::flow::execute(&plan, &dir, None)
        .unwrap_or_else(|e| panic!("the written plan did not replay: {e}"));
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

#[given(
    expr = "a virtual screen showing a bordered pane titled {string} holding the entries {string} and {string} separated by a blank line"
)]
async fn a_screen_with_a_pane_holding_separated_entries(
    world: &mut TinmanWorld,
    title: String,
    first: String,
    second: String,
) {
    let lines = vec![first, String::new(), second];
    world.screen = Some(support::bordered_pane_screen(&title, &lines, None));
    world.pane = Some((title, lines));
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

#[given(expr = "a virtual screen showing {string} at row {int} column {int}")]
async fn a_screen_showing_text_at(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    world.screen = Some(support::text_at_screen(&text, row, col));
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

/// The one region playing `role` and carrying `name`, resolved through the
/// locator seam so the step asserts role and name together rather than finding a
/// name and inspecting it afterwards.
fn only_region_with_role_named(world: &TinmanWorld, role: &str, name: &str) -> tinman::tom::Region {
    match tinman::tom::Locator::new(role, name).resolve(model(world)) {
        tinman::tom::Resolution::One(region) => region,
        tinman::tom::Resolution::NoMatch => {
            panic!("the model contains no region with the role {role:?} named {name:?}")
        }
        tinman::tom::Resolution::Ambiguous(count) => panic!(
            "the model contains {count} regions with the role {role:?} named {name:?}, not one"
        ),
    }
}

#[then(expr = "the model contains a region with the role {string} named {string}")]
async fn the_model_contains_a_region_with_role_named(
    world: &mut TinmanWorld,
    role: String,
    name: String,
) {
    world.found_region = Some(only_region_with_role_named(world, &role, &name));
}

#[then(expr = "the model contains a region with the role {string} labelled {string}")]
async fn the_model_contains_a_region_with_role_labelled(
    world: &mut TinmanWorld,
    role: String,
    label: String,
) {
    world.found_region = Some(only_region_with_role_named(world, &role, &label));
}

#[then(expr = "the second {string} of that region is named {string}")]
async fn the_second_child_with_role_is_named(
    world: &mut TinmanWorld,
    role: String,
    expected: String,
) {
    let region = world.found_region.as_ref().expect("a region was found");
    let matching: Vec<&tinman::tom::Region> = region
        .children
        .iter()
        .filter(|child| child.role() == role)
        .collect();
    let second = matching.get(1).unwrap_or_else(|| {
        panic!(
            "the region has {} children with the role {role:?}, so it has no second one",
            matching.len()
        )
    });
    assert_eq!(
        second.name.as_deref(),
        Some(expected.as_str()),
        "name of the second child with the role {role:?}"
    );
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

#[given(expr = "a virtual screen showing two bordered panes each listing an item named {string}")]
async fn a_screen_with_two_panes(world: &mut TinmanWorld, item: String) {
    world.screen = Some(support::two_bordered_panes_screen(&item));
}

/// Serve an engine reply built by editing the deterministic model of the
/// scenario's screen, so the reply is always a well-formed model of that screen
/// and only the field the scenario names differs from what the screen yields.
fn serve_engine_model(world: &mut TinmanWorld, edit: impl FnOnce(&mut serde_json::Value)) {
    let screen = world.screen.as_ref().expect("a virtual screen");
    let mut reply = to_json(&tinman::tom::build(screen));
    edit(&mut reply);
    let provider = support::LocalProvider::returning(&reply.to_string());
    use_provider(world, provider);
}

#[given(expr = "an engine that names the pane {string}")]
async fn an_engine_that_names_the_pane(world: &mut TinmanWorld, name: String) {
    let proposed = name.clone();
    serve_engine_model(world, move |reply| {
        reply["root"]["children"][0]["name"] = serde_json::json!(proposed);
    });
    world.proposed_name = Some(name);
}

#[given(expr = "an engine that names the first item {string}")]
async fn an_engine_that_names_the_first_item(world: &mut TinmanWorld, name: String) {
    let proposed = name.clone();
    serve_engine_model(world, move |reply| {
        reply["root"]["children"][0]["children"][0]["name"] = serde_json::json!(proposed);
    });
    world.proposed_name = Some(name);
}

#[given(expr = "an engine that names the second item {string}")]
async fn an_engine_that_names_the_second_item(world: &mut TinmanWorld, name: String) {
    let proposed = name.clone();
    serve_engine_model(world, move |reply| {
        reply["root"]["children"][1]["children"][0]["name"] = serde_json::json!(proposed);
    });
    world.proposed_name = Some(name);
}

#[when("the inferred locator is round-tripped against the deterministic model")]
async fn the_inferred_locator_is_round_tripped(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    let name = world
        .proposed_name
        .clone()
        .expect("an engine named a region");
    let screen = world.screen.as_ref().expect("a virtual screen");
    let inferred = tinman::tom::infer(screen, &settings);
    let deterministic = tinman::tom::build(screen);
    let proposed = inferred
        .find_named(&name)
        .unwrap_or_else(|| panic!("the inferred model carries no region named {name:?}"));
    let confirmed = tinman::tom::confirm(&deterministic, proposed.role(), &name);
    world.tom_binding = confirmed.as_ref().map(|c| c.binding.as_str().to_string());
    world.tom_resolution = Some(match &confirmed {
        Some(c) => c.locator.resolve(&deterministic),
        None => tinman::tom::Resolution::NoMatch,
    });
    world.tom = Some(deterministic);
}

#[given(expr = "a virtual screen showing an unbordered pane whose first line reads {string}")]
async fn a_screen_with_an_unbordered_pane(world: &mut TinmanWorld, first_line: String) {
    world.screen = Some(support::unbordered_pane_screen(&first_line));
}

#[given(expr = "an engine that names that region {string}")]
async fn an_engine_that_names_that_region(world: &mut TinmanWorld, name: String) {
    let proposed = name.clone();
    serve_engine_model(world, move |reply| {
        reply["root"]["children"][0]["name"] = serde_json::json!(proposed);
    });
    world.proposed_name = Some(name);
}

/// The region the engine named, taken from the inferred model, with the
/// deterministic model it must confirm against.
fn proposed_region(world: &mut TinmanWorld) -> (tinman::tom::Model, String, String) {
    let settings = resolved_settings(world);
    let name = world
        .proposed_name
        .clone()
        .expect("an engine named a region");
    let screen = world.screen.as_ref().expect("a virtual screen");
    let inferred = tinman::tom::infer(screen, &settings);
    let deterministic = tinman::tom::build(screen);
    let role = inferred
        .find_named(&name)
        .unwrap_or_else(|| panic!("the inferred model carries no region named {name:?}"))
        .role()
        .to_string();
    (deterministic, role, name)
}

#[when("an expectation on that item is recorded")]
async fn an_expectation_on_that_item_is_recorded(world: &mut TinmanWorld) {
    let (model, role, name) = proposed_region(world);
    world.record_error = tinman::record::record_expectation(&model, &role, &name).err();
}

#[then("recording fails and reports the expectation's locator did not bind")]
async fn recording_reports_the_locator_did_not_bind(world: &mut TinmanWorld) {
    let message = world
        .record_error
        .as_ref()
        .expect("recording the expectation failed");
    assert!(
        message.contains("did not bind"),
        "the recording failure reads {message:?}, so it does not report the locator did not bind"
    );
}

#[when("the plan is written")]
async fn the_plan_is_written(world: &mut TinmanWorld) {
    let (model, role, name) = proposed_region(world);
    let plan = tinman::record::plan_expecting(&model, &role, &name)
        .unwrap_or_else(|e| panic!("no plan was written: {e}"));
    world.written_plan_source =
        Some(serde_yaml::to_string(&plan).expect("the plan serializes to YAML"));
}

#[then(expr = "the plan records the locator's binding as {string}")]
async fn the_plan_records_the_binding(world: &mut TinmanWorld, expected: String) {
    let source = world
        .written_plan_source
        .as_ref()
        .expect("a plan was written");
    let written: serde_yaml::Value =
        serde_yaml::from_str(source).expect("the written plan parses as YAML");
    let binding = find_binding(&written).unwrap_or_else(|| {
        panic!("the written plan records no locator binding; it reads:\n{source}")
    });
    assert_eq!(binding, expected, "the binding the written plan records");
}

/// The first `binding` value the written plan carries, found wherever the plan
/// nests its locator, so the step asserts the written value rather than the
/// path the writer chose to nest it under.
fn find_binding(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, nested) in map {
                if key.as_str() == Some("binding")
                    && let Some(found) = nested.as_str()
                {
                    return Some(found.to_string());
                }
                if let Some(found) = find_binding(nested) {
                    return Some(found);
                }
            }
            None
        }
        serde_yaml::Value::Sequence(items) => items.iter().find_map(find_binding),
        _ => None,
    }
}

#[then(expr = "the locator binds to the region named {string}")]
async fn the_locator_binds_to_the_region(world: &mut TinmanWorld, name: String) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::One(region) => assert_eq!(
            region.name.as_deref(),
            Some(name.as_str()),
            "the region the locator bound to"
        ),
        other => panic!("the locator bound to no single region: {other:?}"),
    }
}

#[then("the locator is rejected as unbindable")]
async fn the_locator_is_rejected_as_unbindable(world: &mut TinmanWorld) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::NoMatch => {}
        other => panic!("the locator was not rejected: {other:?}"),
    }
}

#[then("the locator is scoped to the region containing that item")]
async fn the_locator_is_scoped_to_its_parent(world: &mut TinmanWorld) {
    match world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved")
    {
        tinman::tom::Resolution::One(_) => {}
        other => panic!(
            "the locator was not scoped to one region, so the ambiguity was left standing: {other:?}"
        ),
    }
}

#[then(expr = "the locator addresses the first {string} of the region named {string}")]
async fn the_locator_addresses_the_first(world: &mut TinmanWorld, role: String, pane: String) {
    let resolution = world
        .tom_resolution
        .as_ref()
        .expect("a locator was resolved");
    let tinman::tom::Resolution::One(bound) = resolution else {
        panic!("the locator fell back to no single region: {resolution:?}");
    };
    let model = world.tom.as_ref().expect("a deterministic model");
    let parent = model
        .find_named(&pane)
        .unwrap_or_else(|| panic!("the model contains no region named {pane:?}"));
    let first = parent
        .children
        .iter()
        .find(|child| child.role() == role)
        .unwrap_or_else(|| panic!("the region named {pane:?} has no {role:?} child"));
    assert_eq!(
        bound.rect, first.rect,
        "the locator addresses the first {role:?} of {pane:?}"
    );
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

#[given(expr = "an engine that answers {string}")]
async fn an_engine_that_answers(world: &mut TinmanWorld, reply: String) {
    let provider = support::LocalProvider::returning(&reply);
    use_provider(world, provider);
}

// ---------------------------------------------------------------------------
// the configured provider, reached for real on the @inference tier
// ---------------------------------------------------------------------------

/// The failure ceiling one real call to the configured provider carries. It
/// sits above the ceiling production applies to its own request, so a call that
/// ends on a ceiling ends on production's rather than the harness's.
const PROVIDER_ATTEMPT: std::time::Duration = std::time::Duration::from_secs(120);

/// The deadline an @inference step retries toward. A hosted provider that
/// answers nothing once is asked again; one that answers nothing until this
/// passes is reported as no answer, in the terms the scenario asserts.
const PROVIDER_DEADLINE: std::time::Duration = std::time::Duration::from_secs(240);

/// The settings the @inference tier runs against: the operator's real
/// credential and endpoint, read from the process environment and a dotenv
/// file, as the tier policy in `RIGGING.md` states. A tier scenario reaches the
/// configured provider rather than a local stand-in, so it resolves its
/// settings from the environment fitting out provisioned.
fn configured_settings() -> tinman::inference::Settings {
    let settings = tinman::inference::Settings::from_process();
    assert!(
        settings.api_key.is_some(),
        "no inference credential is configured, so this tier cannot reach a provider"
    );
    settings
}

#[when(expr = "the assistant request {string} is sent")]
async fn the_assistant_request_is_sent(world: &mut TinmanWorld, question: String) {
    let settings = configured_settings();
    world.provider_reply = support::within_deadline(
        "the configured inference provider",
        PROVIDER_ATTEMPT,
        PROVIDER_DEADLINE,
        move || tinman::inference::assistant_completion(&settings, &question),
    );
}

#[then("a reply is parsed from the provider's response")]
async fn a_reply_is_parsed_from_the_providers_response(world: &mut TinmanWorld) {
    assert!(
        world.provider_reply.is_some(),
        "no reply was parsed from the configured provider's response"
    );
}

#[then("the parsed reply carries non-empty content")]
async fn the_parsed_reply_carries_non_empty_content(world: &mut TinmanWorld) {
    let reply = world
        .provider_reply
        .as_deref()
        .expect("a reply was parsed from the provider's response");
    assert!(
        !reply.trim().is_empty(),
        "the parsed reply carries no content: {reply:?}"
    );
}

#[given("the fixture terminal program is captured through a PTY")]
async fn the_fixture_is_captured_through_a_pty(world: &mut TinmanWorld) {
    let program = support::fixture_terminal_program();
    let prepared = shell_process(&program.to_string_lossy());
    // The fixture waits for the operator, so it is captured live and its screen
    // read once it has drawn. A capture that waited for the program to exit
    // would wait for a program that never does.
    let mut capture =
        tinman::pty::capture_interactive(&prepared).expect("the fixture program is captured");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let screen = capture.screen();
        if screen.contains("READY") {
            world.screen = Some(screen);
            break;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the fixture program never drew READY; screen:\n{}",
                screen.contents()
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    capture.end_session();
}

#[when("the terminal object model is inferred by the configured engine")]
async fn the_model_is_inferred_by_the_configured_engine(world: &mut TinmanWorld) {
    let settings = configured_settings();
    let screen = world
        .screen
        .as_ref()
        .expect("a captured virtual screen")
        .clone();
    let contents = screen.contents();
    let started = std::time::Instant::now();
    let inferred = support::within_deadline(
        "the configured inference engine",
        PROVIDER_ATTEMPT,
        PROVIDER_DEADLINE,
        move || tinman::inference::tom_completion(&settings, &contents),
    );
    let elapsed = started.elapsed();
    let inferred = inferred.unwrap_or_else(|| {
        panic!(
            "the configured engine answered with no model, after {:.1}s",
            elapsed.as_secs_f64()
        )
    });
    let value: serde_json::Value = serde_json::from_str(&inferred)
        .unwrap_or_else(|e| panic!("the engine's model is not JSON: {e}\nit reads:\n{inferred}"));
    world.serialized = Some(value);
}

#[when("an acronym expansion is generated by the configured engine")]
async fn an_acronym_expansion_is_generated(world: &mut TinmanWorld) {
    let settings = configured_settings();
    let started = std::time::Instant::now();
    world.provider_reply = support::within_deadline(
        "the configured inference engine",
        PROVIDER_ATTEMPT,
        PROVIDER_DEADLINE,
        move || tinman::inference::expansion(&settings),
    );
    world.provider_elapsed = Some(started.elapsed());
}

#[then("a non-empty expansion is produced")]
async fn a_non_empty_expansion_is_produced(world: &mut TinmanWorld) {
    let elapsed = world
        .provider_elapsed
        .map(|e| format!("{:.1}s", e.as_secs_f64()))
        .unwrap_or_else(|| "an unrecorded time".to_string());
    let expansion = world
        .provider_reply
        .as_ref()
        .unwrap_or_else(|| panic!("the configured engine generated no expansion, after {elapsed}"));
    assert!(
        !expansion.trim().is_empty(),
        "the configured engine generated an empty expansion"
    );
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

#[given("the implementation sources and the verification support sources")]
async fn the_implementation_and_verification_support_sources(world: &mut TinmanWorld) {
    world.conformance_scopes = Some(vec!["src".to_string(), "tests".to_string()]);
}

#[when("the verification-conformance rule set is run")]
async fn conformance_rule_set_is_run(world: &mut TinmanWorld) {
    let scopes = world
        .conformance_scopes
        .as_ref()
        .expect("the source scopes were named");
    world.conformance_matches = Some(
        scopes
            .iter()
            .flat_map(|scope| support::run_conformance_scan(scope))
            .collect(),
    );
}

#[then("no rule in the set reports a match")]
async fn no_rule_reports_a_match(world: &mut TinmanWorld) {
    let matches = world
        .conformance_matches
        .as_ref()
        .expect("the conformance rule set ran");
    let reported: Vec<String> = matches.iter().map(|m| m.to_string()).collect();
    assert!(
        reported.is_empty(),
        "the rule set reported {} match(es):\n{}",
        reported.len(),
        reported.join("\n")
    );
}

#[then(
    "the rule set carries at least the plank-form, plank-presence, perturbation-quiescence and forbidden-doubles rules"
)]
async fn the_rule_set_carries_the_named_rules(_world: &mut TinmanWorld) {
    let carried = support::conformance_rule_ids();
    let missing: Vec<&str> = [
        "plank-form",
        "plank-presence",
        "perturbation-quiescence",
        "forbidden-doubles",
    ]
    .into_iter()
    .filter(|named| !carried.iter().any(|id| id == named))
    .collect();
    assert!(
        missing.is_empty(),
        "the rule set is missing {} named rule(s): {} (it carries {})",
        missing.len(),
        missing.join(", "),
        carried.join(", ")
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: the joins over planks, patterns and scenarios
// ---------------------------------------------------------------------------

#[given("the plank inventory and the step-usage pattern set")]
async fn the_plank_inventory_and_the_pattern_set(world: &mut TinmanWorld) {
    world.planks = Some(support::plank_inventory());
    world.step_patterns = Some(support::step_definition_patterns());
}

#[when("each plank string is matched against the pattern set")]
async fn each_plank_string_is_matched(world: &mut TinmanWorld) {
    let planks = world.planks.as_ref().expect("the plank inventory was read");
    let patterns = world
        .step_patterns
        .as_ref()
        .expect("the pattern set was read");
    world.stale_planks = Some(
        planks
            .iter()
            .filter(|plank| !patterns.iter().any(|p| p.pattern == plank.pattern))
            .map(|plank| {
                format!(
                    "{}:{} names {:?}, which no step definition declares",
                    plank.file, plank.line, plank.pattern
                )
            })
            .collect(),
    );
}

#[then("every plank string is a pattern the step definitions declare")]
async fn every_plank_names_a_current_pattern(world: &mut TinmanWorld) {
    let stale = world
        .stale_planks
        .as_ref()
        .expect("each plank string was matched");
    assert!(
        stale.is_empty(),
        "{} plank(s) name no current step-definition pattern:\n{}",
        stale.len(),
        stale.join("\n")
    );
}

#[then("the plank inventory is not empty")]
async fn the_plank_inventory_is_not_empty(world: &mut TinmanWorld) {
    let planks = world.planks.as_ref().expect("the plank inventory was read");
    assert!(
        !planks.is_empty(),
        "the plank inventory reports no plank, so the join asserted nothing"
    );
}

#[given("the provisional plank references and the scenarios in the specs")]
async fn the_provisional_planks_and_the_scenarios(world: &mut TinmanWorld) {
    world.provisional_planks = Some(support::provisional_plank_inventory());
    world.spec_scenarios = Some(support::spec_scenarios());
}

#[when(expr = "each reference is matched against the scenarios the specs still tag {string}")]
async fn each_provisional_reference_is_matched(world: &mut TinmanWorld, tag: String) {
    let inventory = world
        .provisional_planks
        .as_ref()
        .expect("the provisional plank inventory was read");
    let scenarios = world
        .spec_scenarios
        .as_ref()
        .expect("the scenarios were read");
    let awaiting: Vec<String> = scenarios
        .iter()
        .filter(|scenario| scenario.carries_tag(&tag))
        .map(|scenario| scenario.reference())
        .collect();
    world.spent_provisional_planks = Some(
        inventory
            .provisional
            .iter()
            .filter(|plank| !awaiting.contains(&plank.reference))
            .map(|plank| {
                format!(
                    "{}:{} names {:?}, which no scenario still tagged {tag} declares",
                    plank.file, plank.line, plank.reference
                )
            })
            .collect(),
    );
}

#[then(expr = "every provisional plank names a scenario still tagged {string}")]
async fn every_provisional_plank_awaits_review(world: &mut TinmanWorld, tag: String) {
    let spent = world
        .spent_provisional_planks
        .as_ref()
        .expect("each reference was matched");
    assert!(
        spent.is_empty(),
        "{} provisional plank(s) name no scenario still tagged {tag}, so a seam \
         Captain has already disposed of still reads as covered:\n{}",
        spent.len(),
        spent.join("\n")
    );
}

#[then("the inventory the provisional planks were read from is not empty")]
async fn the_provisional_inventory_is_not_empty(world: &mut TinmanWorld) {
    let inventory = world
        .provisional_planks
        .as_ref()
        .expect("the provisional plank inventory was read");
    assert!(
        inventory.annotations > 0,
        "the plank inventory reports no annotation at all, so the provisional set \
         is empty because nothing was read rather than because no provisional \
         plank stands"
    );
}

#[given("the step-usage pattern set and the scenarios in the specs")]
async fn the_pattern_set_and_the_scenarios(world: &mut TinmanWorld) {
    world.step_patterns = Some(support::step_definition_patterns());
    world.spec_scenarios = Some(support::spec_scenarios());
}

#[when("each pattern is matched against the steps the scenarios carry")]
async fn each_pattern_is_matched_against_the_steps(world: &mut TinmanWorld) {
    let patterns = world
        .step_patterns
        .as_ref()
        .expect("the pattern set was read");
    let scenarios = world
        .spec_scenarios
        .as_ref()
        .expect("the scenarios were read");
    world.unbound_patterns = Some(
        patterns
            .iter()
            .filter(|pattern| {
                !scenarios.iter().any(|scenario| {
                    scenario
                        .steps
                        .iter()
                        .any(|step| pattern.matcher.binds(step))
                })
            })
            .map(|pattern| {
                format!(
                    "{}:{} declares {:?}, which binds no scenario",
                    pattern.file, pattern.line, pattern.pattern
                )
            })
            .collect(),
    );
}

#[then("every pattern binds at least one scenario")]
async fn every_pattern_binds_a_scenario(world: &mut TinmanWorld) {
    let unbound = world
        .unbound_patterns
        .as_ref()
        .expect("each pattern was matched");
    assert!(
        unbound.is_empty(),
        "{} step definition(s) bind no scenario:\n{}",
        unbound.len(),
        unbound.join("\n")
    );
}

#[then("the pattern set is not empty")]
async fn the_pattern_set_is_not_empty(world: &mut TinmanWorld) {
    let patterns = world
        .step_patterns
        .as_ref()
        .expect("the pattern set was read");
    assert!(
        !patterns.is_empty(),
        "the step-usage command reports no pattern, so the join asserted nothing"
    );
}

#[given("the scenarios in the specs")]
async fn the_scenarios_in_the_specs(world: &mut TinmanWorld) {
    world.spec_scenarios = Some(support::spec_scenarios());
}

#[when("each scenario name is read as the focused command would pass it")]
async fn each_scenario_name_is_read_as_a_regex(world: &mut TinmanWorld) {
    let scenarios = world
        .spec_scenarios
        .as_ref()
        .expect("the scenarios were read");
    world.metacharacter_names = Some(
        scenarios
            .iter()
            .filter_map(|scenario| {
                let carried: Vec<char> = support::REGEX_METACHARACTERS
                    .chars()
                    .filter(|c| scenario.name.contains(*c))
                    .collect();
                (!carried.is_empty()).then(|| {
                    format!(
                        "{}:{} carries {}",
                        scenario.feature,
                        scenario.name,
                        carried.iter().collect::<String>()
                    )
                })
            })
            .collect(),
    );
}

#[then("no scenario name carries a regex metacharacter")]
async fn no_scenario_name_carries_a_metacharacter(world: &mut TinmanWorld) {
    let carrying = world
        .metacharacter_names
        .as_ref()
        .expect("each scenario name was read");
    assert!(
        carrying.is_empty(),
        "{} scenario name(s) carry a regex metacharacter, which the focused command would pass unescaped:\n{}",
        carrying.len(),
        carrying.join("\n")
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: the tier ceilings the rigging declares
// ---------------------------------------------------------------------------

#[given(expr = "the tier budgets in {string} and the weather record")]
async fn the_tier_budgets_and_the_weather_record(world: &mut TinmanWorld, rigging: String) {
    world.tier_budgets = Some(support::tier_budgets(&rigging));
    world.recorded_sweeps = Some(support::recorded_sweeps(&rigging));
    world.rigging_path = Some(rigging);
}

#[when("the most recent recorded sweep for each tier is read against that tier's budget")]
async fn the_most_recent_sweep_of_each_tier_is_read_against_its_budget(world: &mut TinmanWorld) {
    let budgets = world
        .tier_budgets
        .as_ref()
        .expect("the tier budgets were read");
    let sweeps = world
        .recorded_sweeps
        .as_ref()
        .expect("the weather record was read");
    world.over_budget_sweeps = Some(
        budgets
            .iter()
            .filter_map(|budget| {
                // The record is append-only, so the last entry naming a tier is
                // that tier's most recent sweep. A tier the record never named
                // carries no observation to judge, and the producer assertion is
                // the floor that keeps a silent tier honest.
                let sweep = sweeps
                    .iter()
                    .rev()
                    .find(|sweep| sweep.tier == budget.tier)?;
                let ceiling = budget.ceiling.as_millis();
                (u128::from(sweep.ms) > ceiling).then(|| {
                    format!(
                        "the most recent {} sweep took {}ms, over the {}ms {} allows",
                        sweep.tier, sweep.ms, ceiling, budget.key
                    )
                })
            })
            .collect(),
    );
}

#[then("no tier's most recent sweep exceeds its budget")]
async fn no_tiers_most_recent_sweep_exceeds_its_budget(world: &mut TinmanWorld) {
    let over = world
        .over_budget_sweeps
        .as_ref()
        .expect("the most recent sweeps were read against their budgets");
    assert!(
        over.is_empty(),
        "{} tier(s) outran the ceiling they declare on their most recent sweep:\n{}",
        over.len(),
        over.join("\n")
    );
}

#[then("every tier declaring a budget has a sweep command that records its wall clock")]
async fn every_budgeted_tier_records_its_wall_clock(world: &mut TinmanWorld) {
    let budgets = world
        .tier_budgets
        .as_ref()
        .expect("the tier budgets were read");
    let rigging = world
        .rigging_path
        .as_ref()
        .expect("the rigging was named")
        .clone();
    assert!(
        !budgets.is_empty(),
        "the rigging at {rigging} declares no tier budget, so this scenario would \
         assert nothing"
    );
    let weather = support::weather_record_path(&rigging);
    let silent: Vec<String> = budgets
        .iter()
        .filter(|budget| {
            !budget
                .sweep
                .as_deref()
                .is_some_and(|command| command.contains(&weather))
        })
        .map(|budget| match &budget.sweep {
            None => format!(
                "{} bounds the {} tier, which has no sweep command at all",
                budget.key, budget.tier
            ),
            Some(command) => format!(
                "{} bounds the {} tier, whose sweep command records nothing to \
                 {weather}: {command}",
                budget.key, budget.tier
            ),
        })
        .collect();
    assert!(
        silent.is_empty(),
        "{} budgeted tier(s) record no wall clock, so their ceiling could never be \
         exceeded:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

// ---------------------------------------------------------------------------
// published scantling contracts: dialect conformance and version agreement
// ---------------------------------------------------------------------------

#[given("the scantlings that declare a JSON Schema dialect")]
async fn scantlings_declaring_a_dialect(world: &mut TinmanWorld) {
    world.dialect_scantlings = Some(support::dialect_scantlings());
}

#[when("each is checked against the JSON Schema 2020-12 meta-schema")]
async fn checked_against_the_meta_schema(world: &mut TinmanWorld) {
    let scantlings = world
        .dialect_scantlings
        .as_ref()
        .expect("the dialect-declaring scantlings were read");
    world.meta_schema_results = Some(
        scantlings
            .iter()
            .map(|(path, document)| {
                let failure = jsonschema::draft202012::meta::validate(document)
                    .err()
                    .map(|e| format!("{e} at {}", e.instance_path()));
                (path.clone(), failure)
            })
            .collect(),
    );
}

#[then("all nine validate")]
async fn all_nine_validate(world: &mut TinmanWorld) {
    let results = world
        .meta_schema_results
        .as_ref()
        .expect("each scantling was checked");
    assert_eq!(
        results.len(),
        9,
        "nine scantlings declare a dialect, found {}: {}",
        results.len(),
        results
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let failures: Vec<String> = results
        .iter()
        .filter_map(|(path, failure)| failure.as_ref().map(|f| format!("{path}: {f}")))
        .collect();
    assert!(
        failures.is_empty(),
        "{} scantling(s) do not satisfy the dialect they declare:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[given(expr = "the package version in {string}")]
async fn the_package_version_in(world: &mut TinmanWorld, manifest: String) {
    world.package_version = Some(support::package_version(&manifest));
}

#[when("the schema URIs in the scantlings and the example plans are read")]
async fn the_published_schema_uris_are_read(world: &mut TinmanWorld) {
    world.published_uris = Some(support::published_schema_uris());
}

#[then("all fourteen name that version")]
async fn all_fourteen_name_that_version(world: &mut TinmanWorld) {
    let version = world
        .package_version
        .as_ref()
        .expect("the package version was read");
    let uris = world
        .published_uris
        .as_ref()
        .expect("the published schema URIs were read");
    assert_eq!(
        uris.len(),
        14,
        "fourteen schema URIs are published, found {}: {}",
        uris.len(),
        uris.iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let tag = format!("@v{version}/");
    let stale: Vec<String> = uris
        .iter()
        .filter(|(_, uri)| !uri.contains(&tag))
        .map(|(path, uri)| format!("{path}: {uri}"))
        .collect();
    assert!(
        stale.is_empty(),
        "{} schema URI(s) do not name the packaged version {version}:\n{}",
        stale.len(),
        stale.join("\n")
    );
}

// ---------------------------------------------------------------------------
// proof contracts: the shape of a scantling that declares no dialect
// ---------------------------------------------------------------------------

#[given("the four scantlings that declare no JSON Schema dialect")]
async fn the_scantlings_declaring_no_dialect(world: &mut TinmanWorld) {
    world.proof_contracts = Some(support::nondialect_scantlings());
}

#[when(expr = "each is checked against the meta-schema in {string}")]
async fn checked_against_the_meta_schema_in(world: &mut TinmanWorld, meta_schema: String) {
    let contracts = world
        .proof_contracts
        .as_ref()
        .expect("the proof contracts were read");
    world.meta_schema_results = Some(
        contracts
            .iter()
            .map(|(path, document)| {
                let counterexamples = support::schema_counterexamples(&meta_schema, document);
                (
                    path.clone(),
                    (!counterexamples.is_empty()).then(|| counterexamples.join("; ")),
                )
            })
            .collect(),
    );
    world.meta_schema_path = Some(meta_schema);
}

#[then("all four validate")]
async fn all_four_validate(world: &mut TinmanWorld) {
    let results = world
        .meta_schema_results
        .as_ref()
        .expect("each proof contract was checked");
    assert_eq!(
        results.len(),
        4,
        "four scantlings declare no dialect, found {}: {}",
        results.len(),
        results
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let failures: Vec<String> = results
        .iter()
        .filter_map(|(path, failure)| failure.as_ref().map(|f| format!("{path}: {f}")))
        .collect();
    assert!(
        failures.is_empty(),
        "{} proof contract(s) do not satisfy the proof-contract meta-schema:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[then("the meta-schema forbids a property it does not name")]
async fn the_meta_schema_forbids_an_unnamed_property(world: &mut TinmanWorld) {
    let meta_schema = world
        .meta_schema_path
        .as_ref()
        .expect("the meta-schema was named");
    let contracts = world
        .proof_contracts
        .as_ref()
        .expect("the proof contracts were read");
    let unnamed = "tinmanPropertyTheMetaSchemaDoesNotName";
    let tolerated: Vec<String> = contracts
        .iter()
        .filter(|(_, document)| {
            let mut probed = document.clone();
            probed
                .as_object_mut()
                .expect("a proof contract is a JSON object")
                .insert(unnamed.to_string(), serde_json::Value::Bool(true));
            support::schema_counterexamples(meta_schema, &probed).is_empty()
        })
        .map(|(path, _)| path.clone())
        .collect();
    assert!(
        tolerated.is_empty(),
        "{meta_schema} accepted the unnamed property {unnamed} on {} proof contract(s): {}",
        tolerated.len(),
        tolerated.join(", ")
    );
}

// ---------------------------------------------------------------------------
// scantling enumerations: both directions of the join against the production
// enumerations they constrain
// ---------------------------------------------------------------------------

#[given("the enumerations the scantlings declare")]
async fn the_enumerations_the_scantlings_declare(world: &mut TinmanWorld) {
    world.enumeration_pairs = Some(support::enumeration_pairs());
}

#[when("each declared value is parsed by the type its scantling describes")]
async fn each_declared_value_is_parsed(world: &mut TinmanWorld) {
    let pairs = world
        .enumeration_pairs
        .as_ref()
        .expect("the scantling enumerations were read");
    world.rejected_values = Some(
        pairs
            .iter()
            .flat_map(|pair| {
                pair.declared.iter().filter_map(move |value| {
                    (pair.serialized)(value).err().map(|failure| {
                        format!(
                            "{} at {} declares {value}, which {} rejects: {failure}",
                            pair.scantling, pair.pointer, pair.production
                        )
                    })
                })
            })
            .collect(),
    );
}

#[then("every declared value is accepted")]
async fn every_declared_value_is_accepted(world: &mut TinmanWorld) {
    let rejected = world
        .rejected_values
        .as_ref()
        .expect("each declared value was parsed");
    assert!(
        rejected.is_empty(),
        "{} declared value(s) are values Tinman rejects:\n{}",
        rejected.len(),
        rejected.join("\n")
    );
}

#[then("the enumerations read are not empty")]
async fn the_enumerations_read_are_not_empty(world: &mut TinmanWorld) {
    let pairs = world
        .enumeration_pairs
        .as_ref()
        .expect("the scantling enumerations were read");
    assert!(
        !pairs.is_empty(),
        "no scantling enumeration was joined to a production enumeration"
    );
    let empty: Vec<String> = pairs
        .iter()
        .filter(|pair| pair.declared.is_empty())
        .map(|pair| format!("{} at {}", pair.scantling, pair.pointer))
        .collect();
    assert!(
        empty.is_empty(),
        "{} enumeration(s) declare no value: {}",
        empty.len(),
        empty.join(", ")
    );
}

#[given("the production enumerations the scantlings constrain")]
async fn the_production_enumerations_the_scantlings_constrain(world: &mut TinmanWorld) {
    world.enumeration_pairs = Some(support::enumeration_pairs());
}

#[when("each variant's serialized name is matched against its scantling enumeration")]
async fn each_variants_serialized_name_is_matched(world: &mut TinmanWorld) {
    let pairs = world
        .enumeration_pairs
        .as_ref()
        .expect("the production enumerations were read");
    world.undeclared_variants = Some(
        pairs
            .iter()
            .flat_map(|pair| {
                pair.accepted.iter().filter_map(move |variant| {
                    let name = (pair.serialized)(variant).unwrap_or_else(|failure| {
                        panic!(
                            "{} accepts the variant {variant} but does not serialize it: {failure}",
                            pair.production
                        )
                    });
                    (!pair.declared.contains(&name)).then(|| {
                        format!(
                            "{} serializes a variant as {name}, which {} at {} does not declare (it declares {})",
                            pair.production,
                            pair.scantling,
                            pair.pointer,
                            pair.declared.join(", ")
                        )
                    })
                })
            })
            .collect(),
    );
}

#[then("every variant is a value its scantling declares")]
async fn every_variant_is_declared(world: &mut TinmanWorld) {
    let undeclared = world
        .undeclared_variants
        .as_ref()
        .expect("each variant's serialized name was matched");
    assert!(
        undeclared.is_empty(),
        "{} production variant(s) are not declared by the scantling that constrains them:\n{}",
        undeclared.len(),
        undeclared.join("\n")
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
