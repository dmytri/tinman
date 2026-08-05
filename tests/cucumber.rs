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
use tinman::sandbox::{
    Backend, CommandSpec, EnvGrant, EnvOrigin, Mount, MountMode, Network, SandboxSpec,
};

/// Shared scenario state. Fields are populated by the step definitions that
/// need them. `Option` fields stay `None` until a `Given`/`When` sets them.
#[derive(Debug, Default, World)]
struct TinmanWorld {
    // command invocation
    command: Option<CommandSpec>,
    // backend selection
    requested_backend: Option<Backend>,
    resolution: Option<Result<ResolvedBackend, ResolveError>>,
    // the platform a scenario names, as the identifier the implementation uses,
    // and the outcomes read from the resolution seam itself
    platform: Option<String>,
    resolution_outcomes: Option<Vec<String>>,
    // bwrap availability + launch
    backend: Option<BubblewrapBackend>,
    spec: Option<SandboxSpec>,
    launch_error: Option<String>,
    sentinel: Option<std::path::PathBuf>,
    // the directory the sentinel path sits in, held so a sentinel a breached
    // sandbox managed to write is reclaimed with it rather than left in the
    // temp directory
    sentinel_dir: Option<support::ScratchDir>,
    // bwrap argument generation
    generated_args: Option<Vec<String>>,
    network_denied: bool,
    // pty boundary
    boundary_counterexamples: Option<Vec<String>>,
    // schema-conformance: the serialized artifact under test, as JSON
    serialized: Option<serde_json::Value>,
    // the variable a specification grants from the host, the fixture tree a
    // mount exposes, and the staging directory a copy mount created
    // the plan file a scenario named and made sure is absent
    missing_plan: Option<String>,
    granted_env: Option<String>,
    // The environment the backend is handed, in place of the process table. One
    // table serves the whole process, so staging a grant by mutating it changes
    // what every concurrent scenario reads.
    handed_env: Option<std::collections::BTreeMap<String, String>>,
    fixture_tree: Option<support::ScratchDir>,
    staging_dir: Option<std::path::PathBuf>,
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
    // the outbound targets the rigging declares, and the ship and verify lines
    // read off each, kept beside the target and the kind so a missing line names
    // which one it is
    outbound_targets: Option<Vec<support::OutboundTarget>>,
    outbound_lines: Option<Vec<(String, String, Option<String>)>>,
    // the exceptional-double marks verification support carries, and the subset
    // whose text names no condition the agreement permits
    double_marks: Option<Vec<support::ExceptionalDoubleMark>>,
    unjustified_marks: Option<Vec<String>>,
    // the construction-boundary contract the verifier last checked, so a floor
    // asserting how the checker counts reads the same contract the check did
    boundary_contract: Option<String>,
    // what the duplication scanner reported over the implementation, the
    // minimum unit size it was asked to analyse at, and the path both were read
    // from so a failure names the file that declares the threshold
    duplication: Option<support::DuplicationReport>,
    duplication_minimum_lines: Option<usize>,
    duplication_allowance_path: Option<String>,
    // what the constant census read over the implementation, the groups it
    // reports as duplicated, and the groups the allowance names as coincidence
    constants_read: Option<Vec<support::CensusConstant>>,
    duplicate_constants: Option<Vec<support::DuplicateConstants>>,
    constant_allowance: Option<Vec<Vec<String>>>,
    // the waits on a spawned child the verification support sources make, each
    // with the deadline that bounds it
    process_waits: Option<Vec<support::ProcessWait>>,
    // what tearing a driver session down observed
    teardown: Option<support::TeardownReport>,
    // the proxied-egress scenarios: the real upstream, the proxy under test, the
    // scratch directory the HAR sits in, and what a request through the proxy
    // answered, whether it answered or failed
    upstream: Option<support::Upstream>,
    proxy: Option<tinman::proxy::Proxy>,
    har_dir: Option<support::ScratchDir>,
    har_path: Option<String>,
    rule_placements: Option<Vec<RulePlacement>>,
    proxied: Option<support::ProxiedResponse>,
    proxy_error: Option<String>,
    // the sandbox resources standing after the suite's own reclaim
    sandbox_inventory: Option<support::SandboxInventory>,
    conformance_matches: Option<Vec<support::ConformanceMatch>>,
    // the plank inventory, the step-definition pattern set, the scenarios the
    // specs declare, and each join's counterexamples
    planks: Option<Vec<support::Plank>>,
    step_patterns: Option<Vec<support::StepPattern>>,
    spec_scenarios: Option<Vec<support::SpecScenario>>,
    stale_planks: Option<Vec<String>>,
    unbound_patterns: Option<Vec<String>>,
    metacharacter_names: Option<Vec<String>>,
    // the scenario references the shipped Markdown documents cite, and the ones
    // naming no scenario the specs carry
    shipped_citations: Option<Vec<support::Citation>>,
    unresolved_citations: Option<Vec<String>>,
    // the shipped Markdown documents, and the survey reading each line that
    // names a scenario or a spec for a watchbill membership claim
    shipped_documents: Option<Vec<String>>,
    watchbill_claim_survey: Option<support::WatchbillSurvey>,
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
    // the generated concurrency fixture, held so it outlives the runs that read
    // it and is reclaimed with the scenario, and the wall clock each arm of the
    // comparison observed
    fixture_dir: Option<support::ScratchDir>,
    fixture_features: Option<std::path::PathBuf>,
    one_worker_run: Option<support::FixtureRun>,
    four_worker_run: Option<support::FixtureRun>,
    // the sweep whose per-scenario durations the wake carries, the scenarios
    // that sweep started without recording one, every run the record carries,
    // and the run made with a filter that matched no feature
    sweep_durations: Option<support::RecordedDurations>,
    scenarios_missing_durations: Option<Vec<String>>,
    recorded_runs: Option<Vec<support::RecordedDurations>>,
    empty_filter_run: Option<String>,
    // published scantling contracts: the dialect-declaring scantlings, what the
    // meta-schema said of each, the packaged version, and the URIs consumers
    // fetch
    dialect_scantlings: Option<Vec<(String, serde_json::Value)>>,
    meta_schema_results: Option<Vec<(String, Option<String>)>>,
    package_version: Option<String>,
    published_uris: Option<Vec<(String, String)>>,
    // the packaged manifests read, in the order the scenario names them, so two
    // artifacts shipped from one tree are compared against each other
    package_versions: Vec<(String, String)>,
    // the scantling paths listed, and the ones no scenario and no step
    // definition names
    scantling_paths: Option<Vec<String>>,
    unreached_scantlings: Option<Vec<String>>,
    // the style properties each scantling requires, kept in the order the
    // scenario names them, so two restatements of one shape are compared
    style_property_sets: Vec<(String, Vec<String>)>,
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
    // the documents an asset's fenced blocks carry, each beside the text it was
    // read from, and what the schema they claim to illustrate said of them
    fenced_documents: Option<Vec<(String, serde_json::Value)>>,
    schema_violations: Option<Vec<String>>,
    // the roles a document names, and those the model's own schema never
    // declares
    named_roles: Option<Vec<String>>,
    undeclared_roles: Option<Vec<String>>,
    // help and command line
    asset_text: Option<String>,
    accepted_commands: Option<Vec<String>>,
    read_command_set: Option<Vec<String>>,
    // the commands a scenario names in its own table, held apart from the set
    // read off the parser so the two can be compared against each other
    named_command_set: Option<Vec<String>>,
    // whether that set came from the scenario's own table. A step comparing the
    // parser against the named set asserts nothing where the set was read off
    // the parser, so the two sources are told apart rather than trusted alike
    named_command_set_from_table: bool,
    // what each command answered when asked for help
    command_help: Option<Vec<CommandHelp>>,
    listed_commands: Option<Vec<String>>,
    advertised_options: Option<Vec<String>>,
    option_rejections: Option<Vec<String>>,
    // the Tinman command lines an asset names, and the parser's refusals
    named_command_lines: Option<Vec<String>>,
    // the body of the roff section a step last read, the subcommand pages the
    // man page cross-references, and the ones the binary would not emit
    roff_section: Option<Vec<String>>,
    cross_references: Option<Vec<String>>,
    unemitted_pages: Option<Vec<String>>,
    command_line_rejections: Option<Vec<String>>,
    placeholder_count: Option<usize>,
    // the placeholder names the help asset's Examples block carries, and the
    // example values the asset offers to expand them to, held apart so the two
    // are compared against each other rather than either asserted alone
    example_placeholders: Option<Vec<String>>,
    example_values: Option<Vec<(String, String)>>,
    // the command-line sessions a scenario launched, closed when the scenario
    // ends: a session outlives the command that launched it, so one left by a
    // scenario that failed part-way is a real program still running.
    //
    // Declared before `scratch`: fields drop in declaration order, and the
    // teardown runs `tinman close` from the scratch directory, so a scratch
    // dropped first leaves the close with no working directory to spawn in.
    launched_sessions: LaunchedSessions,
    // the temporary directory this scenario's runs of the binary see. A session
    // is addressed by the name the operator gave it, and two scenarios of one
    // tier name the same session `work`, so workers sharing one temporary
    // directory address one another's sessions. Dropped after
    // `launched_sessions`, whose teardown reads it.
    session_tmp: Option<support::ScratchDir>,
    // running the real binary
    scratch: Option<support::ScratchDir>,
    terminal_session: Option<support::TerminalSession>,
    provider: Option<support::LocalProvider>,
    run_stdout: Option<String>,
    run_stderr: Option<String>,
    run_status: Option<i32>,
    // the argument list the operator's command line was run as, so a step
    // asserting what the run was contingent on restates that same command line
    // rather than carrying a second copy of the scenario's own flags
    run_argv: Option<Vec<String>>,
    // what the plan a scenario staged expects, so a step reading the failure it
    // reported addresses the step that actually failed
    staged_plan_expectation: Option<String>,
    // the file a scenario put in a run's way, so a step reading "that file"
    // addresses the one the starting state created
    existing_file: Option<String>,
    // the command a scenario staged for the sandbox to refuse, so a step
    // reading "that command" addresses the one the starting state staged
    unexecutable_command: Option<String>,
    // how long the command the operator ran took to complete, so a ceiling a
    // scenario puts on it is read off the wall clock rather than assumed
    run_elapsed: Option<std::time::Duration>,
    // what the run wrote to the terminal, control sequences intact, so a screen
    // assertion reads the bytes the terminal received
    run_raw: Option<Vec<u8>>,
    // the width of the terminal the run drew on, so a screen assertion parses
    // the bytes on the grid the program addressed
    run_columns: Option<u16>,
    // a line the operator's terminal already carried before the run started, so
    // a scenario about what the run does to their scrollback meets a terminal
    // that has work on it rather than a blank one
    scrollback_line: Option<String>,
    saved_credential_paths: Option<Vec<std::path::PathBuf>>,
    // the moment the question was sent, and the elapsed seconds the pending
    // call last reported, so a report of how long it has been waiting is read
    // against the wall clock and against its own earlier reading
    question_sent_at: Option<std::time::Instant>,
    reported_elapsed: Option<u64>,
    // flow orchestration
    flow_outcome: Option<tinman::flow::FlowOutcome>,
    flow_error: Option<String>,
    // the Bubblewrap version a scenario staged on the path, for a refusal that
    // names what it found beside what it requires
    staged_bwrap_version: Option<String>,
    // driver protocol
    driver: Option<support::DriverProcess>,
    reply: Option<serde_json::Value>,
    session_id: Option<String>,
    session_dirs: Vec<std::path::PathBuf>,
    next_request_id: u64,
    // the method names read off the protocol scantling's enum, so a scenario
    // exchanging every one of them reads its worklist from the contract rather
    // than from a list somebody kept in step by hand
    protocol_methods: Option<Vec<String>>,
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
    // the region name a replay plan's activation step addresses, so a following
    // step can rename that region in the driven program and a failure assertion
    // can read the name the plan asked for
    replay_activated_region: Option<String>,
    // the name a plan's roleless locator addresses, so the step asserting the
    // failure reads the locator the plan asked for. An activation written as a
    // bare name and an expectation written as a bare locator both reach a replay
    // with a name and no role, so both are reported against the same name
    replay_roleless_locator: Option<String>,
    // the region a scoped activation narrows to, with the role and name it
    // addresses, so the step asserting what the program shows reads the same
    // three the plan was written with
    scoped_activation: Option<ScopedActivation>,
    // the properties the plan schema declares and the ones a round trip through
    // production keeps, held apart so the two are compared against each other
    plan_schema_path: Option<String>,
    declared_plan_properties: Option<Vec<support::DeclaredProperty>>,
    carried_plan_properties: Option<std::collections::BTreeSet<support::DeclaredProperty>>,
    // the written plan forms production's reader refused, so a property lost
    // because its whole form was refused is reported with that reason
    unread_plan_forms: Option<Vec<String>>,
    record_program: Option<String>,
    proposed_name: Option<String>,
    parsed_sandbox: Option<tinman::sandbox::SandboxSpec>,
    parse_error: Option<String>,
    // terminal object model
    pane: Option<(String, Vec<String>)>,
    // the text a scenario placed on the screen and where it placed it, so a
    // later step redraws the same screen with the cursor moved or hidden
    placed_text: Option<(String, u16, u16)>,
    // the text a scenario drew across the top line, so a later step redraws the
    // same line with one of its labels marked
    top_line: Option<String>,
    // the regions an accessible-name sweep read, so the following step can
    // assert the sweep read something rather than passing over an empty model
    named_regions_read: Option<usize>,
    // the region a presentation assertion last found, so a following step
    // reading "that region" reads the one just named
    styled_region: Option<serde_json::Value>,
    // the background block a presentation assertion last found, as a 1-based row
    // with the run's start column and width, so a following step reading "that
    // background" reads the one just found
    drawn_background: Option<(u16, u16, u16)>,
    // the lines a scenario drew above a table, so a following step asserts the
    // regions read from them against the text they were drawn with
    summary_lines: Option<Vec<String>>,
    // each command the parser defines with the long options it declares, and the
    // manual page each of them emits, so the page is read against the definition
    // it was built from
    command_definitions: Option<Vec<(String, Vec<String>)>>,
    manual_pages: Option<Vec<(String, String)>>,
    // the rule bodies the specs carry and, per body, the implementation paths a
    // reading of it found, so the count read is checked before the finding is
    rule_bodies: Option<Vec<support::RuleBody>>,
    rule_body_citations: Option<Vec<(support::RuleBody, Vec<String>)>>,
    // the lifecycle tag lines the specs carry and what each was read to precede
    lifecycle_tags: Option<Vec<support::LifecycleTag>>,
    lifecycle_tags_read: Option<Vec<support::LifecycleTag>>,
    // the crates the released binary carries and what the advisory database
    // answered about each, so the count queried is checked before the finding is
    release_crates: Option<Vec<(String, String)>>,
    advisory_answers: Option<Vec<support::AdvisoryAnswer>>,
    tom: Option<tinman::tom::Model>,
    found_region: Option<tinman::tom::Region>,
    // every region a role sweep read, so a following step reading "each one"
    // reads the set the count was taken over
    found_regions: Option<Vec<tinman::tom::Region>>,
    tom_resolution: Option<tinman::tom::Resolution>,
    tom_binding: Option<String>,
    record_error: Option<String>,
    written_plan_source: Option<String>,
    // interactive assistant
    response: Option<tinman::assistant::Response>,
    parser_arguments: Option<Vec<String>>,
    // the marker a proposing reply carries, read from the assistant itself, so
    // the asset that teaches it is joined to the code that reads it
    proposal_marker: Option<String>,
    // the page source configuration resolved for the tldr page a naming pass
    // reads, kept so the step asserting what it resolved to reads the value
    // production returned rather than the environment the scenario set
    page_source: Option<String>,
    // the parser's answer to one command line a scenario passed it, kept so the
    // step asserting acceptance reads an outcome rather than an absence
    parser_outcome: Option<Result<(), String>>,
    // the pages the scenario's own tldr source carries, program to examples, and
    // the real loopback source serving them as raw markdown
    tldr_pages: std::collections::BTreeMap<String, Vec<String>>,
    tldr_source: Option<support::LocalTldrSource>,
    // the program whose page a following step appends a further example to
    last_tldr_program: Option<String>,
    // the examples a scenario staged into a program's own help, so the step
    // asserting they were reported reads what the scenario staged
    staged_help_examples: Vec<String>,
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
    // the question last sent at the assistant prompt, so a step reading the
    // request it produced finds that request among the ones the provider took
    last_question: Option<String>,
    // what the scenario's local provider answers with, so a step waiting for a
    // turn to finish ends on the answer reaching the terminal
    provider_answer: Option<String>,
    // the timed seams the latency contract declares, and the ceiling production
    // was read to enforce for each of them
    latency_budgets: Option<Vec<support::LatencyBudget>>,
    enforced_ceilings: Option<Vec<(support::LatencyBudget, Option<std::time::Duration>)>>,
    // the short ceiling one seam is driven against and how long that wait
    // lasted, so a ceiling that ends a wait is told from one that never does
    short_ceiling: Option<std::time::Duration>,
    short_ceiling_elapsed: Option<std::time::Duration>,
    // the refusal an unavailable sandbox prints, as the asset carries it, and
    // the link read out of it, so the step checking where it points reads the
    // shipped text rather than a copy of it
    refusal_text: Option<String>,
    refusal_link: Option<String>,
    // the column header a layout scantling constrained by position, so the step
    // asserting what the failure reported reads the position that region was
    // drawn at in the model rather than a value restated here
    constrained_column_header: Option<String>,
    // the region a staged layout demanded and the program does not draw, so the
    // step asserting what the failure named reads the name the layout carried
    // rather than a value restated in the assertion
    attested_absent_region: Option<String>,
    // the file a launch would write and the directory holding the recorder that
    // writes it, held so the recorder outlives the run and is reclaimed with the
    // scenario
    launch_record: Option<std::path::PathBuf>,
    launch_recorder_dir: Option<support::ScratchDir>,
    // the program a scenario staged for inspection, held as the name a launch
    // addresses it by and the path a capture reads it at, so the step reading
    // what the deterministic pass alone makes of it reads the same program the
    // command was pointed at
    inspected_program: Option<String>,
    inspected_program_path: Option<std::path::PathBuf>,
}

/// The scenario's working directory: the one a dotenv file is staged in and the
/// one a launched Tinman runs in. Created on first use, so a scenario that never
/// names a working directory still runs in a directory holding no `.env`.
fn working_dir(world: &mut TinmanWorld) -> std::path::PathBuf {
    if world.scratch.is_none() {
        world.scratch = Some(support::ScratchDir::new("workdir"));
    }
    // Every run of the binary is given this scenario's own temporary directory,
    // so the sessions it creates are addressable by it alone. Set here because
    // every path that runs the binary reads the working directory first and the
    // configured environment after, so a scenario naming no starting state is
    // isolated too.
    scenario_temp_dir(world);
    world
        .scratch
        .as_ref()
        .expect("a working directory")
        .path()
        .to_path_buf()
}

/// The temporary directory this scenario's runs of the binary see, created on
/// first use and put on their environment as `TMPDIR`.
fn scenario_temp_dir(world: &mut TinmanWorld) -> std::path::PathBuf {
    if world.session_tmp.is_none() {
        world.session_tmp = Some(support::ScratchDir::new("tmp"));
    }
    let path = world
        .session_tmp
        .as_ref()
        .expect("a temporary directory")
        .path()
        .to_path_buf();
    world
        .env_vars
        .entry("TMPDIR".to_string())
        .or_insert_with(|| path.display().to_string());
    path
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
/// the binary receives, dropping the program name itself. The line is one an
/// operator types, so it is split the way their shell splits it rather than on
/// whitespace alone: splitting `tinman expect total 'ls -la'` on whitespace
/// hands the binary `'ls` and `-la'`, which is a command line no operator can
/// produce, and the flag the quotes were protecting is read as Tinman's own.
fn tinman_args(line: &str) -> Vec<String> {
    shell_split(line).into_iter().skip(1).collect()
}

/// Split `line` into words the way a shell does: whitespace separates words,
/// and a single- or double-quoted run is one word whose quotes are removed.
fn shell_split(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    for character in line.chars() {
        match quote {
            Some(open) if character == open => quote = None,
            Some(_) => word.push(character),
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                started = true;
            }
            None if character.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            None => {
                word.push(character);
                started = true;
            }
        }
    }
    assert!(
        quote.is_none(),
        "the command line {line:?} opens a quote it never closes, so it is not a line an operator \
         could have typed"
    );
    if started {
        words.push(word);
    }
    words
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

/// The operating-system identifier the implementation names a platform by, for
/// the platform a scenario names in the operator's own terms.
fn platform_identifier(named: &str) -> &'static str {
    match named {
        "Linux" => "linux",
        "macOS" => "macos",
        other => {
            panic!("the scenario names the platform {other}, which carries no identifier here")
        }
    }
}

#[given(expr = "the running platform is {word}")]
async fn the_running_platform_is(world: &mut TinmanWorld, named: String) {
    world.platform = Some(platform_identifier(&named).to_string());
}

#[when("the backend is resolved for that platform")]
async fn the_backend_is_resolved_for_that_platform(world: &mut TinmanWorld) {
    let platform = world.platform.as_ref().expect("a platform was named");
    world.resolution = Some(resolve(platform));
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

#[then("the failure names the platform it could not serve")]
async fn the_failure_names_the_platform(world: &mut TinmanWorld) {
    let platform = world.platform.as_ref().expect("a platform was named");
    let err = world
        .resolution
        .as_ref()
        .expect("resolution ran")
        .as_ref()
        .expect_err("resolution failed");
    let reported = format!("{err:?}");
    assert!(
        reported.to_lowercase().contains(&platform.to_lowercase()),
        "the failure {reported:?} does not name the platform {platform} it could not serve"
    );
}

#[given("the backend resolution seam")]
async fn the_backend_resolution_seam(world: &mut TinmanWorld) {
    world.resolution_outcomes = Some(support::resolution_outcomes());
}

#[when("every outcome resolution can return is enumerated")]
async fn every_resolution_outcome_is_enumerated(world: &mut TinmanWorld) {
    // The seam is read rather than exercised: an outcome no caller reaches is
    // still an outcome the seam offers, and an escape hatch a published surface
    // advertises is a promise whether or not a caller takes it.
    let outcomes = world
        .resolution_outcomes
        .as_ref()
        .expect("the resolution seam was read");
    assert!(
        !outcomes.is_empty(),
        "the resolution seam yielded no outcome to enumerate"
    );
}

#[then("every outcome names a sandbox backend")]
async fn every_outcome_names_a_sandbox_backend(world: &mut TinmanWorld) {
    let outcomes = world
        .resolution_outcomes
        .as_ref()
        .expect("the resolution seam was read");
    let unsandboxed: Vec<String> = outcomes
        .iter()
        .filter(|name| name.as_str() == support::UNSANDBOXED_BACKEND)
        .cloned()
        .collect();
    assert!(
        unsandboxed.is_empty(),
        "resolution can return {} outcome(s) naming no sandbox: {}; it returns {outcomes:?}",
        unsandboxed.len(),
        unsandboxed.join(", ")
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
        let prepared = staged_backend(world)
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

/// A path a sandboxed program can name but must not be able to write: it sits
/// in a scratch directory of its own, outside the workspace a sandbox binds, so
/// nothing but a breach puts a file there. The directory is held by the world,
/// so a sentinel a breach did write is reclaimed with it.
#[given("a sentinel path outside the sandbox where no file exists")]
async fn a_sentinel_path_outside_the_sandbox(world: &mut TinmanWorld) {
    let dir = support::ScratchDir::new("sentinel");
    let sentinel = dir.path().join("breached");
    assert!(
        !sentinel.exists(),
        "the sentinel {} already exists",
        sentinel.display()
    );
    world.sentinel = Some(sentinel);
    world.sentinel_dir = Some(dir);
}

#[then("no file exists at the sentinel path")]
async fn no_file_exists_at_the_sentinel_path(world: &mut TinmanWorld) {
    let sentinel = world.sentinel.as_ref().expect("a sentinel path was given");
    assert!(
        !sentinel.exists(),
        "a file exists at the sentinel path {}: the launched program wrote outside the sandbox",
        sentinel.display()
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

#[given("the sandbox processes and staging directories present after the suite reclaims")]
async fn the_sandbox_resources_present_after_the_reclaim(world: &mut TinmanWorld) {
    world.sandbox_inventory = Some(support::standing_sandbox_resources());
}

#[when("each is matched against the runs that are still live")]
async fn each_is_matched_against_the_live_runs(world: &mut TinmanWorld) {
    // The sweep reads each resource against the process table as it goes, so the
    // match is already in the inventory this step confirms is in hand.
    assert!(
        world.sandbox_inventory.is_some(),
        "the sandbox resources were swept"
    );
}

/// The sweep this scenario is asserting on.
fn sandbox_inventory(world: &TinmanWorld) -> &support::SandboxInventory {
    world
        .sandbox_inventory
        .as_ref()
        .expect("the sandbox resources were swept")
}

#[then("no sandbox process outlives the run that created it")]
async fn no_sandbox_process_outlives_its_run(world: &mut TinmanWorld) {
    let orphans = &sandbox_inventory(world).orphan_processes;
    assert!(
        orphans.is_empty(),
        "{} sandbox process(es) outlive the run that created them: {}",
        orphans.len(),
        orphans.join(", ")
    );
}

#[then("no staging directory outlives the run that created it")]
async fn no_staging_directory_outlives_its_run(world: &mut TinmanWorld) {
    let orphans = &sandbox_inventory(world).orphan_dirs;
    assert!(
        orphans.is_empty(),
        "{} staging director(ies) outlive the run that created them: {}",
        orphans.len(),
        orphans.join(", ")
    );
}

#[then("the inventory searched every temporary-directory prefix the implementation creates")]
async fn the_inventory_searched_every_implementation_prefix(world: &mut TinmanWorld) {
    // A sweep that read neither source reports a clean bill for a system it
    // never looked at, so the sources are named first. Naming them cannot say
    // the sweep read the right directory under the right names, which is the
    // reading that stays green while resources accumulate: the implementation
    // decides which prefixes exist, so it is what the sweep is measured against.
    let inventory = sandbox_inventory(world);
    assert!(
        inventory
            .searched
            .contains(&std::env::temp_dir().display().to_string()),
        "the sweep did not read the temporary directory; it read {:?}",
        inventory.searched
    );
    assert!(
        inventory.searched.contains(&"/proc".to_string()),
        "the sweep did not read the process table; it read {:?}",
        inventory.searched
    );
    let created = support::implementation_temp_prefixes();
    assert!(
        !created.is_empty(),
        "the implementation tree yielded no temporary-directory prefix to measure the sweep against"
    );
    let unswept: Vec<String> = created
        .iter()
        .filter(|prefix| !inventory.searched_prefixes.contains(prefix))
        .cloned()
        .collect();
    assert!(
        unswept.is_empty(),
        "the implementation creates {} temporary-directory prefix(es) the sweep does not search: {}; the sweep searches {:?}",
        unswept.len(),
        unswept.join(", "),
        inventory.searched_prefixes
    );
}

#[given("the scantling paths under the scantlings directory")]
async fn the_scantling_paths_under_the_scantlings_directory(world: &mut TinmanWorld) {
    world.scantling_paths = Some(support::scantling_paths());
}

#[when("each is matched against the specs and the step definitions that read it")]
async fn each_scantling_is_matched_against_its_readers(world: &mut TinmanWorld) {
    let paths = world
        .scantling_paths
        .as_ref()
        .expect("the scantling paths were listed");
    world.unreached_scantlings = Some(support::unreached_scantlings(paths));
}

#[then("every scantling path is reached by at least one of them")]
async fn every_scantling_path_is_reached(world: &mut TinmanWorld) {
    let unreached = world
        .unreached_scantlings
        .as_ref()
        .expect("each scantling path was matched");
    assert!(
        unreached.is_empty(),
        "{} scantling(s) are named by no scenario and no step definition: {}",
        unreached.len(),
        unreached.join(", ")
    );
}

#[then("the scantling paths read are not empty")]
async fn the_scantling_paths_read_are_not_empty(world: &mut TinmanWorld) {
    let paths = world
        .scantling_paths
        .as_ref()
        .expect("the scantling paths were listed");
    assert!(
        !paths.is_empty(),
        "the scantlings directory listing found no scantling to match"
    );
}

// ---------------------------------------------------------------------------
// the operator's working directory: a launched program reads it and leaves it
// as it found it
// ---------------------------------------------------------------------------

#[given(expr = "the operator's working directory carries no file named {string}")]
async fn the_working_directory_carries_no_file_named(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    if path.exists() {
        std::fs::remove_file(&path)
            .unwrap_or_else(|e| panic!("the file {} was not removed: {e}", path.display()));
    }
    assert!(
        !path.exists(),
        "the file {} stands in the working directory",
        path.display()
    );
}

#[given(expr = "the operator's working directory carries a file {string} containing {string}")]
async fn the_working_directory_carries_a_file(
    world: &mut TinmanWorld,
    name: String,
    contents: String,
) {
    let path = working_dir(world).join(&name);
    std::fs::write(&path, &contents)
        .unwrap_or_else(|e| panic!("the file {} was not written: {e}", path.display()));
}

#[then(expr = "no file named {string} exists in the operator's working directory")]
async fn no_file_named_exists_in_the_working_directory(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    assert!(
        !path.exists(),
        "the file {} exists: the launched program wrote into the operator's working directory",
        path.display()
    );
}

/// A shell line that writes `name` into the directory it runs in and then prints
/// `text`. The sandbox carries no `/dev/null`, so stderr is closed rather than
/// redirected: a refusal the sandbox issues must not be drawn on the screen the
/// assertions read.
fn writes_into_its_working_directory(name: &str, text: &str) -> String {
    format!("exec 2>&-; printf probe > {name}; printf {text}")
}

#[when(
    expr = "the operator inspects a command that writes {string} into its working directory and prints {string}"
)]
async fn the_operator_inspects_a_command_writing_into_the_working_directory(
    world: &mut TinmanWorld,
    name: String,
    text: String,
) {
    let command = writes_into_its_working_directory(&name, &text);
    run_tinman_command(world, &["inspect", &command]).await;
}

#[when(
    expr = "the operator records a command that writes {string} into its working directory and prints {string}"
)]
async fn the_operator_records_a_command_writing_into_the_working_directory(
    world: &mut TinmanWorld,
    name: String,
    text: String,
) {
    let program = stage_recorded_program(
        world,
        "writes-into-workdir",
        &format!("#!/bin/sh\nexec 2>&-\nprintf probe > {name}\nprintf {text}\n"),
    );
    run_record(world, &program, &[]);
}

#[given(
    expr = "a plan whose command writes {string} into its working directory and prints {string}"
)]
async fn a_plan_whose_command_writes_into_the_working_directory(
    world: &mut TinmanWorld,
    name: String,
    text: String,
) {
    let command = writes_into_its_working_directory(&name, &text);
    world.replay_plan_source = Some(format!("flow:\n  - run: {command:?}\n"));
}

#[when("the operator runs that plan")]
async fn the_operator_runs_that_plan(world: &mut TinmanWorld) {
    let source = world
        .replay_plan_source
        .clone()
        .expect("a harness plan was given");
    let path = working_dir(world).join("plan.yaml");
    std::fs::write(&path, &source)
        .unwrap_or_else(|e| panic!("the plan {} was not written: {e}", path.display()));
    run_tinman_command(world, &["test", &path.to_string_lossy()]).await;
}

#[then("the plan passes")]
async fn the_plan_passes(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a plan was run");
    let output = run_output(world);
    let errors = run_errors(world);
    assert_eq!(
        status, 0,
        "the plan did not pass; tinman wrote:\n{output}\nand on stderr:\n{errors}"
    );
}

#[when(expr = "the operator inspects a command that prints the contents of {string}")]
async fn the_operator_inspects_a_command_printing_the_contents_of(
    world: &mut TinmanWorld,
    name: String,
) {
    let command = format!("exec 2>&-; cat {name}");
    run_tinman_command(world, &["inspect", &command]).await;
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
    let operator_working_directory = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let counterexamples = support::check_bwrap_policy(
        &policy_path,
        argv,
        world.network_denied,
        &operator_home,
        &operator_working_directory,
    );
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

#[given("the implementation sources")]
async fn the_implementation_sources(_world: &mut TinmanWorld) {
    // The construction boundary contract names the search paths; nothing to
    // stage here.
}

#[when("the verifier checks the backend construction boundary")]
async fn verifier_checks_backend_construction_boundary(world: &mut TinmanWorld) {
    let contract = "scantlings/backend-construction-boundary.json";
    world.boundary_counterexamples = Some(support::check_construction_boundary(contract));
    world.boundary_contract = Some(contract.to_string());
}

#[when("the verifier checks the prepared-process construction boundary")]
async fn verifier_checks_construction_boundary(world: &mut TinmanWorld) {
    let contract = "scantlings/prepared-process-construction-boundary.json";
    world.boundary_counterexamples = Some(support::check_construction_boundary(contract));
    world.boundary_contract = Some(contract.to_string());
}

#[when("the verifier checks the diagnostic stream boundary")]
async fn verifier_checks_diagnostic_stream_boundary(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_seam_references(
        "scantlings/diagnostic-stream-boundary.json",
    ));
}

#[when("the verifier checks the plan deserialization strictness contract")]
async fn verifier_checks_plan_deserialization_strictness(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_deserialization_strictness(
        "scantlings/plan-deserialization-strictness.json",
    ));
}

#[given("the inference module source")]
async fn the_inference_module_source(_world: &mut TinmanWorld) {
    // The encoding boundary contract names the module it guards; nothing to
    // stage here.
}

#[when("the verifier checks the inference encoding boundary")]
async fn verifier_checks_inference_encoding_boundary(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_seam_references(
        "scantlings/inference-encoding-boundary.json",
    ));
}

#[when("the verifier checks the copy catalogue boundary")]
async fn verifier_checks_copy_catalogue_boundary(world: &mut TinmanWorld) {
    world.boundary_counterexamples = Some(support::check_seam_references(
        "scantlings/copy-catalogue-boundary.json",
    ));
}

#[when(expr = "the verifier checks the seams named in {string}")]
async fn verifier_checks_named_seams(world: &mut TinmanWorld, contract: String) {
    world.boundary_counterexamples = Some(support::check_seam_references(&contract));
}

#[then("a construction written with a qualified path is counted the same as a bare one")]
async fn a_qualified_construction_counts_the_same(world: &mut TinmanWorld) {
    let contract = world
        .boundary_contract
        .as_ref()
        .expect("the boundary verifier named the contract it checked");
    let bounded = support::bounded_construction_type(contract);
    let bare = support::constructions_found_in(
        contract,
        &format!("fn probe() {{ let _ = {bounded} {{ field: 1 }}; }}\n"),
    );
    let qualified = support::constructions_found_in(
        contract,
        &format!("fn probe() {{ let _ = outer::inner::{bounded} {{ field: 1 }}; }}\n"),
    );
    // The bare count is the floor. A checker that had stopped seeing either
    // spelling would count both at zero, and an equality alone would read that
    // as the two being counted the same.
    assert!(
        bare > 0,
        "the checker counts no bare construction of {bounded}, so it counts nothing at all"
    );
    assert_eq!(
        qualified, bare,
        "the checker counts {bare} bare construction(s) of {bounded} and {qualified} written \
         with a qualified path, so a launch path spelling the type in full passes the boundary"
    );
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

#[given(expr = "a sandbox specification granting {string} from the host")]
async fn spec_granting_from_the_host(world: &mut TinmanWorld, name: String) {
    let mut spec = SandboxSpec::default_for_record();
    spec.env.insert(
        name.clone(),
        EnvGrant {
            from: EnvOrigin::Host,
        },
    );
    world.granted_env = Some(name);
    world.spec = Some(spec);
}

#[given(expr = "a sandbox specification mounting the fixture tree in {string} mode")]
async fn spec_mounting_the_fixture_tree(world: &mut TinmanWorld, mode: String) {
    // A real tree with a real file in it, so a copy mount has something to copy
    // and the target has something to read.
    let tree = support::ScratchDir::new("fixture-tree");
    std::fs::write(tree.path().join("notes.txt"), "hello from the fixture tree")
        .expect("the fixture tree is seeded");
    let mut spec = SandboxSpec::default_for_record();
    spec.mounts.push(Mount {
        source: tree.path().display().to_string(),
        target: "/fixture".to_string(),
        mode: match mode.as_str() {
            "readonly" => MountMode::Readonly,
            "copy" => MountMode::Copy,
            "writable" => MountMode::Writable,
            other => panic!("the scenario names the mount mode {other}, which is not a mode"),
        },
    });
    world.spec = Some(spec);
    world.fixture_tree = Some(tree);
}

#[given(expr = "the backend is handed an environment defining {string} as {string}")]
async fn the_backend_is_handed_an_environment(
    world: &mut TinmanWorld,
    name: String,
    value: String,
) {
    world
        .handed_env
        .get_or_insert_with(std::collections::BTreeMap::new)
        .insert(name, value);
}

#[then(expr = "the prepared process carries {string} as {string}")]
async fn prepared_process_carries_as(world: &mut TinmanWorld, name: String, value: String) {
    let prepared = world.prepared.as_ref().expect("a process was prepared");
    let carried = prepared
        .env
        .iter()
        .find(|(key, _)| key == &name)
        .unwrap_or_else(|| {
            let named: Vec<&str> = prepared.env.iter().map(|(key, _)| key.as_str()).collect();
            panic!(
                "the prepared process carries no environment pair for {name}; it carries {named:?}"
            )
        });
    assert_eq!(
        carried.1, value,
        "the prepared process carries {name} as {:?} rather than {value:?}",
        carried.1
    );
}

#[when("the Bubblewrap backend prepares the process")]
async fn bwrap_prepares_process(world: &mut TinmanWorld) {
    // A scenario naming a specification is asserting on what the sandbox grants
    // rather than on a command of its own, so it gets the process that reports
    // what it was granted. A scenario staging a command keeps it.
    let command = match world.command.as_ref() {
        Some(command) => command.clone(),
        None => reporting_process(world.granted_env.as_deref().unwrap_or("PATH")),
    };
    // A scenario staging a parsed specification keeps it. One staging its plan
    // section gets that section through the production parser, so the grants
    // the plan names reach the backend rather than an empty default.
    let spec = match world.spec.clone() {
        Some(spec) => spec,
        None if !world.plan_sources.is_empty() => sandbox_section(world),
        None => SandboxSpec::default_for_record(),
    };
    let prepared = staged_backend(world)
        .prepare(&spec, &command)
        .expect("the Bubblewrap backend prepares the process");
    world.serialized = Some(to_json(&prepared));
    world.prepared = Some(prepared);
}

#[then(expr = "the prepared process names {string} among its environment pairs")]
async fn prepared_names_environment_pair(world: &mut TinmanWorld, name: String) {
    let prepared = world.prepared.as_ref().expect("a process was prepared");
    let named: Vec<&str> = prepared.env.iter().map(|(key, _)| key.as_str()).collect();
    assert!(
        named.contains(&name.as_str()),
        "the prepared process carries no environment pair for {name}; it carries {named:?}"
    );
}

#[then(expr = "the launched program reads {string} from that variable")]
async fn launched_program_reads_from_that_variable(world: &mut TinmanWorld, value: String) {
    let prepared = world.prepared.as_ref().expect("a process was prepared");
    world.screen = Some(tinman::pty::capture(prepared).expect("the process is captured"));
    let read = reported_field(world, "ENVVAL");
    assert_eq!(
        read, value,
        "the launched program read {read:?} from the granted variable rather than {value:?}"
    );
}

#[then("the prepared process names that staging directory among the resources to reclaim")]
async fn prepared_names_the_staging_directory(world: &mut TinmanWorld) {
    let prepared = world.prepared.as_ref().expect("a process was prepared");
    let tree = world
        .fixture_tree
        .as_ref()
        .expect("a fixture tree was made");
    let source = tree.path().display().to_string();
    // The staging directory is a copy the backend made of the source tree, so it
    // is a resource the run created and none of the paths the scenario supplied.
    let staged: Vec<&str> = prepared
        .cleanup
        .iter()
        .map(|path| path.as_str())
        .filter(|path| *path != source)
        .collect();
    assert_eq!(
        staged.len(),
        1,
        "the prepared process names {} staging director(ies) to reclaim, expected the one the copy \
         mount created; it names {:?}",
        staged.len(),
        prepared.cleanup
    );
    let staging = std::path::PathBuf::from(staged[0]);
    assert!(
        staging.is_dir(),
        "the prepared process names {} to reclaim, which is no directory",
        staging.display()
    );
    world.staging_dir = Some(staging);
}

#[then("that directory no longer exists once the process has ended")]
async fn that_directory_no_longer_exists(world: &mut TinmanWorld) {
    let staging = world
        .staging_dir
        .clone()
        .expect("a staging directory was named");
    let prepared = world.prepared.as_ref().expect("a process was prepared");
    tinman::pty::capture(prepared).expect("the process is captured");
    assert!(
        !staging.exists(),
        "the staging directory {} outlived the process it was staged for",
        staging.display()
    );
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
    // Hand the secret to the backend as the operator's environment. Bubblewrap
    // is handed this environment and must clear it, so a leak into the sandbox
    // is a real leak of the operator's secret.
    world
        .handed_env
        .get_or_insert_with(std::collections::BTreeMap::new)
        .insert(name.clone(), value.clone());
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
    let prepared = staged_backend(world)
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

/// The single-column rows of a step's data table, the header row dropped.
fn table_column(step: &cucumber::gherkin::Step) -> Vec<String> {
    step.table
        .as_ref()
        .expect("the step carries a data table")
        .rows
        .iter()
        .skip(1)
        .map(|row| {
            row.first()
                .expect("each table row carries a cell")
                .trim()
                .to_string()
        })
        .collect()
}

#[given("the Tinman command lines in the fenced blocks of these documents")]
async fn the_command_lines_in_these_documents(
    world: &mut TinmanWorld,
    step: &cucumber::gherkin::Step,
) {
    let mut lines = Vec::new();
    for document in table_column(step) {
        let text = std::fs::read_to_string(&document)
            .unwrap_or_else(|e| panic!("document {document} unreadable: {e}"));
        lines.extend(support::named_command_lines(&text));
    }
    world.named_command_lines = Some(lines);
}

/// What one command answered when it was asked for help.
#[derive(Debug)]
struct CommandHelp {
    command: String,
    stdout: String,
    stderr: String,
}

impl CommandHelp {
    /// Everything the command wrote, so a step asking what it reported reads
    /// the answer wherever the command chose to put it.
    fn reported(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

/// The commands a scenario names. A scenario naming them in a table pins the
/// set exactly; a scenario naming none means every command the parser accepts,
/// which is the set an operator meets. The two sources are recorded apart, so a
/// step comparing the parser against the named set can refuse the second form
/// rather than compare the parser to itself and pass.
#[given("the commands Tinman names")]
async fn the_commands_tinman_names(world: &mut TinmanWorld, step: &cucumber::gherkin::Step) {
    match step.table.is_some() {
        true => {
            world.named_command_set = Some(table_column(step));
            world.named_command_set_from_table = true;
        }
        false => {
            world.named_command_set = Some(tinman::cli::accepted_commands());
            world.named_command_set_from_table = false;
        }
    }
}

/// The long options the parser accepts for a command, read out of the OPTIONS
/// section of the manual page the parser itself emits. clap builds the page and
/// the help text from one command definition, so the page is the parser's own
/// answer to what a command accepts and no second list is kept to drift from it.
fn documented_options(page: &str) -> Vec<String> {
    let mut options = Vec::new();
    let mut in_options = false;
    for line in page.lines() {
        if let Some(section) = line.strip_prefix(".SH ") {
            in_options = section.trim() == "OPTIONS";
            continue;
        }
        if !in_options {
            continue;
        }
        for chunk in line.split(r"\fB\-\-").skip(1) {
            let Some(name) = chunk.split(r"\fR").next() else {
                continue;
            };
            options.push(format!("--{}", name.replace(r"\-", "-")));
        }
    }
    options
}

/// Ask each named command for help, keeping what it wrote and where it put it.
#[when("each is asked for help")]
async fn each_is_asked_for_help(world: &mut TinmanWorld) {
    let commands = world
        .named_command_set
        .clone()
        .expect("the scenario named its commands");
    let dir = working_dir(world);
    let env = configured_env(world);
    let mut answers = Vec::new();
    for command in commands {
        let outcome = support::run_tinman_awaited(
            dir.clone(),
            vec![command.clone(), "--help".to_string()],
            env.clone(),
            None,
        )
        .await
        .unwrap_or_else(|e| panic!("the tinman binary did not run: {e}"));
        answers.push(CommandHelp {
            command,
            stdout: outcome.stdout,
            stderr: outcome.stderr,
        });
    }
    world.command_help = Some(answers);
}

/// What the commands answered when they were asked for help, refusing an empty
/// set: a step reading no answer at all would assert nothing.
fn asked_for_help(world: &TinmanWorld) -> &[CommandHelp] {
    let answers = world
        .command_help
        .as_deref()
        .expect("the commands were asked for help");
    assert!(
        !answers.is_empty(),
        "no command was asked for help, so this scenario would assert nothing"
    );
    answers
}

#[then("every one reports its own usage")]
async fn every_one_reports_its_own_usage(world: &mut TinmanWorld) {
    for answer in asked_for_help(world) {
        let usage = format!("Usage: tinman {}", answer.command);
        assert!(
            answer.reported().contains(&usage),
            "asked for help, {:?} reported no {usage:?} line, so it does not report its own \
             usage; it wrote to the data stream:\n{}\nand to the error stream:\n{}",
            answer.command,
            answer.stdout,
            answer.stderr
        );
    }
}

#[then("none reports an unexpected argument")]
async fn none_reports_an_unexpected_argument(world: &mut TinmanWorld) {
    for answer in asked_for_help(world) {
        assert!(
            !answer.reported().contains("unexpected argument"),
            "asked for help, {:?} reported an unexpected argument; it wrote to the data \
             stream:\n{}\nand to the error stream:\n{}",
            answer.command,
            answer.stdout,
            answer.stderr
        );
    }
}

#[given("the command definitions the parser carries")]
async fn the_command_definitions_the_parser_carries(world: &mut TinmanWorld) {
    use clap::CommandFactory;
    let mut root = tinman::cli::Cli::command();
    root.build();
    let definitions = root
        .get_subcommands()
        .map(|subcommand| {
            let options = subcommand
                .get_arguments()
                .filter_map(|argument| argument.get_long())
                .map(|long| format!("--{long}"))
                .collect();
            (subcommand.get_name().to_string(), options)
        })
        .collect();
    world.command_definitions = Some(definitions);
}

#[when("the manual page for each command is read")]
async fn the_manual_page_for_each_command_is_read(world: &mut TinmanWorld) {
    let definitions = world
        .command_definitions
        .clone()
        .expect("the starting state read the parser's command definitions");
    let pages = definitions
        .iter()
        .map(|(command, _)| {
            let page = tinman::cli::manual_page(Some(command))
                .unwrap_or_else(|| panic!("the parser emits no manual page for {command:?}"));
            (command.clone(), page)
        })
        .collect();
    world.manual_pages = Some(pages);
}

#[then("every command definition was read")]
async fn every_command_definition_was_read(world: &mut TinmanWorld) {
    let definitions = world
        .command_definitions
        .as_ref()
        .expect("the starting state read the parser's command definitions");
    let pages = world
        .manual_pages
        .as_ref()
        .expect("a manual page was read for each command");
    let accepted = tinman::cli::accepted_commands();
    assert!(
        !definitions.is_empty(),
        "the parser carries no command definitions, so this scenario would assert nothing"
    );
    let read: Vec<&String> = definitions.iter().map(|(command, _)| command).collect();
    assert_eq!(
        read.len(),
        accepted.len(),
        "the definitions read are {read:?}, against the commands the parser accepts {accepted:?}"
    );
    assert_eq!(
        pages.len(),
        definitions.len(),
        "a page was read for {} of the {} commands defined",
        pages.len(),
        definitions.len()
    );
}

#[then("each one names every option it declares")]
async fn each_one_names_every_option_it_declares(world: &mut TinmanWorld) {
    let definitions = world
        .command_definitions
        .as_ref()
        .expect("the starting state read the parser's command definitions");
    let pages = world
        .manual_pages
        .as_ref()
        .expect("a manual page was read for each command");
    assert!(
        definitions.iter().any(|(_, options)| !options.is_empty()),
        "no command was found to declare an option, so this step would assert nothing"
    );
    for ((command, declared), (paged, page)) in definitions.iter().zip(pages) {
        assert_eq!(
            command, paged,
            "the pages were read in a different order from the definitions"
        );
        let documented = documented_options(page);
        let missing: Vec<&String> = declared
            .iter()
            .filter(|option| !documented.contains(option))
            .collect();
        assert!(
            missing.is_empty(),
            "the manual page for {command:?} names none of {missing:?}, which that command \
             declares; the page names {documented:?}"
        );
    }
}

#[then("the parser accepts every command line")]
async fn the_parser_accepts_every_command_line(world: &mut TinmanWorld) {
    let rejected = world
        .command_line_rejections
        .as_ref()
        .expect("the command lines were passed to the parser");
    assert!(
        rejected.is_empty(),
        "command lines the documents name but the parser refuses: {rejected:?}"
    );
}

#[then("the command lines read are not empty")]
async fn the_command_lines_read_are_not_empty(world: &mut TinmanWorld) {
    let lines = world
        .named_command_lines
        .as_ref()
        .expect("the documents' command lines were read");
    assert!(
        !lines.is_empty(),
        "no Tinman command line was read from the documents, \
         so this scenario would assert nothing"
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

#[then("the context carries the body of each of these scantlings")]
async fn the_context_carries_each_scantling(
    world: &mut TinmanWorld,
    step: &cucumber::gherkin::Step,
) {
    let context = world.skill_context.as_ref().expect("a context was built");
    let named = table_column(step);
    assert!(
        !named.is_empty(),
        "the step names no scantling, so this scenario would assert nothing"
    );
    let mut missing = Vec::new();
    for path in named {
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("scantling {path} unreadable: {e}"));
        let body = body.trim().to_string();
        assert!(
            !body.is_empty(),
            "the scantling at {path} is empty, so this scenario would assert nothing"
        );
        if !context.contains(&body) {
            missing.push(path);
        }
    }
    assert!(
        missing.is_empty(),
        "the assistant context omits the body of: {missing:?}"
    );
}

#[given("the scenario names in the specs")]
async fn the_scenario_names_in_the_specs(world: &mut TinmanWorld) {
    world.spec_scenarios = Some(support::spec_scenarios());
}

#[then("the context carries no scenario name from the specs")]
async fn the_context_carries_no_scenario_name(world: &mut TinmanWorld) {
    let context = world.skill_context.as_ref().expect("a context was built");
    let scenarios = world
        .spec_scenarios
        .as_ref()
        .expect("the specs' scenario names were read");
    let carried: Vec<&str> = scenarios
        .iter()
        .map(|scenario| scenario.name.as_str())
        .filter(|name| context.contains(name))
        .collect();
    assert!(
        carried.is_empty(),
        "the assistant context carries scenario names from the specs: {carried:?}"
    );
}

#[then("the scenario names read are not empty")]
async fn the_scenario_names_read_are_not_empty(world: &mut TinmanWorld) {
    let scenarios = world
        .spec_scenarios
        .as_ref()
        .expect("the specs' scenario names were read");
    assert!(
        !scenarios.is_empty(),
        "no scenario name was read from the specs, so this scenario would assert nothing"
    );
}

#[given(expr = "the roles the asset at {string} names")]
async fn the_roles_the_asset_names(world: &mut TinmanWorld, path: String) {
    let asset =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}"));
    world.named_roles = Some(support::named_roles(&asset));
}

#[when(expr = "each is matched against the roles the {string} schema declares")]
async fn each_is_matched_against_declared_roles(world: &mut TinmanWorld, schema: String) {
    let path = format!("scantlings/{schema}.schema.json");
    let declared = support::declared_roles(&path);
    assert!(
        !declared.is_empty(),
        "the schema at {path} declares no role, so this scenario would assert nothing"
    );
    let named = world
        .named_roles
        .as_ref()
        .expect("the asset's roles were read");
    world.undeclared_roles = Some(
        named
            .iter()
            .filter(|role| !declared.contains(role))
            .cloned()
            .collect(),
    );
}

#[then("every role the skill names is a role the model produces")]
async fn every_role_the_skill_names_is_produced(world: &mut TinmanWorld) {
    let undeclared = world
        .undeclared_roles
        .as_ref()
        .expect("the roles were matched against the schema");
    assert!(
        undeclared.is_empty(),
        "roles the skill names that the model never produces: {undeclared:?}"
    );
}

#[then("the roles read are not empty")]
async fn the_roles_read_are_not_empty(world: &mut TinmanWorld) {
    let named = world
        .named_roles
        .as_ref()
        .expect("the asset's roles were read");
    assert!(
        !named.is_empty(),
        "no role was read from the asset, so this scenario would assert nothing"
    );
}

#[given(expr = "the JSON messages in the fenced blocks of the asset at {string}")]
async fn the_json_messages_in_the_fenced_blocks(world: &mut TinmanWorld, path: String) {
    let asset =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}"));
    let mut documents = Vec::new();
    for block in support::fenced_blocks(&asset, "json") {
        for line in block.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("the asset's JSON message {line:?} did not parse: {e}"));
            documents.push((line.trim().to_string(), value));
        }
    }
    world.fenced_documents = Some(documents);
}

#[given(expr = "the YAML plans in the fenced blocks of the asset at {string}")]
async fn the_yaml_plans_in_the_fenced_blocks(world: &mut TinmanWorld, path: String) {
    let asset =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}"));
    let documents = support::fenced_blocks(&asset, "yaml")
        .into_iter()
        .map(|block| {
            let value: serde_json::Value = serde_yaml::from_str(&block).unwrap_or_else(|e| {
                panic!("the asset's YAML plan did not parse: {e}\nit reads:\n{block}")
            });
            (block, value)
        })
        .collect();
    world.fenced_documents = Some(documents);
}

#[when(expr = "each is checked against the {string} schema in {string}")]
async fn each_is_checked_against_the_schema(
    world: &mut TinmanWorld,
    _schema_id: String,
    path: String,
) {
    let documents = world
        .fenced_documents
        .as_ref()
        .expect("the asset's fenced documents were read");
    let mut violations = Vec::new();
    for (label, value) in documents {
        for bad in support::schema_counterexamples(&path, value) {
            violations.push(format!("{label}: {bad}"));
        }
    }
    world.schema_violations = Some(violations);
}

#[then("every message conforms")]
async fn every_message_conforms(world: &mut TinmanWorld) {
    let violations = world
        .schema_violations
        .as_ref()
        .expect("the messages were checked against the schema");
    assert!(
        violations.is_empty(),
        "driver messages the skill teaches that the protocol refuses: {violations:#?}"
    );
}

#[then("the messages read are not empty")]
async fn the_messages_read_are_not_empty(world: &mut TinmanWorld) {
    let documents = world
        .fenced_documents
        .as_ref()
        .expect("the asset's messages were read");
    assert!(
        !documents.is_empty(),
        "no JSON message was read from the asset, so this scenario would assert nothing"
    );
}

#[then("every plan conforms")]
async fn every_plan_conforms(world: &mut TinmanWorld) {
    let violations = world
        .schema_violations
        .as_ref()
        .expect("the plans were checked against the schema");
    assert!(
        violations.is_empty(),
        "plans the skill teaches that the plan schema refuses: {violations:#?}"
    );
}

#[then("the plans read are not empty")]
async fn the_plans_read_are_not_empty(world: &mut TinmanWorld) {
    let documents = world
        .fenced_documents
        .as_ref()
        .expect("the asset's plans were read");
    assert!(
        !documents.is_empty(),
        "no YAML plan was read from the asset, so this scenario would assert nothing"
    );
}

// ---------------------------------------------------------------------------
// conventional help: the bundled help asset, rendered with no inference
// ---------------------------------------------------------------------------

#[given("inference is available")]
async fn inference_is_available(world: &mut TinmanWorld) {
    let answer = "Tinman Inspects Numerous Machine Agent Nodes";
    let provider = support::LocalProvider::returning(answer);
    world.provider_answer = Some(answer.to_string());
    use_provider(world, provider);
}

#[given(expr = "the provider answers {string}")]
async fn the_provider_answers(world: &mut TinmanWorld, answer: String) {
    let provider = support::LocalProvider::returning(&answer);
    world.provider_answer = Some(answer);
    use_provider(world, provider);
}

#[given(expr = "the provider always answers {string}")]
async fn the_provider_always_answers(world: &mut TinmanWorld, answer: String) {
    // A local provider answers every request it receives with the same content,
    // so a second ask gets the same miss as the first, which is the provider
    // that keeps missing.
    let provider = support::LocalProvider::returning(&answer);
    world.provider_answer = Some(answer);
    use_provider(world, provider);
}

/// The wall-clock ceiling the latency contract puts on `seam`.
fn ceiling_for(seam: &str) -> std::time::Duration {
    let contract = std::fs::read_to_string("scantlings/latency-budgets.json")
        .expect("the latency contract scantlings/latency-budgets.json is readable");
    let value: serde_json::Value =
        serde_json::from_str(&contract).expect("the latency contract is JSON");
    let budgets = value["budgets"]
        .as_array()
        .expect("the latency contract lists budgets")
        .clone();
    let budget = budgets
        .iter()
        .find(|budget| budget["seam"] == seam)
        .unwrap_or_else(|| panic!("the latency contract names no seam {seam:?}"));
    let ceiling = budget["ceilingMs"]
        .as_u64()
        .expect("the seam carries a ceiling in milliseconds");
    std::time::Duration::from_millis(ceiling)
}

#[when("the tagline is generated")]
async fn the_tagline_is_generated(world: &mut TinmanWorld) {
    // The tagline fills only on a terminal, so the help runs on one and the
    // line an operator would read is the line the assertions read.
    let dir = working_dir(world);
    let env = configured_env(world);
    let started = std::time::Instant::now();
    let outcome = support::run_tinman_on_a_terminal(&dir, &["--help"], &env)
        .unwrap_or_else(|e| panic!("running the help on a terminal failed: {e}"));
    world.run_elapsed = Some(started.elapsed());
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
    world.run_raw = Some(outcome.raw);
}

#[then(expr = "the tagline reads {string}")]
async fn the_tagline_reads(world: &mut TinmanWorld, expected: String) {
    assert_eq!(tagline_line(world), expected, "tagline");
}

#[then("every raised letter begins a word")]
async fn every_raised_letter_begins_a_word(world: &mut TinmanWorld) {
    let tagline = tagline_line(world);
    // A raised letter is a capital in an expansion whose words are otherwise
    // lower case, so a raised letter that begins no word is exactly a capital
    // with a letter before it.
    let characters: Vec<char> = tagline.chars().collect();
    let stranded: Vec<String> = characters
        .iter()
        .enumerate()
        .filter(|(index, character)| {
            character.is_uppercase() && *index > 0 && characters[index - 1].is_alphanumeric()
        })
        .map(|(index, character)| format!("{character:?} at {index}"))
        .collect();
    assert!(
        stranded.is_empty(),
        "the tagline raises letters that begin no word: {stranded:?}; it reads {tagline:?}"
    );
}

#[then("that expansion is not on the tagline line")]
async fn that_expansion_is_not_on_the_tagline_line(world: &mut TinmanWorld) {
    let answered = world
        .provider_answer
        .clone()
        .expect("the scenario staged the provider's answer");
    let tagline = tagline_line(world);
    assert!(
        !tagline.to_lowercase().contains(&answered.to_lowercase()),
        "the tagline line carries the expansion {answered:?} the provider gave; it reads {tagline:?}"
    );
}

#[then("the tagline line settles inside the tagline ceiling")]
async fn the_tagline_line_settles_inside_the_ceiling(world: &mut TinmanWorld) {
    let ceiling = ceiling_for("tagline generation");
    let elapsed = world.run_elapsed.expect("the help run was timed");
    // Reading the line proves it settled on something rather than being left
    // mid-flight.
    let tagline = tagline_line(world);
    assert!(
        !tagline.is_empty(),
        "the tagline line settled on nothing at all"
    );
    // The ceiling bounds the wait the tagline imposed rather than the whole
    // command, so the same help runs once against a provider that answers at
    // once and that run is the baseline the wait is measured above. The
    // allowance is measured here rather than chosen, so no constant stands in
    // for the machine.
    //
    // The baseline keeps the scenario's own configuration and replaces only the
    // endpoint, so it draws the same help and opens the same assistant. A
    // baseline run with no credential opens no assistant at all, and the cost
    // of opening one would then fall inside the wait rather than beside it.
    let dir = working_dir(world);
    let prompt = support::LocalProvider::returning("Tinman Inspects Numerous Machine Agent Nodes");
    let mut baseline_env = world.env_vars.clone();
    baseline_env.insert("TINMAN_BASE_URL".to_string(), prompt.base_url().to_string());
    let baseline_env: Vec<(String, String)> = baseline_env.into_iter().collect();
    let baseline_started = std::time::Instant::now();
    support::run_tinman_on_a_terminal(&dir, &["--help"], &baseline_env)
        .unwrap_or_else(|e| panic!("running the help against a provider that answers failed: {e}"));
    let baseline = baseline_started.elapsed();
    let waited = elapsed.saturating_sub(baseline);
    // Both runs are timed by polling for the program's exit, so each duration is
    // known only to the poll interval and a wait that ends exactly on its
    // ceiling is observed just past it. The interval is the instrument's
    // resolution, read from the harness that took the measurement; it is not an
    // allowance on the product's budget, and an overshoot of any real size still
    // reddens.
    let resolution = support::EXIT_POLL_INTERVAL;
    assert!(
        waited <= ceiling + resolution,
        "the tagline line settled after a {}ms wait, past its {}ms ceiling read to a {}ms \
         resolution; the run took {}ms against a {}ms baseline; it reads {tagline:?}",
        waited.as_millis(),
        ceiling.as_millis(),
        resolution.as_millis(),
        elapsed.as_millis(),
        baseline.as_millis()
    );
}

#[when(expr = "the operator runs {string} with stdout redirected to a file")]
async fn operator_runs_redirected(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let out = dir.join("stdout.txt");
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let started = std::time::Instant::now();
    let outcome = support::run_tinman(&dir, &argv, &env, Some(&out))
        .unwrap_or_else(|e| panic!("running {line:?} failed: {e}"));
    world.run_elapsed = Some(started.elapsed());
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
}

#[when(expr = "the operator executes {string}")]
async fn operator_executes(world: &mut TinmanWorld, line: String) {
    execute_command_line(world, &line).await;
}

/// A plan a probe already wrote, which is the starting state of a round trip:
/// the command runs for real and has to hold, because a probe that failed
/// writes no plan and the run that follows would be replaying a file that was
/// never there.
#[given(expr = "a plan written by {string}")]
async fn a_plan_written_by(world: &mut TinmanWorld, line: String) {
    execute_command_line(world, &line).await;
    let status = world.run_status.expect("a command was run");
    assert_eq!(
        status,
        0,
        "the probe {line:?} exited {status}, so it wrote no plan to replay; it wrote to the data \
         stream:\n{}\nand to the error stream:\n{}",
        run_output(world),
        run_errors(world)
    );
}

/// Run `line` as the operator's own command line, keeping everything the run
/// produced, so a step asserting on a run and a step staging one from a run
/// observe the same thing.
///
/// The run is awaited rather than blocked on, because this step body is polled
/// on the runner's one executor thread and a blocking call here holds every
/// scenario beside it. The command lines this drives launch real sandboxed
/// programs, so the wait is the whole cost of the scenario and the worker count
/// buys nothing while it is spent blocking.
async fn execute_command_line(world: &mut TinmanWorld, line: &str) {
    let dir = working_dir(world);
    let args = tinman_args(line);
    let env = configured_env(world);
    let started = std::time::Instant::now();
    let outcome = support::run_tinman_awaited(dir, args.clone(), env, None)
        .await
        .unwrap_or_else(|e| panic!("running {line:?} failed: {e}"));
    world.run_elapsed = Some(started.elapsed());
    world.run_stdout = Some(outcome.stdout);
    // A failing run writes its diagnosis to stderr, so a step asserting what a
    // failure reported reads nothing unless the run's error stream is kept.
    world.run_stderr = Some(outcome.stderr);
    world.run_status = Some(outcome.status);
    world.run_argv = Some(args);
}

#[then(expr = "the output begins with a roff title macro naming {string}")]
async fn the_output_begins_with_a_roff_title_macro(world: &mut TinmanWorld, name: String) {
    let output = run_output(world);
    let first = output
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_else(|| panic!("the command wrote nothing"));
    assert!(
        first.starts_with(".TH"),
        "the output opens with {first:?} rather than a roff title macro"
    );
    assert!(
        first.to_lowercase().contains(&name.to_lowercase()),
        "the roff title macro {first:?} does not name {name:?}"
    );
}

#[then("the output names every command the parser accepts")]
async fn the_output_names_every_command(world: &mut TinmanWorld) {
    let output = run_output(world);
    let accepted = tinman::cli::accepted_commands();
    assert!(
        !accepted.is_empty(),
        "the parser accepts no command, so this scenario would assert nothing"
    );
    let missing: Vec<&String> = accepted
        .iter()
        .filter(|command| !output.contains(command.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "the output names no {missing:?}, though the parser accepts them; it reads:\n{output}"
    );
}

#[then(expr = "the output carries no {string} annotation")]
async fn the_output_carries_no_annotation(world: &mut TinmanWorld, marker: String) {
    let output = run_output(world);
    assert!(
        !output.is_empty(),
        "the command wrote nothing, so this step would assert nothing"
    );
    let carrying: Vec<&str> = output
        .lines()
        .filter(|line| line.contains(&marker))
        .collect();
    assert!(
        carrying.is_empty(),
        "the output carries the {marker:?} annotation on {} line(s): {carrying:?}",
        carrying.len()
    );
}

#[then("the NAME section is a single line")]
async fn the_name_section_is_a_single_line(world: &mut TinmanWorld) {
    let section = support::roff_section(run_output(world), "NAME");
    assert_eq!(
        section.len(),
        1,
        "the NAME section runs to {} lines: {section:?}",
        section.len()
    );
    world.roff_section = Some(section);
}

#[then(expr = "it carries the one-line description from the asset at {string}")]
async fn it_carries_the_one_line_description(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    let wanted = support::asset_one_line_description(&asset, tinman::help::TAGLINE_PLACEHOLDER);
    assert!(
        !wanted.is_empty(),
        "the asset at {path} carries no one-line description, so this step would assert nothing"
    );
    let section = world
        .roff_section
        .clone()
        .expect("the NAME section was read");
    let line = section.join(" ");
    assert!(
        line.contains(&wanted),
        "the NAME section reads {line:?}, and the asset's one-line description is {wanted:?}"
    );
}

#[then(expr = "the DESCRIPTION section carries the closing paragraph of the asset at {string}")]
async fn the_description_carries_the_closing_paragraph(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    let wanted = support::asset_closing_paragraph(&asset);
    assert!(
        !wanted.is_empty(),
        "the asset at {path} carries no closing paragraph, so this step would assert nothing"
    );
    let section = support::roff_section(run_output(world), "DESCRIPTION").join(" ");
    // roff wraps the paragraph over several lines, so the section is compared as
    // one run of words rather than line by line.
    let normalise = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalise(&section).contains(&normalise(&wanted)),
        "the DESCRIPTION section reads {section:?}, and the asset's closing paragraph is {wanted:?}"
    );
}

#[given("the subcommand pages the man page cross-references")]
async fn the_subcommand_pages_cross_referenced(world: &mut TinmanWorld) {
    let dir = working_dir(world);
    let env = configured_env(world);
    let outcome = support::run_tinman(&dir, &["man"], &env, None)
        .unwrap_or_else(|e| panic!("the tinman binary did not run: {e}"));
    world.cross_references = Some(support::man_cross_references(&outcome.stdout));
}

#[when(expr = "each is requested from {string}")]
async fn each_is_requested_from(world: &mut TinmanWorld, line: String) {
    let names = world
        .cross_references
        .clone()
        .expect("the cross-references were read");
    let dir = working_dir(world);
    let env = configured_env(world);
    // `tinman_args` strips only the program name, so the line "tinman man"
    // already yields the subcommand the pages are requested from.
    let args = tinman_args(&line);
    let mut missing = Vec::new();
    for name in &names {
        let mut argv: Vec<&str> = args.iter().map(String::as_str).collect();
        argv.push(name);
        match support::run_tinman(&dir, &argv, &env, None) {
            Ok(outcome) if outcome.status == 0 && !outcome.stdout.trim().is_empty() => {}
            Ok(outcome) => missing.push(format!(
                "{name} exited {} and wrote {} bytes",
                outcome.status,
                outcome.stdout.trim().len()
            )),
            Err(e) => missing.push(format!("{name} did not run: {e}")),
        }
    }
    world.unemitted_pages = Some(missing);
}

#[then("every one is emitted")]
async fn every_cross_reference_is_emitted(world: &mut TinmanWorld) {
    let missing = world
        .unemitted_pages
        .as_ref()
        .expect("each cross-reference was requested");
    assert!(
        missing.is_empty(),
        "pages the man page cross-references but the binary will not emit: {}",
        missing.join(", ")
    );
}

#[then("the cross-references read are not empty")]
async fn the_cross_references_read_are_not_empty(world: &mut TinmanWorld) {
    let names = world
        .cross_references
        .as_ref()
        .expect("the cross-references were read");
    assert!(
        !names.is_empty(),
        "the man page cross-references no subcommand page, so this scenario would assert nothing"
    );
}

#[then("mandoc parses the emitted page and reports no error")]
async fn mandoc_parses_the_emitted_page(world: &mut TinmanWorld) {
    let page = run_output(world).to_string();
    let path = working_dir(world).join("tinman.1");
    std::fs::write(&path, &page).expect("the emitted page is written for mandoc to read");
    // mandoc is the language's own parser, so its own reading of the page is the
    // proof. The severity threshold is mandoc's own: `-W error` reports the
    // errors the scenario names and leaves the style hints below them, which are
    // remarks on the source rather than roff a formatter would stumble on.
    let output = std::process::Command::new("mandoc")
        .arg("-T")
        .arg("lint")
        .arg("-W")
        .arg("error")
        .arg(&path)
        .output()
        .expect("mandoc is installed and runs");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success() && diagnostics.trim().is_empty(),
        "mandoc reports on the emitted page:\n{diagnostics}"
    );
}

#[then("the emitted page is not empty")]
async fn the_emitted_page_is_not_empty(world: &mut TinmanWorld) {
    let page = run_output(world);
    assert!(
        !page.trim().is_empty(),
        "the command emitted no page, so this scenario would assert nothing"
    );
}

#[given(expr = "the Tinman command lines in the Examples block of the asset at {string}")]
async fn the_command_lines_in_the_examples_block(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    world.named_command_lines = Some(support::examples_block_command_lines(&asset));
}

#[then(expr = "the output is a completion script naming {string}")]
async fn the_output_is_a_completion_script(world: &mut TinmanWorld, name: String) {
    let output = run_output(world);
    assert!(
        output.contains(&name),
        "the output names no {name:?}; it reads:\n{output}"
    );
    // A completion script is a shell program that registers a completion for the
    // program it names, so the registration is what separates it from any other
    // output that happens to carry the name.
    assert!(
        output.contains("complete"),
        "the output registers no completion, so it is not a completion script; it reads:\n{output}"
    );
}

// ---------------------------------------------------------------------------
// sandbox availability: a version floor refused before anything is executed
// ---------------------------------------------------------------------------

/// Stage a directory the scenario owns and make it the whole path the launched
/// Tinman searches, so what a sandbox lookup finds is what the scenario put
/// there rather than whatever the host happens to carry.
fn stage_path_dir(world: &mut TinmanWorld, name: &str) -> std::path::PathBuf {
    let dir = working_dir(world).join(name);
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("the staged path directory was not created: {e}"));
    world
        .env_vars
        .insert("PATH".to_string(), dir.display().to_string());
    dir
}

/// The Bubblewrap version this project requires, as `RIGGING.md` declares it
/// under `## Dependencies`. The floor is a configuration value rather than a
/// constant restated here, so a refusal and the rigging cannot drift apart.
fn required_bwrap_version() -> String {
    let rigging = std::fs::read_to_string("RIGGING.md")
        .unwrap_or_else(|e| panic!("RIGGING.md unreadable: {e}"));
    rigging
        .lines()
        .find_map(|line| line.trim().strip_prefix("- dependency: bwrap >= "))
        .map(|version| version.trim().to_string())
        .expect("RIGGING.md declares a Bubblewrap version floor under ## Dependencies")
}

#[given("the Bubblewrap binary is not on the path")]
async fn the_bubblewrap_binary_is_not_on_the_path(world: &mut TinmanWorld) {
    stage_path_dir(world, "path-without-bwrap");
}

#[given(expr = "the Bubblewrap binary reports version {string}")]
async fn the_bubblewrap_binary_reports_version(world: &mut TinmanWorld, version: String) {
    let dir = stage_path_dir(world, "path-with-old-bwrap");
    let shim = dir.join("bwrap");
    std::fs::write(
        &shim,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'bubblewrap {version}'\n  exit 0\nfi\nexit 1\n"
        ),
    )
    .unwrap_or_else(|e| panic!("the bwrap shim was not written: {e}"));
    support::set_executable(&shim);
    world.staged_bwrap_version = Some(version);
}

#[then("the failure reports the required Bubblewrap version")]
async fn the_failure_reports_the_required_version(world: &mut TinmanWorld) {
    let required = required_bwrap_version();
    let errors = run_errors(world);
    assert!(
        errors.contains(&required),
        "the failure does not name {required:?} as the Bubblewrap version required; it reads:\n{errors}"
    );
}

#[then("the failure reports the version found and the version required")]
async fn the_failure_reports_found_and_required(world: &mut TinmanWorld) {
    let required = required_bwrap_version();
    let found = world
        .staged_bwrap_version
        .clone()
        .expect("the scenario staged a Bubblewrap version on the path");
    let errors = run_errors(world);
    assert!(
        errors.contains(&found) && errors.contains(&required),
        "the failure does not name both the {found:?} found and the {required:?} required; \
         it reads:\n{errors}"
    );
}

/// The asset carrying the refusal an unavailable sandbox prints.
const SANDBOX_REFUSAL_ASSET: &str = "assets/help/sandbox-unavailable.txt";

/// The refusal the operator met, read from both streams: the scenario asserts
/// the copy reached them, and which stream carried it is a separate concern.
fn refusal_shown(world: &TinmanWorld) -> String {
    run_all_output(world)
}

/// The paragraph of the refusal asset that carries `marker`, as one line with
/// its wrapping removed. The asset wraps its prose for a terminal, so a
/// paragraph is compared as the words it carries rather than as the lines the
/// file happens to break them into.
fn refusal_paragraph(marker: &str) -> String {
    let body = asset_body(SANDBOX_REFUSAL_ASSET);
    let paragraph = body
        .split("\n\n")
        .find(|paragraph| paragraph.contains(marker))
        .unwrap_or_else(|| {
            panic!("the refusal asset carries no paragraph naming {marker:?}; it reads:\n{body}")
        });
    paragraph.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `text` with its line breaks flattened, so copy wrapped at one width is
/// compared against the same copy wrapped at another.
fn flattened(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[then("the refusal names the required Bubblewrap version")]
async fn the_refusal_names_the_required_version(world: &mut TinmanWorld) {
    let required = required_bwrap_version();
    let shown = refusal_shown(world);
    assert!(
        shown.contains(&required),
        "the refusal does not name {required:?} as the Bubblewrap version required; \
         it reads:\n{shown}"
    );
}

#[then("the refusal explains that no flag lifts the sandbox requirement")]
async fn the_refusal_explains_no_flag_lifts_it(world: &mut TinmanWorld) {
    let wanted = refusal_paragraph("no flag lifts it");
    let shown = flattened(&refusal_shown(world));
    assert!(
        shown.contains(&wanted),
        "the refusal does not carry the asset's explanation {wanted:?}; it reads:\n{shown}"
    );
}

#[then("the refusal carries the install instructions link")]
async fn the_refusal_carries_the_install_link(world: &mut TinmanWorld) {
    let body = asset_body(SANDBOX_REFUSAL_ASSET);
    let link = body
        .split_whitespace()
        .find(|word| word.starts_with("https://"))
        .unwrap_or_else(|| panic!("the refusal asset carries no link; it reads:\n{body}"))
        .trim_end_matches(['.', ','])
        .to_string();
    let shown = refusal_shown(world);
    assert!(
        shown.contains(&link),
        "the refusal does not carry the install instructions link {link:?}; it reads:\n{shown}"
    );
}

#[then("the refusal names the commands that still answer without a sandbox")]
async fn the_refusal_names_the_commands_that_still_answer(world: &mut TinmanWorld) {
    let wanted = refusal_paragraph("still works");
    let shown = flattened(&refusal_shown(world));
    assert!(
        shown.contains(&wanted),
        "the refusal does not name the commands that still answer, as the asset carries them \
         {wanted:?}; it reads:\n{shown}"
    );
}

#[given("the refusal text in the assets")]
async fn the_refusal_text_in_the_assets(world: &mut TinmanWorld) {
    world.refusal_text = Some(asset_body(SANDBOX_REFUSAL_ASSET));
}

#[when("the link it carries is read")]
async fn the_link_it_carries_is_read(world: &mut TinmanWorld) {
    let text = world
        .refusal_text
        .as_ref()
        .expect("the refusal text was read from the assets");
    let link = text
        .split_whitespace()
        .find(|word| word.starts_with("https://"))
        .unwrap_or_else(|| panic!("the refusal text carries no link; it reads:\n{text}"))
        .trim_end_matches(['.', ','])
        .to_string();
    world.refusal_link = Some(link);
}

#[then(expr = "it names a heading the file at {string} carries")]
async fn it_names_a_heading_the_file_carries(world: &mut TinmanWorld, path: String) {
    let link = world
        .refusal_link
        .as_ref()
        .expect("the link was read out of the refusal text");
    let (_, fragment) = link
        .split_once('#')
        .unwrap_or_else(|| panic!("the link {link:?} names no heading at all"));
    let document = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the file at {path} is unreadable: {e}"));
    // A heading is addressed by the anchor a rendered document gives it, so the
    // comparison is against that anchor rather than against the heading text.
    let anchors: Vec<String> = document
        .lines()
        .filter(|line| line.starts_with('#'))
        .map(|line| heading_anchor(line.trim_start_matches('#').trim()))
        .collect();
    assert!(
        anchors.iter().any(|anchor| anchor == fragment),
        "the refusal points at {fragment:?}, which is no heading {path} carries; \
         it carries {anchors:?}"
    );
}

/// The anchor a Markdown heading is reached by: its text lowercased, with runs
/// of space hyphenated and anything neither alphanumeric nor a hyphen dropped.
fn heading_anchor(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            c if c.is_alphanumeric() => Some(c),
            ' ' | '-' => Some('-'),
            _ => None,
        })
        .collect()
}

#[then("no program was executed")]
async fn no_program_was_executed(world: &mut TinmanWorld) {
    // A program that ran would have drawn to the screen inspection reads, so a
    // listing is the observable trace of a launch that happened anyway.
    let output = run_output(world);
    assert!(
        output.trim().is_empty(),
        "inspection reported a screen, so a program was executed; it reads:\n{output}"
    );
}

#[when("the operator starts the driver")]
async fn the_operator_starts_the_driver(world: &mut TinmanWorld) {
    let env = configured_env(world);
    let mut driver = support::DriverProcess::start_with(&env);
    // A driver that refuses as it starts exits on its own. One that does not
    // would wait on its input for as long as it stays open, so the input is
    // closed and the wait ends on the process exit rather than on a clock.
    driver.close_stdin();
    let status = driver.wait_for_exit();
    world.run_status = Some(status.code().unwrap_or(-1));
    world.run_stdout = Some(String::new());
    world.driver = Some(driver);
}

#[then("the driver accepted no connection")]
async fn the_driver_accepted_no_connection(world: &mut TinmanWorld) {
    let driver = world.driver.as_ref().expect("the driver was started");
    let exchanged = driver.exchanged();
    assert!(
        exchanged.is_empty(),
        "the driver exchanged {} message(s) before refusing: {exchanged:?}",
        exchanged.len()
    );
}

#[then("the command exits with a non-zero status")]
async fn the_command_exits_non_zero(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a command was run");
    let output = run_output(world);
    assert_ne!(status, 0, "the command exited successfully:\n{output}");
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

#[then(expr = "the command completed within {int} seconds")]
async fn the_command_completed_within_seconds(world: &mut TinmanWorld, seconds: u64) {
    let elapsed = world
        .run_elapsed
        .expect("the operator's command was run and timed");
    let ceiling = std::time::Duration::from_secs(seconds);
    assert!(
        elapsed <= ceiling,
        "the command took {:.1}s, longer than the {seconds} seconds allowed",
        elapsed.as_secs_f64()
    );
}

#[given("the commands the parser accepts")]
async fn the_commands_the_parser_accepts(world: &mut TinmanWorld) {
    world.accepted_commands = Some(tinman::cli::accepted_commands());
}

#[when("the accepted command set is read")]
async fn the_accepted_command_set_is_read(world: &mut TinmanWorld) {
    let mut commands = tinman::cli::accepted_commands();
    commands.sort();
    commands.dedup();
    world.read_command_set = Some(commands);
}

#[then("it is exactly the commands named")]
async fn the_command_set_is_exactly_the_commands_named(world: &mut TinmanWorld) {
    let read = world
        .read_command_set
        .as_ref()
        .expect("the accepted command set was read");
    let mut named = world
        .named_command_set
        .clone()
        .expect("the scenario named its commands");
    assert!(
        world.named_command_set_from_table,
        "the scenario named its commands by reading them off the parser, so comparing the two \
         would assert nothing; this step needs a scenario that names them in its own table"
    );
    assert!(
        !named.is_empty(),
        "the scenario named no command, so this step would assert nothing"
    );
    named.sort();
    named.dedup();
    assert_eq!(
        read, &named,
        "the parser accepts {read:?}, the scenario names {named:?}"
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

#[given(expr = "the commands listed in the Commands block of the asset at {string}")]
async fn the_commands_listed_in_the_commands_block(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    world.read_command_set = Some(support::listed_commands(&asset));
}

#[then("the parser accepts every listed command")]
async fn the_parser_accepts_every_listed_command(world: &mut TinmanWorld) {
    let rejected = world
        .command_line_rejections
        .as_ref()
        .expect("the listed commands were passed to the parser");
    assert!(
        rejected.is_empty(),
        "commands the Commands block lists but the parser refuses: {rejected:?}"
    );
}

#[then("the commands read are not empty")]
async fn the_commands_read_are_not_empty(world: &mut TinmanWorld) {
    let commands = world
        .read_command_set
        .as_ref()
        .expect("the Commands block was read");
    assert!(
        !commands.is_empty(),
        "the Commands block lists nothing, so this scenario would assert nothing"
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
    // Three scenarios stage three different collections and pass each through
    // the same parser. A scenario stages exactly one, so the collection in hand
    // is what this step passes.
    if let Some(options) = world.advertised_options.clone() {
        let mut rejected = Vec::new();
        for option in &options {
            if let Err(e) = tinman::cli::Cli::try_parse_from(["tinman", option]) {
                // Clap reports a help or version flag as a display request
                // rather than a parse failure: the parser took the option and
                // asked to print. Every other error is the parser refusing it.
                if !matches!(
                    e.kind(),
                    clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
                ) {
                    rejected.push(format!("{option} refused as {:?}", e.kind()));
                }
            }
        }
        world.option_rejections = Some(rejected);
        return;
    }
    if let Some(lines) = world.named_command_lines.clone() {
        let mut rejected = Vec::new();
        for line in &lines {
            if let Err(e) = tinman::cli::parse_command_line(line) {
                rejected.push(format!("{line:?} refused: {e}"));
            }
        }
        world.command_line_rejections = Some(rejected);
        return;
    }
    let commands = world
        .read_command_set
        .clone()
        .expect("a collection was staged for the parser");
    let accepted = tinman::cli::accepted_commands();
    world.command_line_rejections = Some(
        commands
            .iter()
            .filter(|command| !accepted.contains(command))
            .map(|command| format!("{command:?} is not a command the parser accepts"))
            .collect(),
    );
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
    // A session still standing is the interactive case: the program's own exit
    // is the signal this step ends on, bounded by a budget sized to a real run.
    let status = match world.run_status {
        Some(status) => status,
        None => {
            let status = world
                .terminal_session
                .as_mut()
                .expect("a command was run")
                .wait(std::time::Duration::from_secs(20));
            world.run_status = Some(status);
            status
        }
    };
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
    // This provider answers nothing, so a step waiting for a turn to finish has
    // no signal to end on and must not wait for one.
    world.provider_answer = None;
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
// latency budgets: every timed seam observed against its declared ceiling
// ---------------------------------------------------------------------------

#[given(expr = "the seams and ceilings in {string}")]
async fn the_seams_and_ceilings_in(world: &mut TinmanWorld, contract: String) {
    world.latency_budgets = Some(support::latency_budgets(&contract));
}

#[given("a dependency that never answers")]
async fn a_dependency_that_never_answers(world: &mut TinmanWorld) {
    // Every inference seam reaches its dependency over the configured endpoint,
    // so one stalled provider withholds the answer from all of them. It accepts
    // the connection and holds it, which is the fault only a ceiling ends: a
    // refused connection would end the wait on its own and time nothing.
    let provider = support::LocalProvider::stalling();
    world.provider_answer = None;
    use_provider(world, provider);
}

/// The ceiling each timed seam enforces, as production reports it. Reading the
/// enforcement is what makes this deterministic: a wait driven out to its
/// ceiling measures the ceiling, and driving every seam that way costs the sum
/// of them on the tier that carries every inner-loop run.
#[when("the ceiling each seam enforces is read")]
async fn the_ceiling_each_seam_enforces_is_read(world: &mut TinmanWorld) {
    let budgets = world
        .latency_budgets
        .clone()
        .expect("the latency contract was read");
    let read = budgets
        .into_iter()
        .map(|budget| {
            let enforced = tinman::enforced_ceiling(&budget.seam);
            (budget, enforced)
        })
        .collect();
    world.enforced_ceilings = Some(read);
}

#[then("every seam enforces the ceiling declared for it")]
async fn every_seam_enforces_the_ceiling_declared_for_it(world: &mut TinmanWorld) {
    let read = world
        .enforced_ceilings
        .as_ref()
        .expect("the ceiling each seam enforces was read");
    let wrong: Vec<String> = read
        .iter()
        .filter_map(|(budget, enforced)| match enforced {
            None => Some(format!(
                "{} on the {} tier declares a {}ms ceiling and no seam of that \
                 name enforces one, so its ceiling is a number nothing reads",
                budget.seam, budget.tier, budget.ceiling_ms
            )),
            Some(enforced) if enforced.as_millis() != u128::from(budget.ceiling_ms) => {
                Some(format!(
                    "{} on the {} tier enforces {}ms against the {}ms its contract \
                     declares",
                    budget.seam,
                    budget.tier,
                    enforced.as_millis(),
                    budget.ceiling_ms
                ))
            }
            Some(_) => None,
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} timed seam(s) do not enforce the ceiling declared for them:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// How far past a short ceiling an observation may land and still read as the
/// wait ending there. What is timed carries building the request and reaching
/// the stalled provider, and no ceiling governs those, so a margin is owed
/// above the ceiling and none below it.
const SHORT_CEILING_MARGIN: std::time::Duration = std::time::Duration::from_secs(2);

#[given(expr = "a seam whose ceiling is {int} milliseconds")]
async fn a_seam_whose_ceiling_is_milliseconds(world: &mut TinmanWorld, ceiling_ms: u64) {
    world.short_ceiling = Some(std::time::Duration::from_millis(ceiling_ms));
}

/// Drive the enforcement mechanism against the dependency that never answers,
/// with the short ceiling the scenario named. The seam is asserted to have
/// answered nothing: a wait that received an answer timed the dependency rather
/// than the enforcement, so its measurement would say nothing about the ceiling.
#[when("that seam is exercised")]
async fn that_seam_is_exercised(world: &mut TinmanWorld) {
    let ceiling = world.short_ceiling.expect("a seam whose ceiling was named");
    let settings = resolved_settings(world);
    let started = std::time::Instant::now();
    let available = tinman::inference::is_available_within(&settings, ceiling);
    world.short_ceiling_elapsed = Some(started.elapsed());
    assert!(
        !available,
        "the stalled provider was reported available, so what was timed was an \
         answer rather than an abandoned wait"
    );
}

#[then(expr = "it gave up inside {int} milliseconds")]
async fn it_gave_up_inside_milliseconds(world: &mut TinmanWorld, ceiling_ms: u64) {
    let elapsed = world.short_ceiling_elapsed.expect("the seam was exercised");
    let ceiling = std::time::Duration::from_millis(ceiling_ms);
    assert!(
        elapsed <= ceiling + SHORT_CEILING_MARGIN,
        "the seam waited {}ms against its {ceiling_ms}ms ceiling, so the ceiling \
         did not end that wait",
        elapsed.as_millis()
    );
}

#[then("the seams read are not empty")]
async fn the_seams_read_are_not_empty(world: &mut TinmanWorld) {
    let budgets = world
        .latency_budgets
        .as_ref()
        .expect("the latency contract was read");
    assert!(
        !budgets.is_empty(),
        "the latency contract declared no timed seam, so the attestation asserted \
         nothing"
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

#[given("a provider endpoint that records the body it receives")]
async fn a_provider_endpoint_that_records_the_body(world: &mut TinmanWorld) {
    // The local provider logs every body it receives, so recording is what it
    // does rather than a mode it is put into. Only the endpoint moves here: the
    // credential the preceding step configured is left standing, because this
    // scenario asserts what reaches the provider under the operator's own
    // configuration.
    let provider =
        support::LocalProvider::returning("Tinman Inspects Numerous Machine Agent Nodes");
    world.env_vars.insert(
        "TINMAN_BASE_URL".to_string(),
        provider.base_url().to_string(),
    );
    world.provider = Some(provider);
}

#[when("the acronym request is sent to that endpoint")]
async fn the_acronym_request_is_sent_to_that_endpoint(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    // One settings value builds the request and drives the send, so the
    // document the assertions read and the document the provider received come
    // from one configuration. That identity is the join this scenario exists to
    // make: any other pairing would compare two requests rather than two
    // encodings of one.
    world.built_requests = vec![tinman::inference::acronym_request(&settings)];
    // The call returns once the provider has answered, so the recorded body is
    // in hand the moment this step ends; nothing here waits on a clock.
    let _ = tinman::inference::tagline_expansion(&settings);
}

/// The one body the provider recorded, parsed as the JSON document a provider
/// receives. An expansion the generator accepts ends the ask loop on its first
/// pass, so a second recorded body means the send was retried and the scenario
/// is no longer reading the send it describes.
fn recorded_body(world: &TinmanWorld) -> serde_json::Value {
    let provider = world
        .provider
        .as_ref()
        .expect("a recording provider endpoint was started");
    let bodies = provider.received_bodies();
    assert_eq!(
        bodies.len(),
        1,
        "the provider recorded {} bodies rather than the one this scenario sends: {bodies:?}",
        bodies.len()
    );
    serde_json::from_str(&bodies[0]).unwrap_or_else(|e| {
        panic!(
            "the body the provider received is not JSON: {e}; it reads:\n{}",
            bodies[0]
        )
    })
}

/// The payload half of a built request's wire form, which is the half a
/// provider receives. The endpoint and the credential are transport and sit
/// beside it, per the provider contract.
fn built_request_body(world: &TinmanWorld) -> serde_json::Value {
    let built = world.built_requests.first().expect("a request was built");
    let wire = to_json(built);
    wire.get("body")
        .unwrap_or_else(|| {
            panic!("the built request's wire form carries no body field; it reads:\n{wire}")
        })
        .clone()
}

#[then("the recorded body names the model the built request names")]
async fn the_recorded_body_names_the_model(world: &mut TinmanWorld) {
    let recorded = recorded_body(world);
    let built = world.built_requests.first().expect("a request was built");
    assert_eq!(
        recorded.get("model").and_then(|model| model.as_str()),
        Some(built.model()),
        "the sent body names a different model than the built request; it reads:\n{recorded}"
    );
}

#[then("the recorded body carries the temperature the built request carries")]
async fn the_recorded_body_carries_the_temperature(world: &mut TinmanWorld) {
    let recorded = recorded_body(world);
    let built = world.built_requests.first().expect("a request was built");
    assert_eq!(
        recorded
            .get("temperature")
            .and_then(|temperature| temperature.as_f64()),
        Some(built.temperature()),
        "the sent body carries a different temperature than the built request; it reads:\n{recorded}"
    );
}

#[then("the recorded body carries the messages the built request carries")]
async fn the_recorded_body_carries_the_messages(world: &mut TinmanWorld) {
    let recorded = recorded_body(world);
    let declared = built_request_body(world);
    assert_eq!(
        recorded.get("messages"),
        declared.get("messages"),
        "the sent body carries different messages than the built request declares; \
         it reads:\n{recorded}"
    );
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

#[then("the request disables reasoning")]
async fn the_request_disables_reasoning(world: &mut TinmanWorld) {
    // Read off the wire form the provider receives, which is the artifact the
    // request scantling governs, rather than off a field only this suite sees.
    for request in built_requests(world) {
        let wire = to_json(request);
        // The reasoning control is payload rather than transport, so it rides
        // inside the body the provider receives, per the request scantling.
        let body = wire
            .get("body")
            .unwrap_or_else(|| panic!("the request's wire form carries no body field:\n{wire:#}"));
        let reasoning = body.get("reasoning").unwrap_or_else(|| {
            panic!("the request carries no reasoning control, so the provider reasons by default:\n{wire:#}")
        });
        assert_eq!(
            reasoning.get("enabled"),
            Some(&serde_json::Value::Bool(false)),
            "the request's reasoning control does not disable reasoning:\n{wire:#}"
        );
    }
}

// ---------------------------------------------------------------------------
// tldr page source: where a naming pass reads a program's page from
// ---------------------------------------------------------------------------

#[given("no tldr page source is configured")]
async fn no_tldr_page_source_is_configured(world: &mut TinmanWorld) {
    world.env_vars.remove("TINMAN_TLDR_BASE_URL");
    let dir = working_dir(world);
    let dotenv = dir.join(".env");
    assert!(
        !dotenv.exists(),
        "a dotenv file is staged at {}, so this scenario's precondition does not hold",
        dotenv.display()
    );
}

#[when("the page source is read")]
async fn the_page_source_is_read(world: &mut TinmanWorld) {
    let settings = resolved_settings(world);
    world.page_source = Some(settings.tldr_base_url.clone());
}

/// The page source the scenario resolved.
fn page_source(world: &TinmanWorld) -> &str {
    world
        .page_source
        .as_deref()
        .expect("the page source was read")
}

#[then("it is the tldr-pages project's raw markdown")]
async fn it_is_the_tldr_projects_raw_markdown(world: &mut TinmanWorld) {
    // The project's own raw markdown, named by host and repository. The branch
    // and the path below it are production's to choose, so pinning the whole
    // string here would assert a spelling rather than the source.
    let source = page_source(world);
    assert!(
        source.starts_with("https://raw.githubusercontent.com/tldr-pages/tldr/"),
        "the page source {source:?} is not the tldr-pages project's raw markdown"
    );
}

#[then(expr = "it is {string}")]
async fn it_is(world: &mut TinmanWorld, expected: String) {
    assert_eq!(page_source(world), expected, "the page source");
}

// ---------------------------------------------------------------------------
// the naming context: the tldr page a pass over a named program carries, and
// the credit the plan written from it gives that page
// ---------------------------------------------------------------------------

#[when(expr = "the terminal object model of {string} is inferred")]
async fn the_model_of_program_is_inferred(world: &mut TinmanWorld, program: String) {
    let settings = resolved_settings(world);
    let screen = world.screen.as_ref().expect("a virtual screen").clone();
    // The request is the seam these scenarios assert on, so it is kept as the
    // request builder produced it, read later off the wire form the provider
    // would receive.
    world.built_requests = vec![tinman::inference::tom_request(
        &settings,
        &screen.contents(),
        &program,
    )];
    world.tom = Some(tinman::tom::infer_for(&screen, &settings, &program));
}

/// The chat message contents one built request carries, read off the wire form
/// the provider receives rather than off anything the request reports about
/// itself.
fn request_contents(world: &TinmanWorld) -> String {
    built_requests(world)
        .iter()
        .map(|request| {
            let wire = to_json(request);
            // The messages are payload, so they ride inside the body the
            // provider receives rather than beside it, per the request
            // scantling.
            wire["body"]["messages"]
                .as_array()
                .unwrap_or_else(|| panic!("the request carries no messages array:\n{wire:#}"))
                .iter()
                .map(|message| message["content"].as_str().unwrap_or_default().to_string())
                .collect::<Vec<String>>()
                .join("\n")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[then(expr = "the inference request carries the tldr page for {string}")]
async fn the_request_carries_the_tldr_page_for(world: &mut TinmanWorld, program: String) {
    let staged = support::tldr_page(&program, &[]);
    let carried = request_contents(world);
    // The page the source served is the page the request must carry, so the
    // assertion is against that text rather than against the program's name,
    // which the request names for other reasons.
    let body: Vec<&str> = staged
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let missing: Vec<&&str> = body
        .iter()
        .filter(|line| !carried.contains(**line))
        .collect();
    assert!(
        missing.is_empty(),
        "the inference request carries no tldr page for {program:?}; it is missing {missing:?}. \
         The request reads:\n{carried}"
    );
}

#[then("the inference request carries no tldr page")]
async fn the_request_carries_no_tldr_page(world: &mut TinmanWorld) {
    let carried = request_contents(world);
    // A page is recognised by the text the style guide fixes on every one of
    // them, so the absence asserted here is the absence of a page rather than
    // the absence of the word the instruction may use for other reasons.
    let markers = ["> Documentation for", "More information:"];
    let found: Vec<&&str> = markers
        .iter()
        .filter(|marker| carried.contains(**marker))
        .collect();
    assert!(
        found.is_empty(),
        "the inference request carries a tldr page though the source has none for this program; \
         it carries {found:?}. The request reads:\n{carried}"
    );
}

#[when(expr = "the plan for {string} is written")]
async fn the_plan_for_program_is_written(world: &mut TinmanWorld, program: String) {
    let settings = resolved_settings(world);
    let name = world
        .proposed_name
        .clone()
        .expect("an engine named a region");
    let screen = world.screen.as_ref().expect("a virtual screen").clone();
    let inferred = tinman::tom::infer_for(&screen, &settings, &program);
    let deterministic = tinman::tom::build(&screen);
    let role = inferred
        .find_named(&name)
        .unwrap_or_else(|| panic!("the inferred model carries no region named {name:?}"))
        .role()
        .to_string();
    let plan =
        tinman::record::plan_expecting_for(&deterministic, &role, &name, &program, &settings)
            .unwrap_or_else(|e| panic!("no plan was written: {e}"));
    world.written_plan_source =
        Some(serde_yaml::to_string(&plan).expect("the plan serializes to YAML"));
}

/// The plan the scenario wrote, as its YAML source.
fn written_plan_source(world: &TinmanWorld) -> &str {
    world
        .written_plan_source
        .as_deref()
        .expect("a plan was written")
}

#[then("the plan credits the tldr-pages project for the page it read")]
async fn the_plan_credits_the_tldr_project(world: &mut TinmanWorld) {
    let source = written_plan_source(world);
    assert!(
        source.contains("tldr-pages"),
        "the plan credits no tldr-pages project for the page it read; it reads:\n{source}"
    );
}

#[then(expr = "the plan names {string} as that page's licence")]
async fn the_plan_names_the_pages_licence(world: &mut TinmanWorld, licence: String) {
    let source = written_plan_source(world);
    assert!(
        source.contains(&licence),
        "the plan names no {licence:?} as the page's licence; it reads:\n{source}"
    );
}

#[then("the plan credits no page")]
async fn the_plan_credits_no_page(world: &mut TinmanWorld) {
    let source = written_plan_source(world);
    assert!(
        !source.contains("tldr-pages"),
        "the plan credits the tldr-pages project though it read no page; it reads:\n{source}"
    );
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
    let started = std::time::Instant::now();
    // A scenario that put work on the operator's terminal first runs against a
    // terminal carrying it, so the run meets the screen that scenario describes.
    let outcome = match world.scrollback_line.clone() {
        Some(carried) => {
            support::run_tinman_on_a_terminal_after_printing(&dir, &argv, &env, &carried)
                .unwrap_or_else(|e| panic!("running {line:?} on a terminal failed: {e}"))
        }
        None => support::run_tinman_on_a_terminal(&dir, &argv, &env)
            .unwrap_or_else(|e| panic!("running {line:?} on a terminal failed: {e}")),
    };
    world.run_elapsed = Some(started.elapsed());
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
    world.run_raw = Some(outcome.raw);
}

#[when(
    expr = "the operator runs {string} in an interactive terminal with stdin redirected from a file"
)]
async fn operator_runs_interactive_with_stdin_from_a_file(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let stdin_path = dir.join("stdin.txt");
    std::fs::write(&stdin_path, "").expect("the redirected input file is written");
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let outcome = support::run_tinman_on_a_terminal_with_stdin_from(&dir, &argv, &env, &stdin_path)
        .unwrap_or_else(|e| panic!("running {line:?} with stdin redirected failed: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
    world.run_raw = Some(outcome.raw);
}

#[when(expr = "the operator runs {string} in an interactive terminal {int} columns wide")]
async fn operator_runs_interactive_at(world: &mut TinmanWorld, line: String, columns: u16) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let outcome = support::run_tinman_on_a_terminal_at(&dir, &argv, &env, columns)
        .unwrap_or_else(|e| panic!("running {line:?} on a {columns} column terminal failed: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_status = Some(outcome.status);
    world.run_raw = Some(outcome.raw);
    world.run_columns = Some(columns);
}

/// The width of the terminal the run drew on: the width the scenario named, or
/// the width a session opens by default when it named none.
fn run_columns(world: &TinmanWorld) -> u16 {
    world.run_columns.unwrap_or(support::TERMINAL_COLS)
}

/// The bytes the run drew its own interface with. A run that put the terminal
/// on the alternate screen drew there and gave the operator's screen back
/// before it exited, so the screen it ended on is the operator's rather than
/// its own; the span it drew on is what an assertion about its interface reads.
/// A run that drew on the operator's screen is read whole, as it always was.
fn interface_raw(world: &TinmanWorld) -> Vec<u8> {
    let raw = world.run_raw.clone().expect("the terminal output was read");
    support::alternate_screen_span(&raw).unwrap_or(raw)
}

/// The region titled `title` in the model built from what the run drew, read on
/// the grid the program addressed.
fn run_region(world: &TinmanWorld, title: &str) -> tinman::tom::Region {
    let raw = interface_raw(world);
    let screen = support::terminal_screen_at(&raw, run_columns(world));
    tinman::tom::build(&screen)
        .find_named(title)
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "the terminal object model reads no region titled {title:?}; screen:\n{}",
                screen.contents()
            )
        })
}

#[then(expr = "the region titled {string} is at most {int} columns wide")]
async fn the_region_titled_is_at_most_columns_wide(
    world: &mut TinmanWorld,
    title: String,
    limit: u16,
) {
    let region = run_region(world, &title);
    assert!(
        region.rect.width <= limit,
        "the region titled {title:?} is {} columns wide, wider than the {limit} columns allowed",
        region.rect.width
    );
}

#[then(expr = "the region titled {string} has the corner glyph {string}")]
async fn the_region_titled_has_the_corner_glyph(
    world: &mut TinmanWorld,
    title: String,
    glyph: String,
) {
    let region = run_region(world, &title);
    // The region reports its own cells on its own 1-based grid, so the top-left
    // corner of the border is the cell at its row 1 column 1.
    assert_eq!(
        region.cell(1, 1),
        glyph,
        "the top-left corner glyph of the region titled {title:?}"
    );
}

#[then(
    expr = "the terminal object model of the screen conforms to the {string} schema in {string}"
)]
async fn the_tom_of_the_screen_conforms(world: &mut TinmanWorld, schema: String, path: String) {
    let raw = interface_raw(world);
    let screen = support::terminal_screen_at(&raw, run_columns(world));
    let model = tinman::tom::build(&screen);
    let instance =
        serde_json::to_value(&model).expect("the terminal object model serializes to JSON");
    let bad = support::schema_counterexamples(&path, &instance);
    assert!(
        bad.is_empty(),
        "the terminal object model of the screen violates the {schema:?} schema:\n{}\nscreen:\n{}",
        bad.join("\n"),
        screen.contents()
    );
}

#[then(expr = "the region titled {string} is drawn in a colour other than the default foreground")]
async fn the_region_titled_is_drawn_in_colour(world: &mut TinmanWorld, title: String) {
    let region = run_region(world, &title);
    let rect = region.rect;
    let raw = interface_raw(world);
    // The model's rectangle is 0-based and the cell report is 1-based, so the
    // reported cell is converted before it is placed in the box.
    let inside = support::coloured_cells(&raw, run_columns(world))
        .into_iter()
        .any(|(row, col)| {
            let (row, col) = (row - 1, col - 1);
            row >= rect.y
                && row < rect.y + rect.height
                && col >= rect.x
                && col < rect.x + rect.width
        });
    assert!(
        inside,
        "no cell of the region titled {title:?} is drawn in a colour other than the default foreground"
    );
}

#[then("no cell is drawn in a colour other than the default foreground")]
async fn no_cell_is_drawn_in_colour(world: &mut TinmanWorld) {
    let raw = world
        .run_raw
        .as_ref()
        .expect("the terminal output was read");
    let coloured = support::coloured_cells(raw, run_columns(world));
    assert!(
        coloured.is_empty(),
        "{} cells are drawn in a colour other than the default foreground, the first at row {} column {}",
        coloured.len(),
        coloured[0].0,
        coloured[0].1
    );
}

/// The bytes the run drew on the terminal: the open session's output where one
/// stands, and the finished run's otherwise. A presentation assertion reads the
/// same bytes whichever way the scenario started the program.
/// Whether the run's output is read whole. A finished run has said everything it
/// is going to say, so nothing it drew is out of reach; a live session is read on
/// the screenful the operator is looking at.
fn read_whole(world: &TinmanWorld) -> bool {
    world.terminal_session.is_none()
}

fn drawn_raw(world: &TinmanWorld) -> Vec<u8> {
    match world.terminal_session.as_ref() {
        Some(session) => session.raw_output(),
        None => world.run_raw.clone().expect("the terminal output was read"),
    }
}

/// The 1-based row, column and width of the cells reading `text`, once they are
/// drawn. An answer reaches the terminal only when the model replies, so an open
/// session is waited on until the text is drawn rather than read once; a
/// finished run drew everything it is going to draw and is read as it stands.
fn await_text_cells(world: &TinmanWorld, text: &str) -> (u16, u16, u16) {
    let columns = run_columns(world);
    let budget = match world.terminal_session {
        Some(_) => std::time::Duration::from_secs(20),
        None => std::time::Duration::ZERO,
    };
    let deadline = std::time::Instant::now() + budget;
    loop {
        let raw = drawn_raw(world);
        if let Some(found) = support::text_cells(&raw, columns, text, read_whole(world)) {
            return found;
        }
        if std::time::Instant::now() >= deadline {
            let screen = support::terminal_screen_at(&raw, columns);
            panic!(
                "the terminal never drew {text:?}; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// Whether every cell of the run reading `text` is drawn in a foreground colour
/// other than the terminal's default.
fn text_is_coloured(world: &TinmanWorld, text: &str) -> bool {
    let (row, col, width) = await_text_cells(world, text);
    let raw = drawn_raw(world);
    let coloured = support::coloured_cells_at(&raw, run_columns(world), read_whole(world));
    (0..width).all(|offset| coloured.contains(&(row, col + offset)))
}

#[then(expr = "the name {string} is drawn in a colour other than the default foreground")]
async fn the_name_is_drawn_in_colour(world: &mut TinmanWorld, name: String) {
    assert!(
        text_is_coloured(world, &name),
        "the name {name:?} is drawn in the terminal's default foreground"
    );
}

#[then("the tagline is drawn in a colour other than the default foreground")]
async fn the_tagline_is_drawn_in_colour(world: &mut TinmanWorld) {
    // The tagline on the screen is the line the configured provider wrote, so it
    // is read from the answer this scenario staged rather than restated here.
    let tagline = world
        .provider_answer
        .clone()
        .expect("the scenario staged the provider's tagline");
    assert!(
        text_is_coloured(world, &tagline),
        "the tagline {tagline:?} is drawn in the terminal's default foreground"
    );
}

/// The characters a terminal spinner cycles through. Every one of them is an
/// animation glyph rather than text, so none can reach the screen as part of the
/// help or of an expansion, and a frame left standing is unambiguous.
const SPINNER_FRAMES: &[char] = &[
    '⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏', '◐', '◓', '◑', '◒', '◴', '◷', '◶', '◵', '▖',
    '▘', '▝', '▗',
];

/// The spinner frames `text` carries, which is empty where nothing is spinning.
fn spinner_frames_in(text: &str) -> Vec<char> {
    let mut found: Vec<char> = text
        .chars()
        .filter(|character| SPINNER_FRAMES.contains(character))
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

#[given("the inference provider does not answer")]
async fn the_inference_provider_does_not_answer(world: &mut TinmanWorld) {
    // A provider that accepts the connection and then withholds its answer is
    // the tagline that never arrives: the call ends only when its ceiling ends
    // it, which is what the settled line has to survive.
    let provider = support::LocalProvider::stalling();
    use_provider(world, provider);
}

#[then("the Commands block is drawn before the tagline is drawn")]
async fn the_commands_block_is_drawn_before_the_tagline(world: &mut TinmanWorld) {
    let block = support::commands_block(&tinman::help::conventional());
    assert!(
        !block.is_empty(),
        "the help text carries no Commands block, so this step would assert nothing"
    );
    let tagline = world
        .provider_answer
        .clone()
        .expect("the scenario staged the provider's tagline");
    // The terminal receives what the program draws in the order it draws it, so
    // the byte stream is the record of which reached the screen first.
    let raw = drawn_raw(world);
    let drawn = String::from_utf8_lossy(&raw);
    let commands_at = block
        .iter()
        .filter_map(|line| drawn.find(line.trim_end()))
        .min()
        .unwrap_or_else(|| {
            panic!("the Commands block was never drawn; the terminal read:\n{drawn}")
        });
    let tagline_at = drawn
        .find(&tagline)
        .unwrap_or_else(|| panic!("the tagline was never drawn; the terminal read:\n{drawn}"));
    assert!(
        commands_at < tagline_at,
        "the tagline was drawn before the Commands block, so the help waited on it"
    );
}

#[then("the tagline line carries the generated expansion")]
async fn the_tagline_line_carries_the_generated_expansion(world: &mut TinmanWorld) {
    let expansion = world
        .provider_answer
        .clone()
        .expect("the scenario staged the provider's tagline");
    // The help is longer than the terminal is tall, so the tagline has scrolled
    // off the final screen by the time the run ends. The line the help reserves
    // for it in the output is where it is read.
    let tagline = tagline_line(world);
    assert!(
        tagline.contains(&expansion),
        "the tagline line carries no expansion the provider generated; it reads {tagline:?}"
    );
}

#[then("no spinner frame is on the screen")]
async fn no_spinner_frame_is_on_the_screen(world: &mut TinmanWorld) {
    let raw = drawn_raw(world);
    let screen = support::terminal_screen_at(&raw, run_columns(world));
    let contents = screen.contents();
    let frames = spinner_frames_in(&contents);
    assert!(
        frames.is_empty(),
        "the screen was left carrying the spinner frames {frames:?}; it reads:\n{contents}"
    );
}

#[then("the Commands block is in the output")]
async fn the_commands_block_is_in_the_output(world: &mut TinmanWorld) {
    let block = support::commands_block(&tinman::help::conventional());
    assert!(
        !block.is_empty(),
        "the help text carries no Commands block, so this step would assert nothing"
    );
    let output = world.run_stdout.as_ref().expect("the help output was read");
    let missing: Vec<&String> = block
        .iter()
        .filter(|line| !output.contains(line.trim_end()))
        .collect();
    assert!(
        missing.is_empty(),
        "the output carries no {missing:?} of the Commands block; it reads:\n{output}"
    );
}

#[then("the output carries no spinner frame")]
async fn the_output_carries_no_spinner_frame(world: &mut TinmanWorld) {
    let output = world.run_stdout.as_ref().expect("the help output was read");
    let frames = spinner_frames_in(output);
    assert!(
        frames.is_empty(),
        "the redirected output carries the spinner frames {frames:?}; it reads:\n{output}"
    );
}

#[then("the Commands block is drawn in the default foreground")]
async fn the_commands_block_is_drawn_plainly(world: &mut TinmanWorld) {
    let block = support::commands_block(&tinman::help::conventional());
    assert!(
        !block.is_empty(),
        "the help text carries no Commands block, so this step would assert nothing"
    );
    let raw = drawn_raw(world);
    let columns = run_columns(world);
    let whole = read_whole(world);
    let coloured = support::coloured_cells_at(&raw, columns, whole);
    let mut read = 0usize;
    let mut painted = Vec::new();
    for line in &block {
        let Some((row, col, width)) = support::text_cells(&raw, columns, line.trim_end(), whole)
        else {
            continue;
        };
        read += 1;
        if (0..width).any(|offset| coloured.contains(&(row, col + offset))) {
            painted.push(format!("{line:?} on row {row}"));
        }
    }
    assert!(
        read > 0,
        "no line of the Commands block is on the screen, so this step would assert nothing"
    );
    assert!(
        painted.is_empty(),
        "lines of the Commands block are drawn in a colour of their own: {}",
        painted.join(", ")
    );
}

#[then(expr = "{string} is drawn on a background other than the default background")]
async fn text_is_drawn_on_a_coloured_background(world: &mut TinmanWorld, text: String) {
    let (row, col, width) = await_text_cells(world, &text);
    let raw = drawn_raw(world);
    let columns = run_columns(world);
    let run =
        support::background_run(&raw, columns, row, col, read_whole(world)).unwrap_or_else(|| {
            panic!("{text:?} on row {row} is drawn on the terminal's default background")
        });
    let (start, painted) = run;
    assert!(
        start <= col && start + painted >= col + width,
        "{text:?} runs from column {col} for {width} columns, and the background block on row {row} runs from column {start} for {painted}"
    );
    world.drawn_background = Some((row, start, painted));
}

#[then("that background runs the full measure of the transcript")]
async fn that_background_runs_the_full_measure(world: &mut TinmanWorld) {
    let (row, start, painted) = world
        .drawn_background
        .expect("a background block was found on the screen");
    // The transcript's measure is the width the box beneath it is drawn to: the
    // interface's own contract binds the box to the measure the transcript is
    // wrapped to, so the measure is read off the screen rather than restated.
    let (_, box_region) = session_region(world, &assistant_box_title());
    let measure = box_region.rect.width;
    assert_eq!(
        painted, measure,
        "the background block on row {row} runs from column {start} for {painted} columns, and the transcript's measure is {measure}"
    );
}

#[then(expr = "{string} is drawn on the default background")]
async fn text_is_drawn_on_the_default_background(world: &mut TinmanWorld, text: String) {
    let (row, col, width) = await_text_cells(world, &text);
    let raw = drawn_raw(world);
    let columns = run_columns(world);
    let painted: Vec<u16> = (0..width)
        .filter(|offset| {
            support::background_run(&raw, columns, row, col + offset, read_whole(world)).is_some()
        })
        .map(|offset| col + offset)
        .collect();
    assert!(
        painted.is_empty(),
        "{} cell(s) of {text:?} on row {row} are drawn on a background of their own, the first at column {}",
        painted.len(),
        painted[0]
    );
}

#[then(expr = "the question is drawn with the leading marker {string}")]
async fn the_question_is_drawn_with_the_leading_marker(world: &mut TinmanWorld, marker: String) {
    let question = world
        .last_question
        .clone()
        .expect("a question was sent at the prompt");
    // The marker earns its place only immediately before the question, so the
    // two are looked for as one run rather than separately.
    let marked = format!("{marker}{question}");
    let _ = await_text_cells(world, &marked);
}

#[then("no cell is drawn on a background other than the default background")]
async fn no_cell_is_drawn_on_a_coloured_background(world: &mut TinmanWorld) {
    let raw = drawn_raw(world);
    let painted = support::background_cells(&raw, run_columns(world), read_whole(world));
    assert!(
        painted.is_empty(),
        "{} cells are drawn on a background other than the default, the first at row {} column {}",
        painted.len(),
        painted[0].0,
        painted[0].1
    );
}

#[then("the help output carries no escape sequence")]
async fn the_help_output_carries_no_escape_sequence(world: &mut TinmanWorld) {
    let output = run_output(world);
    assert!(
        !output.is_empty(),
        "the command wrote nothing, so this step would assert nothing"
    );
    let escapes = output.matches('\u{1b}').count();
    assert_eq!(
        escapes, 0,
        "the redirected help carries {escapes} escape sequence(s); it reads:\n{output:?}"
    );
}

#[then("no line on the screen carries a fence marker")]
async fn no_line_on_the_screen_carries_a_fence_marker(world: &mut TinmanWorld) {
    let raw = drawn_raw(world);
    // Read whole rather than on the visible screenful: a fence that has scrolled
    // out of view is still a fence the program printed, and an absence assertion
    // read on one screenful passes on timing.
    let screen = support::whole_terminal_screen(&raw, run_columns(world));
    let fenced: Vec<String> = screen
        .contents()
        .lines()
        .filter(|line| line.contains("```"))
        .map(str::to_string)
        .collect();
    assert!(
        fenced.is_empty(),
        "lines of the screen carry a fence marker: {fenced:?}"
    );
}

/// The words of a single answer line of exactly 200 characters. Distinct short
/// tokens, so the rows the wrapped answer occupies are read back by finding the
/// words rather than by assuming where the wrap fell.
fn long_answer_words() -> Vec<String> {
    let mut words: Vec<String> = (1..=40).map(|n| format!("w{n:03}")).collect();
    words.last_mut().expect("the line carries words").push('x');
    words
}

/// A single answer line of exactly 200 characters.
fn long_answer_line() -> String {
    let line = long_answer_words().join(" ");
    assert_eq!(
        line.chars().count(),
        200,
        "the staged answer is a single line of 200 characters"
    );
    line
}

/// The rows the answer occupies on the terminal, as 1-based row numbers paired
/// with the drawn width of each, read once the answer's last word has arrived.
fn answer_rows(world: &TinmanWorld) -> Vec<(u16, u16)> {
    let words = long_answer_words();
    let last = words.last().expect("the answer carries words").clone();
    // The answer reaches the terminal only when the model replies, so the read
    // ends on its final word being drawn rather than on a clock.
    let _ = await_text_cells(world, &last);
    let raw = drawn_raw(world);
    let columns = run_columns(world);
    let screen = support::terminal_screen_at(&raw, columns);
    screen
        .rows()
        .iter()
        .enumerate()
        .filter_map(|(index, cells)| {
            let line = cells.concat();
            if !words.iter().any(|word| line.contains(word.as_str())) {
                return None;
            }
            let width = u16::try_from(line.trim_end().chars().count())
                .expect("a row width fits a terminal");
            let row = u16::try_from(index + 1).expect("a row number fits a terminal");
            Some((row, width))
        })
        .collect()
}

#[then("the answer is drawn on more than one line")]
async fn the_answer_is_drawn_on_more_than_one_line(world: &mut TinmanWorld) {
    let rows = answer_rows(world);
    assert!(
        rows.len() > 1,
        "the answer is drawn on {} line(s): {rows:?}",
        rows.len()
    );
}

#[then(expr = "no line of the answer is wider than {int} columns")]
async fn no_line_of_the_answer_is_wider_than(world: &mut TinmanWorld, limit: u16) {
    let rows = answer_rows(world);
    assert!(
        !rows.is_empty(),
        "the answer is drawn on no line, so this step would assert nothing"
    );
    let wide: Vec<String> = rows
        .iter()
        .filter(|(_, width)| *width > limit)
        .map(|(row, width)| format!("row {row} is {width} columns wide"))
        .collect();
    assert!(
        wide.is_empty(),
        "lines of the answer are wider than the {limit} columns allowed: {}",
        wide.join(", ")
    );
}

/// The title the assistant box carries. The asset's first line is the box's
/// title, which is the name the model reads it by, so a step that must find the
/// box takes the name from the asset that draws it rather than restating it.
fn assistant_box_title() -> String {
    asset_body("assets/help/assistant-prompt.txt")
        .lines()
        .next()
        .expect("the assistant prompt asset carries a title")
        .to_string()
}

#[given(expr = "the operator runs {string} in an interactive terminal")]
async fn given_operator_runs_interactive(world: &mut TinmanWorld, line: String) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let session = support::TerminalSession::start(&dir, &argv, &env)
        .unwrap_or_else(|e| panic!("starting {line:?} on a terminal failed: {e}"));
    // The session is up once the box inviting a question stands on the
    // terminal, so nothing is typed before the program can read it.
    session.await_region(&assistant_box_title(), std::time::Duration::from_secs(10));
    world.terminal_session = Some(session);
}

/// Send `question` at the prompt and note the moment it went, so a step reading
/// how long a call has been pending reads it against the wall clock.
///
/// The same action is starting state for a scenario about a second exchange and
/// the action under test for a scenario about the first, so it binds in both
/// phases.
async fn send_question_at_the_assistant_prompt(world: &mut TinmanWorld, question: String) {
    world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session")
        .type_line(&question);
    world.question_sent_at = Some(std::time::Instant::now());
    world.last_question = Some(question);
}

#[given("the assistant is drawing its box")]
async fn the_assistant_is_drawing_its_box(world: &mut TinmanWorld) {
    // The assistant draws its ask box where it can reach a provider, so the
    // starting state provisions one, as every other scenario that wants the box
    // does. The session runs under a shell that outlives it, so the terminal
    // can be asked afterwards what state it was left in.
    let provider =
        support::LocalProvider::returning("Tinman Inspects Numerous Machine Agent Nodes");
    use_provider(world, provider);
    let dir = working_dir(world);
    let env = configured_env(world);
    let session = support::start_interruptible(&dir, &["--help"], &env)
        .unwrap_or_else(|e| panic!("starting the assistant on a terminal failed: {e}"));
    // The box standing on the terminal is the signal the assistant has taken
    // the terminal, so the interrupt lands on a session that is really drawing.
    session.await_region(&assistant_box_title(), std::time::Duration::from_secs(10));
    world.terminal_session = Some(session);
}

#[when("the operator interrupts the session")]
async fn the_operator_interrupts_the_session(world: &mut TinmanWorld) {
    let session = world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session");
    session.interrupt();
    // The shell reports the terminal state once the program has gone, so the
    // wait ends on that report arriving rather than on a clock.
    session.await_output(
        support::TERMINAL_STATE_MARKER,
        std::time::Duration::from_secs(10),
    );
}

/// What the terminal reported about itself once the program had gone: the
/// `stty` reading the surviving shell took after the session ended.
fn terminal_state_report(world: &TinmanWorld) -> String {
    let output = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .output();
    match output.split_once(support::TERMINAL_STATE_MARKER) {
        Some((_, reported)) => reported.to_string(),
        None => panic!(
            "the terminal never reported its state after the session ended; it produced:\n{output}"
        ),
    }
}

#[then("the terminal is out of raw mode")]
async fn the_terminal_is_out_of_raw_mode(world: &mut TinmanWorld) {
    let reported = terminal_state_report(world);
    // `stty` names a disabled flag with a leading minus, so the canonical,
    // echoing terminal an operator gets back is the absence of both minuses.
    let held: Vec<&str> = ["-icanon", "-echo"]
        .into_iter()
        .filter(|flag| {
            reported
                .split_whitespace()
                .any(|word| word.trim_end_matches(';') == *flag)
        })
        .collect();
    assert!(
        held.is_empty(),
        "the terminal was left in raw mode; it reports {held:?} in:\n{reported}"
    );
}

#[then("the alternate screen has been left")]
async fn the_alternate_screen_has_been_left(world: &mut TinmanWorld) {
    let raw = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .raw_output();
    let output = String::from_utf8_lossy(&raw);
    // A terminal switches screens on the DEC private mode 1049 sequences, so
    // the screen the session ended on is whichever of the two came last.
    let entered = output.rfind("\u{1b}[?1049h");
    let left = output.rfind("\u{1b}[?1049l");
    let ended_on_the_alternate_screen = match (entered, left) {
        (Some(at), Some(back)) => back < at,
        (Some(_), None) => true,
        (None, _) => false,
    };
    assert!(
        !ended_on_the_alternate_screen,
        "the session ended on the alternate screen"
    );
}

/// The bytes the finished run wrote while the terminal stood on the alternate
/// screen, failing with the whole stream where the run never put it there. A
/// run that never switched screens drew over whatever the operator had, so the
/// absence is the finding rather than an empty screen to assert against.
fn alternate_screen_bytes(world: &TinmanWorld) -> Vec<u8> {
    let raw = world
        .run_raw
        .clone()
        .expect("the run's terminal bytes were kept");
    support::alternate_screen_span(&raw).unwrap_or_else(|| {
        panic!(
            "the run never put the terminal on the alternate screen, so it drew over \
             the operator's own screen; it wrote:\n{}",
            String::from_utf8_lossy(&raw).replace('\r', "")
        )
    })
}

#[given(expr = "the terminal scrollback carries the line {string}")]
async fn the_terminal_scrollback_carries_the_line(world: &mut TinmanWorld, line: String) {
    world.scrollback_line = Some(line);
}

#[then("the terminal is on the alternate screen")]
async fn the_terminal_is_on_the_alternate_screen(world: &mut TinmanWorld) {
    let bytes = alternate_screen_bytes(world);
    // The assistant's own box is what the alternate screen must carry: a run
    // that switched screens and drew nothing there has not drawn the assistant
    // on the alternate screen, which is what this scenario is about.
    let screen = support::terminal_screen(&bytes);
    let title = assistant_box_title();
    assert!(
        tinman::tom::build(&screen).find_named(&title).is_some(),
        "the alternate screen carries no region named {title:?}, so the assistant drew \
         elsewhere; the alternate screen reads:\n{}",
        screen.contents()
    );
}

#[then(expr = "{string} is not on the screen")]
async fn text_is_not_on_the_screen(world: &mut TinmanWorld, text: String) {
    // The screen the assistant is drawing on is the alternate one, so that is
    // the screen this assertion reads. Reading the whole stream instead would
    // find the operator's own earlier line in the bytes that put it on the
    // screen the assistant left behind, and report it as still showing.
    let screen = support::terminal_screen(&alternate_screen_bytes(world));
    let contents = screen.contents();
    assert!(
        !contents.contains(&text),
        "the screen the assistant drew on carries {text:?}; it reads:\n{contents}"
    );
}

#[given(expr = "the operator runs {string} in an interactive terminal {int} columns wide")]
async fn given_operator_runs_interactive_at(world: &mut TinmanWorld, line: String, columns: u16) {
    let dir = working_dir(world);
    let args = tinman_args(&line);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = configured_env(world);
    let session = support::TerminalSession::start_at(&dir, &argv, &env, columns)
        .unwrap_or_else(|e| panic!("starting {line:?} on a {columns} column terminal failed: {e}"));
    // The session is up once the box inviting a question stands on the terminal,
    // so nothing is typed before the program can read it.
    session.await_region(&assistant_box_title(), std::time::Duration::from_secs(10));
    world.terminal_session = Some(session);
    world.run_columns = Some(columns);
}

#[when(expr = "the operator types {string} at the assistant prompt")]
async fn the_operator_types_at_the_assistant_prompt(world: &mut TinmanWorld, question: String) {
    send_question_at_the_assistant_prompt(world, question).await;
}

#[given(expr = "the operator types {string} at the assistant prompt")]
async fn given_the_operator_types_at_the_assistant_prompt(
    world: &mut TinmanWorld,
    question: String,
) {
    // Starting state is a finished exchange. The prompt drops keystrokes while a
    // call is pending, so the question that follows this one is typed only once
    // the answer to this one has reached the terminal.
    ask_and_await_the_answer(world, &question).await;
}

#[when(expr = "the operator types {string} at the assistant prompt without sending")]
async fn the_operator_types_without_sending(world: &mut TinmanWorld, text: String) {
    let session = world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session");
    session.type_text(&text);
    // The typed characters reach the box only when the program redraws it, so
    // what follows reads a settled screen: the gate ends on the box carrying
    // the text rather than on a clock, and the cursor is then read where the
    // program left it.
    let title = assistant_box_title();
    await_region_showing(session, &title, &text, std::time::Duration::from_secs(10));
}

/// Wait until the region titled `title` shows `expected` on the session's
/// terminal, bounded by `budget`, failing with the screen last observed.
fn await_region_showing(
    session: &support::TerminalSession,
    title: &str,
    expected: &str,
    budget: std::time::Duration,
) {
    let deadline = std::time::Instant::now() + budget;
    loop {
        let screen = support::terminal_screen(&session.raw_output());
        if let Some(region) = tinman::tom::build(&screen).find_named(title)
            && region_shows_line(region, expected)
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the region titled {title:?} never showed {expected:?}; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// The region titled `title` on the session's terminal, read on the grid the
/// program addressed.
fn session_region(world: &TinmanWorld, title: &str) -> (Vec<u8>, tinman::tom::Region) {
    let raw = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .raw_output();
    let screen = support::terminal_screen(&raw);
    let region = tinman::tom::build(&screen)
        .find_named(title)
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "the terminal object model reads no region titled {title:?}; screen:\n{}",
                screen.contents()
            )
        });
    (raw, region)
}

#[then(expr = "the cursor is inside the region titled {string}")]
async fn the_cursor_is_inside_the_region_titled(world: &mut TinmanWorld, title: String) {
    let (raw, region) = session_region(world, &title);
    let rect = region.rect;
    // The cursor is reported on the 1-based grid the terminal addresses and the
    // model's rectangle is 0-based, so the reported cell is converted before it
    // is placed in the box.
    let (row, col) = support::cursor_cell(&raw, support::TERMINAL_COLS);
    let (row, col) = (row - 1, col - 1);
    assert!(
        row >= rect.y && row < rect.y + rect.height && col >= rect.x && col < rect.x + rect.width,
        "the cursor sits at row {} column {} of the terminal, outside the region titled {title:?} \
         at rows {}..{} columns {}..{}",
        row + 1,
        col + 1,
        rect.y,
        rect.y + rect.height,
        rect.x,
        rect.x + rect.width
    );
}

#[then(expr = "the cursor is one column past the {string} it shows")]
async fn the_cursor_is_one_column_past(world: &mut TinmanWorld, text: String) {
    let raw = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .raw_output();
    let screen = support::terminal_screen(&raw);
    let (cursor_row, cursor_col) = support::cursor_cell(&raw, support::TERMINAL_COLS);
    // The text is looked for on the cursor's own row, so a cursor left on
    // another row fails here rather than matching text it does not follow.
    let cells = screen
        .rows()
        .get((cursor_row - 1) as usize)
        .unwrap_or_else(|| {
            panic!(
                "the cursor sits at row {cursor_row}, which the screen does not have; screen:\n{}",
                screen.contents()
            )
        });
    let wanted: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    let starts_at = cells
        .windows(wanted.len())
        .position(|window| window == wanted.as_slice())
        .unwrap_or_else(|| {
            panic!(
                "row {cursor_row}, the row the cursor sits on, does not show {text:?}; screen:\n{}",
                screen.contents()
            )
        });
    let expected = starts_at as u16 + wanted.len() as u16 + 1;
    assert_eq!(
        cursor_col,
        expected,
        "the cursor sits at column {cursor_col}, not one column past the {text:?} the box shows, \
         which ends at column {}",
        expected - 1
    );
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

/// Every line of the conventional help that carries text. These are the lines
/// the operator asked for, and they must still be readable after the assistant
/// opens beneath them.
fn conventional_help_lines() -> Vec<String> {
    tinman::help::conventional()
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// Everything the run wrote to the terminal, with the run brought to its own end
/// first where a session is still open. A scenario reading the screen after the
/// assistant leaves is asserting on what the program left behind, so the read
/// ends on the program's exit rather than on whatever had been drained when the
/// step arrived.
fn finished_terminal_output(world: &mut TinmanWorld) -> String {
    if let Some(mut session) = world.terminal_session.take() {
        let outcome = session.finish(std::time::Duration::from_secs(30));
        world.run_stdout = Some(outcome.stdout);
        world.run_status = Some(outcome.status);
        world.run_raw = Some(outcome.raw);
    }
    world
        .run_stdout
        .clone()
        .expect("the terminal output was read")
}

#[then("the conventional help is on the screen")]
async fn the_conventional_help_is_on_the_screen(world: &mut TinmanWorld) {
    let output = finished_terminal_output(world);
    // The help reaches a terminal with its example placeholders expanded and a
    // redirected stream with them intact, and both renderings are the help this
    // step looks for. Which one a run produces is pinned by the scenarios that
    // own that question, so a line is accepted in either of its rendered forms
    // and the placeholder policy is asserted where it belongs.
    let values = support::example_values(&tinman::help::conventional());
    for line in conventional_help_lines() {
        let expanded = values.iter().fold(line.clone(), |text, (name, value)| {
            text.replace(&format!("{{{{{name}}}}}"), value)
        });
        assert!(
            output.contains(&line) || output.contains(&expanded),
            "the terminal no longer shows the conventional help line {line:?}; output:\n{output}"
        );
    }
}

#[then(expr = "{string} is on the screen")]
async fn text_is_on_the_screen(world: &mut TinmanWorld, text: String) {
    let output = finished_terminal_output(world);
    assert!(
        output.contains(&text),
        "the terminal never showed {text:?}; output:\n{output}"
    );
}

#[then("the Examples block is on the screen")]
async fn the_examples_block_is_on_the_screen(world: &mut TinmanWorld) {
    let block = support::examples_block(&tinman::help::conventional());
    assert!(
        !block.is_empty(),
        "the help text carries no Examples block, so this scenario would assert nothing"
    );
    let output = finished_terminal_output(world);
    // An entry carrying a placeholder reaches the screen expanded, so what
    // survives expansion is the text ahead of the first delimiter, and that is
    // what this step looks for. A line carrying no placeholder is looked for
    // whole. The delimiters themselves are the other step's assertion.
    for line in &block {
        let stable = line.split("{{").next().unwrap_or(line).trim_end();
        if stable.is_empty() {
            continue;
        }
        assert!(
            output.contains(stable),
            "the terminal never showed the Examples block line {stable:?}; output:\n{output}"
        );
    }
}

#[then("no placeholder delimiter is on the screen")]
async fn no_placeholder_delimiter_is_on_the_screen(world: &mut TinmanWorld) {
    let output = finished_terminal_output(world);
    let carried: Vec<&str> = ["{{", "}}"]
        .into_iter()
        .filter(|delimiter| output.contains(delimiter))
        .collect();
    assert!(
        carried.is_empty(),
        "the terminal shows the placeholder delimiters {carried:?} unexpanded; output:\n{output}"
    );
}

#[given(expr = "the placeholders in the Examples block of the asset at {string}")]
async fn the_placeholders_in_the_examples_block(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    world.example_placeholders = Some(support::examples_block_placeholders(&asset));
    world.asset_text = Some(asset);
}

#[when("each is looked up among the example values")]
async fn each_is_looked_up_among_the_example_values(world: &mut TinmanWorld) {
    let asset = world
        .asset_text
        .as_ref()
        .expect("the asset the placeholders were read from was kept");
    world.example_values = Some(support::example_values(asset));
}

#[then("every one has a value")]
async fn every_placeholder_has_a_value(world: &mut TinmanWorld) {
    let placeholders = world
        .example_placeholders
        .as_ref()
        .expect("the placeholders were read");
    let values = world
        .example_values
        .as_ref()
        .expect("the example values were read");
    let missing: Vec<&String> = placeholders
        .iter()
        .filter(|name| !values.iter().any(|(key, _)| key == *name))
        .collect();
    assert!(
        missing.is_empty(),
        "placeholders the Examples block carries and the example values omit: {missing:?}"
    );
}

#[then("the placeholders read are not empty")]
async fn the_placeholders_read_are_not_empty(world: &mut TinmanWorld) {
    let placeholders = world
        .example_placeholders
        .as_ref()
        .expect("the placeholders were read");
    assert!(
        !placeholders.is_empty(),
        "the Examples block carries no placeholder, so this scenario would assert nothing"
    );
}

#[then(expr = "the screen does not carry the Commands block of the asset at {string}")]
async fn the_screen_does_not_carry_the_commands_block(world: &mut TinmanWorld, path: String) {
    let asset = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("help asset {path} unreadable: {e}"));
    let block = support::commands_block(&asset);
    assert!(
        !block.is_empty(),
        "the asset at {path} carries no Commands block, so this scenario would assert nothing"
    );
    let output = world
        .run_stdout
        .as_ref()
        .expect("the terminal output was read");
    let carried: Vec<&String> = block.iter().filter(|line| output.contains(*line)).collect();
    assert!(
        carried.is_empty(),
        "the terminal carries lines of the Commands block: {carried:?}; output:\n{output}"
    );
}

#[then(expr = "a bordered region titled {string} is drawn")]
async fn a_bordered_region_is_drawn(world: &mut TinmanWorld, title: String) {
    // Read through the same virtual screen the negative assertion uses: the
    // title sits on the top border row and the body rows below, so no run of
    // the region's text appears contiguously in the bytes and a search of the
    // raw output would report the box present whether or not it was drawn as a
    // region.
    let raw = interface_raw(world);
    let screen = support::terminal_screen(&raw);
    assert!(
        tinman::tom::build(&screen).find_named(&title).is_some(),
        "the terminal object model reads no bordered region titled {title:?}; screen:\n{}",
        screen.contents()
    );
}

/// The roles requiring an accessible name, read from the terminal object
/// model's own scantling rather than restated here. The scantling already
/// carries the rule and the reasoning WAI-ARIA 1.2 gives for it, so a second
/// copy in verification support is a copy that drifts: it is what let a role
/// added to the model reach this list and not the contract, or the reverse.
fn roles_requiring_a_name() -> Vec<String> {
    let path = "scantlings/tom.schema.json";
    let schema = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("the scantling at {path} is unreadable: {e}"));
    let value: serde_json::Value = serde_json::from_str(&schema)
        .unwrap_or_else(|e| panic!("the scantling at {path} is not JSON: {e}"));
    let clauses = value["$defs"]["region"]["allOf"]
        .as_array()
        .unwrap_or_else(|| panic!("the scantling at {path} states no conditions on a region"));
    let roles: Vec<String> = clauses
        .iter()
        .filter(|clause| {
            clause["then"]["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|key| key == "name"))
        })
        .filter_map(|clause| clause["if"]["properties"]["role"]["enum"].as_array())
        .flatten()
        .filter_map(|role| role.as_str().map(str::to_string))
        .collect();
    assert!(
        !roles.is_empty(),
        "the scantling at {path} names no role requiring an accessible name, so the \
         sweep below would read nothing"
    );
    roles
}

/// Collect every region of `region` and its descendants playing one of `roles`,
/// as pairs of role and the name it carries.
fn regions_requiring_a_name(
    region: &tinman::tom::Region,
    roles: &[String],
    found: &mut Vec<(String, Option<String>)>,
) {
    if roles.iter().any(|role| role == region.role()) {
        found.push((region.role().to_string(), region.name.clone()));
    }
    for child in &region.children {
        regions_requiring_a_name(child, roles, found);
    }
}

#[then("every region whose role requires an accessible name carries one")]
async fn every_region_requiring_a_name_carries_one(world: &mut TinmanWorld) {
    let roles = roles_requiring_a_name();
    // A live session redraws while this step reads it, so a single read can
    // catch a frame the program is halfway through writing, whose torn rows
    // carry regions the settled screen never draws. The read therefore ends on
    // the observed signal its sibling assertions wait for, the interface
    // reading clean, and reports the last state it saw when it never does.
    let budget = std::time::Duration::from_secs(10);
    let deadline = std::time::Instant::now() + budget;
    loop {
        let raw = match world.terminal_session.as_ref() {
            Some(session) => session.raw_output(),
            None => world.run_raw.clone().expect("the terminal output was read"),
        };
        let screen = support::terminal_screen(&raw);
        let mut found = Vec::new();
        regions_requiring_a_name(&tinman::tom::build(&screen).root, &roles, &mut found);
        let unnamed: Vec<&String> = found
            .iter()
            .filter(|(_, name)| name.as_deref().is_none_or(str::is_empty))
            .map(|(role, _)| role)
            .collect();
        if unnamed.is_empty() && !found.is_empty() {
            world.named_regions_read = Some(found.len());
            return;
        }
        if std::time::Instant::now() >= deadline {
            world.named_regions_read = Some(found.len());
            assert!(
                unnamed.is_empty(),
                "these regions play a role requiring an accessible name and carry none: {unnamed:?}; screen:\n{}",
                screen.contents()
            );
            panic!(
                "the interface drew no region whose role requires an accessible name, so this step would assert over nothing; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[then("the regions read are not empty")]
async fn the_regions_read_are_not_empty(world: &mut TinmanWorld) {
    let read = world
        .named_regions_read
        .expect("a step read the regions of the interface");
    assert!(
        read > 0,
        "the sweep read no region whose role requires an accessible name, so the assertion above passed over nothing"
    );
}

#[then(expr = "no bordered region titled {string} is drawn")]
async fn no_bordered_region_is_drawn(world: &mut TinmanWorld, title: String) {
    // Read through the same virtual screen the positive assertion uses: the
    // title sits on the top border row and the body rows below, so no run of
    // the region's text appears contiguously in the bytes and a search of the
    // raw output would report the box absent whether or not it was drawn.
    let raw = world
        .run_raw
        .as_ref()
        .expect("the terminal output was read");
    let screen = support::terminal_screen(raw);
    assert!(
        tinman::tom::build(&screen).find_named(&title).is_none(),
        "the terminal object model reads a bordered region titled {title:?}; screen:\n{}",
        screen.contents()
    );
}

/// Assert the terminal carries a prompt line naming `key` as the key that
/// performs `action`. One line must carry both, so the prompt loses the
/// assertion the moment it stops pairing the key with what it does.
fn assert_prompt_names_key(output: &str, key: &str, action: &str) {
    let named = output
        .lines()
        .any(|line| line.contains(key) && line.contains(action));
    assert!(
        named,
        "no line of the terminal names {key:?} as the key that {action}s; output:\n{output}"
    );
}

#[then(expr = "the assistant prompt names {string} as the key that sends")]
async fn the_assistant_prompt_names_the_sending_key(world: &mut TinmanWorld, key: String) {
    let output = world.run_stdout.as_ref().expect("the help output was read");
    assert_prompt_names_key(output, &key, "send");
}

#[then(expr = "the assistant prompt names {string} as the key that leaves")]
async fn the_assistant_prompt_names_the_leaving_key(world: &mut TinmanWorld, key: String) {
    let output = world.run_stdout.as_ref().expect("the help output was read");
    assert_prompt_names_key(output, &key, "leave");
}

/// Everything a region shows: its own name and text, and those of every region
/// nested inside it.
fn region_contents(region: &tinman::tom::Region) -> String {
    let mut shown = String::new();
    if let Some(name) = region.name.as_deref() {
        shown.push_str(name);
        shown.push('\n');
    }
    if let Some(text) = region.text.as_deref() {
        shown.push_str(text);
        shown.push('\n');
    }
    for child in &region.children {
        shown.push_str(&region_contents(child));
    }
    shown
}

/// Whether one line of what `region` shows reads exactly `expected`. The
/// trailing spaces are the box's padding rather than the text's, so they are cut
/// before the comparison.
///
/// The line is compared whole because a containment reading cannot tell an
/// erased character from one still standing: the shorter text is a prefix of the
/// longer, so `contains` answers yes either way.
fn region_shows_line(region: &tinman::tom::Region, expected: &str) -> bool {
    region_contents(region)
        .lines()
        .any(|line| line.trim_end() == expected)
}

#[then(expr = "the region titled {string} shows {string}")]
async fn the_region_titled_shows(world: &mut TinmanWorld, title: String, expected: String) {
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    // The typed line reaches the box only when the program redraws it, so the
    // wait ends on the box carrying the text rather than on a clock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let screen = support::terminal_screen(&session.raw_output());
        if let Some(region) = tinman::tom::build(&screen).find_named(&title)
            && region_shows_line(region, &expected)
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the region titled {title:?} never showed {expected:?}; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// Every region of the model built from `screen`, the root included.
fn all_regions(screen: &tinman::screen::VirtualScreen) -> Vec<tinman::tom::Region> {
    fn gather(region: &tinman::tom::Region, found: &mut Vec<tinman::tom::Region>) {
        found.push(region.clone());
        for child in &region.children {
            gather(child, found);
        }
    }
    let mut found = Vec::new();
    gather(&tinman::tom::build(screen).root, &mut found);
    found
}

/// The row the transcript can begin on: the one after the last row showing a
/// line of the conventional help, and the top of the screen where none of the
/// help is drawn.
///
/// The help is drawn above the transcript and quotes Tinman command lines in its
/// own Examples block, so a search of everything above the box finds the help's
/// example rather than the answer, and reports the answer drawn before the model
/// has replied.
fn first_transcript_row(screen: &tinman::screen::VirtualScreen) -> usize {
    let help = conventional_help_lines();
    let mut after = 0usize;
    for (index, cells) in screen.rows().iter().enumerate() {
        let row = cells.concat();
        let row = row.trim_end();
        if !row.is_empty() && help.iter().any(|line| line == row) {
            after = index + 1;
        }
    }
    after
}

/// Whether a row of `screen` between the transcript's first row and `above`
/// shows `text`.
fn text_appears_above(screen: &tinman::screen::VirtualScreen, text: &str, above: u16) -> bool {
    let wanted: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    let from = first_transcript_row(screen);
    screen
        .rows()
        .iter()
        .take(above as usize)
        .skip(from)
        .any(|cells| {
            cells.len() >= wanted.len()
                && cells
                    .windows(wanted.len())
                    .any(|window| window == wanted.as_slice())
        })
}

#[then(expr = "{string} appears above the region titled {string}")]
async fn text_appears_above_the_region_titled(
    world: &mut TinmanWorld,
    text: String,
    title: String,
) {
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    // The answer reaches the terminal only when the program has the model's
    // reply, so the wait ends on the text standing above the box rather than on
    // a clock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let screen = support::terminal_screen(&session.raw_output());
        if let Some(region) = tinman::tom::build(&screen).find_named(&title)
            && text_appears_above(&screen, &text, region.rect.y)
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "no row above the region titled {title:?} shows {text:?}; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[then(expr = "the region titled {string} does not show {string}")]
async fn the_region_titled_does_not_show(world: &mut TinmanWorld, title: String, unwanted: String) {
    let (_, region) = session_region(world, &title);
    let shown = region_contents(&region);
    assert!(
        !shown.contains(&unwanted),
        "the region titled {title:?} shows {unwanted:?}; it shows:\n{shown}"
    );
}

#[then(expr = "the region titled {string} is the lowest region on the screen")]
async fn the_region_titled_is_the_lowest_region(world: &mut TinmanWorld, title: String) {
    let raw = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .raw_output();
    let screen = support::terminal_screen(&raw);
    let region = tinman::tom::build(&screen)
        .find_named(&title)
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "the terminal object model reads no region titled {title:?}; screen:\n{}",
                screen.contents()
            )
        });
    // Nothing is drawn below the box: no region begins at or below the row the
    // box ends on. A region containing the box begins above it and a region
    // inside it begins within it, so both pass without being excused.
    let bottom = region.rect.y + region.rect.height;
    let below: Vec<String> = all_regions(&screen)
        .into_iter()
        .filter(|other| other.rect.y >= bottom)
        .map(|other| {
            format!(
                "a {} named {:?} at row {}",
                other.role(),
                other.name,
                other.rect.y
            )
        })
        .collect();
    assert!(
        below.is_empty(),
        "the region titled {title:?} ends at row {bottom}, and {} region(s) are drawn below it: {}; screen:\n{}",
        below.len(),
        below.join(", "),
        screen.contents()
    );
}

#[then(expr = "{string} is drawn in a different colour from {string}")]
async fn text_is_drawn_in_a_different_colour_from(
    world: &mut TinmanWorld,
    one: String,
    other: String,
) {
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    // Both halves of the exchange must be on the terminal before their colours
    // can be compared, and the answer arrives only when the model replies, so
    // the wait ends on both being drawn.
    //
    // The two texts belong to one passage, and the screen can carry the same
    // words elsewhere: the help text's own example line quotes the very command
    // an answer proposes. So the second text fixes the passage, and the first is
    // read from that row down rather than from the top of the screen.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let (first, second) = loop {
        let raw = session.raw_output();
        let passage = support::text_cells(&raw, support::TERMINAL_COLS, &other, false);
        let second = support::text_foreground(&raw, support::TERMINAL_COLS, &other);
        let first = passage.and_then(|(row, _, _)| {
            support::text_foreground_from(&raw, support::TERMINAL_COLS, &one, row)
        });
        if let (Some(first), Some(second)) = (first, second) {
            break (first, second);
        }
        if std::time::Instant::now() >= deadline {
            let screen = support::terminal_screen(&raw);
            panic!(
                "the terminal never showed both {one:?} and {other:?}; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    assert_ne!(
        first, second,
        "{one:?} and {other:?} are both drawn in {first}"
    );
}

#[then(expr = "the region titled {string} is drawn")]
async fn the_region_titled_is_drawn(world: &mut TinmanWorld, title: String) {
    let _ = session_region(world, &title);
}

#[then("the command has not exited")]
async fn the_command_has_not_exited(world: &mut TinmanWorld) {
    let session = world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session");
    assert!(
        !session.has_exited(),
        "the program has exited; terminal output:\n{}",
        session.output()
    );
}

/// The seconds a pending call reports it has been waiting: the number the box
/// shows that reads as the time already spent. The number is matched against
/// the wall clock rather than against a format, so the step asserts what the
/// scenario says, the elapsed seconds, and fixes no way of printing them.
fn reported_elapsed_seconds(region: &tinman::tom::Region, spent: u64) -> Option<u64> {
    let shown = region_contents(region);
    let mut digits = String::new();
    let mut found = Vec::new();
    for character in shown.chars().chain(std::iter::once(' ')) {
        if character.is_ascii_digit() {
            digits.push(character);
        } else if !digits.is_empty() {
            if let Ok(number) = digits.parse::<u64>() {
                found.push(number);
            }
            digits.clear();
        }
    }
    // A reading is the elapsed seconds when it is no further from the wall
    // clock than the second it was read in, so a report one tick behind the
    // step's own reading still counts.
    found.into_iter().find(|number| number.abs_diff(spent) <= 1)
}

#[then(expr = "the region titled {string} shows the elapsed seconds of the pending call")]
async fn the_region_shows_the_elapsed_seconds(world: &mut TinmanWorld, title: String) {
    let sent = world
        .question_sent_at
        .expect("the question was sent at a known moment");
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    // The first report cannot appear before a second has passed, so the wait
    // ends on the box carrying a reading rather than on a clock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let reading = loop {
        let screen = support::terminal_screen(&session.raw_output());
        let spent = sent.elapsed().as_secs();
        if let Some(region) = tinman::tom::build(&screen).find_named(&title)
            && let Some(reading) = reported_elapsed_seconds(region, spent)
        {
            break reading;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the region titled {title:?} never showed the elapsed seconds of the pending call, \
                 {spent} seconds after it was sent; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    world.reported_elapsed = Some(reading);
}

#[then("the reported elapsed seconds advance while the call is pending")]
async fn the_reported_elapsed_seconds_advance(world: &mut TinmanWorld) {
    let sent = world
        .question_sent_at
        .expect("the question was sent at a known moment");
    let before = world
        .reported_elapsed
        .expect("the elapsed seconds were read once");
    let title = assistant_box_title();
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let screen = support::terminal_screen(&session.raw_output());
        let spent = sent.elapsed().as_secs();
        if let Some(region) = tinman::tom::build(&screen).find_named(&title)
            && let Some(reading) = reported_elapsed_seconds(region, spent)
            && reading > before
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the region titled {title:?} still reports {before} seconds, {spent} seconds after \
                 the question was sent; screen:\n{}",
                screen.contents()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// ---------------------------------------------------------------------------
// the assistant session: what the request Tinman builds carries of the exchange
// ---------------------------------------------------------------------------

/// How long a step waits for a turn to reach the provider and come back. The
/// keystrokes cross a real pseudo-terminal and the request crosses a real
/// socket, so each wait below ends on one of those observable arrivals.
const TURN_BUDGET: std::time::Duration = std::time::Duration::from_secs(20);

/// The chat messages of every request the scenario's provider has received.
fn provider_requests(world: &TinmanWorld) -> Vec<Vec<(String, String)>> {
    world
        .provider
        .as_ref()
        .expect("a local provider was started")
        .received_messages()
}

/// The request the question last sent at the prompt produced, waited for until
/// it observably reaches the provider.
fn assistant_request(world: &TinmanWorld) -> Vec<(String, String)> {
    let question = world
        .last_question
        .clone()
        .expect("a question was sent at the assistant prompt");
    let deadline = std::time::Instant::now() + TURN_BUDGET;
    loop {
        let received = provider_requests(world);
        if let Some(request) = received.iter().rev().find(|messages| {
            messages
                .iter()
                .any(|(_, content)| content.contains(&question))
        }) {
            return request.clone();
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "no request carrying {question:?} reached the provider; it received {} request(s)",
                received.len()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// Send `question` at the prompt and wait until the answer to it has reached the
/// terminal, so the next question is typed at a prompt that is listening rather
/// than at one still holding a call.
async fn ask_and_await_the_answer(world: &mut TinmanWorld, question: &str) {
    let configured = world.provider_answer.clone();
    let answered_before = configured
        .as_ref()
        .map(|answer| answers_on_screen(world, answer))
        .unwrap_or_default();
    send_question_at_the_assistant_prompt(world, question.to_string()).await;
    // A provider that answers nothing offers no signal to end on, so a scenario
    // about a pending call carries on the moment the question is sent.
    let Some(answer) = configured else {
        return;
    };
    let deadline = std::time::Instant::now() + TURN_BUDGET;
    loop {
        if answers_on_screen(world, &answer) > answered_before {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "the answer to {question:?} never reached the terminal; it carried {} answer(s) \
                 before the question was sent",
                answered_before
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// How many times the provider's answer stands on the session's terminal, which
/// is one per turn the assistant has finished.
fn answers_on_screen(world: &TinmanWorld, answer: &str) -> usize {
    world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .output()
        .matches(answer)
        .count()
}

/// The contents of the request's messages carrying `role`.
fn messages_with_role(request: &[(String, String)], role: &str) -> Vec<String> {
    request
        .iter()
        .filter(|(carried, _)| carried == role)
        .map(|(_, content)| content.clone())
        .collect()
}

#[given(expr = "the operator has asked {string} and seventeen questions since")]
async fn the_operator_has_asked_and_seventeen_since(world: &mut TinmanWorld, first: String) {
    ask_and_await_the_answer(world, &first).await;
    for number in 2..=18 {
        ask_and_await_the_answer(world, &format!("question number {number}")).await;
    }
}

#[then("the assistant request carries no earlier exchange")]
async fn the_request_carries_no_earlier_exchange(world: &mut TinmanWorld) {
    let request = assistant_request(world);
    let answers = messages_with_role(&request, "assistant");
    assert!(
        answers.is_empty(),
        "the request carries {} earlier answer(s): {answers:?}",
        answers.len()
    );
}

#[then(expr = "the assistant request carries the earlier question {string}")]
async fn the_request_carries_the_earlier_question(world: &mut TinmanWorld, question: String) {
    let request = assistant_request(world);
    assert!(
        request
            .iter()
            .any(|(_, content)| content.contains(&question)),
        "the request carries no earlier question {question:?}; it carries {request:?}"
    );
}

#[then(expr = "the assistant request carries the earlier answer {string}")]
async fn the_request_carries_the_earlier_answer(world: &mut TinmanWorld, answer: String) {
    let request = assistant_request(world);
    let answers = messages_with_role(&request, "assistant");
    assert!(
        answers.iter().any(|content| content.contains(&answer)),
        "the request carries no earlier answer {answer:?}; it carries the answers {answers:?}"
    );
}

#[then(expr = "the assistant request carries the question {string}")]
async fn the_request_carries_the_question(world: &mut TinmanWorld, question: String) {
    let request = assistant_request(world);
    assert!(
        request
            .iter()
            .any(|(_, content)| content.contains(&question)),
        "the request carries no question {question:?}; it carries {request:?}"
    );
}

#[then(expr = "the assistant request carries no answer for {string}")]
async fn the_request_carries_no_answer_for(world: &mut TinmanWorld, question: String) {
    let request = assistant_request(world);
    let asked = request
        .iter()
        .position(|(role, content)| role == "user" && content.contains(&question))
        .unwrap_or_else(|| {
            panic!("the request carries no question {question:?}; it carries {request:?}")
        });
    let answered: Vec<&String> = request[asked + 1..]
        .iter()
        .take_while(|(role, _)| role != "user")
        .filter(|(role, _)| role == "assistant")
        .map(|(_, content)| content)
        .collect();
    assert!(
        answered.is_empty(),
        "the request still carries an answer for {question:?}: {answered:?}"
    );
}

#[then("the assistant request carries seventeen whole exchanges")]
async fn the_request_carries_seventeen_whole_exchanges(world: &mut TinmanWorld) {
    let request = assistant_request(world);
    let answers = messages_with_role(&request, "assistant");
    assert_eq!(
        answers.len(),
        17,
        "whole exchanges the request carries; it carries {request:?}"
    );
}

#[then("the provider received exactly two assistant requests")]
async fn the_provider_received_exactly_two_assistant_requests(world: &mut TinmanWorld) {
    let _ = assistant_request(world);
    let received = provider_requests(world);
    // The help path also expands the tagline acronym, so the provider's log
    // carries requests of both kinds. An assistant request is one built on the
    // assistant context, read from production rather than from a copied literal.
    let context = tinman::skill::assistant_context();
    let assistant = received
        .iter()
        .filter(|messages| {
            messages
                .iter()
                .any(|(_, content)| content.starts_with(&context))
        })
        .count();
    assert_eq!(
        assistant,
        2,
        "assistant requests the provider received, out of {} request(s) in all; \
         each request ends with {:?}",
        received.len(),
        received
            .iter()
            .map(|messages| messages
                .last()
                .map(|(_, content)| content.chars().rev().take(40).collect::<String>())
                .map(|tail| tail.chars().rev().collect::<String>())
                .unwrap_or_default())
            .collect::<Vec<String>>()
    );
}

#[then(expr = "the assistant prompt names {string} as the key that cancels")]
async fn the_assistant_prompt_names_the_cancelling_key(world: &mut TinmanWorld, key: String) {
    let session = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session");
    // The hint changes when the call starts, so the wait ends on the prompt
    // carrying the pending hint rather than on a clock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let output = session.output();
        if output
            .lines()
            .any(|line| line.contains(&key) && line.contains("cancel"))
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "no line of the terminal names {key:?} as the key that cancels; output:\n{output}"
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[when(expr = "the operator presses {string} at the assistant prompt")]
async fn the_operator_presses_at_the_assistant_prompt(world: &mut TinmanWorld, key: String) {
    world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session")
        .press(&key);
}

/// The text of a line, with any styling the program drew it in removed. These
/// steps assert what a line reads, not what colour it was drawn in, so the
/// escape sequences carrying the colour are not part of the comparison; the
/// scenarios that do assert colour read the cell attributes instead.
fn plain_text(line: &str) -> String {
    let mut plain = String::new();
    let mut chars = line.chars();
    while let Some(character) = chars.next() {
        if character != '\u{1b}' {
            plain.push(character);
            continue;
        }
        // A control sequence introducer runs to its final byte, which is the
        // first character in the `@` to `~` range after the `[`.
        if chars.next() != Some('[') {
            continue;
        }
        for byte in chars.by_ref() {
            if ('@'..='~').contains(&byte) {
                break;
            }
        }
    }
    plain
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
        .map(plain_text)
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

#[when("the test runner sends a request naming a method the driver does not serve")]
async fn sends_a_request_naming_an_unserved_method(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "wormhole", serde_json::json!({"session": session})));
    world.reply = Some(reply);
}

#[then("the driver answers with a JSON-RPC error object")]
async fn the_driver_answers_with_an_error_object(world: &mut TinmanWorld) {
    let reply = world.reply.as_ref().expect("the driver answered");
    assert_eq!(reply["jsonrpc"], serde_json::json!("2.0"), "reply framing");
    let error = reply
        .get("error")
        .unwrap_or_else(|| panic!("the reply carries no error object: {reply}"));
    assert!(
        error.get("code").and_then(|code| code.as_i64()).is_some(),
        "the error object carries no numeric code: {reply}"
    );
    assert!(
        error
            .get("message")
            .and_then(|message| message.as_str())
            .is_some_and(|message| !message.is_empty()),
        "the error object carries no message: {reply}"
    );
}

#[then("the session stays open for the next request")]
async fn the_session_stays_open_for_the_next_request(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(id, "screen", serde_json::json!({"session": session})));
    assert!(
        reply.get("result").is_some(),
        "the session did not answer the request after the one it refused: {reply}"
    );
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

/// The text a scenario waits for that the fixture terminal program never draws,
/// so the wait runs to its ceiling against a real program rather than a
/// simulated one.
const NEVER_DRAWN_TEXT: &str = "SUPERNOVA";

#[when(expr = "the test runner waits for the text {string}")]
async fn the_runner_waits_for_the_text(world: &mut TinmanWorld, text: String) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "wait",
        serde_json::json!({"session": session, "text": text}),
    ));
    world.reply = Some(reply);
}

#[when("the test runner waits for text the program never draws")]
async fn the_runner_waits_for_text_never_drawn(world: &mut TinmanWorld) {
    let id = next_id(world);
    let session = session(world);
    let reply = driver(world).request(rpc(
        id,
        "wait",
        serde_json::json!({"session": session, "text": NEVER_DRAWN_TEXT}),
    ));
    world.reply = Some(reply);
}

#[then("the driver reports the session reached that state")]
async fn the_driver_reports_the_session_reached_that_state(world: &mut TinmanWorld) {
    let reply = reply(world);
    assert_eq!(
        result(reply)["ok"],
        serde_json::Value::Bool(true),
        "the driver did not report the session reached that state: {reply}"
    );
}

#[then("the driver reports the session did not reach that state")]
async fn the_driver_reports_the_session_did_not_reach_that_state(world: &mut TinmanWorld) {
    let reply = reply(world);
    assert_eq!(
        result(reply)["ok"],
        serde_json::Value::Bool(false),
        "the driver did not report the session failed to reach that state: {reply}"
    );
}

#[then("the failure names no failed expectation")]
async fn the_failure_names_no_failed_expectation(world: &mut TinmanWorld) {
    // A wait is a barrier rather than a claim, so a wait that never arrives
    // reports the state the session did not reach. Reporting it as a failed
    // expectation is the conflation the two calls exist to separate.
    let reply = reply(world);
    let failure = result(reply)["failure"].as_str().unwrap_or_else(|| {
        panic!("the failed wait carries no failure to read: {reply}");
    });
    assert!(
        !failure.contains("expect"),
        "the wait failure is reported as a failed expectation: {failure:?}"
    );
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

#[given(expr = "the methods named in {string}")]
async fn the_methods_named_in(world: &mut TinmanWorld, path: String) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the protocol scantling {path} was not read: {e}"));
    let schema: serde_json::Value = serde_json::from_str(&source)
        .unwrap_or_else(|e| panic!("the protocol scantling {path} did not parse: {e}"));
    let named = schema["$defs"]["request"]["properties"]["method"]["enum"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} names no method enum for the driver's requests"));
    world.protocol_methods = Some(
        named
            .iter()
            .map(|name| {
                name.as_str()
                    .expect("a method the protocol names is a string")
                    .to_string()
            })
            .collect(),
    );
}

/// The params a call of `method` carries, addressed to `session`, built to the
/// shape the protocol permits. `launch` opens a session rather than addressing
/// one, so it alone carries no session.
///
/// The values name what the fixture really draws: its bordered `Files` pane,
/// its `Settings` menu item, its labelled `Username` field and the `READY` it
/// writes on its status line. A call carrying params the driver rejects would
/// exchange a conforming error and report a method served, so the params are
/// the real ones a caller would send.
fn protocol_call_params(method: &str, session: &str, command: &str) -> serde_json::Value {
    match method {
        "launch" => serde_json::json!({"command": command}),
        "expect" | "wait" => serde_json::json!({"session": session, "text": "READY"}),
        "capture" => serde_json::json!({
            "session": session, "role": "listitem", "within": "Files", "scope": "visible"
        }),
        "activate" => {
            serde_json::json!({"session": session, "role": "menuitem", "name": "Settings"})
        }
        "fill" => serde_json::json!({"session": session, "label": "Username", "value": "dmytri"}),
        "press" => serde_json::json!({"session": session, "key": "q"}),
        _ => serde_json::json!({"session": session}),
    }
}

#[when("each is exchanged with the driver")]
async fn each_is_exchanged_with_the_driver(world: &mut TinmanWorld) {
    let methods = world
        .protocol_methods
        .clone()
        .expect("the protocol's method names were read");
    let session = session(world);
    let command = support::fixture_terminal_source().to_string();
    // `launch` opens a second session and `close` is the only call that ends
    // one, so the two are exchanged together at the end: every other method is
    // sent against the session the background opened, and the session `launch`
    // opens is the one `close` reclaims. Closing the background's session
    // instead would leave every later call addressing a session that is gone.
    let (opening, addressing): (Vec<String>, Vec<String>) = methods
        .iter()
        .cloned()
        .partition(|method| method == "launch" || method == "close");
    let mut opened = session.clone();
    for method in &addressing {
        let id = next_id(world);
        let params = protocol_call_params(method, &session, &command);
        let reply = driver(world).request(rpc(id, method, params));
        world.reply = Some(reply);
    }
    for method in ["launch", "close"] {
        if !opening.iter().any(|named| named == method) {
            continue;
        }
        let id = next_id(world);
        let params = protocol_call_params(method, &opened, &command);
        let reply = driver(world).request(rpc(id, method, params));
        if method == "launch"
            && let Some(name) = reply["result"]["session"].as_str()
        {
            opened = name.to_string();
        }
        world.reply = Some(reply);
    }
}

#[then("every exchange conforms to the protocol")]
async fn every_exchange_conforms_to_the_protocol(world: &mut TinmanWorld) {
    let messages: Vec<serde_json::Value> = driver(world).exchanged().to_vec();
    assert!(
        !messages.is_empty(),
        "no message was exchanged, so this scenario would assert nothing"
    );
    let mut bad = Vec::new();
    for message in &messages {
        for violation in
            support::schema_counterexamples("scantlings/driver-protocol.schema.json", message)
        {
            bad.push(format!("{message}: {violation}"));
        }
    }
    assert!(bad.is_empty(), "schema violations: {bad:#?}");
}

#[then("the methods read are not empty")]
async fn the_methods_read_are_not_empty(world: &mut TinmanWorld) {
    let methods = world
        .protocol_methods
        .as_ref()
        .expect("the protocol's method names were read");
    assert!(
        !methods.is_empty(),
        "the protocol names no method, so this scenario would assert nothing"
    );
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

#[given(expr = "a harness plan whose run step carries the field {string} where {string} was meant")]
async fn a_plan_whose_run_step_carries_the_field(
    world: &mut TinmanWorld,
    written: String,
    _intended: String,
) {
    // The plan carries what the operator wrote. What they meant is the
    // scenario's data rather than the parser's: a field the model does not
    // define is refused whichever field it was mistyped from.
    world.plan_sources.push(format!(
        "flow:\n  - run:\n      command: printf READY\n      {written}: Saved\n"
    ));
}

#[given("a harness plan whose run step names the command 1.0 unquoted")]
async fn a_plan_whose_run_step_names_an_unquoted_number(world: &mut TinmanWorld) {
    // Written as the operator wrote it: YAML reads a bare 1.0 as a number, so
    // the command the plan names never reaches the parser as the text it is.
    world
        .plan_sources
        .push("flow:\n  - run:\n      command: 1.0\n".to_string());
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

#[given(expr = "a harness plan whose step expects the {string} named {string} and states no text")]
async fn a_plan_whose_step_expects_a_locator(world: &mut TinmanWorld, role: String, name: String) {
    // The role and the name are written as quoted scalars, because a name like
    // `%CPU` opens with a YAML indicator and a plain scalar carrying one is not
    // the document the scenario states.
    world.plan_sources.push(format!(
        "tui: top\nsteps:\n  - expect:\n      role: {role:?}\n      name: {name:?}\n"
    ));
}

/// The one expectation the parsed plan states. Production's own reading is what
/// a step about parsing asserts on, so a plan the parser refused is reported as
/// that rather than as a plan carrying nothing.
fn only_parsed_expectation(world: &TinmanWorld) -> tinman::plan::Expectation {
    if let Some(error) = world.parse_error.as_deref() {
        panic!("the plan did not parse: {error}");
    }
    let expectations: Vec<tinman::plan::Expectation> = world
        .parsed_plans
        .first()
        .expect("a harness plan was parsed")
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.clone(),
            tinman::plan::FlowStep::Run(_) => Vec::new(),
        })
        .filter_map(|action| match action {
            tinman::plan::Action::Expect(expectation) => Some(expectation),
            _ => None,
        })
        .collect();
    match expectations.as_slice() {
        [only] => only.clone(),
        _ => panic!(
            "the parsed plan states {} expectations rather than the one the scenario wrote: \
             {expectations:?}",
            expectations.len()
        ),
    }
}

#[then(expr = "the plan's step expects the {string} named {string}")]
async fn the_plans_step_expects_the_role_named(
    world: &mut TinmanWorld,
    role: String,
    name: String,
) {
    let expectation = only_parsed_expectation(world);
    let locator = expectation
        .locator
        .as_ref()
        .unwrap_or_else(|| panic!("the parsed step states no locator; it reads {expectation:?}"));
    assert_eq!(
        locator.role.as_deref(),
        Some(role.as_str()),
        "the role the parsed step's locator addresses"
    );
    assert_eq!(
        locator.name, name,
        "the name the parsed step's locator addresses"
    );
}

/// Every expectation a plan document states, wherever it sits in the flow. The
/// document is read rather than the parsed structure, because whether an
/// expectation carries text is a property of what is written and read back, and
/// a structure normalizing an absent text to an empty one would answer a
/// different question.
fn stated_expectations(document: &serde_json::Value) -> Vec<serde_json::Value> {
    match document {
        serde_json::Value::Object(map) => map
            .iter()
            .flat_map(|(key, nested)| {
                let mut found = stated_expectations(nested);
                if key == "expect" {
                    found.push(nested.clone());
                }
                found
            })
            .collect(),
        serde_json::Value::Array(items) => items.iter().flat_map(stated_expectations).collect(),
        _ => Vec::new(),
    }
}

/// Assert every expectation `document` states carries no text. A bare string is
/// the plan's text form, so an expectation written as one carries text by being
/// that form at all.
fn assert_states_no_text(document: &serde_json::Value, source: &str) {
    let expectations = stated_expectations(document);
    assert!(
        !expectations.is_empty(),
        "the plan states no expectation at all, so there is none to read; it reads:\n{source}"
    );
    for expectation in &expectations {
        assert!(
            !expectation.is_string() && expectation.get("text").is_none(),
            "the expectation carries text, reading {expectation}; the plan reads:\n{source}"
        );
    }
}

#[then("that step states no text")]
async fn that_step_states_no_text(world: &mut TinmanWorld) {
    let source = world
        .plan_sources
        .first()
        .expect("a harness plan was given")
        .clone();
    let document = world
        .serialized
        .clone()
        .expect("the plan was parsed and read back");
    assert_states_no_text(&document, &source);
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

#[then(expr = "parsing fails and reports the unknown field {string}")]
async fn parsing_reports_unknown_field(world: &mut TinmanWorld, field: String) {
    let message = world.parse_error.as_deref().unwrap_or_else(|| {
        panic!("the plan parsed rather than failing: the field {field:?} was accepted and dropped")
    });
    assert!(
        message.contains(&field),
        "the parse failure does not name {field:?}: {message}"
    );
    assert!(
        message.to_lowercase().contains("unknown field"),
        "the parse failure does not report {field:?} as an unknown field: {message}"
    );
}

#[then("parsing fails and reports the value must be quoted to be read as text")]
async fn parsing_reports_the_value_must_be_quoted(world: &mut TinmanWorld) {
    let message = world.parse_error.as_deref().unwrap_or_else(|| {
        panic!("the plan parsed rather than failing: the unquoted number was accepted as a command")
    });
    assert!(
        message.to_lowercase().contains("quote"),
        "the parse failure does not name quoting as the fix: {message}"
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
        .push("home: empty\nnetwork: deny\n".to_string());
}

#[given(expr = "a plan sandbox section that injects {string} from the host")]
async fn a_sandbox_section_injecting_from_the_host(world: &mut TinmanWorld, name: String) {
    world.plan_sources.push(format!(
        "home: empty\nnetwork: deny\nenv:\n  {name}:\n    from: host\n"
    ));
}

#[given(expr = "a plan sandbox section whose path lists {string}")]
async fn a_sandbox_section_whose_path_lists(world: &mut TinmanWorld, entry: String) {
    world
        .plan_sources
        .push(format!("home: empty\nnetwork: deny\npath:\n  - {entry}\n"));
}

#[given(expr = "the operator's environment defines {string} as {string}")]
async fn the_operator_environment_defines(world: &mut TinmanWorld, name: String, value: String) {
    // Hand it to the backend as the operator's environment, which Bubblewrap
    // must clear unless the plan names it, so a leak is a real leak.
    world
        .handed_env
        .get_or_insert_with(std::collections::BTreeMap::new)
        .insert(name.clone(), value.clone());
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

/// The backend the scenario staged: one handed the environment the scenario
/// defined, or one reading the process table where the scenario handed none.
///
/// A scenario stages a host variable by handing the backend a table of its own.
/// One process table serves every worker, so mutating it to stage a grant
/// changes what every concurrent scenario reads, at whatever moment the
/// scheduler interleaves them.
fn staged_backend(world: &TinmanWorld) -> BubblewrapBackend {
    let backend = match world.handed_env.as_ref() {
        Some(env) => BubblewrapBackend::with_environment(env.clone()),
        None => BubblewrapBackend::new(),
    };
    // A scenario that started its own proxy hands the backend that proxy, so a
    // launch is linked to the one its own scenario started. Without the link the
    // association can only be re-derived from ambient state, which is what let
    // two concurrent scenarios bridge each other's ports.
    match world.proxy.as_ref() {
        Some(proxy) => backend.routing_egress_through(proxy),
        None => backend,
    }
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

#[given("a plan whose steps activate a region and fill a field")]
async fn a_plan_activating_and_filling(world: &mut TinmanWorld) {
    // The fixture places its "Save" button inside Settings and draws its
    // labelled input from the first frame, so activation is proved by the
    // button and filling by reading the value back off the field.
    let program = support::fixture_terminal_source();
    let source = format!(
        "tui: {program:?}\nsteps:\n  - expect: READY\n  \
         - activate: {{role: menuitem, name: Settings}}\n  \
         - fill: {{label: Username, value: dmytri}}\n  \
         - expect: dmytri\n"
    );
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the activation plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
}

/// The name the capture scenario's plan keeps its collected items under, written
/// once so the plan that captures and the assertion that reads it name the same
/// capture.
const CAPTURE_NAME: &str = "controls";

#[given(
    expr = "a harness plan whose flow captures the model after activating the {string} named {string}"
)]
async fn a_plan_capturing_after_activating(world: &mut TinmanWorld, role: String, name: String) {
    // The fixture draws its "Save" button only once Settings is open, so a
    // capture placed after the activation reads a screen the earlier steps
    // could not have read. The capture step is written in the form the plan
    // scantling declares: the pane it reads, the role its items play, how much
    // of the pane is read, and the name it keeps them under.
    let program = support::fixture_terminal_source();
    let source = format!(
        "tui: {program:?}\nsteps:\n  - expect: READY\n  \
         - activate: {{role: {role}, name: {name}}}\n  \
         - capture: {{role: application, items: button, scope: visible, as: {CAPTURE_NAME}}}\n"
    );
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the capturing plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
}

#[given(
    expr = "a harness plan driving the fixture terminal program whose final step expects the text {string}"
)]
async fn a_plan_driving_the_fixture_expecting(world: &mut TinmanWorld, text: String) {
    give_fixture_plan(world, Some(&text));
}

/// A plan whose activation step addresses `name`, driving `program`. The locator
/// is written once here so the renaming step below can rebuild the plan against
/// a renamed program while leaving the locator exactly as the plan recorded it.
fn give_activation_plan(world: &mut TinmanWorld, program: &str, name: &str) {
    let source = format!(
        "tui: {program:?}\nsteps:\n  - expect: READY\n  \
         - activate: {{role: menuitem, name: {name}}}\n"
    );
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the activation plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
}

#[given(expr = "a harness plan whose step activates the region named {string}")]
async fn a_plan_whose_step_activates_the_region(world: &mut TinmanWorld, name: String) {
    give_activation_plan(world, support::fixture_terminal_source(), &name);
    world.replay_activated_region = Some(name);
}

#[given(expr = "a harness plan whose step activates the bare name {string}")]
async fn a_plan_whose_step_activates_a_bare_name(world: &mut TinmanWorld, name: String) {
    // A bare name is one of the two written forms of a locator, so the plan is
    // written the way the plan language reads it rather than as a map whose role
    // was left out.
    let program = support::fixture_terminal_source();
    let source = format!("tui: {program:?}\nsteps:\n  - expect: READY\n  - activate: {name:?}\n");
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the bare-name activation plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
    world.replay_roleless_locator = Some(name);
}

#[given(expr = "a harness plan whose step expects the region named {string} and states no role")]
async fn a_plan_whose_step_expects_a_roleless_region(world: &mut TinmanWorld, name: String) {
    // An expectation carrying a locator and no text is the bare-locator form,
    // and the role is what this one leaves out. The name is written as a quoted
    // scalar, because a name opening with a YAML indicator is not the document
    // the scenario states.
    let program = support::fixture_terminal_source();
    let source =
        format!("tui: {program:?}\nsteps:\n  - expect: READY\n  - expect:\n      name: {name:?}\n");
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the bare-locator expectation plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
    world.replay_roleless_locator = Some(name);
}

#[then("the replay fails and reports the locator that named no role")]
async fn the_replay_fails_reporting_the_roleless_locator(world: &mut TinmanWorld) {
    let name = world
        .replay_roleless_locator
        .clone()
        .expect("the plan named the locator its step addresses");
    let reported = world
        .flow_error
        .as_deref()
        .expect("the replay was expected to fail, and it passed");
    assert!(
        reported.contains(&name),
        "the failure does not name the locator that stated no role, the one named {name:?}:\n{reported}"
    );
}

#[then("the replay fails and reports the scope that matched no region")]
async fn the_replay_fails_reporting_the_unmatched_scope(world: &mut TinmanWorld) {
    let (scope, _, _) = world
        .scoped_activation
        .clone()
        .expect("the plan named the region its step narrows to");
    let reported = world
        .flow_error
        .as_deref()
        .expect("the replay was expected to fail, and it passed");
    assert!(
        reported.contains(&scope),
        "the failure does not name the scope that matched no region, {scope:?}:\n{reported}"
    );
}

/// The scoped activation a scenario's plan carries: the region it narrows to,
/// the role and the name it addresses, so the step that follows reads the same
/// three the plan was written with.
type ScopedActivation = (String, String, String);

/// Every region of `model` playing `role` and carrying `name`, wherever it sits.
fn regions_named(model: &tinman::tom::Model, role: &str, name: &str) -> Vec<tinman::tom::Rect> {
    fn walk(
        region: &tinman::tom::Region,
        role: &str,
        name: &str,
        found: &mut Vec<tinman::tom::Rect>,
    ) {
        if region.role() == role && region.name.as_deref() == Some(name) {
            found.push(region.rect);
        }
        for child in &region.children {
            walk(child, role, name, found);
        }
    }
    let mut found = Vec::new();
    walk(&model.root, role, name, &mut found);
    found
}

/// Whether `inner` is drawn inside `outer`, read off the rectangles the model
/// reports. Containment is where a region is drawn rather than where the model
/// files it, so a precondition about what the program shows stays readable
/// whatever the model does with it.
fn drawn_inside(outer: tinman::tom::Rect, inner: tinman::tom::Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// A plan whose activation step narrows to `scope`, driving `program`. The
/// locator is written once here so the step below can rebuild the plan against a
/// program that draws the scope while leaving the locator exactly as the plan
/// recorded it.
fn give_scoped_activation_plan(
    world: &mut TinmanWorld,
    program: &str,
    role: &str,
    name: &str,
    scope: &str,
) {
    // The narrowing key is the one the plan schema declares, so the plan is
    // written the way the contract says an operator writes one.
    let source = format!(
        "tui: {program:?}\nsteps:\n  - expect: READY\n  \
         - activate: {{role: {role}, name: {name}, within: {scope}}}\n"
    );
    world.replay_plan = Some(
        tinman::plan::parse(&source)
            .unwrap_or_else(|e| panic!("the scoped activation plan did not parse: {e}")),
    );
    world.replay_plan_source = Some(source);
}

#[given(
    expr = "a harness plan whose step activates the {string} named {string} within the region named {string}"
)]
async fn a_plan_whose_step_activates_within(
    world: &mut TinmanWorld,
    role: String,
    name: String,
    scope: String,
) {
    // The step states what the plan says, and says nothing about what the driven
    // program draws, so the program is the ordinary fixture until a step states
    // otherwise. A scenario about a scope the screen does not carry is exactly
    // the one that states nothing further, and a fixture built to draw whatever
    // scope it is handed would carry that scope and answer the opposite of what
    // the scenario asks.
    give_scoped_activation_plan(
        world,
        support::fixture_terminal_source(),
        &role,
        &name,
        &scope,
    );
    world.scoped_activation = Some((scope, role, name));
}

#[given(
    expr = "the program under replay shows another {string} named {string} outside that region"
)]
async fn the_program_shows_another_outside(world: &mut TinmanWorld, role: String, name: String) {
    let (scope, activated_role, activated_name) = world
        .scoped_activation
        .clone()
        .expect("the plan named the region its step narrows to");
    // The plan keeps the locator it recorded; only the program changes, so the
    // replay is the recorded flow meeting the program this step describes.
    let program = support::fixture_with_scoped_button_source(&scope, &name);
    give_scoped_activation_plan(world, &program, &activated_role, &activated_name, &scope);
    let screen = support::scoped_button_screen(&scope, &name);
    let model = tinman::tom::build(&screen);
    let region = model
        .find_named(&scope)
        .unwrap_or_else(|| {
            panic!(
                "the program under replay draws no region named {scope:?}; its screen reads:\n{}",
                screen.contents()
            )
        })
        .rect;
    let found = regions_named(&model, &role, &name);
    assert_eq!(
        found.len(),
        2,
        "the program under replay draws {} {role}(s) named {name:?}, so the locator it is \
         given is not the ambiguous one this scenario is about; its screen reads:\n{}",
        found.len(),
        screen.contents()
    );
    let inside = found
        .iter()
        .filter(|rect| drawn_inside(region, **rect))
        .count();
    assert_eq!(
        inside,
        1,
        "{inside} of the {role}s named {name:?} are drawn inside {scope:?}, so one inside and \
         one outside is not what the program shows; its screen reads:\n{}",
        screen.contents()
    );
}

/// The written forms of a plan, together carrying every property the schema
/// declares, so a round trip through production's own reader and writer shows
/// which of those declarations production really keeps. Several forms are
/// needed because the schema declares the full form and the shorthand as
/// alternatives at the root, and no single plan is written both ways. Each step
/// kind the schema declares appears, so a kind production cannot read costs its
/// own properties their credit and leaves the rest of the set creditable.
const RECORDED_PLAN_FORMS: [&str; 5] = [
    "sandbox: {}\n\
     flow:\n\
     \x20 - run:\n\
     \x20     command: \"true\"\n\
     \x20     cwd: \"work\"\n\
     \x20     status: 0\n\
     \x20     stdin: \"input\"\n\
     \x20 - tui:\n\
     \x20     command: \"./fixture-tui\"\n\
     \x20     steps:\n\
     \x20       - expect: {text: \"READY\", within: \"Files\"}\n\
     \x20       - activate: {role: \"button\", name: \"OK\", within: \"Settings\", binding: \"scoped\"}\n\
     \x20       - fill: {label: \"Username\", value: \"ada\"}\n\
     \x20       - press: \"q\"\n",
    "tui: \"./fixture-tui\"\n\
     steps:\n\
     \x20 - press: \"q\"\n",
    "tui: \"./fixture-tui\"\n\
     steps:\n\
     \x20 - capture: {role: \"article\", items: \"log\", scope: \"all\", as: \"conversation\"}\n",
    "tui: \"./fixture-tui\"\n\
     steps:\n\
     \x20 - press: \"q\"\n\
     sources:\n\
     \x20 - source: \"https://github.com/tldr-pages/tldr\"\n\
     \x20   licence: \"CC-BY-4.0\"\n",
    "tui: \"./fixture-tui\"\n\
     steps:\n\
     \x20 - conforms: \"top-layout.json\"\n",
];

#[given(expr = "the properties declared by {string}")]
async fn the_properties_declared_by(world: &mut TinmanWorld, path: String) {
    world.declared_plan_properties = Some(support::declared_properties(&path));
    world.plan_schema_path = Some(path);
}

#[when("a recorded plan is serialized and read back")]
async fn a_recorded_plan_is_serialized_and_read_back(world: &mut TinmanWorld) {
    let path = world
        .plan_schema_path
        .clone()
        .expect("the schema whose properties were read");
    let mut carried = std::collections::BTreeSet::new();
    let mut unread = Vec::new();
    for source in RECORDED_PLAN_FORMS {
        // The round trip is production's own: its reader takes the written plan
        // and its writer puts it back, so a property production never learned to
        // read is absent from what it writes. A form its reader refuses carries
        // nothing forward, and the refusal is kept so the loss is reported with
        // the reason rather than as a bare absence.
        let plan = match tinman::plan::parse(source) {
            Ok(plan) => plan,
            Err(e) => {
                unread.push(e);
                continue;
            }
        };
        let written = serde_yaml::to_string(&plan)
            .unwrap_or_else(|e| panic!("the parsed plan was not written back: {e}"));
        let instance: serde_json::Value = serde_yaml::from_str(&written)
            .unwrap_or_else(|e| panic!("the written plan did not read back: {e}\n{written}"));
        carried.extend(support::carried_properties(&path, &instance));
        // The schema declares the full form and the shorthand as alternatives
        // at the root and says both normalize to the same parsed plan, so a
        // shorthand key is an input form production consumes rather than a
        // property it drops. It is carried where the schema declares it, on a
        // plan production read, and that is where it is credited.
        let given: serde_json::Value = serde_yaml::from_str(source)
            .unwrap_or_else(|e| panic!("the recorded plan did not read as a document: {e}"));
        carried.extend(
            support::carried_properties(&path, &given)
                .into_iter()
                .filter(|(site, _)| site.is_empty()),
        );
    }
    world.carried_plan_properties = Some(carried);
    world.unread_plan_forms = Some(unread);
}

#[then("every declared property survives the round trip")]
async fn every_declared_property_survives(world: &mut TinmanWorld) {
    let declared = world
        .declared_plan_properties
        .as_ref()
        .expect("the schema's declared properties were read");
    let carried = world
        .carried_plan_properties
        .as_ref()
        .expect("the round trip was made");
    let lost: Vec<String> = declared
        .iter()
        .filter(|property| !carried.contains(*property))
        .map(|(site, name)| {
            format!(
                "{name} declared at {}",
                if site.is_empty() { "/" } else { site }
            )
        })
        .collect();
    let unread = world
        .unread_plan_forms
        .as_ref()
        .expect("the round trip was made");
    assert!(
        lost.is_empty(),
        "{} declared propert(ies) do not survive a plan's round trip through production: {}{}",
        lost.len(),
        lost.join(", "),
        if unread.is_empty() {
            String::new()
        } else {
            format!(
                "\nproduction's reader refused {} written form(s): {}",
                unread.len(),
                unread.join("; ")
            )
        }
    );
}

#[given("the program under replay has renamed that region")]
async fn the_program_under_replay_has_renamed_that_region(world: &mut TinmanWorld) {
    let name = world
        .replay_activated_region
        .clone()
        .expect("the plan named the region its step activates");
    // The plan keeps the locator it recorded; only the program changes, so the
    // replay is the recorded flow meeting a program that renamed its region.
    let renamed = support::fixture_terminal_source_renaming(&name, "Preferences");
    give_activation_plan(world, &renamed, &name);
}

#[then("the replay fails and reports the locator that bound no matching region")]
async fn the_replay_fails_reporting_the_unbound_locator(world: &mut TinmanWorld) {
    let name = world
        .replay_activated_region
        .clone()
        .expect("the plan named the region its step activates");
    let reported = world
        .flow_error
        .as_deref()
        .expect("the replay was expected to fail, and it passed");
    assert!(
        reported.contains("menuitem") && reported.contains(&name),
        "the failure does not name the locator that bound no matching region, the {:?} named {name:?}:\n{reported}",
        "menuitem"
    );
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
    run_tinman_command(world, &["test", &path.to_string_lossy()]).await;
}

#[when("the operator tests that plan with the streams captured separately")]
async fn the_operator_tests_that_plan_with_streams_separated(world: &mut TinmanWorld) {
    let source = world
        .replay_plan_source
        .clone()
        .expect("a harness plan was given");
    let dir = working_dir(world);
    let path = dir.join("plan.yaml");
    std::fs::write(&path, &source)
        .unwrap_or_else(|e| panic!("the plan {} was not written: {e}", path.display()));
    run_tinman_command(world, &["test", &path.to_string_lossy()]).await;
}

#[given(expr = "no file named {string} exists")]
async fn no_file_named_exists(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    if path.exists() {
        std::fs::remove_file(&path)
            .unwrap_or_else(|e| panic!("the file {} was not removed: {e}", path.display()));
    }
    world.missing_plan = Some(name);
}

#[when(expr = "the operator tests the plan {string}")]
async fn the_operator_tests_the_plan_named(world: &mut TinmanWorld, name: String) {
    run_tinman_command(world, &["test", &name]).await;
}

#[given("a plan file that is not valid YAML")]
async fn a_plan_file_that_is_not_valid_yaml(world: &mut TinmanWorld) {
    // A document no YAML parser can read: a flow sequence that is never closed.
    world.replay_plan_source = Some("tui: [unclosed\nsteps: {\n".to_string());
}

#[then("the failure names the plan file that was not read")]
async fn the_failure_names_the_plan_file(world: &mut TinmanWorld) {
    let name = world.missing_plan.as_ref().expect("a plan file was named");
    let reported = run_errors(world);
    assert!(
        reported.contains(name.as_str()),
        "the failure does not name the plan file {name}; it reads:\n{reported}"
    );
}

#[then("the failure reports the plan did not parse")]
async fn the_failure_reports_the_plan_did_not_parse(world: &mut TinmanWorld) {
    let reported = run_errors(world);
    assert!(
        reported.to_lowercase().contains("parse"),
        "the failure does not report that the plan did not parse; it reads:\n{reported}"
    );
}

#[then("the output carries no panic")]
async fn the_output_carries_no_panic(world: &mut TinmanWorld) {
    // A panic reaches whichever stream the runtime writes it to, so both are
    // read: a handled failure on one stream beside an abort on the other is the
    // reading a single-stream check would pass.
    let streams = [
        ("the data stream", run_output(world).to_string()),
        ("the error stream", run_errors(world).to_string()),
    ];
    for (stream, text) in streams {
        assert!(
            !text.contains("panicked"),
            "{stream} carries a panic where a reported failure belongs; it reads:\n{text}"
        );
    }
}

#[then(expr = "the error stream reports the step expecting {string}")]
async fn the_error_stream_reports_the_step_expecting(world: &mut TinmanWorld, text: String) {
    let reported = run_errors(world);
    assert!(
        reported.contains(text.as_str()),
        "the error stream does not report the step expecting {text:?}; it reads:\n{reported}"
    );
}

#[then(expr = "the error stream reports Tinman has no {string} command")]
async fn the_error_stream_reports_no_such_command(world: &mut TinmanWorld, name: String) {
    let reported = run_errors(world);
    assert!(
        reported.contains(name.as_str()),
        "the error stream does not name the refused command {name:?}; it reads:\n{reported}"
    );
    // The refusal is read by its sense rather than by one wording, so the
    // parser's own sentence and a sentence the man path returns both satisfy
    // it, and neither is dictated here.
    let lowered = reported.to_lowercase();
    assert!(
        lowered.contains("unrecognized") || lowered.contains("has no"),
        "the error stream names {name:?} without reporting Tinman has no such command; it reads:\n{reported}"
    );
}

#[then("the error stream carries no panic")]
async fn the_error_stream_carries_no_panic(world: &mut TinmanWorld) {
    let reported = run_errors(world);
    assert!(
        !reported.contains("panicked"),
        "the error stream carries a panic where a reported failure belongs; it reads:\n{reported}"
    );
}

#[then("the data stream carries nothing")]
async fn the_data_stream_carries_nothing(world: &mut TinmanWorld) {
    let data = run_output(world);
    assert!(
        data.trim().is_empty(),
        "the data stream carries a diagnostic where a consumer reads only data; it reads:\n{data}"
    );
}

/// Run the real `tinman` binary in the scenario's working directory, keeping
/// what it wrote and the status it left.
///
/// The run is awaited rather than blocked on, so the scenarios beside this one
/// keep being polled while this one waits on its process. Blocking here holds
/// the single executor thread the runner polls every scenario on, which is what
/// made a worker count above one buy nothing.
async fn run_tinman_command(world: &mut TinmanWorld, args: &[&str]) {
    let dir = working_dir(world);
    let env = configured_env(world);
    let owned: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
    let outcome = support::run_tinman_awaited(dir, owned, env, None)
        .await
        .unwrap_or_else(|e| panic!("the tinman binary did not run: {e}"));
    world.run_stdout = Some(outcome.stdout);
    world.run_stderr = Some(outcome.stderr);
    world.run_status = Some(outcome.status);
}

/// What the last run of the real binary wrote to stderr, where a failing run
/// puts its diagnosis.
fn run_errors(world: &TinmanWorld) -> &str {
    world.run_stderr.as_deref().unwrap_or_default()
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
    // Inspection launches its target inside the sandbox, which binds the
    // workspace and nothing else, so the fixture is staged there and named
    // relatively rather than by an absolute path the sandbox cannot reach.
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    run_tinman_command(world, &["inspect", &program]).await;
}

#[when("the operator inspects the fixture terminal program as JSON")]
async fn the_operator_inspects_the_fixture_as_json(world: &mut TinmanWorld) {
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    run_tinman_command(world, &["inspect", &program, "--json"]).await;
}

#[when(expr = "the operator inspects the command {string}")]
async fn the_operator_inspects_the_command(world: &mut TinmanWorld, command: String) {
    run_tinman_command(world, &["inspect", &command]).await;
}

#[when(
    expr = "the operator inspects a command that writes to the sentinel path and prints {string}"
)]
async fn the_operator_inspects_a_command_writing_the_sentinel(
    world: &mut TinmanWorld,
    text: String,
) {
    let sentinel = world
        .sentinel
        .clone()
        .expect("a sentinel path was given")
        .display()
        .to_string();
    // The sandbox carries no /dev/null, so stderr is closed rather than
    // redirected: the refusal the sandbox issues must not be drawn on the
    // screen the assertion below reads.
    let command = format!("exec 2>&-; touch {sentinel}; printf {text}");
    run_tinman_command(world, &["inspect", &command]).await;
}

/// One line of the inspect listing, read as the depth it is indented to, the
/// role it names, and the accessible name it carries. The listing renders a
/// region as its role followed by its name in quotes, indented two spaces per
/// level, so the rendered text is parsed back into the tree it describes.
fn listing_rows(output: &str) -> Vec<(usize, String, Option<String>)> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let depth = (line.len() - line.trim_start().len()) / 2;
            let body = line.trim_start();
            match body.split_once(" \"") {
                Some((role, rest)) => (
                    depth,
                    role.to_string(),
                    Some(rest.trim_end_matches('"').to_string()),
                ),
                None => (depth, body.to_string(), None),
            }
        })
        .collect()
}

#[when(expr = "the operator inspects {string}")]
async fn the_operator_inspects(world: &mut TinmanWorld, program: String) {
    run_tinman_command(world, &["inspect", &program]).await;
}

#[when("the operator inspects that screen")]
async fn the_operator_inspects_that_screen(world: &mut TinmanWorld) {
    // The listing the inspect command prints, rendered from the model of the
    // screen the starting state drew, through the same seam the command itself
    // renders with.
    let screen = world.screen.as_ref().expect("a virtual screen");
    world.run_stdout = Some(tinman::inspect::render(&tinman::tom::build(screen)));
}

#[then(expr = "the inspect output shows the status region carries {string}")]
async fn the_inspect_output_shows_the_status_text(world: &mut TinmanWorld, text: String) {
    let output = run_output(world);
    assert!(
        output
            .lines()
            .any(|line| line.trim_start().starts_with("status") && line.contains(&text)),
        "no status line of the inspect output carries {text:?}; it reads:\n{output}"
    );
}

#[given("a command the sandbox cannot execute")]
async fn a_command_the_sandbox_cannot_execute(world: &mut TinmanWorld) {
    // A program staged in the scenario's own workspace without the executable
    // bit. Inspection binds that workspace, so the sandbox finds the program
    // and still cannot run it, which is the failure the scenario is about, and
    // it is the scenario's own file rather than a name the host happens to
    // lack.
    let workspace = working_dir(world);
    let path = workspace.join("not-executable");
    std::fs::write(&path, "printf READY\n")
        .unwrap_or_else(|e| panic!("the staged program {} was not written: {e}", path.display()));
    world.unexecutable_command = Some("./not-executable".to_string());
}

#[when("the operator inspects that command with the streams captured separately")]
async fn the_operator_inspects_that_command_with_streams_separated(world: &mut TinmanWorld) {
    let command = world
        .unexecutable_command
        .clone()
        .expect("the starting state staged the command the sandbox cannot execute");
    run_tinman_command(world, &["inspect", &command]).await;
}

#[then("the error stream reports the failure")]
async fn the_error_stream_reports_the_failure(world: &mut TinmanWorld) {
    let command = world
        .unexecutable_command
        .clone()
        .expect("the starting state staged the command the sandbox cannot execute");
    let reported = run_errors(world);
    assert!(
        reported.contains(command.as_str()),
        "the error stream does not report the failure of {command:?}; it reads:\n{reported}"
    );
}

#[when("the operator inspects a command that asks terminfo for the terminal width")]
async fn the_operator_inspects_a_terminfo_query(world: &mut TinmanWorld) {
    // `tput cols` is how an ordinary program asks the terminfo database what its
    // terminal can do, and it answers "unknown terminal" where TERM names
    // nothing the database describes. A fixture writing its own escapes never
    // consults the database at all, so it would prove the harness rather than
    // the sandbox.
    run_tinman_command(world, &["inspect", "tput cols"]).await;
}

#[then("the inspect output reports the width rather than an unknown terminal")]
async fn the_inspect_output_reports_the_width(world: &mut TinmanWorld) {
    let output = run_output(world);
    assert!(
        !output.contains("unknown terminal"),
        "terminfo described no terminal for the sandboxed program; the inspect output reads:\n{output}"
    );
    let widths: Vec<String> = listing_rows(output)
        .into_iter()
        .filter_map(|(_, _, name)| name)
        .filter(|name| name.trim().parse::<u32>().is_ok())
        .collect();
    assert!(
        !widths.is_empty(),
        "the inspect output names no terminal width, so terminfo answered nothing:\n{output}"
    );
}

#[then("the inspect output lists at least one region")]
async fn the_inspect_output_lists_at_least_one_region(world: &mut TinmanWorld) {
    let output = run_output(world);
    assert!(
        !output.contains("no regions on screen"),
        "the inspect output reports an empty screen:\n{output}"
    );
    assert!(
        !listing_rows(output).is_empty(),
        "the inspect output lists nothing at all:\n{output}"
    );
}

#[given("the host's terminal database is unavailable")]
async fn the_hosts_terminal_database_is_unavailable(world: &mut TinmanWorld) {
    // ncurses reads TERMINFO first and TERMINFO_DIRS after it, so pointing both
    // at a directory carrying no database is how the host's definitions are
    // taken away from the run. What Tinman carries of its own is then the only
    // thing left for the sandboxed program to read.
    let empty = working_dir(world).join("no-terminfo");
    std::fs::create_dir_all(&empty)
        .unwrap_or_else(|e| panic!("the empty terminfo directory was not made: {e}"));
    let path = empty.display().to_string();
    world.env_vars.insert("TERMINFO".to_string(), path.clone());
    world.env_vars.insert("TERMINFO_DIRS".to_string(), path);
}

#[then("the inspect output reports the program did not start")]
async fn the_inspect_output_reports_the_program_did_not_start(world: &mut TinmanWorld) {
    // What the command reported is read across both its streams. A diagnostic
    // belongs on the error stream, which `features/command-surface.feature`
    // states, so a check reading only the data stream would be asserting which
    // stream the report reached rather than that it was reported at all.
    let output = format!("{}\n{}", run_output(world), run_errors(world));
    // The scenario's own words are the message. The project already says "could
    // not start" of a launch that failed, so either wording names what happened.
    assert!(
        output.contains("did not start") || output.contains("could not start"),
        "the inspect output does not report that the program failed to start:\n{output}"
    );
}

#[then("the inspect output does not report no regions on screen")]
async fn the_inspect_output_does_not_report_no_regions(world: &mut TinmanWorld) {
    let output = run_output(world);
    assert!(
        !output.contains("no regions on screen"),
        "the inspect output reports an empty screen for a program that never started:\n{output}"
    );
}

#[when("the operator inspects a command printing 200 numbered lines in a terminal 24 rows high")]
async fn the_operator_inspects_200_numbered_lines(world: &mut TinmanWorld) {
    // The PTY inspection opens is 24 rows high, so 200 lines is far more than
    // one screenful: only a reading of the whole stream carries the first line.
    let command = "i=1; while [ $i -le 200 ]; do printf 'line %s\\n' \"$i\"; i=$((i+1)); done";
    run_tinman_command(world, &["inspect", command]).await;
}

#[then(expr = "the inspect output carries the line numbered {int}")]
async fn the_inspect_output_carries_the_line_numbered(world: &mut TinmanWorld, number: u32) {
    let output = run_output(world);
    let expected = format!("{:?}", format!("line {number}"));
    assert!(
        output.contains(&expected),
        "the inspect output carries no line numbered {number}; it reads:\n{output}"
    );
}

#[when(
    expr = "the operator inspects a command printing {string}, {string} and {string} on their own lines"
)]
async fn the_operator_inspects_three_lines(
    world: &mut TinmanWorld,
    first: String,
    second: String,
    third: String,
) {
    let command = format!("printf '{first}\\n{second}\\n{third}\\n'");
    run_tinman_command(world, &["inspect", &command]).await;
}

#[when("the operator inspects a command printing two two-line blocks separated by a blank line")]
async fn the_operator_inspects_two_blocks(world: &mut TinmanWorld) {
    let command = "printf 'build started\\nat 10:00\\n\\nbuild finished\\nat 10:05\\n'";
    run_tinman_command(world, &["inspect", command]).await;
}

#[when(expr = "the operator inspects a command printing {string}, a blank line and {string}")]
async fn the_operator_inspects_two_lines_apart(
    world: &mut TinmanWorld,
    first: String,
    second: String,
) {
    let command = format!("printf '{first}\\n\\n{second}\\n'");
    run_tinman_command(world, &["inspect", &command]).await;
}

#[then(expr = "the inspect output lists a {string} holding {int} {string} regions")]
async fn the_inspect_output_lists_holding(
    world: &mut TinmanWorld,
    role: String,
    count: usize,
    child_role: String,
) {
    let output = run_output(world);
    let rows = listing_rows(output);
    let found = rows.iter().enumerate().any(|(index, (depth, listed, _))| {
        listed == &role
            && rows[index + 1..]
                .iter()
                .take_while(|(child_depth, _, _)| child_depth > depth)
                .filter(|(child_depth, child, _)| *child_depth == depth + 1 && child == &child_role)
                .count()
                == count
    });
    assert!(
        found,
        "the inspect output lists no {role:?} holding {count} {child_role:?} regions; it reads:\n{output}"
    );
}

#[when(expr = "the operator inspects a command printing {string} and {string} on their own lines")]
async fn the_operator_inspects_two_lines(world: &mut TinmanWorld, first: String, second: String) {
    // Each line is passed as an argument rather than spliced into the format
    // string, so a line beginning with a dash reaches the program as data
    // instead of being parsed by printf as an option.
    let command = format!("printf '%s\\n' '{first}' '{second}'");
    run_tinman_command(world, &["inspect", &command]).await;
}

#[then(expr = "the inspect output lists a {string} holding at least {int} {string} region")]
async fn the_inspect_output_lists_holding_at_least(
    world: &mut TinmanWorld,
    role: String,
    count: usize,
    child_role: String,
) {
    let output = run_output(world);
    let rows = listing_rows(output);
    let found = rows.iter().enumerate().any(|(index, (depth, listed, _))| {
        listed == &role
            && rows[index + 1..]
                .iter()
                .take_while(|(child_depth, _, _)| child_depth > depth)
                .filter(|(child_depth, child, _)| *child_depth == depth + 1 && child == &child_role)
                .count()
                >= count
    });
    assert!(
        found,
        "the inspect output lists no {role:?} holding at least {count} {child_role:?} regions; it reads:\n{output}"
    );
}

#[then(expr = "the inspect output lists no {string} region")]
async fn the_inspect_output_lists_no_region(world: &mut TinmanWorld, role: String) {
    let output = run_output(world);
    let rows = listing_rows(output);
    let found = rows.iter().any(|(_, listed, _)| listed == &role);
    assert!(
        !found,
        "the inspect output lists a {role:?} region; it reads:\n{output}"
    );
}

/// One row of a listing's table: the accessible name the row carries where it
/// has one, and the cells it holds in the order they were drawn.
type TableRow = (Option<String>, Vec<String>);

/// The table a listing carries, read as the roles the terminal object model
/// names: the column headers the table labels its columns with, and each row
/// with the cells it holds in the order they were drawn. A table with no header
/// row carries no column headers, which is the positional case the scenarios
/// address by ordinal rather than by label.
fn listing_table(output: &str) -> Option<(Vec<String>, Vec<TableRow>)> {
    let rows = listing_rows(output);
    let (index, depth) = rows
        .iter()
        .enumerate()
        .find(|(_, (_, role, _))| role.as_str() == "table")
        .map(|(index, (depth, _, _))| (index, *depth))?;
    let mut headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<(Option<String>, Vec<String>)> = Vec::new();
    for (child_depth, role, name) in rows[index + 1..]
        .iter()
        .take_while(|(child_depth, _, _)| *child_depth > depth)
    {
        match (child_depth - depth, role.as_str()) {
            (1, "columnheader") => headers.push(name.clone().unwrap_or_default()),
            (1, "row") => table_rows.push((name.clone(), Vec::new())),
            (2, "cell" | "columnheader") => {
                if let Some(last) = table_rows.last_mut() {
                    last.1.push(name.clone().unwrap_or_default());
                }
            }
            _ => {}
        }
    }
    Some((headers, table_rows))
}

/// The cells of the row that names the given value, whether the row carries the
/// value as its own accessible name or holds it in one of its cells.
fn row_naming<'a>(table_rows: &'a [TableRow], name: &str) -> Option<&'a Vec<String>> {
    table_rows
        .iter()
        .find(|(row_name, cells)| {
            row_name.as_deref() == Some(name) || cells.iter().any(|cell| cell == name)
        })
        .map(|(_, cells)| cells)
}

#[then(expr = "the {string} cell of the row naming {string} is {string}")]
async fn the_labelled_cell_of_the_row_is(
    world: &mut TinmanWorld,
    column: String,
    row_name: String,
    value: String,
) {
    let output = run_output(world);
    let (headers, table_rows) = listing_table(output)
        .unwrap_or_else(|| panic!("the inspect output lists no table; it reads:\n{output}"));
    let column_index = headers
        .iter()
        .position(|header| header == &column)
        .unwrap_or_else(|| {
            panic!("the inspect output lists no columnheader named {column:?}; it reads:\n{output}")
        });
    let cells = row_naming(&table_rows, &row_name).unwrap_or_else(|| {
        panic!("the inspect output lists no row naming {row_name:?}; it reads:\n{output}")
    });
    let cell = cells.get(column_index).unwrap_or_else(|| {
        panic!("the row naming {row_name:?} holds no cell under {column:?}; it reads:\n{output}")
    });
    assert_eq!(
        cell, &value,
        "the {column:?} cell of the row naming {row_name:?} is {cell:?}; it reads:\n{output}"
    );
}

#[then(expr = "the second cell of the row naming {string} is {string}")]
async fn the_second_cell_of_the_row_is(world: &mut TinmanWorld, row_name: String, value: String) {
    let output = run_output(world);
    let (_, table_rows) = listing_table(output)
        .unwrap_or_else(|| panic!("the inspect output lists no table; it reads:\n{output}"));
    let cells = row_naming(&table_rows, &row_name).unwrap_or_else(|| {
        panic!("the inspect output lists no row naming {row_name:?}; it reads:\n{output}")
    });
    let cell = cells.get(1).unwrap_or_else(|| {
        panic!("the row naming {row_name:?} holds no second cell; it reads:\n{output}")
    });
    assert_eq!(
        cell, &value,
        "the second cell of the row naming {row_name:?} is {cell:?}; it reads:\n{output}"
    );
}

#[then(expr = "the fifth cell of the first row is {string}")]
async fn the_fifth_cell_of_the_first_row_is(world: &mut TinmanWorld, value: String) {
    let output = run_output(world);
    let (_, table_rows) = listing_table(output)
        .unwrap_or_else(|| panic!("the inspect output lists no table; it reads:\n{output}"));
    let (_, cells) = table_rows
        .first()
        .unwrap_or_else(|| panic!("the inspect output lists no row; it reads:\n{output}"));
    let cell = cells
        .get(4)
        .unwrap_or_else(|| panic!("the first row holds no fifth cell; it reads:\n{output}"));
    assert_eq!(
        cell, &value,
        "the fifth cell of the first row is {cell:?}; it reads:\n{output}"
    );
}

#[when(expr = "the operator inspects a command that prints {string} in red")]
async fn the_operator_inspects_a_command_printing_in_red(world: &mut TinmanWorld, text: String) {
    // A real SGR sequence sets the colour and clears it after, so the sandboxed
    // program draws the attribute rather than the fixture asserting one.
    let command = format!("printf '\\033[31m{text}\\033[0m'");
    run_tinman_command(world, &["inspect", &command]).await;
}

#[then(expr = "the inspect output reports that region is drawn in red")]
async fn the_inspect_output_reports_that_region_is_red(world: &mut TinmanWorld) {
    let output = run_output(world);
    assert!(
        output.contains("red"),
        "the inspect output names no colour for the region it lists:\n{output}"
    );
}

/// One of the standard presentations ECMA-48 defines, named as a scenario names
/// it. Two of the six are two words, which alternation in a cucumber expression
/// cannot carry, so the set is declared once here rather than split across six
/// step definitions.
#[derive(Debug, cucumber::Parameter)]
#[param(
    name = "attribute",
    regex = "dim|italic|underlined|hidden|struck through|doubly underlined"
)]
struct StandardAttribute(String);

impl std::str::FromStr for StandardAttribute {
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(StandardAttribute(name.to_string()))
    }
}

impl StandardAttribute {
    /// The SGR parameters that turn this presentation on, as the emulator
    /// Tinman captures through parses them. Double underline is the `4:2`
    /// sub-parameter form rather than a parameter of its own.
    fn sgr(&self) -> &'static str {
        match self.0.as_str() {
            "dim" => "2",
            "italic" => "3",
            "underlined" => "4",
            "hidden" => "8",
            "struck through" => "9",
            "doubly underlined" => "4:2",
            other => panic!("no SGR parameter is known for the attribute {other:?}"),
        }
    }
}

#[when(expr = "the operator inspects a command that prints {string} {attribute}")]
async fn the_operator_inspects_a_command_printing_attributed(
    world: &mut TinmanWorld,
    text: String,
    attribute: StandardAttribute,
) {
    // A real SGR sequence sets the attribute and clears it after, so the
    // sandboxed program draws the presentation rather than the fixture
    // asserting one.
    let command = format!("printf '\\033[{}m{text}\\033[0m'", attribute.sgr());
    run_tinman_command(world, &["inspect", &command]).await;
}

#[then(expr = "the inspect output reports that region is {attribute}")]
async fn the_inspect_output_reports_that_region_is(
    world: &mut TinmanWorld,
    attribute: StandardAttribute,
) {
    let output = run_output(world);
    assert!(
        output.contains(&attribute.0),
        "the inspect output reports no {:?} presentation for the region it lists:\n{output}",
        attribute.0
    );
}

#[then(expr = "the inspect output lists a region named {string}")]
async fn the_inspect_output_lists_a_region_named(world: &mut TinmanWorld, name: String) {
    let output = run_output(world);
    // The listing renders a named region as its role followed by the name in
    // quotes, so the quoted form is what distinguishes a named region from a
    // line that merely carries the text.
    let quoted = format!("{name:?}");
    let listed = output.lines().any(|line| line.contains(&quoted));
    assert!(
        listed,
        "no line of the inspect output lists a region named {name:?}:\n{output}"
    );
}

#[then(expr = "the inspect output names the root region {string}")]
async fn the_inspect_output_names_the_root_region(world: &mut TinmanWorld, name: String) {
    let output = run_output(world);
    // The listing indents each region under its parent, so the root region is
    // the first unindented line, and a named region renders as its role
    // followed by the name in quotes.
    let root = output
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with(char::is_whitespace))
        .unwrap_or_else(|| panic!("the inspect output lists no root region:\n{output}"));
    let quoted = format!("{name:?}");
    assert!(
        root.contains(&quoted),
        "the root region line {root:?} does not name the region {name:?}:\n{output}"
    );
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

/// Everything the last run of the real binary wrote, both streams together. A
/// scenario naming "the output" means what the command wrote to the operator;
/// which stream carried it is the separate concern that
/// `features/command-surface.feature` asserts on, and reading one stream here
/// would make these two readings contradict.
fn run_all_output(world: &TinmanWorld) -> String {
    format!("{}{}", run_output(world), run_errors(world))
}

#[then(expr = "the output reports the step expecting {string}")]
async fn the_output_reports_the_step_expecting(world: &mut TinmanWorld, text: String) {
    let output = run_all_output(world);
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
    let output = run_all_output(world);
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

/// Replay the scenario's plan through the real command, so the status the
/// binary leaves is what the scenario's `Then the plan passes` reads.
#[when("the plan is replayed")]
async fn the_plan_is_replayed(world: &mut TinmanWorld) {
    let source = world.replay_plan_source.clone().expect("a plan was given");
    let path = working_dir(world).join("plan.yaml");
    std::fs::write(&path, &source)
        .unwrap_or_else(|e| panic!("the plan {} was not written: {e}", path.display()));
    run_tinman_command(world, &["test", &path.to_string_lossy()]).await;
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

#[then(expr = "the capture carries a {string} named {string}")]
async fn the_capture_carries_a_named(world: &mut TinmanWorld, role: String, name: String) {
    let plan = world
        .replay_plan
        .as_ref()
        .expect("a harness plan was given");
    // The role is what the plan's capture step declared its items to be, so the
    // assertion reads the plan as well as what replay collected; a capture of
    // some other role carrying the same text is not this assertion passing.
    let declared: Vec<&str> = plan
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.as_slice(),
            tinman::plan::FlowStep::Run(_) => [].as_slice(),
        })
        .filter_map(|action| match action {
            tinman::plan::Action::Capture(capture) => Some(capture.items.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        declared.contains(&role.as_str()),
        "the plan captures items of {declared:?}, so it never collected a {role:?}"
    );
    let outcome = world
        .flow_outcome
        .as_ref()
        .expect("the replay produced no outcome");
    let items = outcome.captures.get(CAPTURE_NAME).unwrap_or_else(|| {
        panic!(
            "the replay produced no capture named {CAPTURE_NAME:?}, so the plan's capture step \
             collected nothing; it produced {:?}",
            outcome.captures.keys().collect::<Vec<_>>()
        )
    });
    assert!(
        items.iter().any(|item| item == &name),
        "the capture named {CAPTURE_NAME:?} carries {items:?}, so no {role:?} named {name:?} was collected"
    );
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
    world.existing_file = Some(name);
}

#[when(expr = "the operator records {string} to that file with the streams captured separately")]
async fn the_operator_records_to_that_file_with_streams_separated(
    world: &mut TinmanWorld,
    command: String,
) {
    let name = world
        .existing_file
        .clone()
        .expect("the starting state named the file already in the way");
    // The refusal is issued before the recorder takes the terminal, so this run
    // needs no PTY. It runs with its streams on separate pipes, which is what a
    // scenario about which stream a diagnostic reaches has to read: a terminal
    // merges the two and could not tell them apart.
    let mut owned = vec!["record".to_string(), "--output".to_string(), name];
    owned.extend(command.split_whitespace().map(str::to_string));
    let args: Vec<&str> = owned.iter().map(String::as_str).collect();
    run_tinman_command(world, &args).await;
}

#[then("the error stream reports the file already exists")]
async fn the_error_stream_reports_the_file_already_exists(world: &mut TinmanWorld) {
    let name = world
        .existing_file
        .clone()
        .expect("the starting state named the file already in the way");
    let reported = run_errors(world);
    assert!(
        reported.contains("already exists") && reported.contains(name.as_str()),
        "the error stream does not report that {name} already exists; it reads:\n{reported}"
    );
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

#[given("the operator records a program that never exits when its input ends")]
async fn records_a_program_that_ignores_the_end_of_input(world: &mut TinmanWorld) {
    // A program that draws and then never returns. Ending its input tells it
    // nothing, so the recorder's own deadline is the only thing that can end the
    // session. The recording runs under a shell that outlives it, so the
    // terminal can be asked afterwards what state it was left in.
    let program = stage_recorded_program(
        world,
        "ignores-end-of-input",
        "printf 'READY\\n'\nwhile :; do sleep 1; done\n",
    );
    let dir = working_dir(world);
    let env = configured_env(world);
    let session = support::start_interruptible(&dir, &["record", &program], &env)
        .unwrap_or_else(|e| panic!("starting the recording on a terminal failed: {e}"));
    // The recorder captures its target into a virtual screen and draws nothing
    // of its own to this terminal, so there is no output to gate on. None is
    // owed: the terminal buffers what is typed at it, so the end of input the
    // scenario delivers is read when the recorder reaches its input loop,
    // whether it has got there yet or not.
    world.terminal_session = Some(session);
}

#[then("recording fails and reports the session reached its deadline")]
async fn recording_reports_it_reached_its_deadline(world: &mut TinmanWorld) {
    let status = world.run_status.expect("the recording ended");
    assert_ne!(
        status, 0,
        "the recording exited successfully where it should have failed at its deadline"
    );
    let output = world
        .terminal_session
        .as_ref()
        .expect("an interactive terminal session")
        .output();
    assert!(
        output.to_lowercase().contains("deadline"),
        "the recording does not report that the session reached its deadline; it produced:\n{output}"
    );
}

#[given("a fixture terminal program whose pane titles change between draws")]
async fn a_fixture_whose_titles_change(world: &mut TinmanWorld) {
    // Recording launches its target inside the sandbox, which binds the
    // workspace and nothing else, so the fixture is staged there and named
    // relatively rather than by an absolute path the sandbox cannot reach.
    let workspace = working_dir(world);
    world.record_program = Some(support::stage_unstable_fixture_in(&workspace));
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

/// Stage `body` as an executable shell program in the scenario's workspace and
/// return the relative name a recording addresses it by. A recorded plan replays
/// inside a sandbox binding the workspace, so the program is staged there and
/// named relatively, as an operator's own project program is.
fn stage_recorded_program(world: &mut TinmanWorld, name: &str, body: &str) -> String {
    let path = working_dir(world).join(name);
    std::fs::write(&path, body)
        .unwrap_or_else(|e| panic!("the program {} was not written: {e}", path.display()));
    support::set_executable(&path);
    format!("./{name}")
}

#[when(
    expr = "the operator records a command that writes to the sentinel path and prints {string}"
)]
async fn the_operator_records_a_command_writing_the_sentinel(
    world: &mut TinmanWorld,
    text: String,
) {
    let sentinel = world
        .sentinel
        .clone()
        .expect("a sentinel path was given")
        .display()
        .to_string();
    let program = stage_recorded_program(
        world,
        "writes-sentinel",
        &format!("#!/bin/sh\nexec 2>&-\ntouch {sentinel}\nprintf {text}\n"),
    );
    run_record(world, &program, &[]);
}

#[when(expr = "the operator records a command that prints {string} and the value of {string}")]
async fn the_operator_records_a_command_printing_the_secret(
    world: &mut TinmanWorld,
    text: String,
    name: String,
) {
    let program = stage_recorded_program(
        world,
        "prints-secret",
        &format!("#!/bin/sh\nprintf '{text} %s' \"${name}\"\n"),
    );
    run_record(world, &program, &[]);
}

#[then(expr = "the recorded snapshots show the text {string}")]
async fn the_recorded_snapshots_show_the_text(world: &mut TinmanWorld, expected: String) {
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
    assert!(
        snapshots
            .iter()
            .any(|snapshot| snapshot.contains(&expected)),
        "the recorded snapshots {snapshots:?} do not show the text {expected:?}; \
         the plan reads:\n{text}"
    );
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

#[given(
    expr = "a virtual screen showing a bordered pane titled {string} whose first line reads {string}"
)]
async fn a_screen_with_a_pane_whose_first_line_reads(
    world: &mut TinmanWorld,
    title: String,
    first_line: String,
) {
    let lines = vec![first_line];
    world.screen = Some(support::bordered_pane_screen(&title, &lines, None));
    world.pane = Some((title, lines));
}

#[given(
    expr = "a virtual screen showing a bordered pane titled {string} whose bottom border reads {string}"
)]
async fn a_screen_with_a_pane_whose_bottom_border_reads(
    world: &mut TinmanWorld,
    title: String,
    hint: String,
) {
    world.screen = Some(support::bordered_pane_screen_with_bottom_border(
        &title,
        &[],
        &hint,
    ));
    world.pane = Some((title, Vec::new()));
}

#[given("the cursor rests inside that pane")]
async fn the_cursor_rests_inside_that_pane(world: &mut TinmanWorld) {
    let (title, items) = world
        .pane
        .as_ref()
        .expect("a bordered pane was drawn")
        .clone();
    // The pane is drawn from row 1 column 1, so row 2 column 2 is its first cell
    // inside the border: the line a character the operator types lands on.
    world.screen = Some(support::bordered_pane_screen_with_cursor_at(
        &title, &items, 2, 2,
    ));
}

#[given("the cursor rests outside that pane")]
async fn the_cursor_rests_outside_that_pane(world: &mut TinmanWorld) {
    let (title, items) = world
        .pane
        .as_ref()
        .expect("a bordered pane was drawn")
        .clone();
    // The pane occupies its top border, one row per item, and its bottom border,
    // so the next row down is the first row clear of it.
    let below = u16::try_from(items.len() + 3).expect("the pane fits a terminal");
    world.screen = Some(support::bordered_pane_screen_with_cursor_at(
        &title, &items, below, 1,
    ));
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

#[given(expr = "a virtual screen whose bottom line reads {string} in red")]
async fn a_screen_whose_bottom_line_reads_in_red(world: &mut TinmanWorld, text: String) {
    world.screen = Some(support::bottom_line_screen_in_red(&text));
}

#[given(expr = "a virtual screen whose bottom line reads {string} in no colour of its own")]
async fn a_screen_whose_bottom_line_reads_plainly(world: &mut TinmanWorld, text: String) {
    world.screen = Some(support::bottom_line_screen(&text));
}

#[given(expr = "the line {string} is rendered in red while the rest of the pane is drawn plainly")]
async fn the_line_is_rendered_in_red(world: &mut TinmanWorld, line: String) {
    let (title, items) = world
        .pane
        .as_ref()
        .expect("a bordered pane was drawn")
        .clone();
    assert!(
        items.contains(&line),
        "the pane lists {items:?}, so it has no line {line:?} to draw in red"
    );
    world.screen = Some(support::bordered_pane_screen_with_line_in_red(
        &title, &items, &line,
    ));
}

#[given(expr = "the cursor rests at row {int} column {int}")]
async fn the_cursor_rests_at(world: &mut TinmanWorld, row: u16, column: u16) {
    let (text, at_row, at_col) = world
        .placed_text
        .clone()
        .expect("text was placed on the screen");
    world.screen = Some(support::text_at_screen_with_cursor_at(
        &text, at_row, at_col, row, column,
    ));
}

#[given("the program has hidden the cursor")]
async fn the_program_has_hidden_the_cursor(world: &mut TinmanWorld) {
    let (text, at_row, at_col) = world
        .placed_text
        .clone()
        .expect("text was placed on the screen");
    world.screen = Some(support::text_at_screen_with_cursor_hidden(
        &text, at_row, at_col,
    ));
}

#[given(expr = "a virtual screen whose top line reads {string}")]
async fn a_screen_whose_top_line_reads(world: &mut TinmanWorld, text: String) {
    world.top_line = Some(text.clone());
    world.screen = Some(support::top_line_screen(&text));
}

#[given(expr = "the label {string} is rendered with reversed video")]
async fn the_label_is_reversed(world: &mut TinmanWorld, label: String) {
    let text = world
        .top_line
        .clone()
        .expect("a virtual screen whose top line was drawn");
    world.screen = Some(support::top_line_screen_with_reversed(&text, &label));
}

#[given(expr = "a virtual screen showing {string} at row {int} column {int}")]
async fn a_screen_showing_text_at(world: &mut TinmanWorld, text: String, row: u16, col: u16) {
    world.placed_text = Some((text.clone(), row, col));
    world.screen = Some(support::text_at_screen(&text, row, col));
}

/// The line count a long listing opens with, and the three listing lines under
/// it. The summary line draws in two fields where the lines beneath it draw in
/// six, which is the shape every real `ls -l` shows.
const TOTAL_LINE: &str = "total 12";
const COLUMNAR_LINES: [&str; 3] = [
    "-rw-r--r--  1 dk  staff   220  report.txt",
    "-rw-r--r--  1 dk  staff    84  notes.txt",
    "-rw-r--r--  1 dk  staff   512  data.csv",
];

/// The block a process monitor draws above its table: the uptime line, the task
/// counts, the processor shares and the memory figures, each the program
/// reporting on itself rather than a row of anything.
const SUMMARY_LINES: [&str; 4] = [
    "top - 14:32:01 up 3 days,  2:14,  1 user,  load average: 0.14",
    "Tasks: 214 total,   1 running, 213 sleeping,   0 stopped",
    "%Cpu(s):  4.2 us,  1.1 sy,  0.0 ni, 94.7 id",
    "MiB Mem :  15872.0 total,  2048.5 free",
];

/// The table that monitor draws beneath its summary block, a header row naming
/// the columns and two process rows sustaining them.
const PROCESS_TABLE: [&str; 3] = [
    "  PID USER   PR  RES    CPU  COMMAND",
    " 1042 dk     20  184.2  4.0  tinman",
    " 2913 dk     20   92.6  1.3  cargo",
];

#[given("a virtual screen showing a total line above three columnar lines")]
async fn a_screen_showing_a_total_line_above_columns(world: &mut TinmanWorld) {
    let mut lines = vec![TOTAL_LINE];
    lines.extend(COLUMNAR_LINES);
    world.summary_lines = Some(vec![TOTAL_LINE.to_string()]);
    world.screen = Some(support::lines_screen(&lines));
}

#[given("a virtual screen showing four summary lines above a table")]
async fn a_screen_showing_four_summary_lines_above_a_table(world: &mut TinmanWorld) {
    // A blank row separates the block from the table, which is where a process
    // monitor puts it and what keeps the two readings distinct.
    let mut lines = SUMMARY_LINES.to_vec();
    lines.push("");
    lines.extend(PROCESS_TABLE);
    world.summary_lines = Some(SUMMARY_LINES.iter().map(|line| line.to_string()).collect());
    world.screen = Some(support::lines_screen(&lines));
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

/// The model the scenario built, as the JSON its scantling describes. The
/// serialized model is the published contract consumers read, so a presentation
/// assertion reads that shape rather than a private field.
fn tom_json(world: &TinmanWorld) -> serde_json::Value {
    let model = world
        .tom
        .as_ref()
        .expect("the terminal object model was built");
    serde_json::to_value(model).expect("the terminal object model serializes")
}

/// The first region of the serialized model that `matches`, searched from the
/// root down.
fn find_region_json<'a>(
    node: &'a serde_json::Value,
    matches: &impl Fn(&serde_json::Value) -> bool,
) -> Option<&'a serde_json::Value> {
    if matches(node) {
        return Some(node);
    }
    node.get("children")?
        .as_array()?
        .iter()
        .find_map(|child| find_region_json(child, matches))
}

/// The region of the serialized model carrying `role`, or a panic naming what
/// the model does carry.
fn region_json_with_role(world: &TinmanWorld, role: &str) -> serde_json::Value {
    let model = tom_json(world);
    let root = model.get("root").expect("the model carries a root region");
    find_region_json(root, &|region| {
        region.get("role").and_then(|r| r.as_str()) == Some(role)
    })
    .unwrap_or_else(|| {
        panic!(
            "the model carries no region with the role {role:?}; it reads:\n{}",
            serde_json::to_string_pretty(&model).expect("the model prints")
        )
    })
    .clone()
}

/// Assert the region is drawn in `expected` on the named channel. A region whose
/// cells disagree carries no style at all, which is reported as the absence it
/// is rather than as a mismatch.
fn assert_drawn_in(region: &serde_json::Value, channel: &str, expected: &str) {
    let style = region
        .get("style")
        .filter(|style| !style.is_null())
        .unwrap_or_else(|| {
            panic!(
                "the region carries no style, so it reports no {channel} colour; it reads:\n{}",
                serde_json::to_string_pretty(region).expect("the region prints")
            )
        });
    let drawn = style
        .get(channel)
        .unwrap_or_else(|| panic!("the region's style names no {channel}: {style}"));
    assert_eq!(
        drawn,
        &serde_json::Value::String(expected.to_string()),
        "the region is drawn in the {channel} colour {drawn}, not {expected:?}"
    );
}

#[then(expr = "the region with the role {string} is drawn in the foreground colour {string}")]
async fn the_region_with_role_is_drawn_in_foreground(
    world: &mut TinmanWorld,
    role: String,
    colour: String,
) {
    let region = region_json_with_role(world, &role);
    assert_drawn_in(&region, "foreground", &colour);
    world.styled_region = Some(region);
}

#[then(expr = "that region is drawn in the background colour {string}")]
async fn that_region_is_drawn_in_background(world: &mut TinmanWorld, colour: String) {
    let region = world
        .styled_region
        .clone()
        .expect("a region's foreground was read");
    assert_drawn_in(&region, "background", &colour);
}

#[then(expr = "the region named {string} carries no style")]
async fn the_region_named_carries_no_style(world: &mut TinmanWorld, name: String) {
    let model = tom_json(world);
    let root = model.get("root").expect("the model carries a root region");
    let region = find_region_json(root, &|region| {
        region.get("name").and_then(|n| n.as_str()) == Some(name.as_str())
    })
    .unwrap_or_else(|| panic!("the model carries no region named {name:?}"));
    let style = region.get("style");
    assert!(
        style.is_none() || style == Some(&serde_json::Value::Null),
        "the region named {name:?} carries the style {}, but its cells are drawn differently",
        style.expect("a style was found")
    );
}

#[then(expr = "the child region showing {string} is drawn in the foreground colour {string}")]
async fn the_child_region_showing_is_drawn_in_foreground(
    world: &mut TinmanWorld,
    text: String,
    colour: String,
) {
    let model = tom_json(world);
    let root = model.get("root").expect("the model carries a root region");
    let region = find_region_json(root, &|region| {
        region.get("text").and_then(|t| t.as_str()) == Some(text.as_str())
            || region.get("name").and_then(|n| n.as_str()) == Some(text.as_str())
    })
    .unwrap_or_else(|| {
        panic!(
            "the model carries no region showing {text:?}; it reads:\n{}",
            serde_json::to_string_pretty(&model).expect("the model prints")
        )
    });
    assert_drawn_in(region, "foreground", &colour);
}

#[then(expr = "the model's cursor is at row {int} column {int}")]
async fn the_models_cursor_is_at(world: &mut TinmanWorld, row: u64, column: u64) {
    let model = tom_json(world);
    let cursor = model
        .get("cursor")
        .filter(|cursor| !cursor.is_null())
        .unwrap_or_else(|| {
            panic!(
                "the model carries no cursor; it reads:\n{}",
                serde_json::to_string_pretty(&model).expect("the model prints")
            )
        });
    assert_eq!(
        cursor.get("row").and_then(|r| r.as_u64()),
        Some(row),
        "the model's cursor reads {cursor}, so it is not at row {row}"
    );
    assert_eq!(
        cursor.get("column").and_then(|c| c.as_u64()),
        Some(column),
        "the model's cursor reads {cursor}, so it is not at column {column}"
    );
}

#[then("the model carries no cursor")]
async fn the_model_carries_no_cursor(world: &mut TinmanWorld) {
    let model = tom_json(world);
    let cursor = model.get("cursor");
    assert!(
        cursor.is_none() || cursor == Some(&serde_json::Value::Null),
        "the model reports the cursor at {}, but the program hid it",
        cursor.expect("a cursor was found")
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

#[then("that region is left as the deterministic reading left it")]
async fn that_region_keeps_its_deterministic_name(world: &mut TinmanWorld) {
    // The region the engine left unnamed, read from both producers of the same
    // shape and matched by the rect they must agree on, so the assertion is
    // that enrichment changed nothing there rather than that some region
    // somewhere carries a name.
    let screen = world.screen.as_ref().expect("a virtual screen");
    let deterministic = tinman::tom::build(screen);
    let expected = deterministic
        .root
        .children
        .first()
        .expect("the deterministic reading built a region beneath the screen");
    let inferred = model(world);
    let actual = inferred
        .root
        .children
        .first()
        .expect("the inferred model carries a region beneath the screen");
    assert_eq!(
        actual.rect, expected.rect,
        "the region read from the inferred model is the one the deterministic reading built"
    );
    assert_eq!(
        actual.name, expected.name,
        "the name the region carries after inference, against the name the deterministic reading gave it"
    );
}

#[then(expr = "the model contains no region with the role {string}")]
async fn the_model_contains_no_region_with_role(world: &mut TinmanWorld, role: String) {
    let found = model(world).find_role(&role).cloned();
    assert!(
        found.is_none(),
        "the model contains a region with the role {role:?}, and it reads {found:?}"
    );
}

/// Write every region of `region` and everything nested inside it that plays
/// `role` into `found`, in the order the reading built them.
fn collect_role(region: &tinman::tom::Region, role: &str, found: &mut Vec<tinman::tom::Region>) {
    if region.role() == role {
        found.push(region.clone());
    }
    for child in &region.children {
        collect_role(child, role, found);
    }
}

#[then(expr = "the model contains {int} regions with the role {string}")]
async fn the_model_contains_regions_with_role(
    world: &mut TinmanWorld,
    expected: usize,
    role: String,
) {
    let mut found = Vec::new();
    collect_role(&model(world).root, &role, &mut found);
    let carried: Vec<Option<String>> = found.iter().map(|region| region.text.clone()).collect();
    assert_eq!(
        found.len(),
        expected,
        "the model contains {} regions with the role {role:?}, and they read {carried:?}",
        found.len()
    );
    world.found_regions = Some(found);
}

#[then("each one carries the text of the line it was read from")]
async fn each_region_carries_its_line(world: &mut TinmanWorld) {
    let found = world
        .found_regions
        .as_ref()
        .expect("a role sweep read the regions");
    let drawn = world
        .summary_lines
        .as_ref()
        .expect("the starting state drew the lines");
    let carried: Vec<String> = found
        .iter()
        .map(|region| region.text.clone().unwrap_or_default())
        .collect();
    assert_eq!(
        carried, *drawn,
        "the regions carry {carried:?}, against the lines they were read from"
    );
}

#[then("the total line is not a row of that table")]
async fn the_total_line_is_not_a_row(world: &mut TinmanWorld) {
    let table = world
        .found_region
        .as_ref()
        .expect("the preceding step found the table");
    let total = world
        .summary_lines
        .as_ref()
        .expect("the starting state drew the total line")[0]
        .clone();
    let rows: Vec<String> = table
        .children
        .iter()
        .filter(|child| child.role() == "row")
        .map(|child| child.text.clone().unwrap_or_default())
        .collect();
    assert!(
        !rows.is_empty(),
        "the table carries no rows at all, so it reads nothing the total line could be held out of"
    );
    assert!(
        !rows.iter().any(|row| row.trim() == total),
        "the table reads {total:?} as one of its rows; its rows are {rows:?}"
    );
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

#[then(expr = "the region named {string} contains a region with the role {string}")]
async fn the_region_named_contains_a_region_with_role(
    world: &mut TinmanWorld,
    name: String,
    role: String,
) {
    let region = model(world)
        .find_named(&name)
        .unwrap_or_else(|| panic!("the model contains no region named {name:?}"))
        .clone();
    let found = descendant_with_role(&region, &role).unwrap_or_else(|| {
        panic!(
            "the region named {name:?} contains no region with the role {role:?}; it contains {:?}",
            region
                .children
                .iter()
                .map(|child| child.role())
                .collect::<Vec<_>>()
        )
    });
    world.found_region = Some(found);
}

/// The first region beneath `region` carrying `role`, searched depth first so a
/// region nested any distance inside the named one is found.
fn descendant_with_role(region: &tinman::tom::Region, role: &str) -> Option<tinman::tom::Region> {
    for child in &region.children {
        if child.role() == role {
            return Some(child.clone());
        }
        if let Some(found) = descendant_with_role(child, role) {
            return Some(found);
        }
    }
    None
}

#[then(expr = "that status region shows {string}")]
async fn that_status_region_shows(world: &mut TinmanWorld, expected: String) {
    let region = world.found_region.as_ref().expect("a region was found");
    assert_eq!(
        region.text.as_deref(),
        Some(expected.as_str()),
        "the status region's text"
    );
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

#[given("an engine that answers with a region carrying no name")]
async fn an_engine_that_leaves_a_region_unnamed(world: &mut TinmanWorld) {
    // A well-formed model of the same screen with the pane's name left null,
    // which is what an engine sends when it has no name to propose for a
    // region. Only that field differs from what the screen yields, so the
    // deterministic reading of that region is the only thing under assertion.
    serve_engine_model(world, |reply| {
        reply["root"]["children"][0]["name"] = serde_json::Value::Null;
    });
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

/// The screen the fixture terminal program draws, read from a real capture of
/// it. The fixture waits for the operator, so it is captured live and read once
/// it has drawn its READY marker; a capture that waited for the program to exit
/// would wait for a program that never does.
async fn captured_fixture_screen() -> tinman::screen::VirtualScreen {
    let program = support::fixture_terminal_program();
    captured_screen_of(&program.to_string_lossy()).await
}

/// The screen `command_line` draws, read from a real capture of it, on the same
/// terms the fixture capture above is read on: the program waits for the
/// operator, so it is captured live and read once it has drawn its READY
/// marker.
async fn captured_screen_of(command_line: &str) -> tinman::screen::VirtualScreen {
    let prepared = shell_process(command_line);
    let mut capture =
        tinman::pty::capture_interactive(&prepared).expect("the fixture program is captured");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let drawn = loop {
        let screen = capture.screen();
        if screen.contains("READY") {
            break screen;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "{command_line} never drew READY; screen:\n{}",
                screen.contents()
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    capture.end_session();
    drawn
}

#[given("the fixture terminal program is captured through a PTY")]
async fn the_fixture_is_captured_through_a_pty(world: &mut TinmanWorld) {
    world.screen = Some(captured_fixture_screen().await);
}

#[given("an inference credential is configured")]
async fn an_inference_credential_is_configured(world: &mut TinmanWorld) {
    // A credential the run hands the command, so the command takes the branch a
    // configured credential opens rather than the one an absent credential
    // closes. What answers it is the scenario's own provider, named by the step
    // that follows this one, so the endpoint is that step's to set. Reaching for
    // the operator's real credential here would put a hosted call on the default
    // tier, which the tier policy in `RIGGING.md` keeps free of both; the real
    // credential and the real call belong to the @inference tier, at the seam
    // that makes them.
    world.env_vars.insert(
        "TINMAN_API_KEY".to_string(),
        "sk-local-provider".to_string(),
    );
}

#[given(
    expr = "a terminal program drawing an untitled bordered pane under a line reading {string}"
)]
async fn a_program_drawing_an_untitled_pane_under_a_line(world: &mut TinmanWorld, heading: String) {
    // Inspection launches its target inside the sandbox, which binds the
    // workspace and nothing else, so the program is staged there and named
    // relatively rather than by an absolute path the sandbox cannot reach. The
    // path it was written to is kept beside that name, because the step reading
    // what the deterministic pass alone makes of this program captures it
    // directly rather than through the command.
    let workspace = working_dir(world);
    let program = support::stage_untitled_pane_fixture_in(&workspace, &heading);
    world.inspected_program_path = Some(workspace.join(program.trim_start_matches("./")));
    world.inspected_program = Some(program);
}

#[when("the operator inspects that program")]
async fn the_operator_inspects_that_program(world: &mut TinmanWorld) {
    let program = world
        .inspected_program
        .clone()
        .expect("a program was staged for inspection");
    run_tinman_command(world, &["inspect", &program]).await;
}

/// Give `name` to the first region the deterministic reading leaves untitled
/// that holds regions of its own, depth first, and report whether one was found.
/// A pane is the container in this reading: the summary and status lines a
/// screen also leaves untitled carry text and no children, so requiring children
/// picks the pane out of them without keying on a role name.
fn name_first_untitled_pane(node: &mut serde_json::Value, name: &str) -> bool {
    let Some(children) = node["children"].as_array_mut() else {
        return false;
    };
    for child in children {
        let untitled = child["name"]
            .as_str()
            .is_none_or(|name| name.trim().is_empty());
        if untitled
            && child["children"]
                .as_array()
                .is_some_and(|held| !held.is_empty())
        {
            child["name"] = serde_json::Value::String(name.to_string());
            return true;
        }
        if name_first_untitled_pane(child, name) {
            return true;
        }
    }
    false
}

#[given(expr = "the configured provider names that pane {string}")]
async fn the_configured_provider_names_that_pane(world: &mut TinmanWorld, name: String) {
    // The reply is the deterministic model of the screen the staged program
    // draws, with one field changed: the name of the pane that reading leaves
    // untitled. Building it from the screen rather than by hand keeps it a
    // well-formed model of that screen, so the only thing the enrichment can add
    // is the field the scenario names. The screen is kept for the assertion,
    // which reads the same capture rather than taking a second one.
    let path = world
        .inspected_program_path
        .clone()
        .expect("a program was staged for inspection");
    let screen = captured_screen_of(&path.to_string_lossy()).await;
    let deterministic = tinman::tom::build(&screen);
    let mut reply = to_json(&deterministic);
    assert!(
        name_first_untitled_pane(&mut reply["root"], &name),
        "the deterministic reading of the staged program leaves no pane untitled, so this \
         scenario has nothing for the enrichment to name; its listing reads:\n{}",
        tinman::inspect::render(&deterministic)
    );
    let provider = support::LocalProvider::returning(&reply.to_string());
    use_provider(world, provider);
    world.screen = Some(screen);
}

#[then(expr = "the inspect output names that pane {string}")]
async fn the_inspect_output_names_that_pane(world: &mut TinmanWorld, name: String) {
    // Read against this project's own unenriched listing of the same screen, so
    // what passes is the enrichment reaching the listing rather than a name the
    // deterministic pass supplies on its own.
    let screen = world
        .screen
        .as_ref()
        .expect("the screen the staged program drew")
        .clone();
    let deterministic = tinman::inspect::render(&tinman::tom::build(&screen));
    let unenriched = listing_rows(&deterministic);
    // The pane is the row the deterministic reading leaves untitled that holds
    // rows of its own. The heading and the ready line are left untitled too, and
    // a status region prints its text where a name would stand, so a search for
    // the name anywhere in the listing would read the heading and pass without
    // the enrichment running at all.
    let pane = (0..unenriched.len())
        .find(|row| {
            let (depth, _, listed) = &unenriched[*row];
            let untitled = listed
                .as_deref()
                .is_none_or(|listed| listed.trim().is_empty());
            untitled && unenriched.get(row + 1).is_some_and(|next| next.0 > *depth)
        })
        .unwrap_or_else(|| {
            panic!(
                "the deterministic reading leaves no pane untitled, so this scenario reads \
                 nothing about the enrichment; its listing reads:\n{deterministic}"
            )
        });
    let output = run_output(world).to_string();
    let enriched = listing_rows(&output);
    let listed = enriched.get(pane).unwrap_or_else(|| {
        panic!(
            "the inspect output carries no row where the deterministic reading carries the \
             untitled pane; it reads:\n{output}\n\nthe deterministic listing reads:\n{deterministic}"
        )
    });
    assert_eq!(
        (listed.0, listed.1.as_str(), listed.2.as_deref()),
        (
            unenriched[pane].0,
            unenriched[pane].1.as_str(),
            Some(name.as_str())
        ),
        "the inspect output does not name the pane {name:?}; it reads:\n{output}\n\nthe \
         deterministic listing reads:\n{deterministic}"
    );
}

#[when("the terminal object model is inferred by the configured engine")]
async fn the_model_is_inferred_by_the_configured_engine(world: &mut TinmanWorld) {
    // What this scenario asserts on is the model Tinman produces, never the
    // reply the provider sent, so the inference seam is what runs here rather
    // than a raw completion call. The seam refuses an enrichment that would not
    // conform and leaves the deterministic reading standing in its place, which
    // is the ordinary answer from a real engine rather than an edge.
    let settings = configured_settings();
    let screen = world
        .screen
        .as_ref()
        .expect("a captured virtual screen")
        .clone();
    let model = tinman::tom::infer(&screen, &settings);
    world.serialized = Some(to_json(&model));
    world.tom = Some(model);
}

#[then(expr = "the model Tinman produces conforms to the {string} schema in {string}")]
async fn the_model_tinman_produces_conforms(
    world: &mut TinmanWorld,
    _schema_id: String,
    path: String,
) {
    let instance = world
        .serialized
        .as_ref()
        .expect("Tinman produced a model from the captured screen");
    let bad = support::schema_counterexamples(&path, instance);
    assert!(bad.is_empty(), "schema violations: {bad:?}");
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
    world.provider_answer = Some(answer);
    use_provider(world, provider);
}

#[given("the assistant answers a single line of 200 characters")]
async fn the_assistant_answers_a_long_single_line(world: &mut TinmanWorld) {
    let answer = long_answer_line();
    let reply = tinman::assistant::model_reply_answering(&answer);
    let provider = support::LocalProvider::returning(&reply);
    world.provider_answer = Some(answer);
    use_provider(world, provider);
}

#[given("the assistant answers:")]
async fn the_assistant_answers_the_docstring(
    world: &mut TinmanWorld,
    step: &cucumber::gherkin::Step,
) {
    let answer = step
        .docstring
        .as_ref()
        .expect("the step carries the answer as a docstring")
        .trim_matches('\n')
        .to_string();
    let reply = tinman::assistant::model_reply_answering(&answer);
    let provider = support::LocalProvider::returning(&reply);
    world.provider_answer = Some(answer);
    use_provider(world, provider);
}

/// The title the setup form's own region carries, read from the asset the form
/// draws it from. Held as a copy here, this was the same hand-copied constant
/// the form itself carried: a title edited in the asset would strand the
/// readiness wait below on text the screen no longer shows, and the scenario
/// proving the catalogue is live would fail on the harness rather than on the
/// product.
fn setup_form_title() -> String {
    setup_form_asset()[0].clone()
}

/// The key a scenario types into the credential field. It is a value no real
/// provider issued, so a run that leaked it would be leaking nothing usable.
const STAGED_KEY: &str = "sk-staged-credential-not-a-real-key";

/// Give the scenario a configuration directory of its own, so a credential the
/// program saves lands in the scenario's scratch rather than in the operator's
/// real home, and report the directory it will be written under.
fn staged_config_home(world: &mut TinmanWorld) -> std::path::PathBuf {
    let home = working_dir(world).join("config-home");
    std::fs::create_dir_all(&home).expect("the scenario's configuration directory is created");
    let home_display = home.display().to_string();
    world
        .env_vars
        .insert("XDG_CONFIG_HOME".to_string(), home_display.clone());
    world.env_vars.insert("HOME".to_string(), home_display);
    home
}

/// The terminal object model of what the scenario's run or open session drew.
fn drawn_model(world: &TinmanWorld) -> (tinman::tom::Model, String) {
    let raw = drawn_raw(world);
    let screen = support::terminal_screen_at(&raw, run_columns(world));
    let contents = screen.contents();
    (tinman::tom::build(&screen), contents)
}

/// Every file under `root`, however deep.
fn files_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match path.is_dir() {
            true => found.extend(files_under(&path)),
            false => found.push(path),
        }
    }
    found
}

#[then(expr = "a region titled {string} is drawn")]
async fn a_region_titled_is_drawn(world: &mut TinmanWorld, title: String) {
    let (model, contents) = drawn_model(world);
    assert!(
        model.find_named(&title).is_some(),
        "the terminal object model reads no region titled {title:?}; screen:\n{contents}"
    );
}

#[then(expr = "no region titled {string} is drawn")]
async fn no_region_titled_is_drawn(world: &mut TinmanWorld, title: String) {
    let (model, contents) = drawn_model(world);
    assert!(
        model.find_named(&title).is_none(),
        "the terminal object model reads a region titled {title:?}; screen:\n{contents}"
    );
}

/// The text the setup form is showing, failing where no form was drawn.
fn setup_form_text(world: &TinmanWorld) -> String {
    let (model, contents) = drawn_model(world);
    let title = setup_form_title();
    let form = model.find_named(&title).unwrap_or_else(|| {
        panic!("the terminal object model reads no region titled {title:?}; screen:\n{contents}")
    });
    let mut text = String::new();
    fn gather(region: &tinman::tom::Region, into: &mut String) {
        if let Some(name) = region.name.as_deref() {
            into.push_str(name);
            into.push('\n');
        }
        if let Some(body) = region.text.as_deref() {
            into.push_str(body);
            into.push('\n');
        }
        for child in &region.children {
            gather(child, into);
        }
    }
    gather(form, &mut text);
    text
}

#[then(expr = "the form offers {string} as the endpoint")]
async fn the_form_offers_as_the_endpoint(world: &mut TinmanWorld, endpoint: String) {
    let text = setup_form_text(world);
    assert!(
        text.contains(&endpoint),
        "the setup form offers no endpoint {endpoint:?}; it reads:\n{text}"
    );
}

#[then(expr = "the form offers {string} as the model")]
async fn the_form_offers_as_the_model(world: &mut TinmanWorld, model: String) {
    let text = setup_form_text(world);
    assert!(
        text.contains(&model),
        "the setup form offers no model {model:?}; it reads:\n{text}"
    );
}

#[then(expr = "the form names {string} as an environment variable it reads")]
async fn the_form_names_the_environment_variable(world: &mut TinmanWorld, variable: String) {
    let text = setup_form_text(world);
    assert!(
        text.contains(&variable),
        "the setup form names no environment variable {variable:?}; it reads:\n{text}"
    );
}

/// The setup form's own copy, as the asset carries it: the title on line one,
/// the key hints on line two, and the sentence naming the environment variable
/// it reads on line three. The steps below take the copy from the asset rather
/// than restating it, so an edit to the catalogue an operator or a translator
/// works in is what the assertion is read against.
fn setup_form_asset() -> Vec<String> {
    let body = asset_body("assets/help/setup-form.txt");
    let lines: Vec<String> = body.lines().map(str::to_string).collect();
    assert!(
        lines.len() >= 3,
        "the setup form asset carries no title, key hints and environment sentence; it reads:\n{body}"
    );
    lines
}

/// The key a hint line names as the one that performs `action`, read as the
/// word before `to <action>`, which is the form these hints are written in.
fn key_named_for(hint: &str, action: &str) -> String {
    let phrase = format!("to {action}");
    let at = hint
        .find(&phrase)
        .unwrap_or_else(|| panic!("the hint {hint:?} names no key that {action}s"));
    hint[..at]
        .split_whitespace()
        .next_back()
        .unwrap_or_else(|| panic!("the hint {hint:?} names no key before {phrase:?}"))
        .to_string()
}

#[then("the form title is the title the setup asset carries")]
async fn the_form_title_is_the_title_the_asset_carries(world: &mut TinmanWorld) {
    let title = setup_form_asset()[0].clone();
    let (model, contents) = drawn_model(world);
    assert!(
        model.find_named(&title).is_some(),
        "the setup form draws no region titled {title:?}, the title the asset carries; screen:\n{contents}"
    );
}

#[then("the form names the keys that asset carries as the keys that save and leave")]
async fn the_form_names_the_keys_the_asset_carries(world: &mut TinmanWorld) {
    let hint = setup_form_asset()[1].clone();
    let text = setup_form_text(world);
    for action in ["save", "leave"] {
        let key = key_named_for(&hint, action);
        assert_prompt_names_key(&text, &key, action);
    }
}

#[then("the form names the environment variable that asset carries")]
async fn the_form_names_the_environment_variable_the_asset_carries(world: &mut TinmanWorld) {
    let sentence = setup_form_asset()[2].clone();
    let text = setup_form_text(world);
    assert!(
        text.contains(&sentence),
        "the setup form does not carry {sentence:?}, the environment sentence the asset carries; it reads:\n{text}"
    );
}

#[given("the operator has opened the setup form")]
async fn the_operator_has_opened_the_setup_form(world: &mut TinmanWorld) {
    staged_config_home(world);
    let dir = working_dir(world);
    let env = configured_env(world);
    // Bare `tinman` is the invocation the scenarios open the form with, so the
    // binary is run with no arguments at all.
    let session = support::TerminalSession::start(&dir, &[], &env)
        .unwrap_or_else(|e| panic!("starting the setup form on a terminal failed: {e}"));
    // The form standing on the terminal is the signal it is ready for input, so
    // nothing is typed before the program can read it.
    session.await_region(&setup_form_title(), std::time::Duration::from_secs(10));
    world.terminal_session = Some(session);
}

#[when("the operator types a key into the credential field")]
async fn the_operator_types_a_key_into_the_credential_field(world: &mut TinmanWorld) {
    let session = world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session");
    session.type_text(STAGED_KEY);
    // The typed characters reach the form only when the program redraws it, so
    // the wait ends on the field reporting itself hidden rather than on a clock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let screen = support::terminal_screen(&session.raw_output());
        let model = tinman::tom::build(&screen);
        if model
            .find_role("textbox")
            .is_some_and(|field| !field.presentations().is_empty())
        {
            return;
        }
        if std::time::Instant::now() >= deadline {
            return;
        }
        std::thread::sleep(support::EXIT_POLL_INTERVAL);
    }
}

#[then("the credential field is hidden")]
async fn the_credential_field_is_hidden(world: &mut TinmanWorld) {
    let (model, contents) = drawn_model(world);
    let field = model
        .find_role("textbox")
        .unwrap_or_else(|| panic!("the setup form draws no credential field; screen:\n{contents}"));
    assert!(
        field.presentations().contains(&"hidden"),
        "the credential field is drawn plainly rather than hidden; it is drawn {:?} and reads {:?}",
        field.presentations(),
        field.text
    );
    assert!(
        !contents.contains(STAGED_KEY),
        "the key typed into the credential field is on the screen; it reads:\n{contents}"
    );
}

#[when("the operator saves a key through the form")]
async fn the_operator_saves_a_key_through_the_form(world: &mut TinmanWorld) {
    let session = world
        .terminal_session
        .as_mut()
        .expect("an interactive terminal session");
    session.type_line(STAGED_KEY);
    // Saving ends the form, so the wait ends on the program exiting.
    session.end_input();
    session.wait(std::time::Duration::from_secs(20));
}

#[then("the credential is written under the configuration directory")]
async fn the_credential_is_written_under_the_configuration_directory(world: &mut TinmanWorld) {
    let home = working_dir(world).join("config-home");
    let carrying: Vec<std::path::PathBuf> = files_under(&home)
        .into_iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .map(|body| body.contains(STAGED_KEY))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !carrying.is_empty(),
        "no file under the configuration directory {} carries the saved credential; it holds {:?}",
        home.display(),
        files_under(&home)
    );
    world.saved_credential_paths = Some(carrying);
}

#[then("that file is readable only by its owner")]
async fn that_file_is_readable_only_by_its_owner(world: &mut TinmanWorld) {
    use std::os::unix::fs::PermissionsExt;
    let paths = world
        .saved_credential_paths
        .clone()
        .expect("a step found the saved credential");
    let readable: Vec<String> = paths
        .iter()
        .filter_map(|path| {
            let mode = std::fs::metadata(path).ok()?.permissions().mode() & 0o077;
            (mode != 0).then(|| format!("{} is mode {:o}", path.display(), mode))
        })
        .collect();
    assert!(
        readable.is_empty(),
        "the saved credential is readable beyond its owner: {readable:?}"
    );
}

#[then("no credential is written to the operator's working directory")]
async fn no_credential_is_written_to_the_working_directory(world: &mut TinmanWorld) {
    let dir = working_dir(world);
    let home = dir.join("config-home");
    let leaked: Vec<std::path::PathBuf> = files_under(&dir)
        .into_iter()
        .filter(|path| !path.starts_with(&home))
        .filter(|path| {
            std::fs::read_to_string(path)
                .map(|body| body.contains(STAGED_KEY))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "the credential was written into the operator's working directory: {leaked:?}"
    );
}

#[given("no inference credential is configured")]
async fn no_inference_credential_is_configured(world: &mut TinmanWorld) {
    // A launched Tinman inherits no TINMAN_ variable from the suite, so a
    // credential can reach it only from what a scenario staged or from a dotenv
    // file in the directory it runs in. Both are cleared here, so the step
    // leaves the condition it names rather than assuming it.
    world.env_vars.remove("TINMAN_API_KEY");
    world.provider = None;
    let dotenv = working_dir(world).join(".env");
    if dotenv.exists() {
        std::fs::remove_file(&dotenv).expect("the staged dotenv file is removed");
    }
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
    // A reply may answer and then propose, so the answer is read from whichever
    // response carries it rather than from one variant alone.
    match world.response.as_ref().expect("the assistant responded") {
        tinman::assistant::Response::Answer(text) => {
            assert_eq!(text, &expected, "displayed answer");
        }
        tinman::assistant::Response::Proposal(proposal) => {
            assert_eq!(
                proposal.answer(),
                Some(expected.as_str()),
                "the answer displayed beside the proposed command"
            );
        }
        other => panic!("the assistant gave no answer: {other:?}"),
    }
}

#[given("the proposal marker the assistant reads")]
async fn the_proposal_marker_the_assistant_reads(world: &mut TinmanWorld) {
    // The assistant builds a proposing reply from the same marker it reads one
    // by, so a reply proposing the empty command line is the marker alone.
    let marker = tinman::assistant::model_reply_proposing("");
    assert!(
        !marker.trim().is_empty(),
        "the assistant reads an empty proposal marker, so this scenario would assert nothing"
    );
    world.proposal_marker = Some(marker);
}

#[when(expr = "the asset at {string} is searched for it")]
async fn the_asset_is_searched_for_the_marker(world: &mut TinmanWorld, path: String) {
    world.asset_text = Some(
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("asset {path} unreadable: {e}")),
    );
}

#[then("the instruction asset carries that marker")]
async fn the_instruction_asset_carries_that_marker(world: &mut TinmanWorld) {
    let marker = world
        .proposal_marker
        .as_ref()
        .expect("the proposal marker was read");
    let text = world.asset_text.as_ref().expect("the asset was read");
    assert!(
        text.contains(marker.as_str()),
        "the instruction asset names no {marker:?}, so a reply written to it never proposes; it reads:\n{text}"
    );
}

#[given(expr = "the assistant replies with the answer {string} and the command {string}")]
async fn the_assistant_replies_with_answer_and_command(
    world: &mut TinmanWorld,
    answer: String,
    command: String,
) {
    // The instruction asset puts the marker on the last line of the reply and
    // reads everything above it as the answer, so the reply is built that way.
    let reply = format!(
        "{}\n{}",
        tinman::assistant::model_reply_answering(&answer),
        tinman::assistant::model_reply_proposing(&command)
    );
    let provider = support::LocalProvider::returning(&reply);
    world.provider_answer = Some(answer);
    use_provider(world, provider);
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
    "the rule set carries at least the plank-form, plank-presence, perturbation-quiescence, process-wide-env-mutation, killed-measured-child and unshared-corpus-read rules"
)]
async fn the_rule_set_carries_the_named_rules(_world: &mut TinmanWorld) {
    let carried = support::conformance_rule_ids();
    let missing: Vec<&str> = [
        "plank-form",
        "plank-presence",
        "perturbation-quiescence",
        "process-wide-env-mutation",
        "killed-measured-child",
        "unshared-corpus-read",
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
// methodology conformance: the release commands the rigging declares
// ---------------------------------------------------------------------------

#[given(expr = "the outbound targets in {string}")]
async fn the_outbound_targets(world: &mut TinmanWorld, rigging: String) {
    world.outbound_targets = Some(support::outbound_targets(&rigging));
    world.rigging_path = Some(rigging);
}

#[when("each target's ship and verify lines are read")]
async fn each_targets_ship_and_verify_lines_are_read(world: &mut TinmanWorld) {
    let targets = world
        .outbound_targets
        .as_ref()
        .expect("the outbound targets were read");
    world.outbound_lines = Some(
        targets
            .iter()
            .flat_map(|target| {
                [
                    (target.name.clone(), "ship".to_string(), target.ship.clone()),
                    (
                        target.name.clone(),
                        "verify".to_string(),
                        target.verify.clone(),
                    ),
                ]
            })
            .collect(),
    );
}

#[then("every target carries both a ship line and a verify line")]
async fn every_outbound_target_carries_both_lines(world: &mut TinmanWorld) {
    let lines = world
        .outbound_lines
        .as_ref()
        .expect("the ship and verify lines were read");
    let missing: Vec<String> = lines
        .iter()
        .filter(|(_, _, line)| line.is_none())
        .map(|(name, kind, _)| format!("the {name} target declares no {kind} line"))
        .collect();
    assert!(
        missing.is_empty(),
        "{} outbound line(s) are missing, so a release runs a step nobody declared:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[then("every one of those lines reports its own exit status")]
async fn every_outbound_line_reports_its_own_exit_status(world: &mut TinmanWorld) {
    let lines = world
        .outbound_lines
        .as_ref()
        .expect("the ship and verify lines were read");
    let silent: Vec<String> = lines
        .iter()
        .filter_map(|(name, kind, line)| {
            let command = line.as_deref()?;
            (!support::reports_its_own_exit_status(command)).then(|| {
                format!("the {name} target's {kind} line reports no status of its own: {command}")
            })
        })
        .collect();
    assert!(
        silent.is_empty(),
        "{} outbound line(s) report no exit status, so a failed release reads as a clean \
         one:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

#[then("the targets read are not empty")]
async fn the_outbound_targets_read_are_not_empty(world: &mut TinmanWorld) {
    let targets = world
        .outbound_targets
        .as_ref()
        .expect("the outbound targets were read");
    let rigging = world.rigging_path.as_ref().expect("the rigging was named");
    assert!(
        !targets.is_empty(),
        "the rigging at {rigging} declares no outbound target, so this scenario would assert \
         nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: the doubles verification support declares
// ---------------------------------------------------------------------------

#[given("the exceptional-double marks in the verification support sources")]
async fn the_exceptional_double_marks_in_verification_support(world: &mut TinmanWorld) {
    world.double_marks = Some(support::exceptional_double_marks());
}

#[when("each mark is read for the condition it names")]
async fn each_mark_is_read_for_the_condition_it_names(world: &mut TinmanWorld) {
    let marks = world
        .double_marks
        .as_ref()
        .expect("the exceptional-double marks were read");
    world.unjustified_marks = Some(
        marks
            .iter()
            .filter(|mark| mark.condition.is_none())
            .map(|mark| format!("{}:{} {}", mark.file, mark.line, mark.text))
            .collect(),
    );
}

#[then("every mark names one of the three conditions the Verification agreement permits")]
async fn every_mark_names_a_permitted_condition(world: &mut TinmanWorld) {
    let unjustified = world
        .unjustified_marks
        .as_ref()
        .expect("each mark was read for the condition it names");
    assert!(
        unjustified.is_empty(),
        "{} exceptional-double mark(s) name no one condition the Verification agreement permits, \
         so each stands as a gesture rather than a justification:\n{}",
        unjustified.len(),
        unjustified.join("\n")
    );
}

#[then("the marks read are not empty")]
async fn the_double_marks_read_are_not_empty(world: &mut TinmanWorld) {
    let marks = world
        .double_marks
        .as_ref()
        .expect("the exceptional-double marks were read");
    assert!(
        !marks.is_empty(),
        "the verification support sources carry no exceptional-double mark at all, so the \
         assertion above read an empty selection and inspected nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: what durable prose in the specs may carry
// ---------------------------------------------------------------------------

/// The rigging the spec-prose checks read their own scope from, so the
/// implementation directory a rule body may not cite is the one the rigging
/// names rather than a second copy of it here.
const RIGGING: &str = "RIGGING.md";

#[given("the rule bodies the specs carry")]
async fn the_rule_bodies_the_specs_carry(world: &mut TinmanWorld) {
    world.rule_bodies = Some(support::rule_bodies());
}

#[when("each is read for a path under the implementation directory")]
async fn each_rule_body_is_read_for_an_implementation_path(world: &mut TinmanWorld) {
    let bodies = world
        .rule_bodies
        .clone()
        .expect("the starting state read the rule bodies");
    let implementation = support::implementation_directory(RIGGING);
    let under = format!("{implementation}/");
    let citations = bodies
        .into_iter()
        .map(|body| {
            // A path is one word of the prose, so the body is read word by word
            // with the punctuation a sentence wraps a path in taken off. A word
            // naming the directory alone names the tree rather than a seam in
            // it, so a citation is one carrying something below it.
            let found = body
                .body
                .split_whitespace()
                .map(|word| {
                    word.trim_matches(|c: char| {
                        !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-'
                    })
                })
                .filter(|word| word.starts_with(&under) && word.len() > under.len())
                .map(|word| word.to_string())
                .collect();
            (body, found)
        })
        .collect();
    world.rule_body_citations = Some(citations);
}

#[then("every rule body was read")]
async fn every_rule_body_was_read(world: &mut TinmanWorld) {
    let bodies = world
        .rule_bodies
        .as_ref()
        .expect("the starting state read the rule bodies");
    let read = world
        .rule_body_citations
        .as_ref()
        .expect("each rule body was read for an implementation path");
    // An unreadable corpus and a clean one both report no citation, so the count
    // is asserted before the finding is: this step is what tells them apart.
    assert!(
        !bodies.is_empty(),
        "the specs carry no rule body at all, so the reading below inspected nothing"
    );
    assert_eq!(
        read.len(),
        bodies.len(),
        "{} of the {} rule bodies the specs carry were read",
        read.len(),
        bodies.len()
    );
}

#[then("no rule body cites such a path")]
async fn no_rule_body_cites_such_a_path(world: &mut TinmanWorld) {
    let read = world
        .rule_body_citations
        .as_ref()
        .expect("each rule body was read for an implementation path");
    let citing: Vec<String> = read
        .iter()
        .filter(|(_, found)| !found.is_empty())
        .map(|(body, found)| format!("{}: {found:?}", body.feature))
        .collect();
    assert!(
        citing.is_empty(),
        "{} rule body(ies) cite a path under the implementation directory, which outlives the \
         seam it names:\n{}",
        citing.len(),
        citing.join("\n")
    );
}

#[given("the crates the release graph carries")]
async fn the_crates_the_release_graph_carries(world: &mut TinmanWorld) {
    world.release_crates = Some(support::release_graph_crates());
}

#[when("each is queried against the advisory database by name and version")]
async fn each_crate_is_queried_against_the_advisory_database(world: &mut TinmanWorld) {
    let crates = world
        .release_crates
        .clone()
        .expect("the starting state read the release graph");
    let answers = crates
        .iter()
        .map(|(name, version)| {
            support::advisories_for(name, version).unwrap_or_else(|| {
                panic!("the advisory database did not answer about {name} {version}")
            })
        })
        .collect();
    world.advisory_answers = Some(answers);
}

#[then("every crate in that graph was queried")]
async fn every_crate_in_that_graph_was_queried(world: &mut TinmanWorld) {
    let crates = world
        .release_crates
        .as_ref()
        .expect("the starting state read the release graph");
    let answers = world
        .advisory_answers
        .as_ref()
        .expect("the crates were queried against the advisory database");
    // A graph that failed to resolve and a clean one both report no advisory, so
    // the count is asserted before the finding is.
    assert!(
        !crates.is_empty(),
        "the release graph carries no crate at all, so the queries below asked about nothing"
    );
    let asked: Vec<(String, String)> = answers
        .iter()
        .map(|answer| (answer.name.clone(), answer.version.clone()))
        .collect();
    assert_eq!(
        asked, *crates,
        "the crates queried are not the crates the release graph carries"
    );
}

#[then("no advisory reaches any of them")]
async fn no_advisory_reaches_any_of_them(world: &mut TinmanWorld) {
    let answers = world
        .advisory_answers
        .as_ref()
        .expect("the crates were queried against the advisory database");
    let reached: Vec<String> = answers
        .iter()
        .filter(|answer| !answer.advisories.is_empty())
        .map(|answer| {
            format!(
                "{} {}: {}",
                answer.name,
                answer.version,
                answer.advisories.join(", ")
            )
        })
        .collect();
    assert!(
        reached.is_empty(),
        "{} crate(s) the released binary carries are reached by an advisory:\n{}",
        reached.len(),
        reached.join("\n")
    );
}

#[given("the lifecycle tag lines the specs carry")]
async fn the_lifecycle_tag_lines_the_specs_carry(world: &mut TinmanWorld) {
    world.lifecycle_tags = Some(support::lifecycle_tags());
}

#[when("each is read for what it precedes")]
async fn each_lifecycle_tag_is_read_for_what_it_precedes(world: &mut TinmanWorld) {
    let tags = world
        .lifecycle_tags
        .clone()
        .expect("the starting state read the lifecycle tag lines");
    // The parser reports a tag against the construct it precedes, so the reading
    // is the tags as parsed rather than a second pass over the files.
    world.lifecycle_tags_read = Some(tags);
}

#[then("every lifecycle tag line was read")]
async fn every_lifecycle_tag_line_was_read(world: &mut TinmanWorld) {
    let tags = world
        .lifecycle_tags
        .as_ref()
        .expect("the starting state read the lifecycle tag lines");
    let read = world
        .lifecycle_tags_read
        .as_ref()
        .expect("each lifecycle tag line was read for what it precedes");
    // An empty set is the healthy resting state here, because a lifecycle tag
    // ends by being deleted. A corpus that failed to parse would report the same
    // empty set, so what is asserted is that the specs were read at all.
    assert!(
        !support::spec_scenarios().is_empty(),
        "no scenario was read from the specs, so the tag reading inspected nothing"
    );
    assert_eq!(
        read.len(),
        tags.len(),
        "{} of the {} lifecycle tag lines the specs carry were read",
        read.len(),
        tags.len()
    );
}

#[then("none of them precedes a rule")]
async fn none_of_them_precedes_a_rule(world: &mut TinmanWorld) {
    let read = world
        .lifecycle_tags_read
        .as_ref()
        .expect("each lifecycle tag line was read for what it precedes");
    let on_rules: Vec<String> = read
        .iter()
        .filter(|tag| tag.precedes == "rule")
        .map(|tag| format!("{}: @{} on rule {:?}", tag.feature, tag.tag, tag.subject))
        .collect();
    assert!(
        on_rules.is_empty(),
        "{} lifecycle tag line(s) sit on a rule, where every scenario inside inherits them and \
         deleting the tag on a scenario leaves the rule's still excluding it:\n{}",
        on_rules.len(),
        on_rules.join("\n")
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: duplication across the implementation
// ---------------------------------------------------------------------------

#[given(expr = "the implementation sources and the threshold in {string}")]
async fn the_sources_and_the_duplication_threshold(world: &mut TinmanWorld, scantling: String) {
    // The size is read from the scantling rather than written here, so the
    // number a run applies is the number the durable file declares.
    world.duplication_minimum_lines = Some(support::duplication_minimum_lines(&scantling));
    world.duplication_allowance_path = Some(scantling);
}

#[when("the duplication scanner reads the sources at that threshold")]
async fn the_duplication_scanner_reads_the_sources_at_that_threshold(world: &mut TinmanWorld) {
    let minimum = world
        .duplication_minimum_lines
        .expect("the threshold was read");
    // The implementation directory the rigging names, which is the scope its
    // own conformance command hands the scanner.
    world.duplication = Some(support::duplication_report("src", minimum));
}

#[then("it reports no duplicate group")]
async fn it_reports_no_duplicate_group(world: &mut TinmanWorld) {
    let report = world
        .duplication
        .as_ref()
        .expect("the duplication scanner ran");
    let path = world
        .duplication_allowance_path
        .as_deref()
        .expect("the threshold was named");
    // Above the declared size the gate is zero and nothing is exempt, so a group
    // is named with its members rather than joined against a list. The
    // fingerprint rides along because it is the scanner's own identity for the
    // group and is what a reader would search its report by.
    let reported: Vec<String> = report
        .groups
        .iter()
        .map(|group| format!("{}: {}", group.fingerprint, group.members.join(", ")))
        .collect();
    assert!(
        reported.is_empty(),
        "{} duplicate group(s) survive at or above the size {path} names, where structural \
         identity is copied logic:\n{}",
        reported.len(),
        reported.join("\n")
    );
}

#[then("the code units it analysed are not empty")]
async fn the_code_units_analysed_are_not_empty(world: &mut TinmanWorld) {
    let report = world
        .duplication
        .as_ref()
        .expect("the duplication scanner ran");
    let path = world
        .duplication_allowance_path
        .as_deref()
        .expect("the threshold was named");
    // Zero groups is the healthy resting state here, and a scanner that read
    // nothing reports it identically. A threshold set above every unit in the
    // tree, or a scope naming no source, would both pass the gate above while
    // inspecting nothing at all.
    assert!(
        report.code_units > 0,
        "the duplication scanner analysed no code unit at the size {path} names, so the gate \
         above passed while reading nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: duplicated constants across the implementation
// ---------------------------------------------------------------------------

#[given(expr = "the implementation sources and the allowance in {string}")]
async fn the_sources_and_the_duplication_allowance(world: &mut TinmanWorld, allowance: String) {
    // The scanner reads functions and carries no unit kind for a constant, so
    // the list this census joins against is the only one the allowance still
    // carries.
    world.constant_allowance = Some(support::duplication_allowance_constants(&allowance));
    world.duplication_allowance_path = Some(allowance);
}

#[when("the constant census reads the sources")]
async fn the_constant_census_reads_the_sources(world: &mut TinmanWorld) {
    // The implementation directory the rigging names, which is the scope the
    // duplication scanner beside this census is handed.
    let constants = support::constant_census("src");
    world.duplicate_constants = Some(support::duplicate_constants(&constants));
    world.constants_read = Some(constants);
}

#[then("every duplicated constant it reports is one the allowance names")]
async fn every_duplicate_constant_is_one_the_allowance_names(world: &mut TinmanWorld) {
    let reported = world
        .duplicate_constants
        .as_ref()
        .expect("the constant census ran");
    let allowed = world
        .constant_allowance
        .as_ref()
        .expect("the constant allowance was read");
    let path = world
        .duplication_allowance_path
        .as_deref()
        .expect("the duplication allowance was named");
    // The census runs no scanner, so it has no fingerprint of its own: the
    // members are the group's identity, and they are carried into the message
    // because a failure naming a declaration alone says nothing about where the
    // copies are.
    let unnamed: Vec<String> = reported
        .iter()
        .filter(|group| !allowed.contains(&group.members))
        .map(|group| format!("{}: {}", group.declaration, group.members.join(", ")))
        .collect();
    assert!(
        unnamed.is_empty(),
        "{} duplicated constant group(s) the census reports are not named in {path}:\n{}",
        unnamed.len(),
        unnamed.join("\n")
    );
}

#[then("every duplicated constant the allowance names is one it reports")]
async fn every_allowed_constant_is_one_the_census_reports(world: &mut TinmanWorld) {
    let reported = world
        .duplicate_constants
        .as_ref()
        .expect("the constant census ran");
    let allowed = world
        .constant_allowance
        .as_ref()
        .expect("the constant allowance was read");
    let path = world
        .duplication_allowance_path
        .as_deref()
        .expect("the duplication allowance was named");
    // The reverse direction of the same join. An entry the census no longer
    // reports excused duplication that is gone, and it stays in the file granting
    // cover to whatever later matches it.
    let unreported: Vec<String> = allowed
        .iter()
        .filter(|members| !reported.iter().any(|group| &group.members == *members))
        .map(|members| members.join(", "))
        .collect();
    assert!(
        unreported.is_empty(),
        "{} allowance entr(ies) in {path} name duplicated constants the census no longer \
         reports, so each excuses duplication that is gone:\n{}",
        unreported.len(),
        unreported.join("\n")
    );
}

#[then("the constants read are not empty")]
async fn the_constants_read_are_not_empty(world: &mut TinmanWorld) {
    let constants = world
        .constants_read
        .as_ref()
        .expect("the constant census ran");
    // The floor guards the reader rather than the count. An empty duplicated set
    // is the healthy resting state here, so the assertion is that the census read
    // the sources at all: a census that found no constant whatsoever and one that
    // found no duplicate among them would otherwise report the same clean bill.
    assert!(
        !constants.is_empty(),
        "the constant census read no constant at all, so the joins above read nothing and this \
         scenario would assert nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: waits on a spawned child
// ---------------------------------------------------------------------------

#[given("the process waits in the verification support sources")]
async fn the_process_waits_in_the_support_sources(world: &mut TinmanWorld) {
    world.process_waits = Some(support::process_wait_census());
}

#[when("each wait is read for the deadline that bounds it")]
async fn each_wait_is_read_for_its_deadline(world: &mut TinmanWorld) {
    // The census names the deadline as it reads each wait, so the reading is
    // already in hand. Asserting it was taken keeps this step from passing over a
    // census that never ran.
    world
        .process_waits
        .as_ref()
        .expect("the process-wait census ran");
}

#[then("every wait on a spawned child reaches a deadline")]
async fn every_wait_reaches_a_deadline(world: &mut TinmanWorld) {
    let waits = world
        .process_waits
        .as_ref()
        .expect("the process-wait census ran");
    let unbounded: Vec<String> = waits
        .iter()
        .filter(|wait| wait.bound.is_none())
        .map(|wait| format!("{}: {}", wait.site(), wait.text))
        .collect();
    assert!(
        unbounded.is_empty(),
        "{} wait(s) on a spawned child reach no deadline, so each hangs the whole sweep rather \
         than failing one scenario:\n{}",
        unbounded.len(),
        unbounded.join("\n")
    );
}

#[then("the waits read are not empty")]
async fn the_waits_read_are_not_empty(world: &mut TinmanWorld) {
    let waits = world
        .process_waits
        .as_ref()
        .expect("the process-wait census ran");
    assert!(
        !waits.is_empty(),
        "the census read no wait on a spawned child at all, so the assertion above inspected \
         nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: tearing down a session whose process has gone
// ---------------------------------------------------------------------------

#[given("a driver session whose process has already exited")]
async fn a_driver_session_whose_process_has_exited(world: &mut TinmanWorld) {
    let mut session = support::DriverProcess::start();
    // Closing stdin is this protocol's own end of input, so the driver runs its
    // own exit path. Waiting on that exit is what makes the process observably
    // gone before teardown reaches it, rather than merely likely to be.
    session.close_stdin();
    session.wait_for_exit();
    world.driver = Some(session);
}

#[when("verification support tears the session down")]
async fn verification_support_tears_the_session_down(world: &mut TinmanWorld) {
    let session = world.driver.as_mut().expect("the driver session started");
    world.teardown = Some(session.tear_down());
}

#[then("the teardown reports the process had already gone")]
async fn the_teardown_reports_the_process_had_gone(world: &mut TinmanWorld) {
    let report = world.teardown.as_ref().expect("the session was torn down");
    assert!(
        report.already_gone,
        "teardown reached a process that had already exited and did not report it as gone, so it \
         would signal one that cannot be stopped"
    );
}

#[then("no diagnostic from the system's kill reaches the runner's error stream")]
async fn no_kill_diagnostic_reaches_the_error_stream(world: &mut TinmanWorld) {
    let report = world.teardown.as_ref().expect("the session was torn down");
    // Teardown captures whatever the kill writes rather than letting it inherit
    // the runner's error stream, so what it captured is what would have reached
    // that stream. Escalation is owed when a termination deadline passes, and
    // only then, so a teardown reaching a gone process signals nothing at all.
    assert!(
        report.kill_diagnostics.is_empty(),
        "teardown signalled a process that had already gone, and the system's kill answered:\n{}",
        report.kill_diagnostics
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
// methodology conformance: the scenarios the shipped prose cites
// ---------------------------------------------------------------------------

#[given("the scenario references cited in the shipped Markdown documents")]
async fn the_scenario_references_the_shipped_documents_cite(world: &mut TinmanWorld) {
    world.shipped_citations = Some(support::cited_scenario_references());
}

#[when("each is matched against the scenarios the specs declare")]
async fn each_citation_is_matched_against_the_specs(world: &mut TinmanWorld) {
    let citations = world
        .shipped_citations
        .as_ref()
        .expect("the citations were read");
    let carried: std::collections::HashSet<String> = support::spec_scenarios()
        .iter()
        .map(support::SpecScenario::reference)
        .collect();
    world.unresolved_citations = Some(
        citations
            .iter()
            .filter(|citation| !carried.contains(&citation.reference))
            .map(|citation| {
                format!(
                    "{}:{} cites {}",
                    citation.document, citation.line, citation.reference
                )
            })
            .collect(),
    );
}

#[then("every citation names a scenario the specs carry")]
async fn every_citation_names_a_scenario_the_specs_carry(world: &mut TinmanWorld) {
    let unresolved = world
        .unresolved_citations
        .as_ref()
        .expect("each citation was matched");
    assert!(
        unresolved.is_empty(),
        "{} citation(s) name a scenario the specs do not carry:\n{}",
        unresolved.len(),
        unresolved.join("\n")
    );
}

#[then("the citations read are not empty")]
async fn the_citations_read_are_not_empty(world: &mut TinmanWorld) {
    let citations = world
        .shipped_citations
        .as_ref()
        .expect("the citations were read");
    assert!(
        !citations.is_empty(),
        "the shipped Markdown documents cite no scenario, so the join asserted nothing"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: the watchbill membership the shipped prose claims
// ---------------------------------------------------------------------------

#[given("the shipped Markdown documents")]
async fn the_shipped_markdown_documents(world: &mut TinmanWorld) {
    world.shipped_documents = Some(support::shipped_markdown_documents());
}

#[when("each line naming a scenario or a spec is read for a watchbill membership claim")]
async fn each_line_naming_a_spec_is_read_for_a_membership_claim(world: &mut TinmanWorld) {
    let documents = world
        .shipped_documents
        .as_ref()
        .expect("the shipped documents were listed");
    world.watchbill_claim_survey = Some(support::watchbill_membership_claims(documents));
}

#[then("no line claims the scenario it names is on a watchbill")]
async fn no_line_claims_the_scenario_it_names_is_on_a_watchbill(world: &mut TinmanWorld) {
    let survey = world
        .watchbill_claim_survey
        .as_ref()
        .expect("each line naming a scenario or a spec was read");
    let reported: Vec<String> = survey
        .claims
        .iter()
        .map(|claim| {
            format!(
                "{}:{} claims membership with \"{}\":\n    {}",
                claim.document, claim.line, claim.construction, claim.text
            )
        })
        .collect();
    assert!(
        reported.is_empty(),
        "{} shipped line(s) claim a scenario they name is on a watchbill, which decays silently the moment custody strikes the file:\n{}",
        reported.len(),
        reported.join("\n")
    );
}

#[then("every shipped Markdown document was read")]
async fn every_shipped_markdown_document_was_read(world: &mut TinmanWorld) {
    let shipped = world
        .shipped_documents
        .as_ref()
        .expect("the shipped documents were listed");
    let survey = world
        .watchbill_claim_survey
        .as_ref()
        .expect("each line naming a scenario or a spec was read");
    assert!(
        !shipped.is_empty(),
        "the packaged manifest excludes every tracked Markdown document, so the survey read nothing"
    );
    assert_eq!(
        &survey.documents_read, shipped,
        "the survey read a different set of documents than the shipped set"
    );
}

// ---------------------------------------------------------------------------
// methodology conformance: what a worker count buys
// ---------------------------------------------------------------------------

#[given("a fixture feature whose four scenarios each block on a real sandboxed process")]
async fn a_fixture_feature_blocking_on_sandboxed_processes(world: &mut TinmanWorld) {
    let dir = support::ScratchDir::new("concurrency-fixture");
    let path = support::write_concurrency_fixture(dir.path());
    // The fixture is what the whole measurement rests on, so its shape is read
    // back rather than assumed. A fixture that lost a scenario would still run
    // and still report a ratio, and the ratio would describe a workload the
    // scenario never asked for.
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the fixture at {} was not read back: {e}", path.display()));
    let scenarios = text.matches("  Scenario: ").count();
    assert_eq!(
        scenarios,
        support::FIXTURE_SCENARIO_COUNT,
        "the fixture carries {scenarios} scenario(s) where this scenario names \
         {}; it reads:\n{text}",
        support::FIXTURE_SCENARIO_COUNT
    );
    world.fixture_features = Some(path);
    world.fixture_dir = Some(dir);
}

#[when("it is run at one worker and again at four")]
async fn the_fixture_is_run_at_one_worker_and_at_four(world: &mut TinmanWorld) {
    let features = world
        .fixture_features
        .clone()
        .expect("the concurrency fixture was written");
    // The serial arm runs first so the parallel arm cannot be the run that pays
    // a cold cache the other avoided. Both arms launch the same real sandboxed
    // processes, so anything cached is cached before the comparison is made.
    let one = support::run_suite_over(&features, 1);
    let four = support::run_suite_over(&features, 4);
    for (workers, run) in [(1, &one), (4, &four)] {
        // A run that selected no scenario finishes fast and passes, and two of
        // them would compare as a ratio nobody exercised. Each arm reports the
        // scenarios it ran, so each arm is held to the fixture's own count.
        let ran = format!("{} scenarios (", support::FIXTURE_SCENARIO_COUNT);
        assert!(
            run.output.contains(&ran),
            "the {workers}-worker run did not report {} scenarios; it reported:\n{}",
            support::FIXTURE_SCENARIO_COUNT,
            run.output
        );
    }
    world.one_worker_run = Some(one);
    world.four_worker_run = Some(four);
}

#[then("the four-worker run takes less than three quarters of the one-worker wall clock")]
async fn the_four_worker_run_is_materially_faster(world: &mut TinmanWorld) {
    let one = world
        .one_worker_run
        .as_ref()
        .expect("the one-worker run was measured");
    let four = world
        .four_worker_run
        .as_ref()
        .expect("the four-worker run was measured");
    let ceiling = one.elapsed.mul_f64(0.75);
    assert!(
        four.elapsed < ceiling,
        "four workers took {:?} against {:?} at one worker, which is no better \
         than three quarters of it ({:?}). The worker count the rigging declares \
         buys nothing while a step body holds the executor thread it shares with \
         every scenario beside it.",
        four.elapsed,
        one.elapsed,
        ceiling
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
// methodology conformance: where a tier sweep spent its wall clock
// ---------------------------------------------------------------------------

#[given("the most recent tier sweep the wake recorded")]
async fn the_most_recent_tier_sweep_the_wake_recorded(world: &mut TinmanWorld) {
    // The sweep read here is one the record offers as a sweep, and one other
    // than the run carrying this scenario. A run reading its own record would
    // see every scenario that had not finished yet as a gap, and a focused run
    // would see no scenario but this one, so neither could judge what a sweep
    // recorded.
    let current = support::current_run_id();
    let swept = support::recorded_durations()
        .into_iter()
        .rfind(|run| run.is_sweep() && run.run != current);
    world.sweep_durations = Some(swept.unwrap_or_else(|| {
        panic!(
            "the wake at {} carries no tier sweep, so nothing recorded how long a \
             scenario took",
            support::DURATIONS_RECORD
        )
    }));
}

#[when("the durations it recorded are read")]
async fn the_durations_it_recorded_are_read(world: &mut TinmanWorld) {
    let swept = world
        .sweep_durations
        .as_ref()
        .expect("a tier sweep was read from the wake");
    world.scenarios_missing_durations = Some(swept.missing_durations());
}

#[then("every scenario it ran carries its own duration")]
async fn every_scenario_it_ran_carries_its_own_duration(world: &mut TinmanWorld) {
    let swept = world
        .sweep_durations
        .as_ref()
        .expect("a tier sweep was read from the wake");
    let missing = world
        .scenarios_missing_durations
        .as_ref()
        .expect("the durations the sweep recorded were read");
    assert!(
        missing.is_empty(),
        "{} of the {} scenarios the {} sweep started carry no duration of their own:\n{}",
        missing.len(),
        swept.started.len(),
        swept.run,
        missing.join("\n")
    );
}

#[then("the sweep read covered a whole tier rather than a generated fixture")]
async fn the_sweep_read_covered_a_whole_tier(world: &mut TinmanWorld) {
    // The record offers a run as a sweep on the root it declared it swept. This
    // reads the specs it actually ran, which is the other half of the same
    // claim: a recorder naming the specs directory while running a generated
    // fixture would otherwise hand this scenario four fixture scenarios to
    // attest while it reported on a tier.
    let swept = world
        .sweep_durations
        .as_ref()
        .expect("a tier sweep was read from the wake");
    let outside = swept.scenarios_outside_the_specs();
    assert!(
        outside.is_empty(),
        "the {} sweep declared it swept {} and ran {} scenario(s) from outside it:\n{}",
        swept.run,
        support::SPECS_DIRECTORY,
        outside.len(),
        outside.join("\n")
    );
}

#[then("the durations read are not empty")]
async fn the_durations_read_are_not_empty(world: &mut TinmanWorld) {
    let swept = world
        .sweep_durations
        .as_ref()
        .expect("a tier sweep was read from the wake");
    assert!(
        !swept.timed.is_empty(),
        "the sweep {} recorded in the wake at {} carries no scenario duration at all",
        swept.run,
        support::DURATIONS_RECORD
    );
}

#[given("a run whose filter matched no feature")]
async fn a_run_whose_filter_matched_no_feature(world: &mut TinmanWorld) {
    world.empty_filter_run = Some(support::run_suite_matching_no_feature());
}

#[when("the durations the wake recorded are read")]
async fn the_durations_the_wake_recorded_are_read(world: &mut TinmanWorld) {
    world.recorded_runs = Some(support::recorded_durations());
}

#[then("that run is not offered as a sweep")]
async fn that_run_is_not_offered_as_a_sweep(world: &mut TinmanWorld) {
    let empty = world
        .empty_filter_run
        .as_ref()
        .expect("a run whose filter matched no feature was made");
    let runs = world
        .recorded_runs
        .as_ref()
        .expect("the durations the wake recorded were read");
    let recorded = runs
        .iter()
        .find(|run| &run.run == empty)
        .unwrap_or_else(|| panic!("the wake carries no run {empty}"));
    assert!(
        !recorded.is_sweep(),
        "the run {empty} matched no feature and is offered as a sweep, so the next \
         reader attests an empty set: it started {} scenario(s), it timed {}, and it \
         declared it swept {:?}",
        recorded.started.len(),
        recorded.timed.len(),
        recorded.features
    );
}

#[then("the runs read are not empty")]
async fn the_runs_read_are_not_empty(world: &mut TinmanWorld) {
    let runs = world
        .recorded_runs
        .as_ref()
        .expect("the durations the wake recorded were read");
    // The floor guards the reader rather than the count. A record nothing could
    // read and one carrying no run report the same clean bill above.
    assert!(
        !runs.is_empty(),
        "the wake at {} carries no run at all, so the assertion above read nothing",
        support::DURATIONS_RECORD
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

#[then("all ten validate")]
async fn all_ten_validate(world: &mut TinmanWorld) {
    let results = world
        .meta_schema_results
        .as_ref()
        .expect("each scantling was checked");
    assert_eq!(
        results.len(),
        10,
        "ten scantlings declare a dialect, found {}: {}",
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
    let version = support::package_version(&manifest);
    world.package_version = Some(version.clone());
    world.package_versions.push((manifest, version));
}

#[when("the two are compared")]
async fn the_two_manifest_versions_are_compared(world: &mut TinmanWorld) {
    // Both manifests are read by the steps above, so what this step settles is
    // that two were read: one manifest compared against itself agrees always.
    let read = &world.package_versions;
    assert_eq!(
        read.len(),
        2,
        "two packaged manifests are compared, {} was read: {}",
        read.len(),
        read.iter()
            .map(|(manifest, _)| manifest.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[then("they are the same version")]
async fn they_are_the_same_version(world: &mut TinmanWorld) {
    let read = &world.package_versions;
    let (first_manifest, first_version) = read.first().expect("a manifest version was read");
    let disagreeing: Vec<String> = read
        .iter()
        .filter(|(_, version)| version != first_version)
        .map(|(manifest, version)| format!("{manifest} names {version}"))
        .collect();
    assert!(
        disagreeing.is_empty(),
        "{first_manifest} names {first_version}, but {}",
        disagreeing.join(", ")
    );
}

#[when("the schema URIs in the scantlings and the example plans are read")]
async fn the_published_schema_uris_are_read(world: &mut TinmanWorld) {
    world.published_uris = Some(support::published_schema_uris());
}

#[then("all twenty-four name that version")]
async fn all_twenty_four_name_that_version(world: &mut TinmanWorld) {
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
        24,
        "twenty-four schema URIs are published, found {}: {}",
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

#[given("the thirteen scantlings that declare no JSON Schema dialect")]
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

#[then("all thirteen proof contracts validate")]
async fn all_thirteen_proof_contracts_validate(world: &mut TinmanWorld) {
    let results = world
        .meta_schema_results
        .as_ref()
        .expect("each proof contract was checked");
    assert_eq!(
        results.len(),
        13,
        "thirteen scantlings declare no dialect, found {}: {}",
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
// the style object two scantlings restate: the join that keeps them in step
// ---------------------------------------------------------------------------

#[given(expr = "the style properties required by {string}")]
async fn the_style_properties_required_by(world: &mut TinmanWorld, path: String) {
    let properties = support::required_style_properties(&path);
    world.style_property_sets.push((path, properties));
}

#[when("the two sets are compared")]
async fn the_two_sets_are_compared(world: &mut TinmanWorld) {
    assert_eq!(
        world.style_property_sets.len(),
        2,
        "two style property sets are compared, {} were read: {}",
        world.style_property_sets.len(),
        world
            .style_property_sets
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[then("they are the same set")]
async fn they_are_the_same_set(world: &mut TinmanWorld) {
    let (first_path, first) = &world.style_property_sets[0];
    let (second_path, second) = &world.style_property_sets[1];
    let missing: Vec<&str> = first
        .iter()
        .filter(|name| !second.contains(name))
        .map(String::as_str)
        .collect();
    let extra: Vec<&str> = second
        .iter()
        .filter(|name| !first.contains(name))
        .map(String::as_str)
        .collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "{first_path} and {second_path} require different style properties; \
         {second_path} is missing {missing:?} and adds {extra:?}"
    );
}

#[then("the properties read are not empty")]
async fn the_properties_read_are_not_empty(world: &mut TinmanWorld) {
    // The step guards whichever reading its scenario made, and a scenario that
    // read nothing at all is the case it exists to catch: an empty set compared
    // against an empty set passes every assertion above it while proving
    // nothing.
    let mut read = 0usize;
    for (path, properties) in &world.style_property_sets {
        assert!(
            !properties.is_empty(),
            "{path} requires no style property, so the comparison read nothing there"
        );
        read += properties.len();
    }
    if let Some(declared) = &world.declared_plan_properties {
        let path = world.plan_schema_path.as_deref().unwrap_or("the schema");
        assert!(
            !declared.is_empty(),
            "{path} declares no property, so the round trip was compared against nothing"
        );
        read += declared.len();
    }
    assert!(
        read > 0,
        "no properties were read, so the comparison above asserted nothing"
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

// ---------------------------------------------------------------------------
// the command parser, asked about one line at a time
// ---------------------------------------------------------------------------

#[when(expr = "{string} is passed to the command parser")]
async fn the_line_is_passed_to_the_command_parser(world: &mut TinmanWorld, line: String) {
    use clap::Parser;
    let mut argv = vec!["tinman".to_string()];
    argv.extend(line.split_whitespace().map(str::to_string));
    world.parser_outcome = Some(match tinman::cli::Cli::try_parse_from(&argv) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}: {e}", e.kind())),
    });
}

#[then("the parser accepts it")]
async fn the_parser_accepts_it(world: &mut TinmanWorld) {
    match world
        .parser_outcome
        .as_ref()
        .expect("a line was passed to the command parser")
    {
        Ok(()) => {}
        Err(refusal) => panic!("the parser refused the line: {refusal}"),
    }
}

// ---------------------------------------------------------------------------
// documented examples: the tldr page source and the program's own help
// ---------------------------------------------------------------------------

/// Restart the scenario's own tldr page source over the pages it now carries and
/// point Tinman at it. One source serves a scenario however many pages it
/// stages, and the source's own readiness gate means no step reaches it before
/// it serves.
fn restage_tldr_source(world: &mut TinmanWorld) {
    let pages: Vec<(String, String)> = world
        .tldr_pages
        .iter()
        .map(|(program, examples)| {
            let lines: Vec<&str> = examples.iter().map(String::as_str).collect();
            (program.clone(), support::tldr_page(program, &lines))
        })
        .collect();
    // The standing source is dropped before the replacement binds, so a scenario
    // staging several pages leaves one listener rather than one per staging step.
    world.tldr_source = None;
    let source = support::LocalTldrSource::serving(&pages);
    world.env_vars.insert(
        "TINMAN_TLDR_BASE_URL".to_string(),
        source.base_url().to_string(),
    );
    world.tldr_source = Some(source);
}

#[given(expr = "the configured tldr page source has a page for {string}")]
async fn the_page_source_has_a_page_for(world: &mut TinmanWorld, program: String) {
    world.tldr_pages.entry(program.clone()).or_default();
    world.last_tldr_program = Some(program);
    restage_tldr_source(world);
}

#[given(
    expr = "the configured tldr page source has a page for {string} carrying the example {string}"
)]
async fn the_page_source_has_a_page_carrying(
    world: &mut TinmanWorld,
    program: String,
    example: String,
) {
    world
        .tldr_pages
        .entry(program.clone())
        .or_default()
        .push(example);
    world.last_tldr_program = Some(program);
    restage_tldr_source(world);
}

#[given(expr = "that page also carries the example {string}")]
async fn that_page_also_carries_the_example(world: &mut TinmanWorld, example: String) {
    let program = world
        .last_tldr_program
        .clone()
        .expect("a page was staged for a program");
    world.tldr_pages.entry(program).or_default().push(example);
    restage_tldr_source(world);
}

#[given(expr = "the configured tldr page source has no page for {string}")]
async fn the_page_source_has_no_page_for(world: &mut TinmanWorld, program: String) {
    world.tldr_pages.remove(&program);
    restage_tldr_source(world);
}

#[given("the configured tldr page source has no page for the fixture terminal program")]
async fn the_page_source_has_no_page_for_the_fixture(world: &mut TinmanWorld) {
    // The fixture is inspected under the name its help shim is staged as, so
    // that is the name a page would be looked up by and the name the source is
    // staged without.
    world.tldr_pages.remove("fixture");
    restage_tldr_source(world);
}

#[given(
    expr = "the configured tldr page source has a page for {string} carrying an example that writes to the sentinel path and prints {string}"
)]
async fn the_page_source_has_a_destructive_example(
    world: &mut TinmanWorld,
    program: String,
    text: String,
) {
    let sentinel = world
        .sentinel
        .clone()
        .expect("a sentinel path was given")
        .display()
        .to_string();
    // The sandbox carries no /dev/null, so stderr is closed rather than
    // redirected: the refusal the sandbox issues must not be drawn on the screen
    // the assertion reads.
    let example = format!("{program} -c 'exec 2>&-; touch {sentinel}; printf {text}'");
    world
        .tldr_pages
        .entry(program.clone())
        .or_default()
        .push(example);
    world.last_tldr_program = Some(program);
    restage_tldr_source(world);
}

/// Stage an executable named `name` in the scenario's working directory whose
/// help carries `examples`, and which otherwise runs the real program of that
/// name. No stock binary carries the example a scenario names, so the program
/// under inspection is one the scenario built. The help follows the `Examples:`
/// block convention this project's own help asset uses, a description line above
/// each indented command line.
fn stage_help_shim(world: &mut TinmanWorld, name: &str, target: &str, examples: &[String]) {
    let dir = working_dir(world);
    let mut help = format!("Usage: {name} [OPTION]...\n\nExamples:\n");
    for example in examples {
        help.push_str(&format!("  Documented example:\n    {example}\n\n"));
    }
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--help\" ]; then\ncat <<'TINMAN_HELP'\n{help}TINMAN_HELP\nexit 0\nfi\nexec {target} \"$@\"\n"
    );
    let path = dir.join(name);
    std::fs::write(&path, script)
        .unwrap_or_else(|e| panic!("the {name} help shim was not written: {e}"));
    let mut permissions = std::fs::metadata(&path)
        .unwrap_or_else(|e| panic!("the {name} help shim was not read back: {e}"))
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(&path, permissions)
        .unwrap_or_else(|e| panic!("the {name} help shim was not made executable: {e}"));
}

#[given(expr = "the {string} help carries the example {string}")]
async fn the_named_help_carries_the_example(
    world: &mut TinmanWorld,
    program: String,
    example: String,
) {
    world.staged_help_examples.push(example);
    let staged = world.staged_help_examples.clone();
    let target = format!("/usr/bin/{program}");
    stage_help_shim(world, &program, &target, &staged);
}

#[given(expr = "the fixture terminal program's help carries the example {string}")]
async fn the_fixture_help_carries_the_example(world: &mut TinmanWorld, example: String) {
    let workspace = working_dir(world);
    let staged = support::stage_fixture_in(&workspace);
    world.staged_help_examples.push(example);
    let examples = world.staged_help_examples.clone();
    stage_help_shim(world, "fixture", &staged, &examples);
}

#[given(expr = "that help carries the keypress example {string}")]
async fn that_help_carries_the_keypress_example(world: &mut TinmanWorld, keypress: String) {
    world.staged_help_examples.push(keypress);
    let examples = world.staged_help_examples.clone();
    let workspace = working_dir(world);
    let staged = support::stage_fixture_in(&workspace);
    stage_help_shim(world, "fixture", &staged, &examples);
}

// ---------------------------------------------------------------------------
// inspection driven by a program's documented examples
// ---------------------------------------------------------------------------

#[when(expr = "the operator inspects {string} with its documented examples")]
async fn the_operator_inspects_with_documented_examples(world: &mut TinmanWorld, program: String) {
    run_tinman_command(world, &["inspect", "--examples", &program]).await;
}

#[when(expr = "the operator inspects {string} with its documented examples and writes a plan")]
async fn the_operator_inspects_with_documented_examples_writing_a_plan(
    world: &mut TinmanWorld,
    program: String,
) {
    run_tinman_command(
        world,
        &["inspect", "--examples", &program, "--output", "tinman.yaml"],
    )
    .await;
}

#[when("the operator inspects the fixture terminal program with its documented examples")]
async fn the_operator_inspects_the_fixture_with_documented_examples(world: &mut TinmanWorld) {
    run_tinman_command(world, &["inspect", "--examples", "./fixture"]).await;
}

#[when(
    "the operator inspects the fixture terminal program with its documented examples and writes a plan"
)]
async fn the_operator_inspects_the_fixture_with_documented_examples_writing_a_plan(
    world: &mut TinmanWorld,
) {
    run_tinman_command(
        world,
        &[
            "inspect",
            "--examples",
            "./fixture",
            "--output",
            "tinman.yaml",
        ],
    )
    .await;
}

/// Whether the inspect output reports `subject` marked as `marker`. The marker is
/// matched on the line carrying the subject, with case and hyphenation
/// normalised, so the rendering keeps its latitude while the term the scenario
/// names is what the assertion pins.
fn reports_marked(output: &str, subject: &str, marker: &str) -> bool {
    let wanted = marker.to_lowercase().replace('-', " ");
    output.lines().any(|line| {
        line.contains(subject) && line.to_lowercase().replace('-', " ").contains(&wanted)
    })
}

#[then("the page is read as raw markdown from the tldr project")]
async fn the_page_is_read_as_raw_markdown(world: &mut TinmanWorld) {
    let source = world
        .tldr_source
        .as_ref()
        .expect("the scenario staged a tldr page source");
    let asked = source.requested_paths();
    assert!(
        !asked.is_empty(),
        "no request reached the tldr page source, so no page was read from it"
    );
    // The project serves its pages as raw markdown at `<platform>/<name>.md`, so
    // a reader asking for that path is reading the page rather than a rendering
    // of it.
    assert!(
        asked.iter().any(|path| path.ends_with(".md")),
        "the tldr page source was asked for {asked:?}, none of them a raw markdown page"
    );
}

#[then(expr = "the inspect output reports {string} as the ground-truth example")]
async fn the_output_reports_as_ground_truth(world: &mut TinmanWorld, example: String) {
    let output = run_output(world);
    assert!(
        reports_marked(output, &example, "ground truth"),
        "the inspect output does not report {example:?} as the ground-truth example; it reads:\n{output}"
    );
}

#[then(expr = "the inspect output reports {string} as a hint")]
async fn the_output_reports_as_a_hint(world: &mut TinmanWorld, example: String) {
    let output = run_output(world);
    assert!(
        reports_marked(output, &example, "hint"),
        "the inspect output does not report {example:?} as a hint; it reads:\n{output}"
    );
}

#[then(expr = "the inspect output reports the example {string} was refused")]
async fn the_output_reports_the_example_refused(world: &mut TinmanWorld, example: String) {
    let output = run_output(world);
    assert!(
        reports_marked(output, &example, "refused"),
        "the inspect output does not report the example {example:?} was refused; it reads:\n{output}"
    );
}

#[then(expr = "the inspect output reports {string} as untestable as written")]
async fn the_output_reports_untestable_as_written(world: &mut TinmanWorld, example: String) {
    let output = run_output(world);
    assert!(
        reports_marked(output, &example, "untestable as written"),
        "the inspect output does not report {example:?} as untestable as written; it reads:\n{output}"
    );
}

#[then("the inspect output reports no tldr page was read")]
async fn the_output_reports_no_tldr_page(world: &mut TinmanWorld) {
    let output = run_output(world);
    let normalised = output.to_lowercase();
    assert!(
        normalised.contains("no tldr page"),
        "the inspect output does not report that no tldr page was read; it reads:\n{output}"
    );
}

#[then("the inspect output reports the examples the program's own help carries")]
async fn the_output_reports_the_help_examples(world: &mut TinmanWorld) {
    let staged = world.staged_help_examples.clone();
    assert!(
        !staged.is_empty(),
        "the scenario staged no example into the program's own help, so this step would assert nothing"
    );
    let output = run_output(world);
    let missing: Vec<&String> = staged
        .iter()
        .filter(|example| !output.contains(example.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "the inspect output reports none of {missing:?} from the program's own help; it reads:\n{output}"
    );
}

// ---------------------------------------------------------------------------
// the plan a documented-example probe writes
// ---------------------------------------------------------------------------

/// The tldr-pages project's own name, as a plan crediting it carries it.
const TLDR_PROJECT: &str = "tldr-pages";

#[then("the written plan is not empty")]
async fn the_written_plan_is_not_empty(world: &mut TinmanWorld) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    assert!(
        !text.trim().is_empty(),
        "the written plan carries nothing at all"
    );
}

#[then(expr = "the written plan carries no expectation for {string}")]
async fn the_written_plan_carries_no_expectation_for(world: &mut TinmanWorld, subject: String) {
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
        !expectations.iter().any(|found| found.contains(&subject)),
        "the written plan carries the expectations {expectations:?}, one of them for {subject:?}; it reads:\n{text}"
    );
}

#[then(expr = "the written plan carries a key step sending {string}")]
async fn the_written_plan_carries_a_key_step(world: &mut TinmanWorld, key: String) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    let written: serde_yaml::Value =
        serde_yaml::from_str(&text).expect("the written plan parses as YAML");
    assert!(
        records_press(&written, &key),
        "the written plan carries no key step sending {key:?}; it reads:\n{text}"
    );
}

#[then(expr = "the written plan carries no key step for {string}")]
async fn the_written_plan_carries_no_key_step_for(world: &mut TinmanWorld, notation: String) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    let written: serde_yaml::Value =
        serde_yaml::from_str(&text).expect("the written plan parses as YAML");
    // The notation is what the page wrote; a key step for it would either send
    // the notation verbatim or send the key it spells, so neither may appear.
    let spelled = notation.trim_matches(|c| c == '<' || c == '>').to_string();
    assert!(
        !records_press(&written, &notation) && !records_press(&written, &spelled),
        "the written plan carries a key step for {notation:?}; it reads:\n{text}"
    );
}

#[then("the written plan credits the tldr-pages project for the page it read")]
async fn the_written_plan_credits_the_project(world: &mut TinmanWorld) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    assert!(
        text.contains(TLDR_PROJECT),
        "the written plan credits no {TLDR_PROJECT} for the page it read; it reads:\n{text}"
    );
}

#[then("the written plan credits no page")]
async fn the_written_plan_credits_no_page(world: &mut TinmanWorld) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    assert!(
        !text.contains(TLDR_PROJECT),
        "the written plan credits {TLDR_PROJECT} though it read no page; it reads:\n{text}"
    );
}

#[then(expr = "the written plan names {string} as that page's licence")]
async fn the_written_plan_names_the_licence(world: &mut TinmanWorld, licence: String) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    assert!(
        text.contains(&licence),
        "the written plan names no {licence:?} as the page's licence; it reads:\n{text}"
    );
}

#[then("the written plan names the command Tinman ran with its substituted path")]
async fn the_written_plan_names_the_substituted_command(world: &mut TinmanWorld) {
    let text = written_plan_text(world, "tinman.yaml");
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
        named
            .iter()
            .any(|command| !command.contains("{{") && !command.contains("}}")),
        "the written plan names {named:?}, so none is a command line with its placeholder substituted; it reads:\n{text}"
    );
}

#[then(expr = "the written plan records {string} as the documented example it came from")]
async fn the_written_plan_records_the_documented_example(world: &mut TinmanWorld, example: String) {
    let text = written_plan_text(world, "tinman.yaml");
    written_plan(world, "tinman.yaml");
    assert!(
        text.contains(&example),
        "the written plan records no {example:?} as the documented example it came from; it reads:\n{text}"
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
    //
    // The hooks record each scenario's own wall clock into the wake, so the
    // weather record's per-tier ceiling has a per-scenario attribution beside
    // it. The run is closed by hand rather than by `run_and_exit` because the
    // record's completion line has to land before the failure is raised: a
    // sweep that ends red still says where its time went, and a reader tells
    // that sweep from one that was killed part way.
    //
    // The feature root is the specs directory unless the environment names
    // another. The concurrency measurement names a generated fixture, so it can
    // run this same binary over four scenarios of its own and read the wall
    // clock back; every tier leaves the variable unset and sweeps the specs.
    use cucumber::writer::Stats as _;
    let features = std::env::var(support::FEATURE_ROOT_VARIABLE)
        .unwrap_or_else(|_| support::SPECS_DIRECTORY.to_string());
    let writer = TinmanWorld::cucumber()
        .before(|feature, _rule, scenario, _world| {
            let key = support::scenario_key(feature, scenario);
            Box::pin(async move {
                support::record_scenario_started(&key);
            })
        })
        .after(|feature, _rule, scenario, _finished, _world| {
            let key = support::scenario_key(feature, scenario);
            Box::pin(async move {
                support::record_scenario_finished(&key);
            })
        })
        .fail_on_skipped()
        .run(&features)
        .await;
    support::record_run_complete(&features, support::run_is_narrowed());
    assert!(
        !writer.execution_has_failed(),
        "{} step(s) failed, {} parsing error(s), {} hook error(s)",
        writer.failed_steps(),
        writer.parsing_errors(),
        writer.hook_errors()
    );
}

// ---------------------------------------------------------------------------
// proxied egress
// ---------------------------------------------------------------------------

/// The body the upstream answers every request with, so a scenario asserting
/// that the upstream's own answer was served has something to name.
const UPSTREAM_BODY: &str = r#"{"answer":"from the upstream"}"#;

/// The body the upstream is switched to after a recording is made, so a replay
/// that reached the network would answer with this rather than the recording.
const UPSTREAM_CURRENT_BODY: &str = r#"{"answer":"from the upstream, changed"}"#;

/// The path the recorded exchange is made against.
const RECORDED_PATH: &str = "/exchange";

/// The URL a request through the proxy names, in the absolute form a client
/// configured to use a proxy sends.
fn upstream_url(world: &TinmanWorld, path: &str) -> String {
    let upstream = world.upstream.as_ref().expect("the upstream started");
    format!("{}{path}", upstream.base_url())
}

/// Start the proxy under the given egress policy and hold it on the world.
fn start_proxy(world: &mut TinmanWorld, egress: tinman::sandbox::Egress) {
    let proxy = tinman::proxy::Proxy::start(&egress).expect("the proxy starts");
    world.proxy = Some(proxy);
}

/// The HAR path this scenario records to and replays from, created under a
/// scratch directory whose teardown is registered with it.
fn har_path(world: &mut TinmanWorld) -> String {
    if let Some(path) = world.har_path.as_ref() {
        return path.clone();
    }
    let dir = support::ScratchDir::new("har");
    let path = dir.path().join("egress.har").display().to_string();
    world.har_dir = Some(dir);
    world.har_path = Some(path.clone());
    path
}

/// Make one request through the proxy and record what it answered.
fn cross_the_proxy(world: &mut TinmanWorld, path: &str) {
    let url = upstream_url(world, path);
    let address = world
        .proxy
        .as_ref()
        .expect("the proxy started")
        .address()
        .to_string();
    match support::request_through_proxy(&address, &url) {
        Ok(response) => world.proxied = Some(response),
        Err(e) => world.proxy_error = Some(e),
    }
}

#[given("a recorded upstream that answers on a real port")]
async fn a_recorded_upstream(world: &mut TinmanWorld) {
    world.upstream = Some(support::Upstream::answering(UPSTREAM_BODY));
}

#[given("the proxy is recording to a HAR file")]
async fn the_proxy_is_recording_to_a_har_file(world: &mut TinmanWorld) {
    // The HAR does not exist yet, which is what puts the proxy in recording
    // rather than replaying: the recording is the file's presence.
    let har = har_path(world);
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: Some(har),
            fault: None,
        },
    );
}

#[when("a request crosses the proxy and is answered")]
async fn a_request_crosses_the_proxy_and_is_answered(world: &mut TinmanWorld) {
    cross_the_proxy(world, RECORDED_PATH);
    let response = world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the proxy answered: {:?}", world.proxy_error));
    assert_eq!(
        response.status, 200,
        "the exchange was not answered: {response:?}"
    );
}

#[then("the HAR file carries one entry for that request")]
async fn the_har_carries_one_entry(world: &mut TinmanWorld) {
    let entries = support::har_entries(&har_path(world));
    assert_eq!(
        entries.len(),
        1,
        "the HAR carries {} entr(ies) rather than the one exchange that crossed the proxy",
        entries.len()
    );
}

#[then("the entry records the request method, the request URL and the response status")]
async fn the_entry_records_method_url_and_status(world: &mut TinmanWorld) {
    let url = upstream_url(world, RECORDED_PATH);
    let entries = support::har_entries(&har_path(world));
    let entry = entries.first().expect("the HAR carries the entry");
    assert_eq!(entry.method, "GET", "the entry records the wrong method");
    assert_eq!(entry.url, url, "the entry records the wrong URL");
    assert_eq!(entry.status, 200, "the entry records the wrong status");
}

#[given("a HAR file carrying one recorded exchange")]
async fn a_har_carrying_one_recorded_exchange(world: &mut TinmanWorld) {
    let har = har_path(world);
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: Some(har.clone()),
            fault: None,
        },
    );
    cross_the_proxy(world, RECORDED_PATH);
    world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the exchange was recorded: {:?}", world.proxy_error));
    // The recording is closed before it is replayed from, so the replay reads a
    // complete file rather than one still being written.
    world.proxy.take().expect("the proxy started").stop();
    let entries = support::har_entries(&har);
    assert_eq!(
        entries.len(),
        1,
        "the recording carries {} entr(ies) rather than one",
        entries.len()
    );
    // Producing the recording meant one real exchange with the upstream, which
    // is this step's own setup and not the request the scenario goes on to
    // assert about. The log starts where the action does, the same way it does
    // after the readiness probe.
    world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .forget_received();
    world.proxied = None;
    world.proxy_error = None;
}

#[given("the upstream now answers that same request with a different body")]
async fn the_upstream_now_answers_with_a_different_body(world: &mut TinmanWorld) {
    // The upstream stays up and serving. Changing what it would answer is what
    // makes the replay assertion falsifiable: a proxy that forwarded instead of
    // replaying now returns a body the recording does not carry, and the
    // scenario can tell the two apart.
    world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .answer_with(UPSTREAM_CURRENT_BODY);
}

#[when("the same request is made")]
async fn the_same_request_is_made(world: &mut TinmanWorld) {
    let har = har_path(world);
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: Some(har),
            fault: None,
        },
    );
    cross_the_proxy(world, RECORDED_PATH);
}

#[when("a request the recording does not carry is made")]
async fn a_request_the_recording_does_not_carry(world: &mut TinmanWorld) {
    let har = har_path(world);
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: Some(har),
            fault: None,
        },
    );
    cross_the_proxy(world, "/unrecorded");
}

#[then("the recorded body is served rather than the upstream's current one")]
async fn the_recorded_body_is_served(world: &mut TinmanWorld) {
    let current = world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .body();
    let response = world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the replay answered: {:?}", world.proxy_error));
    assert_eq!(response.status, 200, "the replay served the wrong status");
    assert_eq!(
        response.body.trim(),
        UPSTREAM_BODY,
        "the replay served a body the recording does not carry"
    );
    assert_ne!(
        response.body.trim(),
        current.trim(),
        "the replay served the upstream's current body rather than the recorded one"
    );
}

#[then("the request fails naming the entry it did not match")]
async fn the_request_fails_naming_the_entry(world: &mut TinmanWorld) {
    // A miss is a failure the client can see, whether the proxy refuses the
    // exchange outright or answers with an error naming it. Either way the
    // request it could not match is what the failure has to name, so a reader of
    // the failure knows which entry to record.
    let named = match (world.proxied.as_ref(), world.proxy_error.as_ref()) {
        (Some(response), _) => {
            assert!(
                response.status >= 400,
                "the unmatched request was answered with {} rather than failing",
                response.status
            );
            response.body.clone()
        }
        (None, Some(error)) => error.clone(),
        (None, None) => panic!("the unmatched request neither answered nor failed"),
    };
    assert!(
        named.contains("/unrecorded"),
        "the failure does not name the entry it did not match: {named}"
    );
}

#[given(expr = "the proxy is injecting the status {int}")]
async fn the_proxy_is_injecting_the_status(world: &mut TinmanWorld, status: u16) {
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: None,
            fault: Some(tinman::sandbox::Fault {
                status: Some(status),
                latency: None,
                offline: false,
            }),
        },
    );
}

#[given(expr = "the proxy is injecting a latency of {int} seconds")]
async fn the_proxy_is_injecting_a_latency(world: &mut TinmanWorld, seconds: u64) {
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: None,
            fault: Some(tinman::sandbox::Fault {
                status: None,
                latency: Some(format!("{seconds}s")),
                offline: false,
            }),
        },
    );
}

#[given("the proxy is injecting the offline fault")]
async fn the_proxy_is_injecting_the_offline_fault(world: &mut TinmanWorld) {
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: None,
            fault: Some(tinman::sandbox::Fault {
                status: None,
                latency: None,
                offline: true,
            }),
        },
    );
}

#[when("a request crosses the proxy")]
async fn a_request_crosses_the_proxy(world: &mut TinmanWorld) {
    cross_the_proxy(world, RECORDED_PATH);
}

#[then(expr = "the response carries the status {int}")]
async fn the_response_carries_the_status(world: &mut TinmanWorld, status: u16) {
    let response = world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the proxy answered: {:?}", world.proxy_error));
    assert_eq!(
        response.status, status,
        "the proxy served {} rather than the injected status",
        response.status
    );
}

#[then("the upstream received no request")]
async fn the_upstream_received_no_request(world: &mut TinmanWorld) {
    let received = world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .received();
    assert!(
        received.is_empty(),
        "the upstream received {} request(s) although the fault was to be served instead: \
         {received:?}",
        received.len()
    );
}

#[then(expr = "the response arrives no sooner than {int} seconds after the request")]
async fn the_response_arrives_no_sooner_than(world: &mut TinmanWorld, seconds: u64) {
    let response = world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the proxy answered: {:?}", world.proxy_error));
    let configured = std::time::Duration::from_secs(seconds);
    assert!(
        response.elapsed >= configured,
        "the response arrived after {:?}, sooner than the injected latency of {configured:?}",
        response.elapsed
    );
}

#[then("the response carries the upstream's own body")]
async fn the_response_carries_the_upstreams_body(world: &mut TinmanWorld) {
    let expected = world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .body()
        .to_string();
    let response = world
        .proxied
        .as_ref()
        .unwrap_or_else(|| panic!("the proxy answered: {:?}", world.proxy_error));
    assert_eq!(
        response.body.trim(),
        expected,
        "the delayed response did not carry the upstream's own body"
    );
}

#[then("the connection is refused")]
async fn the_connection_is_refused(world: &mut TinmanWorld) {
    let error = world.proxy_error.as_ref().unwrap_or_else(|| {
        panic!(
            "the proxy answered rather than refusing the connection: {:?}",
            world.proxied
        )
    });
    assert!(
        error.contains("refused") || error.contains("closed"),
        "the connection failed for a reason other than refusal: {error}"
    );
}

#[given(expr = "a sandbox specification setting network to {string}")]
async fn spec_setting_network_to(world: &mut TinmanWorld, network: String) {
    let mut spec = SandboxSpec::default_for_record();
    spec.network = match network.as_str() {
        "allow" => Network::Allow,
        "deny" => Network::Deny,
        "proxy" => Network::Proxy,
        other => panic!("no sandbox network policy is named {other}"),
    };
    world.spec = Some(spec);
}

#[when("the specification is validated")]
async fn the_specification_is_validated(world: &mut TinmanWorld) {
    let spec = world.spec.as_ref().expect("a sandbox specification");
    world.serialized = Some(to_json(spec));
}

#[then(expr = "the specification conforms to the {string} schema in {string}")]
async fn the_specification_conforms_to_schema(
    world: &mut TinmanWorld,
    _schema_id: String,
    path: String,
) {
    let instance = world
        .serialized
        .as_ref()
        .expect("the specification was serialized");
    let bad = support::schema_counterexamples(&path, instance);
    assert!(bad.is_empty(), "schema violations: {bad:?}");
}

// ---------------------------------------------------------------------------
// rule placement conformance
// ---------------------------------------------------------------------------

/// A feature file read for where its `Rule:` lines sit relative to its
/// `Background:`.
#[derive(Debug)]
struct RulePlacement {
    path: String,
    /// The line numbers of every `Rule:` declared above the `Background:`. A
    /// rule opens a block, so a background below one belongs to that rule
    /// rather than to the feature, and any later rule then takes the scenarios
    /// under it out of that background's scope.
    rules_above_background: Vec<usize>,
}

/// Read every feature file under the specs directory for a rule declared after
/// its background. The read is line-based on the Gherkin keywords, which is
/// what the parser itself keys on, so the check sees the file the way the
/// runner does.
fn rule_placements() -> Vec<RulePlacement> {
    let specs = std::path::Path::new("features");
    let mut placements = Vec::new();
    let entries = std::fs::read_dir(specs).expect("the specs directory is readable");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("feature") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("the feature file is readable");
        let mut background: Option<usize> = None;
        let mut rules_seen = Vec::new();
        let mut rules_above_background = Vec::new();
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("Rule:") {
                rules_seen.push(index + 1);
                continue;
            }
            if background.is_none() && trimmed.starts_with("Background:") {
                background = Some(index + 1);
                rules_above_background = rules_seen.clone();
            }
        }
        if background.is_some() {
            placements.push(RulePlacement {
                path: path.display().to_string(),
                rules_above_background,
            });
        }
    }
    placements
}

#[given("the feature files in the specs")]
async fn the_feature_files_in_the_specs(world: &mut TinmanWorld) {
    world.rule_placements = Some(rule_placements());
}

#[when("each feature carrying a background is read for a rule declared above it")]
async fn each_feature_read_for_a_rule_above_its_background(world: &mut TinmanWorld) {
    // The read happened when the features were gathered; this step names the
    // action the scenario describes so the assertion below reads what it says.
    world
        .rule_placements
        .as_ref()
        .expect("the feature files were read");
}

#[then("every background is declared before any rule")]
async fn every_background_is_declared_before_any_rule(world: &mut TinmanWorld) {
    let placements = world
        .rule_placements
        .as_ref()
        .expect("the feature files were read");
    let offenders: Vec<String> = placements
        .iter()
        .filter(|placement| !placement.rules_above_background.is_empty())
        .map(|placement| {
            format!(
                "{} declares its background below rule(s) at line(s) {:?}",
                placement.path, placement.rules_above_background
            )
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "a background declared below a rule belongs to that rule rather than to the feature, so \
         a later rule takes the scenarios under it out of scope and leaves their given state \
         unprovisioned: {offenders:?}"
    );
}

#[then("the features read are not empty")]
async fn the_features_read_are_not_empty(world: &mut TinmanWorld) {
    let placements = world
        .rule_placements
        .as_ref()
        .expect("the feature files were read");
    // The floor guards the reader: a listing that found nothing and a tree with
    // no backgrounds both report zero, and only one of those is a clean bill.
    assert!(
        !placements.is_empty(),
        "no feature carrying a background was read, so the check asserted nothing"
    );
}

// ---------------------------------------------------------------------------
// proxied egress under the sandbox
// ---------------------------------------------------------------------------

/// The inference credential Tinman holds. One value serves both directions a
/// scenario asserts, that the sandbox never saw it and that the upstream did,
/// so a leak names itself.
const HELD_CREDENTIAL: &str = "sk-tinman-held-credential";

/// Run `script` as the sandboxed target under the staged specification and
/// capture what it wrote, so what the target observed is read off its own
/// output rather than from the configuration it was launched with.
fn run_sandboxed_target(world: &mut TinmanWorld, script: &str) {
    let spec = world.spec.clone().expect("a sandbox specification");
    let command = CommandSpec {
        program: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
    };
    let prepared = staged_backend(world)
        .prepare(&spec, &command)
        .expect("the Bubblewrap backend prepares the process");
    world.screen = Some(tinman::pty::capture(&prepared).expect("the process is captured"));
    world.prepared = Some(prepared);
}

/// Stage the proxy and the sandbox specification a plan names, reading the
/// section through the production parser so the policy the plan states is the
/// one the backend is handed.
fn stage_proxied_plan(world: &mut TinmanWorld, network: &str) {
    start_proxy(
        world,
        tinman::sandbox::Egress {
            har: None,
            fault: None,
        },
    );
    world
        .plan_sources
        .push(format!("home: empty\nnetwork: {network}\n"));
    world.spec = Some(sandbox_section(world));
}

#[given(expr = "a plan whose sandbox sets network to {string}")]
async fn a_plan_whose_sandbox_sets_network_to(world: &mut TinmanWorld, network: String) {
    stage_proxied_plan(world, &network);
}

#[when("the target requests the allowed host directly by address")]
async fn the_target_requests_the_allowed_host_directly(world: &mut TinmanWorld) {
    let url = upstream_url(world, RECORDED_PATH);
    let proxy = world
        .proxy
        .as_ref()
        .expect("the proxy started")
        .address()
        .to_string();
    // Both routes are attempted in the one sandbox, so the outcome the policy
    // must refuse and the outcome it must allow are read from a single run and
    // cannot disagree about which sandbox produced them.
    let script = format!(
        "if /bin/curl -s -m 5 -o /dev/null {url}; then echo DIRECT_ANSWERED; else echo DIRECT_REFUSED; fi; \
         if /bin/curl -s -m 5 -o /dev/null -x http://{proxy} {url}; then echo PROXIED_ANSWERED; else echo PROXIED_REFUSED; fi"
    );
    run_sandboxed_target(world, &script);
}

#[then("the direct request is refused")]
async fn the_direct_request_is_refused(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let contents = screen.contents();
    // The target reports both outcomes, so an absent verdict is a target that
    // never ran rather than a refusal, and it fails as loudly as a breach.
    assert!(
        contents.contains("DIRECT_REFUSED") || contents.contains("DIRECT_ANSWERED"),
        "the sandboxed target reported no verdict on the direct request; screen:\n{contents}"
    );
    assert!(
        contents.contains("DIRECT_REFUSED"),
        "the sandboxed target reached the allowed host directly rather than being refused; \
         screen:\n{contents}"
    );
}

#[then("the same request through the proxy is answered")]
async fn the_same_request_through_the_proxy_is_answered(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let contents = screen.contents();
    assert!(
        contents.contains("PROXIED_ANSWERED"),
        "the sandboxed target's request through the proxy was not answered; screen:\n{contents}"
    );
}

#[given(
    expr = "a plan whose sandbox sets network to {string} and an inference credential held by Tinman"
)]
async fn a_plan_with_network_and_a_held_credential(world: &mut TinmanWorld, network: String) {
    // The credential is held in Tinman's own environment, which is what the
    // backend reads, and is never granted into the sandbox. That is the whole
    // of "held by Tinman": present where Tinman runs, absent where the target
    // runs.
    world
        .handed_env
        .get_or_insert_with(std::collections::BTreeMap::new)
        .insert("TINMAN_API_KEY".to_string(), HELD_CREDENTIAL.to_string());
    stage_proxied_plan(world, &network);
}

#[when("the target prints its whole environment and then calls the provider")]
async fn the_target_prints_its_environment_and_calls_the_provider(world: &mut TinmanWorld) {
    let url = upstream_url(world, RECORDED_PATH);
    let proxy = world
        .proxy
        .as_ref()
        .expect("the proxy started")
        .address()
        .to_string();
    // `export -p` is a shell builtin, so the environment is read without a
    // binary the sandbox would also have to grant.
    let script = format!(
        "export -p; echo ENV_PRINTED; /bin/curl -s -m 5 -o /dev/null -x http://{proxy} {url}"
    );
    run_sandboxed_target(world, &script);
}

#[then("the printed environment carries no inference credential")]
async fn the_printed_environment_carries_no_credential(world: &mut TinmanWorld) {
    let screen = world.screen.as_ref().expect("a captured virtual screen");
    let contents = screen.contents();
    // The marker proves the environment was actually printed, so an empty
    // capture cannot pass as an absent credential.
    assert!(
        contents.contains("ENV_PRINTED"),
        "the sandboxed target never printed its environment; screen:\n{contents}"
    );
    assert!(
        !contents.contains(HELD_CREDENTIAL),
        "the inference credential {HELD_CREDENTIAL} reached the sandbox; screen:\n{contents}"
    );
}

#[then("the request the upstream received carries the credential")]
async fn the_request_the_upstream_received_carries_the_credential(world: &mut TinmanWorld) {
    let received = world
        .upstream
        .as_ref()
        .expect("the upstream started")
        .received();
    let request = received.first().unwrap_or_else(|| {
        panic!("the upstream received no request, so nothing carried the credential")
    });
    assert!(
        request
            .headers
            .iter()
            .any(|header| header.contains(HELD_CREDENTIAL)),
        "the {} request the upstream received for {} carries no inference credential; its headers \
         are {:?}",
        request.method,
        request.path,
        request.headers
    );
}

// ---------------------------------------------------------------------------
// command-line expectation
// ---------------------------------------------------------------------------

#[then("the command exits with a zero status")]
async fn the_command_exits_zero(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a command was run");
    assert_eq!(
        status,
        0,
        "the command exited {status}; it wrote to the data stream:\n{}\nand to the error \
         stream:\n{}",
        run_output(world),
        run_errors(world)
    );
}

/// A failed expectation shows the screen it read, so the assertion is that the
/// failure carries what the program itself drew rather than a bare status. The
/// scenarios state their expectation against `top`, whose column header `PID`
/// is the text the feature's holding expectation names, so a failure carrying
/// it is carrying the screen.
#[then("the failure carries the screen the program drew")]
async fn the_failure_carries_the_screen(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    assert!(
        errors.contains("PID"),
        "the failure carries nothing the program drew, so it does not show the screen it read; \
         the error stream reads:\n{errors}"
    );
}

/// The keys the serialized terminal object model is written with. A region
/// carries its place as `rect` and its parts as `children`, and neither name
/// reaches a screen a program drew, so a failure carrying one is carrying the
/// document rather than the screen. This is the whole of what a run can decide
/// here: a failure quoting some other part of the model is not findable this
/// way and no check here will catch one.
const SERIALIZED_MODEL_KEYS: [&str; 2] = ["\"rect\"", "\"children\""];

#[then("the failure carries no serialized model")]
async fn the_failure_carries_no_serialized_model(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    let carried: Vec<&str> = SERIALIZED_MODEL_KEYS
        .iter()
        .copied()
        .filter(|key| errors.contains(key))
        .collect();
    assert!(
        carried.is_empty(),
        "the failure carries the serialized model, naming {carried:?}; the error stream \
         reads:\n{errors}"
    );
}

#[then(expr = "the failure names {string} as the region it looked for")]
async fn the_failure_names_the_region_it_looked_for(world: &mut TinmanWorld, region: String) {
    let errors = run_errors(world);
    assert!(
        errors.contains(&region),
        "the failure does not name {region:?} as the region it looked for; the error stream \
         reads:\n{errors}"
    );
}

/// The wrapper syntax Rust's own debug rendering of an optional or a result
/// puts around a value. A message carrying one is the language leaking through
/// prose written for an operator, and it is mechanically findable, which is the
/// whole of what this step can decide: a message that is merely unclear is not
/// findable and no check here will catch one.
const RUST_WRAPPER_SYNTAX: [&str; 4] = ["Some(", "None", "Ok(", "Err("];

#[then("the failure carries no optional or result wrapper syntax")]
async fn the_failure_carries_no_wrapper_syntax(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    let carried: Vec<&str> = RUST_WRAPPER_SYNTAX
        .iter()
        .copied()
        .filter(|wrapper| errors.contains(wrapper))
        .collect();
    assert!(
        carried.is_empty(),
        "the failure carries the wrapper syntax {carried:?}, which is Rust's own rendering of an \
         optional or a result reaching an operator; the error stream reads:\n{errors}"
    );
}

#[then(expr = "the failure reports that {string} is not a role the model produces")]
async fn the_failure_reports_an_unproduced_role(world: &mut TinmanWorld, role: String) {
    let errors = run_errors(world);
    assert!(
        errors.contains(&role) && errors.contains("role"),
        "the failure does not report {role:?} as a role the model does not produce; the error \
         stream reads:\n{errors}"
    );
}

/// Ambiguity is reported as ambiguity, so the failure names how many regions
/// the locator reached and that count is more than one. A message reporting a
/// single match would be a resolution reported as a failure.
#[then("the failure reports the locator matched more than one region")]
async fn the_failure_reports_the_ambiguity(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    assert!(
        errors.contains("matches"),
        "the failure does not report what the locator matched; the error stream reads:\n{errors}"
    );
    let count: usize = errors
        .split(|c: char| !c.is_ascii_digit())
        .filter(|piece| !piece.is_empty())
        .filter_map(|piece| piece.parse().ok())
        .max()
        .unwrap_or_else(|| panic!("the failure reports no count of matches; it reads:\n{errors}"));
    assert!(
        count > 1,
        "the failure reports {count} matches, so it does not report an ambiguity; the error \
         stream reads:\n{errors}"
    );
}

/// A locator carrying no name has no name to report, so a message naming one
/// describes a locator the operator never wrote. The name clause is the
/// findable half of that fault: a failure for a nameless locator carries no
/// `named` clause at all. This is distinct from the wrapper-syntax step, which
/// catches the rendering of the absent value rather than the wrong sentence,
/// and a message reading `named ""` would satisfy that step while still
/// reporting a name.
#[then("the failure reports no name for a locator that carries none")]
async fn the_failure_reports_no_name_for_a_nameless_locator(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    assert!(
        !errors.contains("named"),
        "the failure reports a name for a locator that carries none; the error stream \
         reads:\n{errors}"
    );
}

#[then(expr = "no file is written at {string}")]
async fn no_file_is_written_at(world: &mut TinmanWorld, name: String) {
    let path = working_dir(world).join(&name);
    assert!(
        !path.exists(),
        "a file was written at {}, and it reads:\n{}",
        path.display(),
        std::fs::read_to_string(&path).unwrap_or_default()
    );
}

#[then(expr = "{string} is a plan whose expectation names the role {string} and the name {string}")]
async fn the_plan_expectation_names_role_and_name(
    world: &mut TinmanWorld,
    file: String,
    role: String,
    name: String,
) {
    // The written text is kept as well as the parsed plan, so a following step
    // reading what that expectation carries reads the file the operator commits.
    let text = written_plan_text(world, &file);
    world.written_plan_source = Some(text);
    let plan = written_plan(world, &file);
    let locators: Vec<tinman::plan::Locator> = plan
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.clone(),
            tinman::plan::FlowStep::Run(_) => Vec::new(),
        })
        .filter_map(|action| match action {
            tinman::plan::Action::Expect(expectation) => expectation.locator,
            _ => None,
        })
        .collect();
    assert!(
        locators
            .iter()
            .any(|locator| locator.role.as_deref() == Some(role.as_str()) && locator.name == name),
        "the plan carries no expectation naming the {role} {name:?}; its expectation locators \
         are {locators:?}"
    );
}

#[then("that expectation carries no text")]
async fn that_expectation_carries_no_text(world: &mut TinmanWorld) {
    let source = written_plan_source(world).to_string();
    let document: serde_json::Value = serde_yaml::from_str(&source)
        .unwrap_or_else(|e| panic!("the written plan is not YAML: {e}\nit reads:\n{source}"));
    assert_states_no_text(&document, &source);
}

// ---------------------------------------------------------------------------
// command-line isolation
// ---------------------------------------------------------------------------

/// The value the operator's own environment carries. A sandbox that let it
/// through would put this text on the target's screen, which is what the
/// scenarios assert never happens.
const OPERATOR_SECRET: &str = "hunter2";

/// What the target prints when the variable did not reach it.
const SECRET_ABSENT: &str = "SECRET=[]";

/// Stage a target that prints whether it read `name` and then holds its screen,
/// so an expectation stated against it reads a drawn screen rather than racing
/// a program that has already exited.
fn stage_secret_reporting_target(world: &mut TinmanWorld, name: &str) -> String {
    stage_recorded_program(
        world,
        "reports-secret",
        &format!("#!/bin/sh\nprintf 'SECRET=[%s]\\n' \"${name}\"\nsleep 5\n"),
    )
}

#[given(expr = "the operator's environment carries {string}")]
async fn the_operators_environment_carries(world: &mut TinmanWorld, name: String) {
    world
        .env_vars
        .insert(name.clone(), OPERATOR_SECRET.to_string());
    world.secret_name = Some(name);
    world.secret_value = Some(OPERATOR_SECRET.to_string());
}

// The pattern stays on one line: a Rust line continuation puts the escape and
// the indent into the literal `step-usage` reports, so an exact-string join
// against a plank carrying the pattern the runner binds could never match.
#[when(
    expr = "the operator states an expectation against a target that prints whether it read {string}"
)]
async fn the_operator_states_an_expectation_against_a_secret_reporting_target(
    world: &mut TinmanWorld,
    name: String,
) {
    let program = stage_secret_reporting_target(world, &name);
    run_tinman_command(world, &["expect", SECRET_ABSENT, &program]).await;
}

/// The target reports the variable was absent exactly when the expectation
/// naming the absent form held. A leak draws the operator's own value on the
/// screen instead, the expectation fails, and the failure carries the screen
/// that shows it.
#[then("the target reports the variable was absent")]
async fn the_target_reports_the_variable_absent(world: &mut TinmanWorld) {
    let status = world.run_status.expect("a command was run");
    assert_eq!(
        status,
        0,
        "the target did not report {:?} absent, so the operator's environment reached inside the \
         sandbox; the data stream reads:\n{}\nand the error stream reads:\n{}",
        world
            .secret_name
            .clone()
            .expect("the scenario named the variable"),
        run_output(world),
        run_errors(world)
    );
}

// ---------------------------------------------------------------------------
// the locator form of an expectation, against the plan scantling
// ---------------------------------------------------------------------------

/// A plan whose one step states an expectation as a locator: a role and a name,
/// the same locator an activation addresses a region with.
const LOCATOR_EXPECTATION_PLAN: &str =
    "tui: top\nsteps:\n  - expect:\n      role: columnheader\n      name: PID\n";

#[given("the plan fixture carrying an expectation naming a role and a name")]
async fn the_plan_fixture_carrying_a_locator_expectation(world: &mut TinmanWorld) {
    world
        .plan_sources
        .push(LOCATOR_EXPECTATION_PLAN.to_string());
}

/// Validation reads the plan as it is written. Reading it back through
/// production would attest whatever production normalized it to, which is a
/// different document from the one an operator commits and this scantling
/// governs.
#[when("the plan is validated")]
async fn the_plan_is_validated(world: &mut TinmanWorld) {
    let source = world
        .plan_sources
        .first()
        .expect("a harness plan was given")
        .clone();
    let value: serde_yaml::Value = serde_yaml::from_str(&source)
        .unwrap_or_else(|e| panic!("the plan is not YAML: {e}\nit reads:\n{source}"));
    world.serialized = Some(to_json(&value));
}

// ---------------------------------------------------------------------------
// command-line session
// ---------------------------------------------------------------------------

/// The command-line sessions a scenario launched, with the directory they were
/// launched from. A session outlives the command that launched it, so a
/// scenario that fails between launch and close leaves a real program running
/// and a real sandbox standing. Teardown is registered before the launch and
/// reports what it could not close, since a quiet failure here leaks the very
/// resource the feature exists to reclaim.
#[derive(Debug, Default)]
struct LaunchedSessions {
    dir: Option<std::path::PathBuf>,
    names: Vec<String>,
    /// The program each launched session drives, in the order the names are
    /// held, so a step reading a listing asserts against what the scenario
    /// launched rather than against a program name written into the step.
    programs: Vec<String>,
    /// The temporary directory the runs that created these sessions were given,
    /// so teardown addresses the same sessions the scenario launched.
    tmp: Option<std::path::PathBuf>,
    /// What stood in that directory before the first launch, so a resource the
    /// session created is one that appeared after it. Matching on the session
    /// name alone reads the suite's own scratch directory,
    /// `tinman-workdir-<pid>-<nanos>`, as a resource of the session `work`.
    before: Option<std::collections::BTreeSet<std::path::PathBuf>>,
}

/// What stands in `dir` now.
fn temp_entries(dir: &std::path::Path) -> std::collections::BTreeSet<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return std::collections::BTreeSet::new();
    };
    entries.flatten().map(|entry| entry.path()).collect()
}

impl LaunchedSessions {
    /// Register `name`, driving `program`, before the launch that creates it.
    fn register(
        &mut self,
        dir: &std::path::Path,
        tmp: &std::path::Path,
        name: &str,
        program: &str,
    ) {
        self.before.get_or_insert_with(|| temp_entries(tmp));
        self.tmp = Some(tmp.to_path_buf());
        self.dir = Some(dir.to_path_buf());
        self.names.push(name.to_string());
        self.programs.push(program.to_string());
    }

    /// What appeared in the temporary directory since the first launch and
    /// carries `name`, which is what a session of that name created.
    fn created_for(&self, name: &str) -> Vec<std::path::PathBuf> {
        let Some(tmp) = self.tmp.as_ref() else {
            return Vec::new();
        };
        let before = self.before.clone().unwrap_or_default();
        temp_entries(tmp)
            .into_iter()
            .filter(|path| !before.contains(path))
            .filter(|path| {
                path.file_name()
                    .and_then(|entry| entry.to_str())
                    .is_some_and(|entry| entry.contains(name))
            })
            .collect()
    }
}

impl Drop for LaunchedSessions {
    fn drop(&mut self) {
        let Some(dir) = self.dir.clone() else { return };
        let env = match self.tmp.as_ref() {
            Some(tmp) => vec![("TMPDIR".to_string(), tmp.display().to_string())],
            None => Vec::new(),
        };
        for name in &self.names {
            match support::run_tinman(&dir, &["close", "--session", name], &env, None) {
                // Closing an already-closed session is the ordinary case, so
                // only the report of what stands is worth the noise.
                Ok(outcome) if outcome.status != 0 => {
                    let standing = self.created_for(name);
                    if !standing.is_empty() {
                        eprintln!(
                            "the session {name:?} was not closed at teardown and left \
                             {standing:?} standing: {}",
                            outcome.stderr
                        );
                    }
                }
                Err(e) => eprintln!("the session {name:?} could not be closed at teardown: {e}"),
                _ => {}
            }
        }
    }
}

/// The text `top` draws that every session scenario addresses it by, proven
/// present by `a text expectation that holds exits zero`. Addressing a session
/// with an expectation that holds is how a scenario reads whether the session
/// is running, through the operator's own surface rather than through a store
/// the command line does not expose.
const TOP_SCREEN_TEXT: &str = "PID";

/// Launch `program` as the session `name`, registering its teardown first.
async fn launch_session(world: &mut TinmanWorld, name: &str, program: &str) {
    let dir = working_dir(world);
    let tmp = scenario_temp_dir(world);
    world.launched_sessions.register(&dir, &tmp, name, program);
    run_tinman_command(world, &["launch", program, "--session", name]).await;
}

/// Whether the operator can still address the session `name`, read by stating
/// an expectation that holds against it.
fn session_answers(world: &mut TinmanWorld, name: &str) -> (i32, String) {
    let dir = working_dir(world);
    let env = configured_env(world);
    let outcome = support::run_tinman(
        &dir,
        &["expect", TOP_SCREEN_TEXT, "--session", name],
        &env,
        None,
    )
    .unwrap_or_else(|e| panic!("the tinman binary did not run: {e}"));
    (outcome.status, outcome.stderr)
}

#[given(expr = "a session named {string} running {string}")]
async fn a_session_named_running(world: &mut TinmanWorld, name: String, program: String) {
    launch_session(world, &name, &program).await;
    let status = world.run_status.expect("the launch ran");
    assert_eq!(
        status,
        0,
        "the session {name:?} running {program:?} was not launched, so the scenario starts from a \
         state it did not reach; the error stream reads:\n{}",
        run_errors(world)
    );
}

#[then(expr = "a session named {string} is running")]
async fn a_session_named_is_running(world: &mut TinmanWorld, name: String) {
    let (status, errors) = session_answers(world, &name);
    assert_eq!(
        status, 0,
        "the session {name:?} does not answer, so it is not running; the error stream \
         reads:\n{errors}"
    );
}

#[then(expr = "the session named {string} is still running")]
async fn the_session_named_is_still_running(world: &mut TinmanWorld, name: String) {
    let (status, errors) = session_answers(world, &name);
    assert_eq!(
        status, 0,
        "the session {name:?} no longer answers, so the verb that addressed it ended it; the \
         error stream reads:\n{errors}"
    );
}

/// A one-shot verb closes around its step, so what it launched is gone when it
/// returns. A session left standing is a sandbox whose launcher has exited,
/// which is what the suite's own sweep reads off the system.
#[then("no session is left running")]
async fn no_session_is_left_running(world: &mut TinmanWorld) {
    for name in world.launched_sessions.names.clone() {
        let (status, _) = session_answers(world, &name);
        assert_ne!(
            status, 0,
            "the session {name:?} still answers, so it was left running"
        );
    }
    // Read against this scenario's own temporary directory rather than the
    // system's: a system-wide sweep reads a session another worker is legally
    // holding as this run's leak, since a live session is by design a sandbox
    // whose launching command has exited.
    let tmp = scenario_temp_dir(world);
    let standing: Vec<std::path::PathBuf> = temp_entries(&tmp)
        .into_iter()
        .filter(|path| !is_staging_cache(path))
        .collect();
    assert!(
        standing.is_empty(),
        "the run left {standing:?} standing, so a session outlived the command that launched it"
    );
}

/// Whether `path` is one of the staging caches a run builds beside its
/// sessions, such as the terminfo database a sandboxed terminal is given.
/// These are provisioned per run rather than per session, so a session ending
/// says nothing about them and a check for a standing session must not read
/// one as a session.
fn is_staging_cache(path: &std::path::Path) -> bool {
    let Some(name) = path.file_name().and_then(|entry| entry.to_str()) else {
        return false;
    };
    ["tinman-terminfo-", "tinman-copy-mount-"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

#[then(expr = "the failure reports no session named {string} is running")]
async fn the_failure_reports_no_such_session(world: &mut TinmanWorld, name: String) {
    let errors = run_errors(world);
    assert!(
        errors.contains(&name) && errors.contains("session"),
        "the failure does not report that no session named {name:?} is running; the error stream \
         reads:\n{errors}"
    );
}

#[when(
    expr = "the operator launches a session against a target that prints whether it read {string}"
)]
async fn the_operator_launches_a_session_against_a_secret_reporting_target(
    world: &mut TinmanWorld,
    name: String,
) {
    let program = stage_secret_reporting_target(world, &name);
    launch_session(world, "work", &program).await;
    let launch_status = world.run_status.expect("the launch ran");
    assert_eq!(
        launch_status,
        0,
        "the session was not launched, so nothing read the target's screen; the error stream \
         reads:\n{}",
        run_errors(world)
    );
    run_tinman_command(world, &["expect", SECRET_ABSENT, "--session", "work"]).await;
}

#[then(expr = "no sandbox resource created for {string} remains")]
async fn no_sandbox_resource_created_for_remains(world: &mut TinmanWorld, name: String) {
    let standing = world.launched_sessions.created_for(&name);
    assert!(
        standing.is_empty(),
        "closing the session {name:?} left {standing:?} standing"
    );
}

#[then("no sandbox resource created for either remains")]
async fn no_sandbox_resource_created_for_either_remains(world: &mut TinmanWorld) {
    let names = world.launched_sessions.names.clone();
    assert!(
        !names.is_empty(),
        "the scenario launched no session, so this step would assert nothing"
    );
    for name in names {
        let standing = world.launched_sessions.created_for(&name);
        assert!(
            standing.is_empty(),
            "purging left {standing:?} standing for the session {name:?}"
        );
    }
}

/// What every session directory is named for, which is how a scenario reads off
/// its own temporary directory whether a session stands in it.
const SESSION_DIRECTORY_PREFIX: &str = "tinman-session-";

/// The session directories standing in this scenario's own temporary directory.
/// Read against that directory rather than the system's, since a session
/// another worker is legally holding is not this scenario's state.
fn standing_sessions(world: &mut TinmanWorld) -> Vec<std::path::PathBuf> {
    let tmp = scenario_temp_dir(world);
    temp_entries(&tmp)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|entry| entry.to_str())
                .is_some_and(|entry| entry.starts_with(SESSION_DIRECTORY_PREFIX))
        })
        .collect()
}

#[given("no session is running")]
async fn no_session_is_running(world: &mut TinmanWorld) {
    let standing = standing_sessions(world);
    assert!(
        standing.is_empty(),
        "the scenario starts with {standing:?} standing, so it does not start from no session \
         running"
    );
}

#[then("the listing reports no session is running")]
async fn the_listing_reports_no_session_is_running(world: &mut TinmanWorld) {
    let listing = run_output(world);
    assert!(
        !listing.trim().is_empty(),
        "the listing is empty, so it reports nothing rather than reporting that no session is \
         running"
    );
    assert!(
        listing.to_lowercase().contains("no session"),
        "the listing does not report that no session is running; it reads:\n{listing}"
    );
}

#[then(expr = "the listing names {string} and {string}")]
async fn the_listing_names_both(world: &mut TinmanWorld, first: String, second: String) {
    let listing = run_output(world);
    for name in [&first, &second] {
        assert!(
            listing.contains(name.as_str()),
            "the listing does not name the session {name:?}; it reads:\n{listing}"
        );
    }
}

#[then("each is listed against the program it drives")]
async fn each_is_listed_against_its_program(world: &mut TinmanWorld) {
    let listing = run_output(world).to_string();
    let sessions: Vec<(String, String)> = world
        .launched_sessions
        .names
        .iter()
        .cloned()
        .zip(world.launched_sessions.programs.iter().cloned())
        .collect();
    assert!(
        !sessions.is_empty(),
        "the scenario launched no session, so this step would assert nothing"
    );
    for (name, program) in sessions {
        let line = listing
            .lines()
            .find(|line| line.contains(&name))
            .unwrap_or_else(|| {
                panic!("the listing names no session {name:?}; it reads:\n{listing}")
            });
        assert!(
            line.contains(&program),
            "the session {name:?} is listed as {line:?}, which does not name the program \
             {program:?} it drives"
        );
    }
}

// ---------------------------------------------------------------------------
// command-line plan serialization
// ---------------------------------------------------------------------------

/// Run one verb against the session a scenario is already holding, and fail
/// where it did not hold: a plan written from steps that never ran would be
/// asserted against below as though they had.
async fn verb_against_session(world: &mut TinmanWorld, args: &[&str]) {
    let session = world
        .launched_sessions
        .names
        .last()
        .cloned()
        .expect("a session was launched");
    let mut argv = args.to_vec();
    argv.push("--session");
    argv.push(&session);
    run_tinman_command(world, &argv).await;
}

#[given(expr = "the operator has expected {string} and pressed {string} in that session")]
async fn the_operator_has_expected_and_pressed(world: &mut TinmanWorld, text: String, key: String) {
    verb_against_session(world, &["expect", &text]).await;
    let expected = world.run_status.expect("the expectation ran");
    assert_eq!(
        expected,
        0,
        "the expectation {text:?} did not hold, so the session never performed it; the error \
         stream reads:\n{}",
        run_errors(world)
    );
    verb_against_session(world, &["press", &key]).await;
    let pressed = world.run_status.expect("the keypress ran");
    assert_eq!(
        pressed,
        0,
        "the key {key:?} was not pressed, so the session never performed it; the error stream \
         reads:\n{}",
        run_errors(world)
    );
}

#[given(expr = "the operator has expected {string} and then expected {string} in that session")]
async fn the_operator_has_expected_and_then_expected(
    world: &mut TinmanWorld,
    held: String,
    failed: String,
) {
    verb_against_session(world, &["expect", &held]).await;
    let status = world.run_status.expect("the expectation ran");
    assert_eq!(
        status,
        0,
        "the expectation {held:?} did not hold, so the session never performed it; the error \
         stream reads:\n{}",
        run_errors(world)
    );
    verb_against_session(world, &["expect", &failed]).await;
    let status = world.run_status.expect("the expectation ran");
    assert_ne!(
        status, 0,
        "the expectation {failed:?} held, so the scenario has no failed step to leave out"
    );
}

#[given(expr = "a plan written from a session that expected {string} against {string}")]
async fn a_plan_written_from_a_session(world: &mut TinmanWorld, text: String, program: String) {
    launch_session(world, "written", &program).await;
    let status = world.run_status.expect("the launch ran");
    assert_eq!(
        status,
        0,
        "the session was not launched; the error stream reads:\n{}",
        run_errors(world)
    );
    verb_against_session(world, &["expect", &text]).await;
    let status = world.run_status.expect("the expectation ran");
    assert_eq!(
        status,
        0,
        "the expectation {text:?} did not hold; the error stream reads:\n{}",
        run_errors(world)
    );
    run_tinman_command(
        world,
        &["close", "--session", "written", "--output", "probe.yaml"],
    )
    .await;
    let status = world.run_status.expect("the close ran");
    assert_eq!(
        status,
        0,
        "the session was not closed, so no plan was written; the error stream reads:\n{}",
        run_errors(world)
    );
    let text = written_plan_text(world, "probe.yaml");
    world.plan_sources.push(text);
}

/// The actions a written plan carries, in the order it carries them.
fn written_plan_actions(world: &mut TinmanWorld, file: &str) -> Vec<tinman::plan::Action> {
    written_plan(world, file)
        .flow
        .iter()
        .flat_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => tui.steps.clone(),
            tinman::plan::FlowStep::Run(_) => Vec::new(),
        })
        .collect()
}

/// What an expectation names: its text, or the name its locator addresses.
fn expectation_names(expectation: &tinman::plan::Expectation) -> String {
    match expectation.locator.as_ref() {
        Some(locator) => locator.name.clone(),
        None => expectation.text.clone(),
    }
}

#[then(expr = "{string} is a plan whose steps are the expectation and the keypress in that order")]
async fn the_plan_carries_the_expectation_then_the_keypress(world: &mut TinmanWorld, file: String) {
    let actions = written_plan_actions(world, &file);
    assert_eq!(
        actions.len(),
        2,
        "the plan carries {} steps rather than the expectation and the keypress: {actions:?}",
        actions.len()
    );
    assert!(
        matches!(actions.first(), Some(tinman::plan::Action::Expect(_))),
        "the plan's first step is not the expectation: {actions:?}"
    );
    assert!(
        matches!(actions.get(1), Some(tinman::plan::Action::Press(_))),
        "the plan's second step is not the keypress: {actions:?}"
    );
}

#[then(expr = "{string} is a plan whose only step is the expectation naming {string}")]
async fn the_plan_carries_only_the_expectation_naming(
    world: &mut TinmanWorld,
    file: String,
    named: String,
) {
    let actions = written_plan_actions(world, &file);
    assert_eq!(
        actions.len(),
        1,
        "the plan carries {} steps rather than the one expectation: {actions:?}",
        actions.len()
    );
    match actions.first() {
        Some(tinman::plan::Action::Expect(expectation)) => assert_eq!(
            expectation_names(expectation),
            named,
            "the plan's only step names something other than {named:?}: {expectation:?}"
        ),
        other => panic!("the plan's only step is not an expectation: {other:?}"),
    }
}

#[then(expr = "{string} names the program {string} and carries no session name")]
async fn the_plan_names_the_program_and_no_session(
    world: &mut TinmanWorld,
    file: String,
    program: String,
) {
    let session = world
        .launched_sessions
        .names
        .last()
        .cloned()
        .expect("a session was launched");
    let text = written_plan_text(world, &file);
    let commands: Vec<String> = written_plan(world, &file)
        .flow
        .iter()
        .filter_map(|step| match step {
            tinman::plan::FlowStep::Tui(tui) => Some(tui.command.clone()),
            tinman::plan::FlowStep::Run(_) => None,
        })
        .collect();
    assert!(
        commands.iter().any(|command| command.contains(&program)),
        "the plan names no program {program:?}; it drives {commands:?}"
    );
    let document: serde_yaml::Value = serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("the written plan is not YAML: {e}\nit reads:\n{text}"));
    assert!(
        !yaml_carries(&document, &session),
        "the plan carries the session name {session:?}, which the plan language has no concept \
         of; it reads:\n{text}"
    );
    assert!(
        !yaml_carries(&document, "session"),
        "the plan carries a session key, which the plan language has no concept of; it \
         reads:\n{text}"
    );
}

/// Whether `wanted` appears in `document` as a key or as a whole scalar value.
///
/// Matched on whole nodes rather than as a substring of the serialized text: a
/// plan declaring `network: deny` carries the session name `work` inside the
/// key `network`, and a substring search reads that as the session name the
/// plan is being checked for.
fn yaml_carries(document: &serde_yaml::Value, wanted: &str) -> bool {
    match document {
        serde_yaml::Value::String(text) => text == wanted,
        serde_yaml::Value::Mapping(map) => map
            .iter()
            .any(|(key, nested)| key.as_str() == Some(wanted) || yaml_carries(nested, wanted)),
        serde_yaml::Value::Sequence(items) => items.iter().any(|item| yaml_carries(item, wanted)),
        _ => false,
    }
}

/// The plan declares its isolation rather than leaning on the reader's default:
/// a plan that omits the sandbox replays under whatever the defaults are on the
/// day it is replayed, which is not the run it describes. So the declaration is
/// read off the written text rather than off a parsed plan, which would fill
/// the default in and report an omission as a declaration.
///
/// What is read is that the block is written and names the isolation, which is
/// what the scenario pins: isolation is not selectable from the command line
/// today, so a step asserting which policy the block names would hold whether
/// serialization wrote the session's isolation or a hardcoded default.
#[then(expr = "{string} declares the sandbox the session ran under")]
async fn the_plan_declares_the_sandbox_the_session_ran_under(
    world: &mut TinmanWorld,
    file: String,
) {
    let text = written_plan_text(world, &file);
    let document: serde_yaml::Value = serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("the written plan is not YAML: {e}\nit reads:\n{text}"));
    let sandbox = document
        .get("sandbox")
        .unwrap_or_else(|| panic!("the plan declares no sandbox; it reads:\n{text}"));
    let declared = sandbox.as_mapping().unwrap_or_else(|| {
        panic!("the plan's sandbox is not a block of isolation settings; it reads:\n{text}")
    });
    assert!(
        !declared.is_empty(),
        "the plan's sandbox block names no isolation, so it declares nothing the reader's own \
         defaults would not have supplied; it reads:\n{text}"
    );
}

#[given(expr = "the operator has expected {string} in that session")]
async fn the_operator_has_expected_in_that_session(world: &mut TinmanWorld, text: String) {
    verb_against_session(world, &["expect", &text]).await;
    let status = world.run_status.expect("the expectation ran");
    assert_eq!(
        status,
        0,
        "the expectation {text:?} did not hold, so the session never performed it; the error \
         stream reads:\n{}",
        run_errors(world)
    );
}

/// A replay that ran no step at all exits zero exactly as one that ran every
/// step does, so the zero status the scenario already asserted cannot tell the
/// two apart. The expectation is read as contingent instead: the same plan,
/// carrying an expectation the program never draws, must fail and must report
/// that expectation. A replay that reported the written plan's steps ran while
/// executing none of them is the reframing this feature exists to catch.
#[then("the replay ran the expectation the session performed")]
async fn the_replay_ran_the_expectation(world: &mut TinmanWorld) {
    let status = world.run_status.expect("the replay ran");
    assert_eq!(
        status,
        0,
        "the replay did not pass, so it ran no expectation the session performed; the error \
         stream reads:\n{}",
        run_errors(world)
    );
    let performed = written_plan_text(world, "probe.yaml");
    let never_drawn = "NOSUCHTEXTONTHISSCREEN";
    let mutated = performed.replace(TOP_SCREEN_TEXT, never_drawn);
    assert_ne!(
        mutated, performed,
        "the written plan does not carry the expectation {TOP_SCREEN_TEXT:?} the session \
         performed, so nothing here reads whether the replay ran it; it reads:\n{performed}"
    );
    stage_working_file(world, "replay-probe.yaml", &mutated);
    run_tinman_command(world, &["test", "replay-probe.yaml"]).await;
    let status = world.run_status.expect("the contrary replay ran");
    assert_ne!(
        status, 0,
        "the same plan expecting {never_drawn:?} replayed green, so the replay does not read \
         the expectation it carries and the passing replay proved nothing"
    );
    let errors = run_errors(world);
    assert!(
        errors.contains(never_drawn),
        "the replay's failure does not report the expectation it ran, so what it executed is \
         not readable from it; the error stream reads:\n{errors}"
    );
}

// ---------------------------------------------------------------------------
// an expectation stated after a plan
// ---------------------------------------------------------------------------

/// Write `text` into the scenario's working directory as `name`. The command
/// line addresses it relatively, as an operator's own file is.
fn stage_working_file(world: &mut TinmanWorld, name: &str, text: &str) {
    let path = working_dir(world).join(name);
    std::fs::write(&path, text)
        .unwrap_or_else(|e| panic!("the file {} was not written: {e}", path.display()));
}

/// The value the operator's command line gave `flag`, read off the argument
/// list the run was made with rather than restated here, so a step asserting
/// what the run reported addresses the run the scenario actually made.
fn argv_value(world: &TinmanWorld, flag: &str) -> Option<String> {
    let argv = world.run_argv.as_ref()?;
    let at = argv.iter().position(|arg| arg == flag)?;
    argv.get(at + 1).cloned()
}

/// The plan reaches a screen only a keypress draws, so it is staged against the
/// fixture terminal program rather than a named one: the fixture's opening frame
/// carries `READY` and only the keypress replaces it, so an expectation naming
/// what the keypress draws cannot hold on a screen the plan never advanced. The
/// fixture is staged in the workspace the sandbox binds and named relatively, as
/// every other command-line step does, because a program outside that workspace
/// is unreachable from inside the sandbox.
#[given(
    expr = "a plan at {string} whose steps press {string} against the fixture terminal program"
)]
async fn a_plan_whose_steps_press_the_fixture(world: &mut TinmanWorld, file: String, key: String) {
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    stage_working_file(
        world,
        &file,
        &format!("tui: {program}\nsteps:\n  - press: \"{key}\"\n"),
    );
}

/// The expectation is stated against the same staged fixture the plan drives, so
/// the plan's steps and the expectation address one program. Staging is by name
/// and content, so naming it again here answers the same relative path the
/// `Given` wrote into the plan.
#[when(expr = "the operator expects {string} after {string} against the fixture terminal program")]
async fn the_operator_expects_after_a_plan_against_the_fixture(
    world: &mut TinmanWorld,
    text: String,
    plan: String,
) {
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    run_tinman_command(world, &["expect", &text, "--after", &plan, &program]).await;
}

#[given(expr = "a plan at {string} whose steps expect {string} against {string}")]
async fn a_plan_whose_steps_expect(
    world: &mut TinmanWorld,
    file: String,
    text: String,
    program: String,
) {
    world.staged_plan_expectation = Some(text.clone());
    stage_working_file(
        world,
        &file,
        &format!("tui: {program}\nsteps:\n  - expect: {text}\n"),
    );
}

/// The plan carries the bare locator form a probe writes, an expectation
/// stating no text and naming a region the program never draws. The plan is
/// staged rather than probed because a probe that failed writes no plan, so the
/// failing round trip has no other route to a committed plan whose locator
/// resolves to nothing. The program is the one the passing round trip replays,
/// so the role is a role the model really produces and only the name is absent:
/// a program that drew neither would report a failure the scenario could not
/// tell from a locator that matched.
#[given(expr = "a plan at {string} whose expectation names the {string} named {string}")]
async fn a_plan_whose_expectation_names(
    world: &mut TinmanWorld,
    file: String,
    role: String,
    name: String,
) {
    stage_working_file(
        world,
        &file,
        &format!("tui: top\nsteps:\n  - expect: {{role: {role}, name: {name}}}\n"),
    );
}

/// The plan's own failing step is what the operator has to fix, so the failure
/// names the expectation the plan carried rather than the one the command was
/// asked to state.
#[then("the failure reports the plan step that failed")]
async fn the_failure_reports_the_plan_step_that_failed(world: &mut TinmanWorld) {
    let expectation = world
        .staged_plan_expectation
        .clone()
        .expect("the staged plan named an expectation");
    let errors = run_errors(world);
    assert!(
        errors.contains(&expectation),
        "the failure does not report the plan step expecting {expectation:?}, so the operator is \
         not told which step broke; the error stream reads:\n{errors}"
    );
}

/// Reporting the command's own expectation would say it was read against a
/// screen the plan never reached, which is the failure being reported as held
/// wearing a different coat.
#[then("the failure does not report the expectation as read")]
async fn the_failure_does_not_report_the_expectation_as_read(world: &mut TinmanWorld) {
    let named = argv_value(world, "--name").expect("the command line named an expectation");
    let errors = run_errors(world);
    assert!(
        !errors.contains(&named),
        "the failure reports the expectation naming {named:?}, which was never read because the \
         plan failed on its way to the screen; the error stream reads:\n{errors}"
    );
}

// ---------------------------------------------------------------------------
// a session whose program has gone
// ---------------------------------------------------------------------------

/// How long a staged program is given to run to completion before the wait on
/// it is called a failure. A ceiling rather than a delay: the wait ends on the
/// holding process going, which happens as soon as the program does.
const PROGRAM_EXIT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

/// Whether any process on the system is running the program `mark`, read off
/// `/proc`, which is the kernel's own answer about what is running and which
/// sees a sandboxed process as readily as any other.
///
/// Matched against the head of the command line rather than the whole of it.
/// Every process in the chain that launched the program names it somewhere in
/// its arguments, the holding process and the sandbox launcher included, and a
/// match anywhere would read the holder standing over a program that has gone
/// as the program itself still running.
fn a_process_runs(mark: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    entries.flatten().any(|entry| {
        std::fs::read(entry.path().join("cmdline")).is_ok_and(|line| {
            String::from_utf8_lossy(&line)
                .split('\0')
                .take(2)
                .any(|argument| argument.contains(mark))
        })
    })
}

#[given(expr = "a session named {string} whose program has exited")]
async fn a_session_whose_program_has_exited(world: &mut TinmanWorld, name: String) {
    // A program that draws a screen and returns: the launch reads the screen it
    // drew, and the program is gone directly after, which is the stale session
    // the scenario starts from. A program the sandbox never drew would fail the
    // launch instead and the scenario would assert reclamation of nothing.
    //
    // Its name carries a nonce, so the wait below reads this scenario's own
    // process rather than one a concurrent worker is legally still running.
    let mark = format!(
        "{name}-exits-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is set")
            .as_nanos()
    );
    let program = stage_recorded_program(world, &mark, "#!/bin/sh\nprintf 'READY\\n'\nsleep 1\n");
    launch_session(world, &name, &program).await;
    let status = world.run_status.expect("the launch ran");
    assert_eq!(
        status,
        0,
        "the session {name:?} was not launched, so it never held a program to lose; the error \
         stream reads:\n{}",
        run_errors(world)
    );
    // Two signals, both observed, because the program's exit and the session's
    // settling into staleness are not the same moment. The holding process ends
    // because its program did, so a wait on the program alone returns while the
    // holder is still on its way out, and the launch under assertion then races
    // it. That race lives in the starting state rather than in the behaviour,
    // and a scenario that fails on it reports the wrong thing.
    let holder = scenario_temp_dir(world)
        .join(format!("tinman-session-{name}"))
        .join("holder");
    let deadline = std::time::Instant::now() + PROGRAM_EXIT_DEADLINE;
    while a_process_runs(&mark) || still_held(&holder) {
        assert!(
            std::time::Instant::now() < deadline,
            "the session {name:?} had not gone stale after {PROGRAM_EXIT_DEADLINE:?}, so the \
             scenario starts from a state it did not reach; its program is running: {}, and its \
             holder at {} is running: {}",
            a_process_runs(&mark),
            holder.display(),
            still_held(&holder)
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Whether the process recorded in the holder file at `holder` is still on the
/// system. A holder file that has gone was reclaimed, which is the same answer
/// as a holder that has ended.
fn still_held(holder: &std::path::Path) -> bool {
    let Ok(recorded) = std::fs::read_to_string(holder) else {
        return false;
    };
    let pid = recorded.trim();
    !pid.is_empty() && std::path::Path::new("/proc").join(pid).exists()
}

// ---------------------------------------------------------------------------
// layout attestation
// ---------------------------------------------------------------------------

/// The JSON Schema dialect every scantling in this project declares, so a layout
/// a scenario stages is the same kind of document as one an operator commits.
const LAYOUT_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Stage a layout scantling in the workspace the sandbox binds, named relatively
/// as every other command-line step names its files, because a path outside that
/// workspace is unreachable from inside the sandbox.
fn stage_layout_scantling(world: &mut TinmanWorld, file: &str, schema: &serde_json::Value) {
    let text = serde_json::to_string_pretty(schema).expect("the layout scantling serializes");
    stage_working_file(world, file, &text);
}

/// A layout constraining the children of the model's root region. The root is
/// the only node every model carries, so a layout reaches everything below it
/// from there, and constraining `children` rather than the root itself leaves
/// the root's own name to the program.
fn layout_over_root_children(children: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "$schema": LAYOUT_DIALECT,
        "type": "object",
        "required": ["root"],
        "properties": {
            "root": {
                "type": "object",
                "required": ["children"],
                "properties": { "children": children }
            }
        }
    })
}

/// A layout demanding that a table stand among the regions, carrying a column
/// header of each named name. `contains` is the form a demand for presence
/// takes: the model orders its children, and a layout naming what must exist
/// says nothing about where in that order it sits.
#[given(
    expr = "a layout scantling at {string} requiring a table whose column headers name {string} and {string}"
)]
async fn a_layout_requiring_a_table_with_column_headers(
    world: &mut TinmanWorld,
    file: String,
    first: String,
    second: String,
) {
    let schema = layout_requiring_table_column_headers(&first, &second);
    stage_layout_scantling(world, &file, &schema);
}

/// The layout the step above states, built apart from it so a plan's own
/// attestation step is staged against the same demand the command line is.
fn layout_requiring_table_column_headers(first: &str, second: &str) -> serde_json::Value {
    let header = |name: &str| {
        serde_json::json!({
            "type": "array",
            "contains": {
                "type": "object",
                "required": ["role", "name"],
                "properties": {
                    "role": { "const": "columnheader" },
                    "name": { "const": name }
                }
            }
        })
    };
    layout_over_root_children(serde_json::json!({
        "type": "array",
        "contains": {
            "type": "object",
            "required": ["role", "children"],
            "properties": { "role": { "const": "table" } },
            "allOf": [
                { "properties": { "children": header(first) } },
                { "properties": { "children": header(second) } }
            ]
        }
    }))
}

/// A layout demanding a region carrying `name` stand among the root's children.
#[given(expr = "a layout scantling at {string} requiring a region named {string}")]
async fn a_layout_requiring_a_region_named(world: &mut TinmanWorld, file: String, name: String) {
    let schema = layout_requiring_region_named(&name);
    stage_layout_scantling(world, &file, &schema);
}

/// The layout the step above states, built apart from it so a plan's own
/// attestation step is staged against the same demand the command line is.
fn layout_requiring_region_named(name: &str) -> serde_json::Value {
    layout_over_root_children(serde_json::json!({
        "type": "array",
        "contains": {
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "const": name } }
        }
    }))
}

/// A layout constraining where a named column header is drawn: wherever a region
/// carrying that name stands, its rectangle starts at or beyond `column`. The
/// constraint is conditional on the name rather than pinned to an ordinal, so it
/// addresses the region the operator means on a screen whose other columns may
/// come and go, and the constraint that fails is the position itself rather than
/// a demand for presence that reports only an absence.
#[given(
    expr = "a layout scantling at {string} requiring the {string} column header to start beyond column {int}"
)]
async fn a_layout_requiring_a_column_header_beyond(
    world: &mut TinmanWorld,
    file: String,
    header: String,
    column: usize,
) {
    let schema = layout_over_root_children(serde_json::json!({
        "type": "array",
        "items": {
            "if": {
                "type": "object",
                "required": ["role"],
                "properties": { "role": { "const": "table" } }
            },
            "then": {
                "required": ["children"],
                "properties": {
                    "children": {
                        "type": "array",
                        "items": {
                            "if": {
                                "type": "object",
                                "required": ["name"],
                                "properties": { "name": { "const": header } }
                            },
                            "then": {
                                "required": ["rect"],
                                "properties": {
                                    "rect": {
                                        "required": ["x"],
                                        "properties": { "x": { "minimum": column } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }));
    world.constrained_column_header = Some(header);
    stage_layout_scantling(world, &file, &schema);
}

/// Arrange the scenario's run so that launching a program leaves a durable
/// trace. Every route to a launch goes through the sandbox backend, which names
/// `bwrap` rather than locating it, so a `PATH` leading with a directory of our
/// own and a recording `bwrap` standing in it turns a launch into a file on
/// disk. The `expect` command reaches Bubblewrap only through that launch, so
/// nothing else on these runs writes the record.
///
/// The recorder refuses after writing, which is what a launch that got as far
/// as the sandbox and no further would do anyway. What the run then reports is
/// beside the point: the record is written before the refusal, so the trace
/// stands however the run ends.
fn record_any_launch(world: &mut TinmanWorld) {
    let dir = support::ScratchDir::new("launch-record");
    let record = dir.path().join("launched");
    let recorder = dir.path().join("bwrap");
    std::fs::write(
        &recorder,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\nexit 1\n",
            record.display()
        ),
    )
    .unwrap_or_else(|e| panic!("the recorder {} was not written: {e}", recorder.display()));
    support::set_executable(&recorder);
    // Led rather than replaced: the run reaches every other tool it would
    // normally reach, and only the one name the sandbox launches by is answered
    // by the recorder.
    let inherited = std::env::var("PATH").unwrap_or_default();
    world.env_vars.insert(
        "PATH".to_string(),
        format!("{}:{inherited}", dir.path().display()),
    );
    world.launch_record = Some(record);
    world.launch_recorder_dir = Some(dir);
}

/// A document that parses as JSON and is not a schema: `type` takes a simple
/// type name, and a string that names no type is a value the meta-schema
/// refuses. Real invalidity rather than a mangled file, because a file that does
/// not parse is refused by the parser and never reaches the question this asks.
///
/// The run is arranged to record a launch, because the scenario asserts the
/// program was never launched and a refusal reported after a launch carries the
/// same message as one reported before it.
#[given(expr = "a layout scantling at {string} that is valid JSON and not a valid schema")]
async fn a_layout_that_is_not_a_schema(world: &mut TinmanWorld, file: String) {
    let schema = serde_json::json!({
        "$schema": LAYOUT_DIALECT,
        "type": "quadrilateral"
    });
    record_any_launch(world);
    stage_layout_scantling(world, &file, &schema);
}

/// A schema carrying no keyword any instance can fail, which every model
/// satisfies. This is the attestation that cannot fail, and the scenario asks
/// that it be refused rather than reported as held.
///
/// The run is arranged to record a launch, for the reason the step above states.
#[given(expr = "a layout scantling at {string} that constrains nothing")]
async fn a_layout_constraining_nothing(world: &mut TinmanWorld, file: String) {
    let schema = serde_json::json!({});
    record_any_launch(world);
    stage_layout_scantling(world, &file, &schema);
}

/// The record a launch would have written, read after the run. Absence is the
/// assertion: the scantling was judged and the run ended without the sandbox
/// ever being reached, which is what puts these scenarios in a tier that needs
/// no external tool.
#[then("the program was never launched")]
async fn the_program_was_never_launched(world: &mut TinmanWorld) {
    let record = world
        .launch_record
        .clone()
        .expect("the run was arranged to record a launch");
    let launched = std::fs::read_to_string(&record).unwrap_or_default();
    assert!(
        launched.is_empty(),
        "the run reached the sandbox and launched the program before refusing the scantling, so \
         the operator waits on a real process to be told their schema is broken; the launch was \
         made with:\n{launched}"
    );
}

#[then(expr = "the failure names {string} as the region the model lacked")]
async fn the_failure_names_the_absent_region(world: &mut TinmanWorld, name: String) {
    let errors = run_errors(world);
    assert!(
        errors.contains(&name),
        "the failure does not name {name:?}, so the operator is not told which region departed; \
         the error stream reads:\n{errors}"
    );
}

/// The position a region was drawn at is a fact about the model, so it is read
/// off the model rather than restated here: a value written into the assertion
/// would hold against a report naming any position at all, including one the
/// program never drew.
#[then("the failure reports the position the region was drawn at")]
async fn the_failure_reports_the_drawn_position(world: &mut TinmanWorld) {
    let header = world
        .constrained_column_header
        .clone()
        .expect("the layout constrained a column header by position");
    let drawn = drawn_x_of_region(world, &header);
    let errors = run_errors(world);
    assert!(
        errors.contains(&drawn.to_string()),
        "the failure does not report that the {header:?} column header was drawn at column \
         {drawn}, so the operator is not told where it went; the error stream reads:\n{errors}"
    );
}

/// Where the region carrying `name` was drawn, read from the model of the same
/// program the run under assertion was made against. The program is taken off
/// the argument list that run was made with, so the model read is the model that
/// run read.
fn drawn_x_of_region(world: &mut TinmanWorld, name: &str) -> i64 {
    let program = world
        .run_argv
        .as_ref()
        .and_then(|argv| argv.last())
        .cloned()
        .expect("the run named a program");
    let dir = working_dir(world);
    let env = configured_env(world);
    let outcome = support::run_tinman(&dir, &["inspect", "--json", &program], &env, None)
        .unwrap_or_else(|e| panic!("the model of {program:?} was not read: {e}"));
    let model: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap_or_else(|e| {
        panic!(
            "the model of {program:?} is not JSON: {e}\nit reads:\n{}\nand its error stream \
             reads:\n{}",
            outcome.stdout, outcome.stderr
        )
    });
    let root = model.get("root").expect("the model carries a root region");
    region_named(root, name)
        .and_then(|region| region.get("rect")?.get("x")?.as_i64())
        .unwrap_or_else(|| panic!("the model of {program:?} carries no region named {name:?}"))
}

/// The first region carrying `name`, anywhere below `region`.
fn region_named<'a>(region: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    if region.get("name").and_then(serde_json::Value::as_str) == Some(name) {
        return Some(region);
    }
    region
        .get("children")?
        .as_array()?
        .iter()
        .find_map(|child| region_named(child, name))
}

#[then("the failure reports the scantling is not a schema")]
async fn the_failure_reports_not_a_schema(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    assert!(
        errors.contains("not a schema"),
        "the failure does not report that the scantling is not a schema, so the operator reads it \
         as a layout the model failed; the error stream reads:\n{errors}"
    );
}

#[then("the failure reports the scantling constrains nothing")]
async fn the_failure_reports_no_constraint(world: &mut TinmanWorld) {
    let errors = run_errors(world);
    assert!(
        errors.contains("constrains nothing"),
        "the failure does not report that the scantling constrains nothing, so an attestation that \
         cannot fail is reported as one that held; the error stream reads:\n{errors}"
    );
}

/// The keypress the plan sends draws `Quit?` on the fixture's status line, so
/// the layout the attestation is read against demands that region and nothing
/// the opening screen already carries. A layout satisfied before the plan ran
/// would hold whether or not its steps reached anything.
#[given(expr = "a layout scantling at {string} requiring the region the keypress draws")]
async fn a_layout_requiring_the_region_the_keypress_draws(world: &mut TinmanWorld, file: String) {
    let schema = layout_over_root_children(serde_json::json!({
        "type": "array",
        "contains": {
            "type": "object",
            "required": ["role", "text"],
            "properties": {
                "role": { "const": "status" },
                "text": { "const": "Quit?" }
            }
        }
    }));
    stage_layout_scantling(world, &file, &schema);
}

/// The attestation is stated against the same staged fixture the plan drives, so
/// the plan's steps and the layout address one program. Staging is by name and
/// content, so naming it again here answers the same relative path the `Given`
/// wrote into the plan.
#[when(expr = "the operator attests {string} after {string} against the fixture terminal program")]
async fn the_operator_attests_after_a_plan_against_the_fixture(
    world: &mut TinmanWorld,
    scantling: String,
    plan: String,
) {
    let workspace = working_dir(world);
    let program = support::stage_fixture_in(&workspace);
    run_tinman_command(
        world,
        &[
            "expect",
            "--conforms",
            &scantling,
            "--after",
            &plan,
            &program,
        ],
    )
    .await;
}

/// The steps a plan document carries, from whichever of the two roots the plan
/// scantling declares it is written in: the shorthand form names one program and
/// carries `steps` at the top level, and the full form carries them under the
/// `tui` entry of its `flow`. A reader pinned to one root reports the other as a
/// plan carrying no steps, which is a fact about the document's form rather than
/// about what it attests.
fn plan_document_steps(document: &serde_yaml::Value) -> Option<&Vec<serde_yaml::Value>> {
    if let Some(steps) = document
        .get("steps")
        .and_then(serde_yaml::Value::as_sequence)
    {
        return Some(steps);
    }
    document
        .get("flow")?
        .as_sequence()?
        .iter()
        .find_map(|step| step.get("tui")?.get("steps")?.as_sequence())
}

/// The written plan is read as the document it is rather than through the
/// production types, for the same reason validation reads it that way: the file
/// an operator commits and the plan scantling governs is this document, and
/// reading it back through production would attest whatever production
/// normalized it to.
#[then(expr = "{string} is a plan whose only step attests {string}")]
async fn the_plan_carries_only_the_attestation(
    world: &mut TinmanWorld,
    file: String,
    scantling: String,
) {
    let text = written_plan_text(world, &file);
    let document: serde_yaml::Value = serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("the written plan is not YAML: {e}\nit reads:\n{text}"));
    let steps = plan_document_steps(&document)
        .unwrap_or_else(|| panic!("the written plan carries no steps; it reads:\n{text}"));
    assert_eq!(
        steps.len(),
        1,
        "the plan carries {} steps rather than the one attestation; it reads:\n{text}",
        steps.len()
    );
    let step = steps.first().expect("the plan carries one step");
    let mapping = step
        .as_mapping()
        .unwrap_or_else(|| panic!("the plan's only step is not a step; it reads:\n{text}"));
    assert_eq!(
        mapping.len(),
        1,
        "the plan's only step names {} things rather than the one attestation; it reads:\n{text}",
        mapping.len()
    );
    assert_eq!(
        step.get("conforms").and_then(serde_yaml::Value::as_str),
        Some(scantling.as_str()),
        "the plan's only step does not attest {scantling:?}; it reads:\n{text}"
    );
}

/// The scantling a replayed plan's attestation step names. Staged in the
/// workspace the plan is read from and named relatively, because replay resolves
/// the scantling against that workspace exactly as the command line does.
const REPLAYED_LAYOUT: &str = "attested-layout.json";

/// The region a layout demands of a program that does not draw it. `top` draws a
/// process table and the regions around it; a footer is not among them, so the
/// demand reaches the model and departs from it.
const UNDRAWN_REGION: &str = "Footer";

/// Write a plan driving `program` whose one step attests the scantling at
/// `scantling`. The shorthand root is the form the plan scantling declares for a
/// plan naming one program, and it is the form the attesting command writes.
fn stage_attesting_plan(world: &mut TinmanWorld, file: &str, program: &str, scantling: &str) {
    stage_working_file(
        world,
        file,
        &format!("tui: {program}\nsteps:\n  - conforms: {scantling}\n"),
    );
}

/// A committed plan reaching attestation through replay rather than through the
/// command line. The layout demands the table the program draws, which is the
/// same demand the command-line attestation is stated against, so a replay that
/// ran the step attests a real constraint rather than a document every model
/// holds against.
#[given(expr = "a plan at {string} whose attestation names a layout {string} satisfies")]
async fn a_plan_whose_attestation_names_a_satisfied_layout(
    world: &mut TinmanWorld,
    file: String,
    program: String,
) {
    let schema = layout_requiring_table_column_headers("PID", "%CPU");
    stage_layout_scantling(world, REPLAYED_LAYOUT, &schema);
    stage_attesting_plan(world, &file, &program, REPLAYED_LAYOUT);
}

/// The same plan against a layout demanding a region the program does not draw,
/// so the attestation is reached and departs. The region is kept on the world
/// because the failure has to name what departed, and a name restated in the
/// assertion would hold against a report naming any region at all.
#[given(expr = "a plan at {string} whose attestation names a region {string} does not draw")]
async fn a_plan_whose_attestation_names_an_undrawn_region(
    world: &mut TinmanWorld,
    file: String,
    program: String,
) {
    let schema = layout_requiring_region_named(UNDRAWN_REGION);
    stage_layout_scantling(world, REPLAYED_LAYOUT, &schema);
    world.attested_absent_region = Some(UNDRAWN_REGION.to_string());
    stage_attesting_plan(world, &file, &program, REPLAYED_LAYOUT);
}

/// The region that departed is read back off the layout the `Given` staged, so
/// the assertion names what the scenario set up rather than a value written here
/// that a failure reporting any region at all would satisfy.
#[then("the failure names the region the model lacked")]
async fn the_failure_names_the_region_the_model_lacked(world: &mut TinmanWorld) {
    let region = world
        .attested_absent_region
        .clone()
        .expect("the staged layout demanded a region the program does not draw");
    let errors = run_errors(world);
    assert!(
        errors.contains(&region),
        "the failure does not name {region:?}, so the operator is not told which region departed; \
         the error stream reads:\n{errors}"
    );
}

/// A plan whose one step attests a layout, the form the plan scantling grows to
/// carry when a command able to write it exists.
const ATTESTATION_PLAN: &str = "tui: top\nsteps:\n  - conforms: top-layout.json\n";

#[given("the plan fixture carrying a step that attests a layout scantling")]
async fn the_plan_fixture_carrying_an_attestation(world: &mut TinmanWorld) {
    world.plan_sources.push(ATTESTATION_PLAN.to_string());
}
