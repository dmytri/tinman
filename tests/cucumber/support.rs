//! Verification support for the Tinman cucumber suite.
//!
//! Real-by-default: these checkers read the durable scantling policy files and
//! the real generated artifacts, and return counterexamples. They hold no
//! product behaviour and carry no planks.
//!
//! JSON policy files are parsed with `serde_yaml`: JSON is a subset of YAML, so
//! the already-rigged YAML deserializer reads them without a new dependency.

use serde::Deserialize;

/// The Bubblewrap isolation policy, `scantlings/bwrap-isolation-policy.json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BwrapPolicy {
    pub required_flags: Vec<String>,
    pub network_deny_requires_flag: String,
    pub home_setenv: HomeSetenv,
    pub workspace_mount: WorkspaceMount,
    pub forbid: Forbid,
    pub system_mount_flag: String,
}

/// How the workspace is mounted: the directory the requirement is scoped to, the
/// flag naming the real tree the program reads through, and the flag providing
/// the tmpfs its writes land in. `applies_to` names the subject, because two
/// workspace modes are legitimate and only the operator's own directory must be
/// overlaid.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMount {
    pub applies_to: String,
    pub source_flag: String,
    pub mount_flag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSetenv {
    pub flag: String,
    pub name: String,
    pub must_not_equal_operator_home: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Forbid {
    pub operator_home_as_bind_source: bool,
    pub operator_working_directory_as_bind_source: bool,
    pub host_path_inheritance: bool,
    pub writable_system_mounts: bool,
}

/// A source boundary policy: the module it governs, the references that module
/// must never carry, and the references it must carry. Both
/// `scantlings/pty-sandbox-boundary.json` and
/// `scantlings/assistant-command-boundary.json` are read into this shape; a
/// policy naming no required references constrains only the forbidden ones.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryPolicy {
    pub module: String,
    pub forbidden_references: Vec<String>,
    #[serde(default)]
    pub required_references: Vec<String>,
}

/// A construction boundary contract: the type whose construction is bounded,
/// the modules allowed to construct it, and the roots scanned for construction.
/// `scantlings/prepared-process-construction-boundary.json` is read into this
/// shape. The contract bounds a whole tree rather than one module, so a module
/// added later is guarded by a contract nobody edited.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructionBoundary {
    pub constructed_type: String,
    pub permitted_modules: Vec<String>,
    pub search_paths: Vec<String>,
}

/// A seam-set reference contract: the sources it names and the references none
/// of them may carry. `scantlings/panic-free-seams.json` is read into this
/// shape. The contract carries its own search paths, so the set it guards grows
/// by a line in the contract rather than by a checker somebody has to remember
/// to extend.
///
/// A contract may name its sources either way, which is the split the
/// proof-contract meta-schema's reference-boundary branch already carries:
/// `searchPaths` guards a tree, and `module` guards one file. A contract may
/// also require references as well as forbid them, because forbidding the
/// encoder a module must not use says nothing about whether it uses the one it
/// must, and a module that reaches for neither would satisfy a
/// forbidden-only contract while encoding nothing at all.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeamReferenceContract {
    #[serde(default)]
    pub search_paths: Vec<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub forbidden_references: Vec<String>,
    #[serde(default)]
    pub required_references: Vec<String>,
}

/// A deserialization strictness contract: the derive attribute every bound type
/// must carry, the paths whose deserializable types the requirement binds, and
/// the types it does not bind.
/// `scantlings/plan-deserialization-strictness.json` is read into this shape.
/// The contract bounds paths rather than listing types, so a type a later
/// feature adds is guarded by a contract nobody edited.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeserializationStrictness {
    pub required_derive_attribute: String,
    pub search_paths: Vec<String>,
    #[serde(default)]
    pub exempt_types: Vec<String>,
}

/// The wall-clock ceiling one timed seam must be observed to meet.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyBudget {
    pub seam: String,
    pub ceiling_ms: u64,
    pub tier: String,
}

/// The latency budget contract: every timed seam and its ceiling.
/// `scantlings/latency-budgets.json` is read into this shape.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyBudgets {
    pub budgets: Vec<LatencyBudget>,
}

fn read_policy<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("policy file {path} unreadable: {e}"));
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("policy file {path} did not parse: {e}"))
}

/// Check a generated Bubblewrap argument vector against the isolation policy.
/// `network_denied` states whether the sandbox specification denies network
/// access. `operator_home` is the operator's real HOME. Returns the list of
/// counterexamples; an empty list means the argv satisfies the policy.
pub fn check_bwrap_policy(
    policy_path: &str,
    argv: &[String],
    network_denied: bool,
    operator_home: &str,
    operator_working_directory: &str,
) -> Vec<String> {
    let policy: BwrapPolicy = read_policy(policy_path);
    let mut bad = Vec::new();

    for flag in &policy.required_flags {
        if !argv.iter().any(|a| a == flag) {
            bad.push(format!("required flag {flag} missing"));
        }
    }

    if network_denied && !argv.iter().any(|a| a == &policy.network_deny_requires_flag) {
        bad.push(format!(
            "network denied but {} absent",
            policy.network_deny_requires_flag
        ));
    }

    // HOME must be set, and must not equal the operator's real home.
    let home_value = setenv_value(argv, &policy.home_setenv.flag, &policy.home_setenv.name);
    match home_value {
        None => bad.push(format!("{} for HOME not set", policy.home_setenv.flag)),
        Some(v) => {
            if policy.home_setenv.must_not_equal_operator_home && v == operator_home {
                bad.push(format!("HOME set to operator home {operator_home}"));
            }
        }
    }

    // The operator's real home must never appear as a bind source.
    if policy.forbid.operator_home_as_bind_source && argv.iter().any(|a| a == operator_home) {
        bad.push(format!("operator home {operator_home} used as a bind path"));
    }

    // The workspace is mounted through the overlay pair: the program reads the
    // real tree through the source flag, and every write lands in the tmpfs the
    // mount flag provides, which goes when the sandbox goes. The rule is scoped
    // by `appliesTo` rather than flat, because two workspace modes are
    // legitimate and only the directory `appliesTo` names must be overlaid, so
    // the requirement has a subject only where that directory is mounted at all.
    // An `appliesTo` the checker cannot resolve fails loudly: a scope nobody
    // reads is a rule nobody applies.
    let workspace = match policy.workspace_mount.applies_to.as_str() {
        "operatorWorkingDirectory" => operator_working_directory,
        other => panic!(
            "the isolation policy scopes the workspace mount to {other:?}, which this checker cannot resolve"
        ),
    };
    if argv.iter().any(|a| a == workspace) {
        let read_through_the_overlay = argv
            .windows(2)
            .any(|pair| pair[0] == policy.workspace_mount.source_flag && pair[1] == workspace);
        if !read_through_the_overlay {
            bad.push(format!(
                "workspace {workspace} is mounted without {}",
                policy.workspace_mount.source_flag
            ));
        }
        if !argv.iter().any(|a| a == &policy.workspace_mount.mount_flag) {
            bad.push(format!(
                "workspace mount flag {} missing where {workspace} is the workspace",
                policy.workspace_mount.mount_flag
            ));
        }
    }

    // The operator's working directory must never be bound: binding it is what
    // put a probed command's writes into the tree the operator was standing in.
    if policy.forbid.operator_working_directory_as_bind_source
        && argv
            .windows(2)
            .any(|pair| is_bind_flag(&pair[0]) && pair[1] == operator_working_directory)
    {
        bad.push(format!(
            "operator working directory {operator_working_directory} used as a bind source"
        ));
    }

    // System paths must be mounted read-only, never writable, and the host root
    // must never be mounted wholesale. Each bind flag is followed by its source
    // path, so read the argv as (flag, source) pairs.
    for pair in argv.windows(2) {
        let (flag, source) = (&pair[0], &pair[1]);
        if !is_bind_flag(flag) {
            continue;
        }
        if policy.forbid.host_path_inheritance && source == "/" {
            bad.push(format!(
                "host root / mounted with {flag}: host path inheritance forbidden"
            ));
        }
        if policy.forbid.writable_system_mounts
            && is_system_path(source)
            && !is_readonly_flag(flag, &policy.system_mount_flag)
        {
            bad.push(format!(
                "system path {source} mounted with {flag}: policy requires {}",
                policy.system_mount_flag
            ));
        }
    }

    bad
}

/// Whether an argument is a Bubblewrap bind flag, writable or read-only.
fn is_bind_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--bind" | "--bind-try" | "--dev-bind" | "--dev-bind-try" | "--ro-bind" | "--ro-bind-try"
    )
}

/// Whether a bind flag is the read-only system-mount flag the policy names.
fn is_readonly_flag(flag: &str, system_mount_flag: &str) -> bool {
    flag == system_mount_flag || flag == format!("{system_mount_flag}-try")
}

/// Whether a path is a system directory that must never be writably mounted.
fn is_system_path(path: &str) -> bool {
    const ROOTS: &[&str] = &["/usr", "/bin", "/sbin", "/lib", "/lib64", "/etc"];
    path == "/"
        || ROOTS
            .iter()
            .any(|r| path == *r || path.starts_with(&format!("{r}/")))
}

/// Value of a `--setenv NAME VALUE` triple in the argv, if present.
fn setenv_value(argv: &[String], flag: &str, name: &str) -> Option<String> {
    argv.windows(3)
        .find(|w| w[0] == flag && w[1] == name)
        .map(|w| w[2].clone())
}

/// Validate a serialized instance against the JSON Schema at `schema_path`.
/// Returns the list of counterexamples; an empty list means the instance
/// conforms. The schema file is read as JSON.
pub fn schema_counterexamples(schema_path: &str, instance: &serde_json::Value) -> Vec<String> {
    let text = std::fs::read_to_string(schema_path)
        .unwrap_or_else(|e| panic!("schema file {schema_path} unreadable: {e}"));
    let schema: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("schema file {schema_path} did not parse: {e}"));
    let validator = jsonschema::validator_for(&schema)
        .unwrap_or_else(|e| panic!("schema file {schema_path} is not a valid schema: {e}"));
    validator
        .iter_errors(instance)
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect()
}

/// Render the production capture view for a virtual screen to a test terminal
/// of the given size, and return the rendered buffer as text, one line per row.
/// The capture view is production; this harness drives it through a real
/// Ratatui `TestBackend` render, exactly as the scenario asks.
pub fn render_capture_view(
    screen: &tinman::screen::VirtualScreen,
    width: u16,
    height: u16,
) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal is created");
    terminal
        .draw(|frame| {
            frame.render_widget(tinman::view::CaptureView::new(screen), frame.area());
        })
        .expect("the capture view renders");

    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    let mut lines = Vec::with_capacity(area.height as usize);
    for y in 0..area.height {
        let mut line = String::with_capacity(area.width as usize);
        for x in 0..area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                line.push_str(cell.symbol());
            }
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// A namespaced scratch directory for one scenario. Every path a scenario
/// writes lives under it, so scenarios stay isolated when workers run
/// concurrently. Removed on drop; a failed removal is reported loudly.
#[derive(Debug)]
pub struct ScratchDir {
    path: std::path::PathBuf,
}

impl ScratchDir {
    /// Create a scratch directory whose name no other scenario can collide with.
    pub fn new(label: &str) -> ScratchDir {
        reclaim_stale_staging_dirs();
        let unique = format!(
            "tinman-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the clock is after the epoch")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("scratch directory {} not created: {e}", path.display()));
        ScratchDir { path }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        if self.path.exists()
            && let Err(e) = std::fs::remove_dir_all(&self.path)
        {
            panic!(
                "scratch directory {} not reclaimed: {e}",
                self.path.display()
            );
        }
    }
}

/// The fixture terminal program the suite drives: a real full-screen program,
/// not a stand-in. It draws a bordered pane and a status bar with real ANSI
/// positioning, waits for the operator, and redraws when driven, so a plan can
/// address it by role and name through the same terminal object model any other
/// program reaches.
///
/// The program carries a real menu: it holds which item the selection sits on,
/// redraws the menu bar with that item in reverse video, and opens what the
/// selection lands on. Opening `Settings` draws a `Save` button the first frame
/// does not show, so an assertion naming that button tells an opened pane from
/// an unopened one. The labelled input stays on the first frame, because the
/// failure-report scenarios in `features/replay.feature` and
/// `features/test-command.feature` assert on that text.
///
/// A driver activation arrives here as one line: the directional key, with the
/// confirming carriage return ending it. So a directional line moves the
/// selection one item and opens what it reaches, which is the gesture the
/// driver sends. A selection driven past either end stays on the end item, so a
/// program driven towards an item it cannot reach opens something the naming
/// assertion does not match rather than pretending to arrive.
const FIXTURE_TUI: &str = "\
#!/bin/sh
sel=0

menu() {
  case $sel in
    0) printf '\\033[1;1H  \\033[7mFiles\\033[0m   Settings   Quit  ' ;;
    1) printf '\\033[1;1H  Files   \\033[7mSettings\\033[0m   Quit  ' ;;
    2) printf '\\033[1;1H  Files   Settings   \\033[7mQuit\\033[0m  ' ;;
  esac
}

open_pane() {
  case $sel in
    1) printf '\\033[11;1H[Save]' ;;
    *) printf '\\033[11;1H\\033[K' ;;
  esac
}

opened() {
  case $sel in
    0) printf '\\033[24;1H\\033[KOPEN:Files' ;;
    1) printf '\\033[24;1H\\033[KOPEN:Settings' ;;
    2) printf '\\033[24;1H\\033[KOPEN:Quit' ;;
  esac
}

printf '\\033[2J'
menu
printf '\\033[3;1H\u{250c}Files\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}'
printf '\\033[4;1H\u{2502}src            \u{2502}'
printf '\\033[5;1H\u{2502}tests          \u{2502}'
printf '\\033[6;1H\u{2502}README         \u{2502}'
printf '\\033[7;1H\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}'
printf '\\033[9;1HUsername: ________'
printf '\\033[22;1HHOME:%s' \"$HOME\"
cols=`stty size | cut -d' ' -f2`
printf '\\033[23;1HWIDTH:%s' \"${cols:-80}\"
printf '\\033[24;1HREADY'
while read -r key; do
  case \"$key\" in
    *'[C')
      sel=$((sel + 1))
      if [ \"$sel\" -gt 2 ]; then sel=2; fi
      menu
      open_pane
      opened
      ;;
    *'[D')
      sel=$((sel - 1))
      if [ \"$sel\" -lt 0 ]; then sel=0; fi
      menu
      open_pane
      opened
      ;;
    q)
      printf '\\033[24;1H\\033[KQuit?'
      ;;
    *)
      printf '\\033[24;1H\\033[KSaved'
      ;;
  esac
done
";

/// A fixture terminal program that draws a different pane title on each draw,
/// so a plan recorded against it cannot replay itself. Real instability, not a
/// simulated failure: the program genuinely renames its pane between draws.
const FIXTURE_TUI_UNSTABLE: &str = "\
#!/bin/sh
printf '\\033[2J'
printf '\\033[3;1H\u{250c}Files\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}'
printf '\\033[4;1H\u{2502}src            \u{2502}'
printf '\\033[5;1H\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}'
printf '\\033[24;1HREADY'
read -r _key
printf '\\033[2J'
printf '\\033[3;1H\u{250c}Folders\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}'
printf '\\033[4;1H\u{2502}src            \u{2502}'
printf '\\033[5;1H\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}'
printf '\\033[24;1HREADY'
read -r _key
";

/// A fixture terminal program whose selection never moves: it draws the same
/// menu bar with "Files" selected and redraws on every key without ever
/// shifting the highlight, so a selection sent towards another item never
/// arrives. Real inertness, not a simulated failure.
const FIXTURE_TUI_NO_ARROWS: &str = "\
#!/bin/sh
printf '\\033[2J'
printf '\\033[1;1H  \\033[7mFiles\\033[0m   Settings   Quit  '
printf '\\033[9;1HUsername: ________'
printf '\\033[24;1HREADY'
while read -r _key; do
  printf '\\033[24;1H\\033[KREADY'
done
";

/// Stage the fixture terminal program inside `workspace` and answer the
/// relative name it is reachable by. A recorded plan replays inside a sandbox
/// binding the workspace, so a program named there is reachable at replay time,
/// as an operator's own program in their project is. A program in a temporary
/// directory the sandbox does not bind is not.
pub fn stage_fixture_in(workspace: &std::path::Path) -> String {
    let program = workspace.join("fixture-tui");
    std::fs::write(&program, FIXTURE_TUI)
        .unwrap_or_else(|e| panic!("fixture program {} not written: {e}", program.display()));
    set_executable(&program);
    "./fixture-tui".to_string()
}

/// Stage the fixture terminal program whose pane titles change between draws in
/// `workspace` and return the relative name a launch addresses it by. A launch
/// binds the workspace and nothing else, so a fixture named by an absolute path
/// under the shared fixture directory is unreachable from inside the sandbox.
pub fn stage_unstable_fixture_in(workspace: &std::path::Path) -> String {
    let program = workspace.join("fixture-tui-unstable");
    std::fs::write(&program, FIXTURE_TUI_UNSTABLE)
        .unwrap_or_else(|e| panic!("fixture program {} not written: {e}", program.display()));
    set_executable(&program);
    "./fixture-tui-unstable".to_string()
}

/// The source of the fixture terminal program that ignores directional keys.
pub fn fixture_ignoring_directional_keys_source() -> &'static str {
    FIXTURE_TUI_NO_ARROWS
}

/// The fixture terminal program's source with the menu item named `from` drawn
/// as `to` instead. Real renaming, not a simulated failure: the program draws
/// the new name everywhere it drew the old one, so a locator naming the old one
/// finds nothing on a screen that is otherwise the one the plan was captured
/// against.
pub fn fixture_terminal_source_renaming(from: &str, to: &str) -> String {
    assert!(
        FIXTURE_TUI.contains(from),
        "the fixture terminal program draws no region named {from:?}, so renaming it would change nothing"
    );
    FIXTURE_TUI.replace(from, to)
}

/// The source of a fixture terminal program drawing two buttons that carry the
/// same name, on separate rows, so a locator naming that button matches both.
/// The terminal object model reads one bracketed label per row as a button.
pub fn fixture_with_two_buttons_source(name: &str) -> String {
    format!(
        "\
#!/bin/sh
printf '\\033[2J'
printf '\\033[3;1H[{name}]'
printf '\\033[5;1H[{name}]'
printf '\\033[24;1HREADY'
while read -r _key; do
  printf '\\033[24;1H\\033[KREADY'
done
"
    )
}

/// The inner width of the bordered pane the scoped-button fixture draws.
const SCOPED_PANE_WIDTH: usize = 20;

/// The drawing a fixture terminal program makes when it shows one bracketed
/// button inside a bordered pane titled `scope` and a second, identically
/// named, outside that pane. The two buttons are the same by name and differ
/// only in where they are drawn, which is the one thing a scoped locator has to
/// tell apart. Real ambiguity, not a simulated one: a locator naming the button
/// without saying where it is matches both.
fn scoped_button_drawing(scope: &str, name: &str) -> String {
    let rule = "\u{2500}".repeat(SCOPED_PANE_WIDTH.saturating_sub(scope.chars().count()));
    let label = format!("[{name}]");
    let pad = " ".repeat(SCOPED_PANE_WIDTH.saturating_sub(label.chars().count()));
    let bottom = "\u{2500}".repeat(SCOPED_PANE_WIDTH);
    format!(
        "printf '\\033[2J'\n\
         printf '\\033[3;1H\u{250c}{scope}{rule}\u{2510}'\n\
         printf '\\033[4;1H\u{2502}{label}{pad}\u{2502}'\n\
         printf '\\033[5;1H\u{2514}{bottom}\u{2518}'\n\
         printf '\\033[8;1H{label}'\n\
         printf '\\033[24;1HREADY'\n"
    )
}

/// The source of that fixture terminal program: it draws the two buttons and
/// then answers every key by changing its bottom line, which is the answer an
/// activation waits for before the step ends.
pub fn fixture_with_scoped_button_source(scope: &str, name: &str) -> String {
    format!(
        "#!/bin/sh\n{drawing}while read -r _key; do\n  printf '\\033[24;1H\\033[KActivated'\ndone\n",
        drawing = scoped_button_drawing(scope, name)
    )
}

/// The screen that fixture draws, read by really running its drawing under a
/// shell and parsing what it wrote. The program itself is what a precondition
/// about what it shows is read from, so nothing here restates the drawing.
pub fn scoped_button_screen(scope: &str, name: &str) -> tinman::screen::VirtualScreen {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(scoped_button_drawing(scope, name))
        .output()
        .unwrap_or_else(|e| panic!("the scoped-button fixture drawing did not run: {e}"));
    terminal_screen_at(&output.stdout, 80)
}

/// Where the shared fixture programs for this run live, provisioned once.
static FIXTURE_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// The path of the fixture terminal program, provisioned once per run and
/// shared by every scenario that drives it. The program is ambient state no
/// scenario asserts on, so it is built once rather than per scenario. Leftover
/// fixture directories from earlier runs are reclaimed here, at first use,
/// because a killed run cannot be trusted to have torn down after itself.
pub fn fixture_terminal_program() -> std::path::PathBuf {
    let dir = FIXTURE_DIR.get_or_init(|| {
        reclaim_stale_fixture_dirs();
        let path = std::env::temp_dir().join(format!("tinman-fixtures-{}", std::process::id()));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("fixture directory {} not created: {e}", path.display()));
        path
    });
    let program = dir.join("fixture-tui");
    if !program.exists() {
        std::fs::write(&program, FIXTURE_TUI)
            .unwrap_or_else(|e| panic!("fixture program {} not written: {e}", program.display()));
        set_executable(&program);
    }
    program
}

/// The fixture terminal program's own source. A driver session runs its command
/// inside a sandbox binding the system directories and the session home, so a
/// program written to a temporary directory is not there to run; the shell the
/// session launches reads the program on its command line instead.
pub fn fixture_terminal_source() -> &'static str {
    FIXTURE_TUI
}

/// Remove fixture directories an earlier run left behind, so a crashed run does
/// not leak them. A directory belonging to a process still alive is left alone.
fn reclaim_stale_fixture_dirs() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(pid) = name.strip_prefix("tinman-fixtures-") else {
            continue;
        };
        if pid == std::process::id().to_string() {
            continue;
        }
        if std::path::Path::new("/proc").join(pid).exists() {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

/// Give `path` the owner execute bit, so the shell can launch it as a program.
pub fn set_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .unwrap_or_else(|e| panic!("fixture program {} unreadable: {e}", path.display()))
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).unwrap_or_else(|e| {
        panic!(
            "fixture program {} not made executable: {e}",
            path.display()
        )
    });
}

/// The inner width of the scrolling log panes the capture fixtures draw.
const LOG_PANE_WIDTH: usize = 30;

/// The shell function every scrolling log fixture waits on. It blocks for the
/// first byte, then drains whatever else arrived within a tenth of a second, so
/// one input advances the window exactly once whether the runtime scrolls with
/// a single character or with a multi-byte escape sequence. A read yielding
/// nothing is the terminal closing and a read yielding the end-of-transmission
/// character is the session ending; both end the program, so a closed session
/// reclaims rather than waits.
///
/// Output the screen must not carry is captured into a shell variable rather
/// than redirected away. The sandbox a driver session runs under mounts the
/// system directories and the session home, so it has no `/dev` and therefore
/// no `/dev/null`: a redirect to it fails, and the command it was meant to
/// quieten never runs at all. Command substitution needs no device.
const SCROLL_READER: &str = "\
scroll() {
  scroll_stty=`stty raw -echo min 1 time 0 2>&1`
  scroll_bytes=`dd bs=1 count=1 status=none | tr -d '\\004' | wc -c | tr -d ' '`
  scroll_stty=`stty min 0 time 1 2>&1`
  scroll_drained=`dd bs=64 count=1 status=none`
  [ \"$scroll_bytes\" -gt 0 ]
}
";

/// The shell commands that draw one window of a log pane: a bordered pane
/// titled `title` whose `window` inner lines carry `messages` separated by
/// blank lines, which is what the terminal object model reads as a log of
/// articles. A window holding fewer messages than it has room for leaves the
/// remaining lines blank.
fn draw_log_window(title: &str, messages: &[String], window: usize) -> String {
    let mut out = String::from("printf '\\033[2J'\n");
    let title_rule = "\u{2500}".repeat(LOG_PANE_WIDTH.saturating_sub(title.chars().count()));
    out.push_str(&format!(
        "printf '\\033[1;1H\u{250c}{title}{title_rule}\u{2510}'\n"
    ));
    for line in 0..window {
        let text = if line.is_multiple_of(2) {
            messages.get(line / 2).cloned().unwrap_or_default()
        } else {
            String::new()
        };
        let pad = " ".repeat(LOG_PANE_WIDTH.saturating_sub(text.chars().count()));
        out.push_str(&format!(
            "printf '\\033[{};1H\u{2502}{text}{pad}\u{2502}'\n",
            line + 2
        ));
    }
    let rule = "\u{2500}".repeat(LOG_PANE_WIDTH);
    out.push_str(&format!(
        "printf '\\033[{};1H\u{2514}{rule}\u{2518}'\n",
        window + 2
    ));
    out
}

/// How many messages a window of `window` inner lines shows, given each message
/// is separated from the next by one blank line.
fn messages_per_window(window: usize) -> usize {
    window.div_ceil(2)
}

/// A real full-screen program drawing a scrolling log pane, written from the
/// windows it shows in order. Each scroll advances to the next window; once the
/// last window is reached the program redraws it for every later scroll, so a
/// runtime collecting items sees nothing new and stops.
fn log_fixture_script(title: &str, windows: &[Vec<String>], window: usize) -> String {
    let mut script = format!("#!/bin/sh\n{SCROLL_READER}");
    let (last, leading) = windows.split_last().expect("the fixture shows a window");
    for messages in leading {
        script.push_str(&draw_log_window(title, messages, window));
        script.push_str("scroll || exit 0\n");
    }
    script.push_str("while :; do\n");
    script.push_str(&draw_log_window(title, last, window));
    script.push_str("scroll || exit 0\ndone\n");
    script
}

/// The windows a log of `count` messages shows, `step` messages further on at
/// each scroll position, with each window holding as many messages as the
/// window has room for. A step smaller than the window's capacity repeats the
/// tail of the previous window at the head of the next.
fn log_windows(count: usize, window: usize, step: usize) -> Vec<Vec<String>> {
    let per_window = messages_per_window(window);
    let mut windows = Vec::new();
    let mut start = 1;
    while start <= count {
        windows.push(
            (start..=count.min(start + per_window - 1))
                .map(|n| format!("message {n}"))
                .collect(),
        );
        start += step;
    }
    windows
}

/// A scrolling log fixture whose windows follow one another with no message
/// shown twice: a program holding `count` messages, showing them through a
/// pane titled `title` whose window is `window` lines.
///
/// The fixture is the program's own source rather than a path to it. A driver
/// session runs its command inside a sandbox binding the system directories and
/// the session home, and nothing else, so a program written to a temporary
/// directory is not there to run. The shell the session already launches reads
/// the program on its command line, and every tool these fixtures use lives in
/// `/bin`, which the sandbox binds.
pub fn log_fixture_program(title: &str, count: usize, window: usize) -> String {
    let per_window = messages_per_window(window);
    log_fixture_script(title, &log_windows(count, window, per_window), window)
}

/// A scrolling log fixture that shows the last `repeat` messages of each window
/// again at the head of the next, so collecting every item reaches the same
/// message at more than one scroll position.
pub fn repeating_log_fixture_program(
    title: &str,
    count: usize,
    window: usize,
    repeat: usize,
) -> String {
    let step = messages_per_window(window) - repeat;
    log_fixture_script(title, &log_windows(count, window, step), window)
}

/// A scrolling log fixture that never reaches an end: every scroll draws a
/// window of messages none of the earlier windows carried, so a runtime
/// collecting every item scrolls until its own limit stops it.
pub fn endless_log_fixture_program(title: &str, window: usize) -> String {
    let per_window = messages_per_window(window);
    let title_rule = "\u{2500}".repeat(LOG_PANE_WIDTH.saturating_sub(title.chars().count()));
    let rule = "\u{2500}".repeat(LOG_PANE_WIDTH);
    let script = format!(
        "#!/bin/sh\n{SCROLL_READER}n=0\n\
while :; do\n\
  printf '\\033[2J'\n\
  printf '\\033[1;1H\u{250c}{title}{title_rule}\u{2510}'\n\
  r=2\n\
  i=1\n\
  while [ $i -le {per_window} ]; do\n\
    printf '\\033[%d;1H\u{2502}%-{LOG_PANE_WIDTH}s\u{2502}' \"$r\" \"message $((n+i))\"\n\
    r=$((r+1))\n\
    if [ $i -lt {per_window} ]; then\n\
      printf '\\033[%d;1H\u{2502}%-{LOG_PANE_WIDTH}s\u{2502}' \"$r\" \"\"\n\
      r=$((r+1))\n\
    fi\n\
    i=$((i+1))\n\
  done\n\
  printf '\\033[%d;1H\u{2514}{rule}\u{2518}' \"$r\"\n\
  n=$((n+{per_window}))\n\
  scroll || exit 0\n\
done\n"
    );
    script
}

/// What the local provider answers a chat-completions request with.
#[derive(Debug, Clone)]
pub enum ProviderReply {
    /// A successful completion carrying this message content.
    Content(String),
    /// A rejected credential.
    Unauthorized,
    /// Nothing at all: the connection is accepted and held open, and no reply is
    /// ever written. A refused connection fails a client on its own, so only a
    /// client-side ceiling ends this one.
    Stall,
}

/// A real HTTP server on loopback that speaks the OpenAI-compatible
/// chat-completions protocol. Tinman reaches it over a real socket, sends a
/// real request and parses a real JSON reply, so the client path under test is
/// exercised end to end.
///
/// @exceptional-double: the canned reply satisfies the Verification agreement's
/// first named condition, a specific condition the real environment cannot
/// produce on demand. The default tier's scenarios name the exact expansion a
/// provider returns, an empty generation, and a rejected credential; a real
/// model produces none of those on request. The real provider is exercised for
/// real by the @inference tier, which is where normal-path coverage lives.
#[derive(Debug)]
pub struct LocalProvider {
    base_url: String,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    received: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl LocalProvider {
    /// Start a provider that answers every completion request with `content`.
    pub fn returning(content: &str) -> LocalProvider {
        LocalProvider::start(ProviderReply::Content(content.to_string()))
    }

    /// Start a provider that rejects the credential it is given.
    pub fn rejecting() -> LocalProvider {
        LocalProvider::start(ProviderReply::Unauthorized)
    }

    /// Start a provider that accepts the connection, reads the request, and
    /// never answers, holding the socket open for as long as it runs.
    pub fn stalling() -> LocalProvider {
        LocalProvider::start(ProviderReply::Stall)
    }

    fn start(reply: ProviderReply) -> LocalProvider {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::sync::atomic::Ordering;

        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("the local provider binds a loopback port");
        let addr = listener
            .local_addr()
            .expect("the local provider reports its address");
        listener
            .set_nonblocking(true)
            .expect("the local provider listener is non-blocking");

        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop = std::sync::Arc::clone(&shutdown);
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let arrived = std::sync::Arc::clone(&received);

        let handle = std::thread::spawn(move || {
            // A stalled connection is held rather than dropped: dropping the
            // socket would answer the client with an end of stream, which is a
            // fault it can see, and this provider withholds its answer instead.
            let mut held: Vec<std::net::TcpStream> = Vec::new();
            while !stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("the accepted connection is blocking");
                        let mut reader =
                            BufReader::new(stream.try_clone().expect("the connection is cloned"));
                        let mut length = 0usize;
                        let mut saw_request = false;
                        loop {
                            let mut line = String::new();
                            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                                break;
                            }
                            saw_request = true;
                            if let Some(value) =
                                line.to_ascii_lowercase().strip_prefix("content-length:")
                            {
                                length = value.trim().parse().unwrap_or(0);
                            }
                            if line == "\r\n" || line == "\n" {
                                break;
                            }
                        }
                        let mut body = vec![0u8; length];
                        let _ = reader.read_exact(&mut body);

                        // The readiness probe connects and sends nothing, so a
                        // request is what a read line reports, not an accept.
                        if saw_request {
                            arrived
                                .lock()
                                .expect("the received request log is readable")
                                .push(String::from_utf8_lossy(&body).into_owned());
                        }
                        if matches!(reply, ProviderReply::Stall) {
                            held.push(stream);
                            continue;
                        }

                        let mut stream = stream;
                        let response = match &reply {
                            ProviderReply::Unauthorized => http_response(
                                401,
                                "Unauthorized",
                                r#"{"error":{"message":"invalid credential"}}"#,
                            ),
                            ProviderReply::Content(content) => {
                                let payload = serde_json::json!({
                                    "id": "local-provider",
                                    "object": "chat.completion",
                                    "choices": [{
                                        "index": 0,
                                        "message": {"role": "assistant", "content": content},
                                        "finish_reason": "stop"
                                    }]
                                });
                                http_response(200, "OK", &payload.to_string())
                            }
                            ProviderReply::Stall => {
                                unreachable!("a stalled provider writes no response")
                            }
                        };
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        let provider = LocalProvider {
            base_url: format!("http://{addr}/v1"),
            shutdown,
            received,
            handle: Some(handle),
        };
        provider.await_ready();
        provider
    }

    /// Whether a request has reached this provider. The readiness probe sends
    /// nothing, so this reports a real request rather than a connection.
    pub fn received_request(&self) -> bool {
        !self.received_bodies().is_empty()
    }

    /// The body of each request that reached this provider, in arrival order.
    pub fn received_bodies(&self) -> Vec<String> {
        self.received
            .lock()
            .expect("the received request log is readable")
            .clone()
    }

    /// The chat messages of each request that reached this provider, one
    /// `(role, content)` list per request, read from the real body the client
    /// sent rather than from anything the client reports about itself.
    pub fn received_messages(&self) -> Vec<Vec<(String, String)>> {
        self.received_bodies()
            .iter()
            .map(|body| {
                let value: serde_json::Value = serde_json::from_str(body).unwrap_or_else(|e| {
                    panic!("the request body is not JSON: {e}\nit reads:\n{body}")
                });
                value["messages"]
                    .as_array()
                    .unwrap_or_else(|| {
                        panic!("the request body carries no messages array:\n{body}")
                    })
                    .iter()
                    .map(|message| {
                        (
                            message["role"].as_str().unwrap_or_default().to_string(),
                            message["content"].as_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    /// Poll the listening port until it observably accepts a connection, so no
    /// scenario reaches the provider before it serves.
    fn await_ready(&self) {
        let authority = self
            .base_url
            .trim_start_matches("http://")
            .trim_end_matches("/v1")
            .to_string();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if std::net::TcpStream::connect(&authority).is_ok() {
                return;
            }
            if std::time::Instant::now() >= deadline {
                panic!("the local provider at {authority} never accepted a connection");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// The OpenAI-compatible base URL to configure Tinman with.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for LocalProvider {
    fn drop(&mut self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn http_response(status: u16, reason: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn markdown_response(status: u16, reason: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/markdown\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// A tldr page for `program` carrying `examples`, in the form the tldr-pages
/// style guide fixes: a heading, a description, and one titled example per
/// command line, each written on its own line between backticks. Built rather
/// than copied from the project, so the text an assertion depends on is text
/// this project controls.
pub fn tldr_page(program: &str, examples: &[&str]) -> String {
    let mut page = format!(
        "# {program}\n\n> Documentation for {program}.\n> More information: <https://example.invalid/{program}>.\n"
    );
    for (index, example) in examples.iter().enumerate() {
        page.push_str(&format!("\n- Example {}:\n\n`{example}`\n", index + 1));
    }
    page
}

/// A real HTTP source of tldr pages on loopback, serving raw markdown under the
/// tldr project's own `<platform>/<name>.md` layout. Verification points Tinman
/// at this source rather than at the public project, so the page an assertion
/// depends on is one this project controls rather than whatever the project
/// happens to serve today. The fetch itself stays real, because faking it would
/// be the doubling the Real-by-default Article forbids.
///
/// Only the `common` platform carries pages here. A request for any other
/// platform is answered as absent, so a client resolving a platform page before
/// falling back to the common one is exercised rather than short-circuited.
#[derive(Debug)]
pub struct LocalTldrSource {
    base_url: String,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    requested: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl LocalTldrSource {
    /// Start a source carrying one page per `(program, markdown)` pair. A
    /// program named in no pair has no page here, which is the answer this
    /// source gives for a program the project does not carry.
    pub fn serving(pages: &[(String, String)]) -> LocalTldrSource {
        use std::io::{BufRead, BufReader, Write};
        use std::sync::atomic::Ordering;

        let carried: std::collections::BTreeMap<String, String> = pages.iter().cloned().collect();

        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("the local tldr source binds a loopback port");
        let addr = listener
            .local_addr()
            .expect("the local tldr source reports its address");
        listener
            .set_nonblocking(true)
            .expect("the local tldr source listener is non-blocking");

        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop = std::sync::Arc::clone(&shutdown);
        let requested = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let asked = std::sync::Arc::clone(&requested);

        let handle = std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("the accepted connection is blocking");
                        let mut reader =
                            BufReader::new(stream.try_clone().expect("the connection is cloned"));
                        let mut request_line = String::new();
                        if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
                            // The readiness probe connects and sends nothing, so
                            // it is not recorded as a request for a page.
                            continue;
                        }
                        // Drain the headers so the client sees its whole request
                        // consumed before the answer arrives.
                        loop {
                            let mut header = String::new();
                            if reader.read_line(&mut header).unwrap_or(0) == 0 {
                                break;
                            }
                            if header == "\r\n" || header == "\n" {
                                break;
                            }
                        }
                        let target = request_line
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or_default()
                            .to_string();
                        asked
                            .lock()
                            .expect("the requested path log is readable")
                            .push(target.clone());

                        let page = target
                            .strip_prefix("/common/")
                            .and_then(|rest| rest.strip_suffix(".md"))
                            .and_then(|name| carried.get(name));
                        let response = match page {
                            Some(markdown) => markdown_response(200, "OK", markdown),
                            None => markdown_response(404, "Not Found", "not found\n"),
                        };
                        let mut stream = stream;
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        let source = LocalTldrSource {
            base_url: format!("http://{addr}"),
            shutdown,
            requested,
            handle: Some(handle),
        };
        source.await_ready();
        source
    }

    /// The base URL to configure Tinman's page source with.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The request targets that reached this source, in arrival order.
    pub fn requested_paths(&self) -> Vec<String> {
        self.requested
            .lock()
            .expect("the requested path log is readable")
            .clone()
    }

    /// Poll the listening port until it observably accepts a connection, so no
    /// scenario reaches the source before it serves.
    fn await_ready(&self) {
        let authority = self.base_url.trim_start_matches("http://").to_string();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if std::net::TcpStream::connect(&authority).is_ok() {
                return;
            }
            if std::time::Instant::now() >= deadline {
                panic!("the local tldr source at {authority} never accepted a connection");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

impl Drop for LocalTldrSource {
    fn drop(&mut self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// A loopback address with nothing listening on it, for the scenario that names
/// an unreachable provider. The port is bound to learn a free one and then
/// released, so a connection to it is refused rather than hanging.
pub fn unreachable_base_url() -> String {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port is bound to learn it");
    let addr = listener
        .local_addr()
        .expect("the probe listener reports its address");
    drop(listener);
    format!("http://{addr}/v1")
}

/// The outcome of running the real `tinman` binary.
///
/// `stdout` is the text an assertion reads, with carriage returns removed.
/// `raw` is what the program actually wrote, carriage returns and control
/// sequences intact, so a screen assertion parses the same bytes a terminal
/// received rather than a text view that has already lost the column resets.
#[derive(Debug)]
pub struct RunOutcome {
    pub stdout: String,
    pub status: i32,
    pub raw: Vec<u8>,
    /// What the run wrote to stderr. A run that fails writes its diagnosis
    /// here, so a step reporting only stdout reports a failure with nothing in
    /// it.
    pub stderr: String,
}

/// Run the real `tinman` binary with the given arguments, in `dir`, with only
/// the environment named in `env`. Tinman's own `TINMAN_*` configuration is
/// cleared first, so a scenario's configuration is the whole configuration and
/// the operator's own credential can never reach the run. When `stdout_file` is
/// given, the child's stdout is redirected to that real file and read back from
/// it, so the run sees a file rather than a terminal.
pub fn run_tinman(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
    stdout_file: Option<&std::path::Path>,
) -> std::io::Result<RunOutcome> {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_tinman"));
    command.args(args).current_dir(dir);
    for key in ["TINMAN_API_KEY", "TINMAN_BASE_URL", "TINMAN_MODEL"] {
        command.env_remove(key);
    }
    for (key, value) in env {
        command.env(key, value);
    }
    match stdout_file {
        Some(path) => {
            let file = std::fs::File::create(path)?;
            // The caller's file stands for the terminal stdout would otherwise
            // be, so stderr takes a file beside it rather than the runner's own
            // stderr, where a failing run's diagnosis would be lost.
            let errors_path = path.with_extension("stderr");
            let errors = std::fs::File::create(&errors_path)?;
            let status = command
                .stdout(std::process::Stdio::from(file))
                .stderr(std::process::Stdio::from(errors))
                .status()?;
            Ok(RunOutcome {
                stdout: std::fs::read_to_string(path)?,
                status: status.code().unwrap_or(-1),
                raw: std::fs::read(path)?,
                stderr: std::fs::read_to_string(&errors_path).unwrap_or_default(),
            })
        }
        None => {
            let output = command.output()?;
            Ok(RunOutcome {
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                status: output.status.code().unwrap_or(-1),
                raw: output.stdout,
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
    }
}

/// The same run, awaited rather than blocked on.
///
/// The runner polls every scenario future on one executor thread, so a step
/// body that blocks that thread holds every scenario beside it and a worker
/// count above one buys nothing. Handing the blocking work to a blocking thread
/// and awaiting it lets the executor poll the scenarios that are waiting on
/// their own processes, which is what makes the worker count real.
///
/// The arguments arrive owned because the work outlives this call's borrow.
pub async fn run_tinman_awaited(
    dir: std::path::PathBuf,
    args: Vec<String>,
    env: Vec<(String, String)>,
    stdout_file: Option<std::path::PathBuf>,
) -> std::io::Result<RunOutcome> {
    tokio::task::spawn_blocking(move || {
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        run_tinman(&dir, &borrowed, &env, stdout_file.as_deref())
    })
    .await
    .expect("the tinman run finishes rather than being cancelled")
}

/// A running `tinman driver` process, spoken to over its real stdin and stdout
/// with one JSON message per line, exactly as a test runner in any language
/// would drive it. Every message exchanged is kept, so a scenario can attest
/// the whole conversation against the protocol scantling.
pub struct DriverProcess {
    child: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
    replies: std::sync::mpsc::Receiver<String>,
    exchanged: Vec<serde_json::Value>,
}

impl std::fmt::Debug for DriverProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriverProcess").finish_non_exhaustive()
    }
}

impl DriverProcess {
    /// Start the real driver. Its stdout is drained on a reader thread, so a
    /// reply is waited for on an observed signal rather than a blocking read
    /// that could never return.
    pub fn start() -> DriverProcess {
        DriverProcess::start_with(&[])
    }

    /// Start the real driver with `env` set in its environment, so a scenario
    /// can settle what the driver finds when it looks for the sandbox. Anything
    /// else the driver reads is inherited, as it is for an ordinary start.
    pub fn start_with(env: &[(String, String)]) -> DriverProcess {
        use std::io::{BufRead, BufReader};

        SESSIONS_RECLAIMED.call_once(reclaim_stale_session_dirs);
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_tinman"));
        command.arg("driver");
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("the tinman driver starts");
        let stdin = child.stdin.take().expect("the driver has a stdin pipe");
        let stdout = child.stdout.take().expect("the driver has a stdout pipe");
        let (sender, replies) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        DriverProcess {
            child,
            stdin: Some(stdin),
            replies,
            exchanged: Vec::new(),
        }
    }

    /// The process identifier the driver runs under. Its session sandbox
    /// directories are named after it, so this identifies what it owns.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Close the driver's stdin, as a test runner dropping its end of the pipe
    /// does. Dropping the handle is what the driver observes as end of input.
    pub fn close_stdin(&mut self) {
        self.stdin = None;
    }

    /// Wait for the driver to exit and report the status it left, retrying in
    /// short intervals toward a deadline because a process exit is observable
    /// only by asking. A driver that never exits fails here with a budget rather
    /// than hanging the run.
    pub fn wait_for_exit(&mut self) -> std::process::ExitStatus {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match self
                .child
                .try_wait()
                .expect("the driver's exit status is readable")
            {
                Some(status) => return status,
                None if std::time::Instant::now() >= deadline => {
                    panic!("the driver was still running 30s after its stdin closed")
                }
                None => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    }

    /// Send one request line and read the driver's reply, bounded by a budget
    /// generous enough for a real sandboxed launch.
    pub fn send_line(&mut self, line: &str) -> serde_json::Value {
        use std::io::Write;

        let request: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("the request line is not JSON: {e}\n{line}"));
        self.exchanged.push(request);
        let stdin = self
            .stdin
            .as_mut()
            .expect("the driver's stdin is still open");
        writeln!(stdin, "{line}").expect("the request reaches the driver");
        stdin.flush().expect("the request is flushed");
        let reply = self
            .replies
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap_or_else(|e| panic!("the driver sent no reply to {line}: {e}"));
        let value: serde_json::Value = serde_json::from_str(&reply)
            .unwrap_or_else(|e| panic!("the driver reply is not JSON: {e}\n{reply}"));
        self.exchanged.push(value.clone());
        value
    }

    /// Send one request and read its reply.
    pub fn request(&mut self, request: serde_json::Value) -> serde_json::Value {
        self.send_line(&request.to_string())
    }

    /// Every message exchanged with the driver, requests and replies alike, in
    /// the order they crossed the pipe.
    pub fn exchanged(&self) -> &[serde_json::Value] {
        &self.exchanged
    }
}

/// The budget a child gets to run its own exit path after being asked to stop.
const TERMINATION_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// Send `signal` to the process `pid` through the system's own `kill`, the route
/// `interrupt` already takes, which needs no signalling dependency to reach.
///
/// Whatever the kill writes is captured and returned rather than inherited. The
/// shell running the signal would otherwise write to the runner's own error
/// stream, and a stream carrying routine noise is one nobody reads.
fn signal_process(pid: u32, signal: &str) -> String {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("kill -{signal} {pid}"))
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stderr).into_owned(),
        Err(e) => format!("the {signal} signal to {pid} could not be sent: {e}"),
    }
}

/// What tearing a session down observed.
#[derive(Debug, Clone)]
pub struct TeardownReport {
    /// Whether the process had already gone when teardown reached it. Escalation
    /// is owed when a termination deadline passes, and only then: signalling a
    /// process that has already gone produces a diagnostic and stops nothing.
    pub already_gone: bool,
    /// Whatever the system's own kill wrote while teardown signalled, captured
    /// rather than inherited.
    pub kill_diagnostics: String,
}

/// Wait up to `budget` for a process to exit, asking in short intervals because
/// an exit is observable only by asking. Reports whether it exited in time.
fn exited_within(budget: std::time::Duration, mut exited: impl FnMut() -> bool) -> bool {
    let deadline = std::time::Instant::now() + budget;
    loop {
        if exited() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

impl DriverProcess {
    /// Tear the session down and report what teardown observed.
    ///
    /// Closing stdin is this protocol's own end of input, so the driver runs its
    /// exit path and an instrumented build flushes the coverage counters that
    /// say what it executed. A process stopped outright flushes nothing, and the
    /// seam then reads as unattributed rather than unreached. The unconditional
    /// stop stays as the fallback the deadline reaches.
    ///
    /// A process that has already gone is reported rather than signalled.
    pub fn tear_down(&mut self) -> TeardownReport {
        self.stdin.take();
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return TeardownReport {
                already_gone: true,
                kill_diagnostics: String::new(),
            };
        }
        let exited = exited_within(TERMINATION_BUDGET, || {
            matches!(self.child.try_wait(), Ok(Some(_)))
        });
        let mut kill_diagnostics = String::new();
        if !exited {
            kill_diagnostics = signal_process(self.child.id(), "KILL");
        }
        let _ = self.child.wait();
        TeardownReport {
            already_gone: false,
            kill_diagnostics,
        }
    }
}

impl Drop for DriverProcess {
    fn drop(&mut self) {
        self.tear_down();
    }
}

/// Whether this run has already reclaimed the session homes earlier runs left.
static SESSIONS_RECLAIMED: std::sync::Once = std::sync::Once::new();

/// Remove the sandbox home directories driver sessions of earlier runs left
/// behind. A scenario that never closes its session, and a run that is killed,
/// both leave one standing, so the reclaim at first driver start is the net
/// that keeps them from accumulating. A directory whose driver is still alive
/// is left alone.
fn reclaim_stale_session_dirs() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(rest) = name.strip_prefix("tinman-sess-") else {
            continue;
        };
        let Some((pid, _)) = rest.split_once('-') else {
            continue;
        };
        if std::path::Path::new("/proc").join(pid).exists() {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

/// The temporary sandbox directories a driver session owns, found by the
/// session identifier their names carry.
pub fn session_sandbox_dirs(session: &str) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains(session))
        })
        .collect()
}

/// The temporary sandbox directories a driver still owns, found by the process
/// identifier their names carry. Scoped to one driver, so a concurrent
/// scenario's own session directories are none of this driver's business.
pub fn standing_session_dirs(driver_pid: u32) -> Vec<std::path::PathBuf> {
    let prefix = format!("tinman-sess-{driver_pid}-");
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
        })
        .collect()
}

/// What a sweep of the system found standing: the staging directories and
/// sandbox processes it searched, and the ones that outlived the run that made
/// them.
#[derive(Debug, Default)]
pub struct SandboxInventory {
    /// The sources the sweep read, named as it read them.
    pub searched: Vec<String>,
    /// The temporary-directory name prefixes the sweep recognizes, named as it
    /// declares them rather than derived from the implementation, so a prefix
    /// the implementation gains and this list does not is a difference the
    /// scenario can see.
    pub searched_prefixes: Vec<String>,
    pub searched_dirs: Vec<String>,
    pub searched_processes: Vec<String>,
    pub orphan_dirs: Vec<String>,
    pub orphan_processes: Vec<String>,
}

/// The temporary-directory name prefixes this sweep searches.
///
/// The implementation decides which prefixes exist, and the scenario joins this
/// declared list against the prefixes read from the implementation tree, so a
/// prefix added there and not added here reddens rather than accumulating
/// unswept. Longer prefixes come first: the bare `tinman-` prefix is a prefix of
/// the others, so it stands last as the catch-all.
pub const SWEPT_TEMP_PREFIXES: &[&str] = &["tinman-copy-mount-", "tinman-terminfo-", "tinman-"];

static STAGING_RECLAIMED: std::sync::Once = std::sync::Once::new();

/// Reclaim every Tinman staging directory whose owning run has ended.
///
/// This is the suite's primary safety net, so it runs at suite start rather than
/// only where a driver happens to be started: a crashed or killed run cannot be
/// trusted to have torn down after itself, and what it left is exactly what no
/// later teardown will reach. Every scenario that stages anything passes through
/// here, so the sweep has run before any scenario reads the system.
pub fn reclaim_stale_staging_dirs() {
    STAGING_RECLAIMED.call_once(|| {
        let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(owner) = staging_dir_owner(name) else {
                continue;
            };
            if process_is_live(owner) {
                continue;
            }
            let _ = std::fs::remove_dir_all(entry.path());
        }
    });
}

/// Whether a process identifier names a live process.
fn process_is_live(pid: u32) -> bool {
    std::path::Path::new("/proc").join(pid.to_string()).exists()
}

/// The process identifier a Tinman temporary directory's name carries. Tinman
/// names one `<prefix><pid>-<tail>`, so the run that made it is named in the
/// directory itself rather than in a record a killed run never wrote. A kind
/// carries dashes of its own, such as the copy-mount staging prefix, so the
/// owner is the first component after the prefix that reads as an identifier
/// rather than a component counted from the front.
fn staging_dir_owner(name: &str) -> Option<u32> {
    let prefix = SWEPT_TEMP_PREFIXES
        .iter()
        .find(|prefix| name.starts_with(**prefix))?;
    name[prefix.len()..]
        .split('-')
        .find_map(|part| part.parse::<u32>().ok())
}

// ---------------------------------------------------------------------------
// shared corpora
// ---------------------------------------------------------------------------
//
// The specs, the scantlings, the implementation sources and the shipped example
// plans are ambient state: no scenario asserts that a directory can be listed,
// and a scenario reading one re-reads and re-parses the whole tree to reach the
// one fact it came for. That cost is paid by every scenario in the tier, so it
// is paid on every inner-loop run. Each corpus is read once per run here and
// every step reads the parsed result. Sharing stays safe because these corpora
// are read-only: nothing mutates them, so no scenario can observe another's use
// of them.

/// The implementation sources, as `(path, text)` pairs ordered by path.
static IMPLEMENTATION_SOURCES: std::sync::LazyLock<Vec<(String, String)>> =
    std::sync::LazyLock::new(|| {
        let mut found = Vec::new();
        for entry in std::fs::read_dir("src").expect("the implementation directory is readable") {
            let path = entry.expect("an implementation directory entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let display = path.display().to_string();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("implementation source {display} unreadable: {e}"));
            found.push((display, text));
        }
        found.sort();
        found
    });

/// Every scantling in the scantlings directory, as `(path, document)` pairs
/// ordered by path.
static SCANTLING_DOCUMENTS: std::sync::LazyLock<Vec<(String, serde_json::Value)>> =
    std::sync::LazyLock::new(|| {
        let mut found = Vec::new();
        for entry in std::fs::read_dir("scantlings").expect("the scantlings directory is readable")
        {
            let path = entry.expect("a scantlings directory entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let display = path.display().to_string();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("scantling {display} unreadable: {e}"));
            let document: serde_json::Value = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("scantling {display} did not parse: {e}"));
            found.push((display, document));
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    });

/// Every spec path under the specs directory, ordered by path.
static SPEC_PATHS: std::sync::LazyLock<Vec<std::path::PathBuf>> = std::sync::LazyLock::new(|| {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir("features")
        .expect("the specs directory is readable")
        .map(|entry| entry.expect("a specs directory entry").path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("feature"))
        .collect();
    paths.sort();
    paths
});

/// The shipped example plans, as `(path, text)` pairs ordered by path.
static EXAMPLE_PLANS: std::sync::LazyLock<Vec<(String, String)>> = std::sync::LazyLock::new(|| {
    let mut found = Vec::new();
    for entry in
        std::fs::read_dir("assets/examples").expect("the example plan directory is readable")
    {
        let path = entry.expect("an example plan directory entry").path();
        let display = path.display().to_string();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("example plan {display} unreadable: {e}"));
        found.push((display, text));
    }
    found.sort();
    found
});

/// The verification support sources, as `(path, text)` pairs ordered by path.
/// These are read from named paths rather than from a listing, because the
/// support tier is two files the runner itself names.
static SUPPORT_SOURCES: std::sync::LazyLock<Vec<(String, String)>> =
    std::sync::LazyLock::new(|| {
        ["tests/cucumber.rs", "tests/cucumber/support.rs"]
            .into_iter()
            .map(|path| {
                let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
                    panic!("verification support source {path} unreadable: {e}")
                });
                (path.to_string(), text)
            })
            .collect()
    });

/// The temporary-directory name prefixes the implementation creates, read from
/// the implementation tree. Each creation site joins a `format!` literal onto
/// the system temporary directory, so the prefix is that literal up to its first
/// interpolation.
pub fn implementation_temp_prefixes() -> Vec<String> {
    let mut found = std::collections::BTreeSet::new();
    for (_, text) in IMPLEMENTATION_SOURCES.iter() {
        for (index, _) in text.match_indices("temp_dir()") {
            // The creation site is the join that follows, so the literal is read
            // from a bounded tail rather than from the rest of the file: a
            // `temp_dir()` used for anything else finds no literal of its own
            // instead of borrowing the next one in the file.
            let mut end = (index + 200).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let tail = &text[index..end];
            let Some(open) = tail.find('"') else { continue };
            let after = &tail[open + 1..];
            let Some(close) = after.find('"') else {
                continue;
            };
            let literal = &after[..close];
            let prefix = literal.split('{').next().unwrap_or(literal);
            if prefix.starts_with("tinman-") {
                found.insert(prefix.to_string());
            }
        }
    }
    found.into_iter().collect()
}

/// The parent of `pid`, read from the process table. `None` when the process is
/// gone or its record cannot be read.
fn parent_of(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command field is parenthesised and may itself hold spaces, so the
    // fields after it are counted from the closing parenthesis.
    let after = stat.rsplit_once(") ")?.1;
    after.split_whitespace().nth(1)?.parse().ok()
}

/// One sweep of the temporary directory and the process table.
fn sweep_sandbox_resources() -> SandboxInventory {
    let mut found = SandboxInventory {
        searched_prefixes: SWEPT_TEMP_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect(),
        ..SandboxInventory::default()
    };
    let temp = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&temp) {
        found.searched.push(temp.display().to_string());
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(owner) = staging_dir_owner(name) else {
                continue;
            };
            let path = entry.path().display().to_string();
            found.searched_dirs.push(path.clone());
            if !process_is_live(owner) {
                found.orphan_dirs.push(path);
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir("/proc") {
        found.searched.push("/proc".to_string());
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid) = name.to_str().and_then(|name| name.parse::<u32>().ok()) else {
                continue;
            };
            let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
                continue;
            };
            if comm.trim() != "bwrap" {
                continue;
            }
            let record = format!("{} (pid {pid})", comm.trim());
            found.searched_processes.push(record.clone());
            // A sandbox whose launcher is gone has been reparented to init, so
            // the process table itself says the run that made it has ended.
            if !parent_of(pid).is_some_and(process_is_live) {
                found.orphan_processes.push(record);
            }
        }
    }
    found
}

/// The sandbox resources standing after the suite's own reclaim, read from the
/// system rather than from a record the suite kept: a resource a killed run
/// leaked is exactly the one no record mentions.
///
/// The suite runs many workers, so a run mid-exit can be caught with its sandbox
/// briefly reparented and its directory briefly unclaimed. An orphan is
/// therefore one that survives a second sweep: a leaked resource persists, and a
/// resource still being torn down is gone or claimed by the time the second
/// sweep reads it.
pub fn standing_sandbox_resources() -> SandboxInventory {
    // The scenario reads the system the suite's reclaim has already swept, so a
    // focused run of it alone sweeps first rather than reading a system no
    // scenario in this run had reason to clear.
    reclaim_stale_staging_dirs();
    let first = sweep_sandbox_resources();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let second = sweep_sandbox_resources();
    SandboxInventory {
        orphan_dirs: second
            .orphan_dirs
            .iter()
            .filter(|path| first.orphan_dirs.contains(path))
            .cloned()
            .collect(),
        orphan_processes: second
            .orphan_processes
            .iter()
            .filter(|record| first.orphan_processes.contains(record))
            .cloned()
            .collect(),
        searched: second.searched,
        searched_prefixes: second.searched_prefixes,
        searched_dirs: second.searched_dirs,
        searched_processes: second.searched_processes,
    }
}

/// The inner width of the bordered panes these fixtures draw.
const PANE_WIDTH: usize = 24;

/// Draw a bordered pane carrying `title`, listing `items`, as a real terminal
/// would receive it: box-drawing characters and, for the reversed item, the
/// real SGR reverse-video sequence. The result is parsed by the production
/// virtual screen, so the fixture reaches the model through the same terminal
/// emulation a captured program does.
pub fn bordered_pane_screen(
    title: &str,
    items: &[String],
    reversed: Option<&str>,
) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&bordered_pane_text(title, items, reversed))
}

/// Draw the same bordered pane and leave the terminal's cursor at the 1-based
/// `row` and `col`, addressed with a real ANSI positioning sequence, so the
/// bytes carry where the cursor rests as well as what was drawn. A pane with the
/// cursor parked in it and one with the cursor left elsewhere hold the same
/// characters, and only the cursor tells a field being typed into from a panel
/// being displayed.
pub fn bordered_pane_screen_with_cursor_at(
    title: &str,
    items: &[String],
    row: u16,
    col: u16,
) -> tinman::screen::VirtualScreen {
    let mut out = bordered_pane_text(title, items, None);
    out.push_str(&format!("\x1b[{row};{col}H"));
    tinman::screen::VirtualScreen::from_text(&out)
}

/// Draw a bordered pane carrying `title` in its top border and `hint` in its
/// bottom border, the way a program draws a key hint into the lower rule of the
/// box it belongs to. The pane is widened to whichever of the title, the hint,
/// and the standard width is longest, so the hint reaches the screen whole
/// rather than clipped by the fixture.
pub fn bordered_pane_screen_with_bottom_border(
    title: &str,
    items: &[String],
    hint: &str,
) -> tinman::screen::VirtualScreen {
    let width = PANE_WIDTH
        .max(title.chars().count())
        .max(hint.chars().count());
    let mut out = String::new();
    out.push('\u{250c}');
    out.push_str(title);
    out.push_str(&"\u{2500}".repeat(width - title.chars().count()));
    out.push('\u{2510}');
    out.push_str("\r\n");
    for item in items {
        let pad = " ".repeat(width.saturating_sub(item.chars().count()));
        out.push('\u{2502}');
        out.push_str(item);
        out.push_str(&pad);
        out.push('\u{2502}');
        out.push_str("\r\n");
    }
    out.push('\u{2514}');
    out.push_str(hint);
    out.push_str(&"\u{2500}".repeat(width - hint.chars().count()));
    out.push('\u{2518}');
    out.push_str("\r\n");
    tinman::screen::VirtualScreen::from_text(&out)
}

/// The bytes a bordered pane carrying `title` and listing `items` is drawn with.
fn bordered_pane_text(title: &str, items: &[String], reversed: Option<&str>) -> String {
    let mut out = String::new();
    let title_width = title.chars().count();
    out.push('\u{250c}');
    out.push_str(title);
    out.push_str(&"\u{2500}".repeat(PANE_WIDTH.saturating_sub(title_width)));
    out.push('\u{2510}');
    out.push_str("\r\n");
    for item in items {
        let pad = " ".repeat(PANE_WIDTH.saturating_sub(item.chars().count()));
        out.push('\u{2502}');
        if reversed == Some(item.as_str()) {
            out.push_str("\x1b[7m");
            out.push_str(item);
            out.push_str(&pad);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(item);
            out.push_str(&pad);
        }
        out.push('\u{2502}');
        out.push_str("\r\n");
    }
    out.push('\u{2514}');
    out.push_str(&"\u{2500}".repeat(PANE_WIDTH));
    out.push('\u{2518}');
    out.push_str("\r\n");
    out
}

/// Draw a screen showing two bordered panes side by side, titled `Left` and
/// `Right`, each listing one line carrying `item`, so one name matches in two
/// regions and a locator naming it alone is ambiguous.
pub fn two_bordered_panes_screen(item: &str) -> tinman::screen::VirtualScreen {
    let width = 18;
    let pad = " ".repeat(width - item.chars().count());
    let rule = "\u{2500}".repeat(width);
    let mut out = String::new();
    for (row, (title, gap)) in [("Left", 1), ("Right", 24)].iter().enumerate() {
        let _ = row;
        let title_rule = "\u{2500}".repeat(width - title.chars().count());
        out.push_str(&format!("\x1b[1;{gap}H\u{250c}{title}{title_rule}\u{2510}"));
        out.push_str(&format!("\x1b[2;{gap}H\u{2502}{item}{pad}\u{2502}"));
        out.push_str(&format!("\x1b[3;{gap}H\u{2514}{rule}\u{2518}"));
    }
    tinman::screen::VirtualScreen::from_text(&out)
}

/// Draw a pane carrying no border, whose first line reads `first_line` and whose
/// remaining lines are entries beneath it. The deterministic pass has no title to
/// read, so the region it builds carries no name and only inference can supply
/// one.
pub fn unbordered_pane_screen(first_line: &str) -> tinman::screen::VirtualScreen {
    let mut out = String::new();
    for line in [first_line, "report.txt", "notes.txt"] {
        out.push_str(line);
        out.push_str("\r\n");
    }
    tinman::screen::VirtualScreen::from_text(&out)
}

/// Draw a screen split by a vertical rule at `column`, zero-based, so the model
/// reads two sibling regions either side of it.
pub fn vertical_split_screen(cols: u16, column: u16) -> tinman::screen::VirtualScreen {
    let mut out = String::new();
    for _ in 0..24 {
        out.push_str(&" ".repeat(column as usize));
        out.push('\u{2502}');
        out.push_str(&" ".repeat((cols - column - 1) as usize));
        out.push_str("\r\n");
    }
    tinman::screen::VirtualScreen::from_text(&out)
}

/// Draw a screen whose bottom row carries `text`, addressed with real ANSI
/// cursor positioning so the text lands on the last row of the grid.
pub fn bottom_line_screen(text: &str) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[24;1H{text}"))
}

/// Draw a screen whose bottom row carries `text` in the named foreground
/// colour, set with a real SGR sequence and cleared after, so the grid carries
/// the attribute the program set rather than a flag the fixture invented.
pub fn bottom_line_screen_in_red(text: &str) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[24;1H\x1b[31m{text}\x1b[0m"))
}

/// Draw the bordered pane with `line` rendered in red and every other line drawn
/// plainly, so the pane's own cells disagree on colour while its neighbours
/// agree.
pub fn bordered_pane_screen_with_line_in_red(
    title: &str,
    items: &[String],
    line: &str,
) -> tinman::screen::VirtualScreen {
    let mut out = String::new();
    let title_width = title.chars().count();
    out.push('\u{250c}');
    out.push_str(title);
    out.push_str(&"\u{2500}".repeat(PANE_WIDTH.saturating_sub(title_width)));
    out.push('\u{2510}');
    out.push_str("\r\n");
    for item in items {
        let pad = " ".repeat(PANE_WIDTH.saturating_sub(item.chars().count()));
        out.push('\u{2502}');
        if item == line {
            out.push_str("\x1b[31m");
            out.push_str(item);
            out.push_str(&pad);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(item);
            out.push_str(&pad);
        }
        out.push('\u{2502}');
        out.push_str("\r\n");
    }
    out.push('\u{2514}');
    out.push_str(&"\u{2500}".repeat(PANE_WIDTH));
    out.push('\u{2518}');
    out.push_str("\r\n");
    tinman::screen::VirtualScreen::from_text(&out)
}

/// Draw `text` at the 1-based `row` and `col` and leave the cursor at the
/// 0-based `cursor_row` and `cursor_column`, addressed with a real ANSI
/// positioning sequence.
pub fn text_at_screen_with_cursor_at(
    text: &str,
    row: u16,
    col: u16,
    cursor_row: u16,
    cursor_column: u16,
) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!(
        "\x1b[{row};{col}H{text}\x1b[{};{}H",
        cursor_row + 1,
        cursor_column + 1
    ))
}

/// Draw `text` at the 1-based `row` and `col` with the cursor hidden, using the
/// real DECTCEM sequence a program hides it with.
pub fn text_at_screen_with_cursor_hidden(
    text: &str,
    row: u16,
    col: u16,
) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[{row};{col}H{text}\x1b[?25l"))
}

/// Draw a screen whose top row carries `text`.
pub fn top_line_screen(text: &str) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[1;1H{text}"))
}

/// Draw a screen carrying `text` at the 1-based `row` and `col`, addressed with
/// real ANSI cursor positioning so the text lands where the scenario places it
/// and the rest of the grid stays blank.
pub fn text_at_screen(text: &str, row: u16, col: u16) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[{row};{col}H{text}"))
}

/// Run the real `tinman` binary to completion on a real pseudo-terminal, so the
/// run sees a terminal rather than a pipe, with `dir` as its working directory
/// and only the `TINMAN_*` configuration named in `env`. Everything the program
/// wrote is returned together with its exit status. The terminal is opened large
/// enough to hold the whole help output, so no line the scenario asserts on
/// scrolls away before the program exits.
///
/// The operator's run ends where this call does, so the input is ended straight
/// away: a program that waits for a question sees the input end and completes,
/// and the wait for it is bounded rather than a read that could never return.
pub fn run_tinman_on_a_terminal(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
) -> Result<RunOutcome, String> {
    run_tinman_on_a_terminal_at(dir, args, env, TERMINAL_COLS)
}

/// Run the real `tinman` binary to completion on a real pseudo-terminal
/// `columns` cells wide, so a scenario reads what the program drew at the width
/// it was given. Everything else matches `run_tinman_on_a_terminal`.
pub fn run_tinman_on_a_terminal_at(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
    columns: u16,
) -> Result<RunOutcome, String> {
    let mut session = TerminalSession::start_at(dir, args, env, columns)?;
    session.end_input();
    Ok(session.finish(std::time::Duration::from_secs(30)))
}

/// Run the real `tinman` binary to completion on a real pseudo-terminal that
/// already carries `line` on it, so the run meets a terminal an operator has
/// been working on rather than a blank one. The line is printed by the shell
/// that then becomes the binary, so it reaches the terminal exactly as the
/// operator's own earlier work did, on the normal screen and in the scrollback.
pub fn run_tinman_on_a_terminal_after_printing(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
    line: &str,
) -> Result<RunOutcome, String> {
    let mut shell_line = format!(
        "printf '%s\\n' {}; exec {}",
        shell_quote(line),
        shell_quote(env!("CARGO_BIN_EXE_tinman"))
    );
    for arg in args {
        shell_line.push(' ');
        shell_line.push_str(&shell_quote(arg));
    }
    let mut session = TerminalSession::start_shell_line(dir, &shell_line, env, TERMINAL_COLS)?;
    session.end_input();
    Ok(session.finish(std::time::Duration::from_secs(30)))
}

/// The bytes a run wrote while the terminal stood on the alternate screen, or
/// `None` where the run never put it there.
///
/// A terminal switches screens on the DEC private mode 1049 sequences, so the
/// span begins at the last switch onto the alternate screen and ends at the
/// switch back, or at the end of the stream where the run never switched back.
/// Reading the span rather than the whole stream is what separates what the
/// program drew for itself from what the operator already had on their
/// terminal: the two are the same bytes in one stream and two different screens
/// on the terminal.
pub fn alternate_screen_span(raw: &[u8]) -> Option<Vec<u8>> {
    let enter = b"\x1b[?1049h";
    let leave = b"\x1b[?1049l";
    let at = raw
        .windows(enter.len())
        .rposition(|window| window == enter)?;
    let body = &raw[at + enter.len()..];
    let end = body
        .windows(leave.len())
        .position(|window| window == leave)
        .unwrap_or(body.len());
    Some(body[..end].to_vec())
}

/// Run the real `tinman` binary to completion on a real pseudo-terminal with
/// its standard input redirected from `stdin_path`. The program's output is
/// still a terminal, so this is the shape a shell redirection gives it: a file
/// handed to a program nobody is standing in front of. The redirection is done
/// by the shell that runs the binary, which is the same mechanism an operator's
/// own `tinman < plan.yaml` uses, so no separate plumbing stands in for it.
pub fn run_tinman_on_a_terminal_with_stdin_from(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
    stdin_path: &std::path::Path,
) -> Result<RunOutcome, String> {
    let mut line = format!("exec {}", shell_quote(env!("CARGO_BIN_EXE_tinman")));
    for arg in args {
        line.push(' ');
        line.push_str(&shell_quote(arg));
    }
    line.push_str(" < ");
    line.push_str(&shell_quote(&stdin_path.display().to_string()));
    let mut session = TerminalSession::start_shell_line(dir, &line, env, TERMINAL_COLS)?;
    Ok(session.finish(std::time::Duration::from_secs(30)))
}

/// How often a terminal session asks whether the program has exited. A run's
/// measured duration is therefore accurate only to this interval, so a step
/// timing a run reads its instrument's resolution here rather than assuming the
/// measurement is exact.
pub const EXIT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);

/// What the shell prints once the program has exited, ahead of the terminal's
/// own report of the state it was left in.
pub const TERMINAL_STATE_MARKER: &str = "<<<terminal-state>>>";

/// Start `tinman` on a real terminal under a shell that outlives it and then
/// asks the terminal itself what state the program left it in. `stty` inherits
/// that terminal, so its reading is the operator's own terminal answering
/// rather than the program reporting on itself.
///
/// The shell catches the interrupt rather than ignoring it: an ignored
/// disposition is inherited by the program under test, which would then never
/// receive the interrupt the scenario delivers.
pub fn start_interruptible(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
) -> Result<TerminalSession, String> {
    let mut line = format!(
        "trap ':' INT; {}",
        shell_quote(env!("CARGO_BIN_EXE_tinman"))
    );
    for arg in args {
        line.push(' ');
        line.push_str(&shell_quote(arg));
    }
    // The shell's own status is the last command's, which would be the trailing
    // `stty`, so the program's status is kept before the terminal is read and
    // the shell leaves with it. Without this the session reports success
    // whatever the program did.
    line.push_str(&format!(
        "; __tinman_status=$?; printf '\\n{TERMINAL_STATE_MARKER}\\n'; stty -a; exit $__tinman_status"
    ));
    TerminalSession::start_shell_line(dir, &line, env, TERMINAL_COLS)
}

/// `word` as a single shell word, so a path carrying a space or a quote reaches
/// the shell as one argument.
fn shell_quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// A live `tinman` run on a real pseudo-terminal, kept open so a scenario can
/// type at its prompt and end its input exactly as an operator does. Everything
/// the program writes is drained on a reader thread into a shared buffer, so
/// every wait here ends on observed output rather than on a clock. The child is
/// asked to stop on drop and waited on, so a scenario failing mid-session leaks
/// no process and an instrumented build still flushes what it measured.
pub struct TerminalSession {
    _master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn std::io::Write + Send>,
    output: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

/// The size of the pseudo-terminal a session opens. The width is the width a
/// screen assertion parses the run's bytes at, so the model reads the program's
/// cursor addressing on the same grid the program drew on.
const TERMINAL_ROWS: u16 = 60;
pub const TERMINAL_COLS: u16 = 120;

impl std::fmt::Debug for TerminalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalSession").finish_non_exhaustive()
    }
}

impl TerminalSession {
    /// Start `tinman` on a real terminal in `dir`, with only the `TINMAN_*`
    /// configuration named in `env`, and leave it running.
    pub fn start(
        dir: &std::path::Path,
        args: &[&str],
        env: &[(String, String)],
    ) -> Result<TerminalSession, String> {
        Self::start_at(dir, args, env, TERMINAL_COLS)
    }

    /// Start `tinman` as `start` does, on a terminal `columns` cells wide, so a
    /// scenario naming a width drives the program at that width.
    pub fn start_at(
        dir: &std::path::Path,
        args: &[&str],
        env: &[(String, String)],
        columns: u16,
    ) -> Result<TerminalSession, String> {
        let mut command = portable_pty::CommandBuilder::new(env!("CARGO_BIN_EXE_tinman"));
        for arg in args {
            command.arg(arg);
        }
        Self::spawn_on_pty(command, dir, env, columns)
    }

    /// Start `line` under `sh` on a real terminal in `dir`, so a scenario whose
    /// subject is the shape of the invocation, such as a redirected stream, gets
    /// that shape from the shell that really makes it.
    pub fn start_shell_line(
        dir: &std::path::Path,
        line: &str,
        env: &[(String, String)],
        columns: u16,
    ) -> Result<TerminalSession, String> {
        let mut command = portable_pty::CommandBuilder::new("sh");
        command.arg("-c");
        command.arg(line);
        Self::spawn_on_pty(command, dir, env, columns)
    }

    fn spawn_on_pty(
        mut command: portable_pty::CommandBuilder,
        dir: &std::path::Path,
        env: &[(String, String)],
        columns: u16,
    ) -> Result<TerminalSession, String> {
        use portable_pty::{PtySize, native_pty_system};
        use std::io::Read;

        let pair = native_pty_system()
            .openpty(PtySize {
                rows: TERMINAL_ROWS,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        command.cwd(dir);
        for (key, value) in std::env::vars() {
            if !key.starts_with("TINMAN_") {
                command.env(key, value);
            }
        }
        for (key, value) in env {
            command.env(key, value);
        }

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|e| e.to_string())?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

        let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let sink = std::sync::Arc::clone(&output);
        let drain = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        sink.lock()
                            .expect("the terminal output buffer is not poisoned")
                            .extend_from_slice(&buffer[..read]);
                    }
                }
            }
        });

        Ok(TerminalSession {
            _master: pair.master,
            child,
            writer,
            output,
            reader: Some(drain),
        })
    }

    /// Everything the program has written to the terminal so far, as text: the
    /// carriage returns a terminal ends a line with are removed, so an
    /// assertion reads the lines rather than the terminal's own line endings.
    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.raw_output()).replace('\r', "")
    }

    /// The bytes the program has written to the terminal so far, exactly as the
    /// terminal received them.
    pub fn raw_output(&self) -> Vec<u8> {
        self.output
            .lock()
            .expect("the terminal output buffer is not poisoned")
            .clone()
    }

    /// Wait until Tinman's own model reads a region named `name` on the
    /// terminal, bounded by `budget`, failing with the screen last observed.
    ///
    /// A bordered region is the signal a drawn box gives: its title sits on the
    /// top border row and its body rows below, so no run of the region's text
    /// appears contiguously in the bytes and only the model can see it stand.
    pub fn await_region(&self, name: &str, budget: std::time::Duration) {
        let deadline = std::time::Instant::now() + budget;
        loop {
            let screen = terminal_screen(&self.raw_output());
            if tinman::tom::build(&screen).find_named(name).is_some() {
                return;
            }
            if std::time::Instant::now() >= deadline {
                panic!(
                    "the terminal never showed a region named {name:?}; screen:\n{}",
                    screen.contents()
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// Wait until `text` appears in what the terminal has produced, bounded by
    /// `budget`, failing with everything observed so far.
    pub fn await_output(&self, text: &str, budget: std::time::Duration) {
        let deadline = std::time::Instant::now() + budget;
        loop {
            let seen = self.output();
            if seen.contains(text) {
                return;
            }
            if std::time::Instant::now() >= deadline {
                panic!("the terminal never produced {text:?}; it produced:\n{seen}");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// Interrupt the session the way a terminal driver does when the operator
    /// presses Ctrl-C: SIGINT to the terminal's foreground process group.
    ///
    /// A program holding the terminal in raw mode has turned off the driver's
    /// own signal generation, so typing the interrupt character would arrive as
    /// an ordinary byte and interrupt nothing. The signal goes through the
    /// system's own `kill`, which needs no signalling dependency to reach.
    pub fn interrupt(&self) {
        let group = self
            ._master
            .process_group_leader()
            .expect("the terminal reports a foreground process group");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("kill -INT -{group}"))
            .status()
            .expect("the interrupt is delivered to the session");
        assert!(
            status.success(),
            "interrupting the session's process group {group} failed"
        );
    }

    /// Type `text` at the prompt and press Enter, as an operator does.
    ///
    /// A write that fails because the program has already exited is left to the
    /// assertion that follows, which reports the whole terminal output and so
    /// names the real fault rather than a broken pipe.
    pub fn type_line(&mut self, text: &str) {
        use std::io::Write;
        let _ = write!(self.writer, "{text}\r");
        let _ = self.writer.flush();
    }

    /// Type `text` at the prompt and leave it there, as an operator part way
    /// through a question does. No line ending follows, so the program sees
    /// exactly the characters typed and nothing that would send them.
    pub fn type_text(&mut self, text: &str) {
        use std::io::Write;
        let _ = write!(self.writer, "{text}");
        let _ = self.writer.flush();
    }

    /// Press one key, as an operator does: the key's own bytes reach the
    /// program with no line ending, so a program reading keys sees exactly the
    /// key pressed. Enter is sent as a terminal sends it, a carriage return.
    pub fn press(&mut self, key: &str) {
        use std::io::Write;
        let bytes = match key {
            "Enter" => "\r",
            // The escape key sends the escape character itself, which is what a
            // program reading keys sees when the operator presses it.
            "esc" | "Esc" => "\u{1b}",
            // The backspace key sends the delete character, which is what a
            // terminal in its normal configuration puts in the input queue when
            // the operator presses it.
            "backspace" | "Backspace" => "\u{7f}",
            other => other,
        };
        let _ = write!(self.writer, "{bytes}");
        let _ = self.writer.flush();
    }

    /// End the input, as an operator pressing Ctrl-D does: the terminal's own
    /// end-of-transmission character is how a program reading a terminal sees
    /// the input end.
    pub fn end_input(&mut self) {
        use std::io::Write;
        let _ = self.writer.write_all(&[0x04]);
        let _ = self.writer.flush();
    }

    /// Whether the program has exited, asked of the child itself rather than of
    /// what it drew, so a program still holding the terminal is told apart from
    /// one that has left it.
    pub fn has_exited(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(status) => status.is_some(),
            Err(e) => panic!("asking whether the terminal session exited failed: {e}"),
        }
    }

    /// Wait for the program to exit and report its status, bounded by `budget`.
    pub fn wait(&mut self, budget: std::time::Duration) -> i32 {
        let deadline = std::time::Instant::now() + budget;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return status.exit_code() as i32,
                Ok(None) => {}
                Err(e) => panic!("waiting for the terminal session failed: {e}"),
            }
            if std::time::Instant::now() >= deadline {
                panic!(
                    "the program did not exit within {budget:?}; terminal output:\n{}",
                    self.output()
                );
            }
            std::thread::sleep(EXIT_POLL_INTERVAL);
        }
    }

    /// Wait for the program to exit, then for the terminal to close, and report
    /// everything it wrote together with its exit status. Joining the drain is
    /// what makes the output complete: the child's exit is observed before the
    /// last bytes it wrote have necessarily been read.
    pub fn finish(&mut self, budget: std::time::Duration) -> RunOutcome {
        let status = self.wait(budget);
        if let Some(drain) = self.reader.take() {
            let _ = drain.join();
        }
        RunOutcome {
            stdout: self.output(),
            status,
            raw: self.raw_output(),
            // A program on a real terminal writes its errors to that same
            // terminal, so the output above already carries them.
            stderr: String::new(),
        }
    }
}

/// Read what a program wrote to a terminal into Tinman's own virtual screen,
/// parsed at the width the session opened the terminal at, so a region
/// assertion reads the program's drawing through the same emulation a captured
/// program goes through.
pub fn terminal_screen(raw: &[u8]) -> tinman::screen::VirtualScreen {
    terminal_screen_at(raw, TERMINAL_COLS)
}

/// Read what a program wrote to a terminal `columns` cells wide into Tinman's
/// own virtual screen, so a run driven at a stated width is parsed on the grid
/// the program addressed.
pub fn terminal_screen_at(raw: &[u8], columns: u16) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_pty_output_at(raw, columns)
}

/// The whole of what a run drew, rather than the last screenful of it.
///
/// An assertion that something is absent has to read everything the program
/// drew: read on one screenful, it passes the moment the offending line scrolls
/// out of view, so its result turns on timing rather than on the program.
pub fn whole_terminal_screen(raw: &[u8], columns: u16) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_pty_stream(raw, columns)
}

/// What the terminal received, parsed by the same emulator the virtual screen
/// is built by, kept whole so the cell attributes and the cursor position the
/// virtual screen does not carry can be read off it. Reports the term and the
/// number of rows the grid holds.
///
/// The grid is as tall as Tinman's own virtual screen, taken from a parse of
/// the same bytes rather than restated here. A run taller than the screen
/// scrolls, and two emulations of different heights scroll it by different
/// amounts, so a read on a grid of its own reports cells at rows the model
/// never places a region at, and the join with a region rectangle is false
/// however the program drew.
/// The height the cell grid is read on. A live session is read on the screenful
/// the operator is looking at; a finished run is read whole, because a program
/// that has exited has finished speaking and nothing it drew has scrolled out of
/// reach. The suite opens its terminal taller than the screenful reader's own
/// grid, so a run that drew more than that reader holds loses its opening lines
/// unless it is read whole.
fn grid_rows(raw: &[u8], columns: u16, whole: bool) -> usize {
    if whole {
        tinman::screen::VirtualScreen::from_pty_stream(raw, columns)
            .rows()
            .len()
    } else {
        terminal_screen_at(raw, columns).rows().len()
    }
}

fn emulated_terminal(
    raw: &[u8],
    columns: u16,
) -> (
    alacritty_terminal::term::Term<alacritty_terminal::event::VoidListener>,
    usize,
) {
    emulated_terminal_at(raw, columns, false)
}

fn emulated_terminal_at(
    raw: &[u8],
    columns: u16,
    whole: bool,
) -> (
    alacritty_terminal::term::Term<alacritty_terminal::event::VoidListener>,
    usize,
) {
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::grid::Dimensions;
    use alacritty_terminal::term::{Config, Term};
    use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

    struct Size {
        columns: usize,
        rows: usize,
    }

    impl Dimensions for Size {
        fn total_lines(&self) -> usize {
            self.rows
        }

        fn screen_lines(&self) -> usize {
            self.rows
        }

        fn columns(&self) -> usize {
            self.columns
        }
    }

    let rows = grid_rows(raw, columns, whole);
    let size = Size {
        columns: columns as usize,
        rows,
    };
    let config = Config {
        scrolling_history: 0,
        ..Config::default()
    };
    let mut term = Term::new(config, &size, VoidListener);
    let mut processor = Processor::<StdSyncHandler>::new();
    processor.advance(&mut term, raw);
    (term, rows)
}

/// The 1-based row and column the terminal's cursor sits on, read from the
/// bytes the terminal received through the same emulator the virtual screen is
/// parsed by.
///
/// Cursor position is an attribute the virtual screen does not carry, and the
/// terminal object model carries it no more than it carries colour, so it is
/// read here from the real output. A box drawn with the cursor parked in it and
/// one drawn with the cursor left at the bottom of the screen hold the same
/// characters, and only the terminal's own cursor tells them apart.
pub fn cursor_cell(raw: &[u8], columns: u16) -> (u16, u16) {
    let (term, _) = emulated_terminal(raw, columns);
    let point = term.grid().cursor.point;
    let row = u16::try_from(point.line.0.max(0)).expect("the cursor row fits a terminal");
    let col = u16::try_from(point.column.0).expect("the cursor column fits a terminal");
    (row + 1, col + 1)
}

/// The foreground colour the run of cells reading `text` was drawn in, named as
/// the emulator reports it, or `None` when no row of the screen shows `text`.
///
/// Colour is an attribute the virtual screen does not carry, so two pieces of
/// text drawn in different colours hold the same characters and only the cell
/// attributes tell them apart. The run's first cell carries the colour of the
/// text: a piece of text drawn in one style is drawn in it whole.
pub fn text_foreground(raw: &[u8], columns: u16, text: &str) -> Option<String> {
    text_foreground_from(raw, columns, text, 1)
}

/// As `text_foreground`, reading the first run reading `text` on or below the
/// 1-based row `from_row`.
///
/// A screen can show the same words in two passages, such as a help text's own
/// example line and an answer quoting it back. A comparison between two texts of
/// one passage reads each from that passage rather than from whichever the top
/// of the screen happens to carry.
pub fn text_foreground_from(raw: &[u8], columns: u16, text: &str, from_row: u16) -> Option<String> {
    use alacritty_terminal::index::{Column, Line};

    let (term, rows) = emulated_terminal(raw, columns);
    let grid = term.grid();
    let wanted: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    for row in (from_row.saturating_sub(1) as usize)..rows {
        let cells: Vec<String> = (0..columns as usize)
            .map(|col| grid[Line(row as i32)][Column(col)].c.to_string())
            .collect();
        if let Some(at) = cells
            .windows(wanted.len())
            .position(|window| window == wanted.as_slice())
        {
            return Some(format!("{:?}", grid[Line(row as i32)][Column(at)].fg));
        }
    }
    None
}

/// The 1-based row and column of every cell a program drew in a foreground
/// colour other than the terminal's default, read from the bytes the terminal
/// received through the same emulator the virtual screen is parsed by.
///
/// Colour is an attribute the virtual screen does not carry, so it is read here
/// from the real output rather than asserted against the rendered text: a box
/// drawn in colour and a box drawn plain hold the same characters, and only the
/// cell attributes tell them apart.
pub fn coloured_cells(raw: &[u8], columns: u16) -> Vec<(u16, u16)> {
    coloured_cells_at(raw, columns, false)
}

/// As `coloured_cells`, read on the screenful or on the whole run.
pub fn coloured_cells_at(raw: &[u8], columns: u16, whole: bool) -> Vec<(u16, u16)> {
    use alacritty_terminal::index::{Column, Line};
    use alacritty_terminal::vte::ansi::{Color, NamedColor};

    let (term, rows) = emulated_terminal_at(raw, columns, whole);
    let grid = term.grid();
    let mut found = Vec::new();
    for row in 0..rows {
        for col in 0..columns as usize {
            let cell = &grid[Line(row as i32)][Column(col)];
            if cell.fg != Color::Named(NamedColor::Foreground) {
                found.push((row as u16 + 1, col as u16 + 1));
            }
        }
    }
    found
}

/// The 1-based row and column of the first run of cells reading `text`, with the
/// run's length, or `None` when no row of the screen shows `text`.
///
/// A presentation assertion names the text it is about rather than a coordinate,
/// so the cells that text was drawn on are located here and their attributes read
/// from the same emulator the virtual screen is parsed by.
pub fn text_cells(raw: &[u8], columns: u16, text: &str, whole: bool) -> Option<(u16, u16, u16)> {
    use alacritty_terminal::index::{Column, Line};

    let wanted: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    if wanted.is_empty() || wanted.len() > columns as usize {
        return None;
    }
    let (term, rows) = emulated_terminal_at(raw, columns, whole);
    let grid = term.grid();
    for row in 0..rows {
        let cells: Vec<String> = (0..columns as usize)
            .map(|col| grid[Line(row as i32)][Column(col)].c.to_string())
            .collect();
        if let Some(at) = cells
            .windows(wanted.len())
            .position(|window| window == wanted.as_slice())
        {
            let width = u16::try_from(wanted.len()).expect("the text fits a terminal row");
            return Some((row as u16 + 1, at as u16 + 1, width));
        }
    }
    None
}

/// The 1-based row and column of every cell a program drew on a background other
/// than the terminal's default.
///
/// Background is an attribute the virtual screen does not carry, so a block drawn
/// behind the words and a run of plain text hold the same characters, and only the
/// cell attributes tell them apart.
pub fn background_cells(raw: &[u8], columns: u16, whole: bool) -> Vec<(u16, u16)> {
    use alacritty_terminal::index::{Column, Line};
    use alacritty_terminal::vte::ansi::{Color, NamedColor};

    let (term, rows) = emulated_terminal_at(raw, columns, whole);
    let grid = term.grid();
    let mut found = Vec::new();
    for row in 0..rows {
        for col in 0..columns as usize {
            let cell = &grid[Line(row as i32)][Column(col)];
            if cell.bg != Color::Named(NamedColor::Background) {
                found.push((row as u16 + 1, col as u16 + 1));
            }
        }
    }
    found
}

/// The contiguous run of cells drawn on a background other than the terminal's
/// default that covers the 1-based cell at `row`, `col`, as a 1-based start
/// column and a width. `None` when that cell carries the default background.
///
/// A block is read as the run it occupies rather than as the words inside it,
/// because the whole point of drawing one is that it extends past them.
pub fn background_run(
    raw: &[u8],
    columns: u16,
    row: u16,
    col: u16,
    whole: bool,
) -> Option<(u16, u16)> {
    use alacritty_terminal::index::{Column, Line};
    use alacritty_terminal::vte::ansi::{Color, NamedColor};

    let (term, rows) = emulated_terminal_at(raw, columns, whole);
    if row == 0 || row as usize > rows || col == 0 || col > columns {
        return None;
    }
    let grid = term.grid();
    let line = Line(row as i32 - 1);
    let painted = |column: u16| {
        grid[line][Column(column as usize - 1)].bg != Color::Named(NamedColor::Background)
    };
    if !painted(col) {
        return None;
    }
    let mut start = col;
    while start > 1 && painted(start - 1) {
        start -= 1;
    }
    let mut end = col;
    while end < columns && painted(end + 1) {
        end += 1;
    }
    Some((start, end - start + 1))
}

impl TerminalSession {
    /// Tear the session down and report what teardown observed.
    ///
    /// The program is asked to stop and its exit is waited on, so an instrumented
    /// build flushes what it measured rather than losing it to an outright stop.
    /// The unconditional stop is the fallback the deadline reaches.
    ///
    /// A process that has already gone is reported rather than signalled.
    pub fn tear_down(&mut self) -> TeardownReport {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return TeardownReport {
                already_gone: true,
                kill_diagnostics: String::new(),
            };
        }
        let mut kill_diagnostics = String::new();
        if let Some(pid) = self.child.process_id() {
            kill_diagnostics.push_str(&signal_process(pid, "TERM"));
            let exited = exited_within(TERMINATION_BUDGET, || {
                matches!(self.child.try_wait(), Ok(Some(_)))
            });
            if !exited {
                kill_diagnostics.push_str(&signal_process(pid, "KILL"));
            }
        }
        let _ = self.child.wait();
        TeardownReport {
            already_gone: false,
            kill_diagnostics,
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.tear_down();
        if let Some(drain) = self.reader.take() {
            let _ = drain.join();
        }
    }
}

/// The options a help asset advertises: every `-x` and `--name` token listed in
/// its `Options:` section, in the order the asset lists them. The section runs
/// from the `Options:` heading to the first line that is not one of its indented
/// entries.
pub fn advertised_options(asset: &str) -> Vec<String> {
    let mut options = Vec::new();
    let mut in_section = false;
    for line in asset.lines() {
        if line.trim_end() == "Options:" {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        for token in line.split_whitespace() {
            let token = token.trim_end_matches(',');
            if token.starts_with('-') {
                options.push(token.to_string());
            }
        }
    }
    options
}

/// The commands a help asset lists in its `Commands:` block: the first token of
/// every indented entry, in the order the asset lists them. The block runs from
/// the `Commands:` heading to the first line that is not one of its indented
/// entries. The asset's prose uses command names as ordinary English words, so
/// a command counts as documented only when this block lists it.
pub fn listed_commands(asset: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_block = false;
    for line in asset.lines() {
        if line.trim_end() == "Commands:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        if let Some(name) = line.split_whitespace().next() {
            commands.push(name.to_string());
        }
    }
    commands
}

/// The Tinman command lines an asset names: every inline code span opening with
/// `tinman`, in the order the asset names them. A fenced code block is skipped,
/// so a JSON or YAML sample is never read as a command line.
pub fn named_command_lines(asset: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut fenced = false;
    for line in asset.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        for (index, span) in line.split('`').enumerate() {
            if index % 2 == 1 && (span == "tinman" || span.starts_with("tinman ")) {
                lines.push(span.to_string());
            }
        }
    }
    lines
}

/// The body lines of a roff page's named section: everything between its `.SH`
/// heading and the next one, with the roff macro lines dropped and the escapes
/// roff spells with a backslash read back as the characters they stand for.
pub fn roff_section(page: &str, name: &str) -> Vec<String> {
    let mut body = Vec::new();
    let mut inside = false;
    for line in page.lines() {
        if let Some(heading) = line.strip_prefix(".SH") {
            inside = heading.trim().trim_matches('"') == name;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with('.') {
            continue;
        }
        let text = line.replace("\\-", "-").replace("\\&", "");
        if !text.trim().is_empty() {
            body.push(text.trim().to_string());
        }
    }
    body
}

/// The subcommand pages a roff page cross-references: every `name(1)` reference
/// it prints, in the order it prints them, without the section suffix.
pub fn man_cross_references(page: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in page.lines() {
        let text = line.replace("\\-", "-");
        for token in text.split_whitespace() {
            let Some(name) = token.strip_suffix("(1)") else {
                continue;
            };
            let name = name.trim_matches(|c: char| !c.is_ascii_graphic());
            if let Some(subcommand) = name.strip_prefix("tinman-")
                && !subcommand.is_empty()
                && !found.iter().any(|seen| seen == subcommand)
            {
                found.push(subcommand.to_string());
            }
        }
    }
    found
}

/// The one-line description a help asset carries: the first non-empty line after
/// the line the tagline placeholder reserves. The name and the tagline come
/// first, and the line beneath them is the program said in one sentence.
pub fn asset_one_line_description(asset: &str, placeholder: &str) -> String {
    let mut past_tagline = false;
    for line in asset.lines() {
        if line.contains(placeholder) {
            past_tagline = true;
            continue;
        }
        if past_tagline && !line.trim().is_empty() {
            return line.trim().to_string();
        }
    }
    String::new()
}

/// The closing paragraph of a help asset: the last run of non-empty unindented
/// lines, joined into one paragraph.
pub fn asset_closing_paragraph(asset: &str) -> String {
    let mut paragraphs: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in asset.lines() {
        if line.trim().is_empty() || line.starts_with(' ') {
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(line.trim().to_string());
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }
    paragraphs
        .last()
        .map(|lines| lines.join(" "))
        .unwrap_or_default()
}

/// The Tinman command lines a help asset's `Examples:` block carries: every
/// indented line beneath the heading that opens with `tinman`, in the order the
/// asset lists them. The block runs from the heading to the first non-blank line
/// that is not indented, so the blank lines separating entries stay inside it and
/// the closing prose paragraph ends it.
pub fn examples_block_command_lines(asset: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in asset.lines() {
        if line.trim_end() == "Examples:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let entry = line.trim();
        if entry == "tinman" || entry.starts_with("tinman ") {
            lines.push(entry.to_string());
        }
    }
    lines
}

/// The lines of a help asset's `Commands:` block: the heading and every
/// indented entry beneath it, trimmed of trailing space. The block runs from the
/// heading to the first line that is not one of its indented entries.
pub fn commands_block(asset: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in asset.lines() {
        if line.trim_end() == "Commands:" {
            lines.push(line.trim_end().to_string());
            continue;
        }
        if lines.is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        lines.push(line.trim_end().to_string());
    }
    lines
}

/// The lines of a help asset's `Examples:` block: the heading and every
/// indented line beneath it, trimmed of trailing space. The block runs from the
/// heading to the first non-blank line that is not indented, so the blank lines
/// separating entries stay inside it and the `Example values:` heading ends it.
pub fn examples_block(asset: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in asset.lines() {
        if line.trim_end() == "Examples:" {
            in_block = true;
            lines.push(line.trim_end().to_string());
            continue;
        }
        if !in_block {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        lines.push(line.trim_end().to_string());
    }
    lines
}

/// The placeholder names a help asset's `Examples:` block carries, in the order
/// the block writes them. Each is read from the tldr-pages `{{name}}` notation
/// and reported without its delimiters.
pub fn examples_block_placeholders(asset: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in examples_block(asset) {
        let mut rest = line.as_str();
        while let Some(open) = rest.find("{{") {
            let after = &rest[open + 2..];
            let Some(close) = after.find("}}") else {
                break;
            };
            names.push(after[..close].to_string());
            rest = &after[close + 2..];
        }
    }
    names
}

/// The example values a help asset's `Example values:` block offers: each
/// indented `name: value` entry beneath the heading, in the order the asset
/// lists them. The block runs from the heading to the first non-blank line that
/// is not indented, so the closing prose paragraph ends it.
pub fn example_values(asset: &str) -> Vec<(String, String)> {
    let mut values = Vec::new();
    let mut in_block = false;
    for line in asset.lines() {
        if line.trim_end() == "Example values:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let Some((name, value)) = line.trim().split_once(':') else {
            continue;
        };
        values.push((name.trim().to_string(), value.trim().to_string()));
    }
    values
}

/// The bodies of a document's fenced code blocks whose info string is
/// `language`, in the order the document carries them. A fence opening with a
/// different info string is skipped whole, so its closing fence is never read
/// as an opening one.
pub fn fenced_blocks(asset: &str, language: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut inside = false;
    let mut collecting = false;
    let mut body: Vec<&str> = Vec::new();
    for line in asset.lines() {
        if let Some(info) = line.trim_start().strip_prefix("```") {
            if inside {
                if collecting {
                    blocks.push(body.join("\n"));
                    body.clear();
                    collecting = false;
                }
                inside = false;
            } else {
                inside = true;
                collecting = info.trim() == language;
            }
            continue;
        }
        if collecting {
            body.push(line);
        }
    }
    blocks
}

/// The roles a document names: the inline code spans of the sentence declaring
/// the full set, and the value every `role` key takes in its examples. A
/// document teaches roles both ways, and a reader of one alone reports a clean
/// bill for the half it never read.
pub fn named_roles(asset: &str) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();
    let mut awaiting_set = false;
    for line in asset.lines() {
        if line.contains("The full set:") {
            awaiting_set = true;
            continue;
        }
        if awaiting_set {
            if line.trim().is_empty() {
                continue;
            }
            for (index, span) in line.split('`').enumerate() {
                if index % 2 == 1 {
                    add_role(&mut roles, span);
                }
            }
            awaiting_set = false;
            continue;
        }
        for value in role_values(line) {
            add_role(&mut roles, &value);
        }
    }
    roles
}

/// Add `candidate` to `roles` as a bare role name, once.
fn add_role(roles: &mut Vec<String>, candidate: &str) {
    let role = candidate
        .trim()
        .trim_matches(|c: char| c == '"' || c == ',' || c == '.');
    if !role.is_empty() && !roles.iter().any(|carried| carried == role) {
        roles.push(role.to_string());
    }
}

/// Every value a `role` key takes on one line, in either the JSON or the YAML
/// spelling the examples use. A `role` the prose merely mentions carries no
/// colon after it and is passed over.
fn role_values(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("role") {
        rest = &rest[at + "role".len()..];
        let after = rest.trim_start().trim_start_matches('"').trim_start();
        if let Some(value) = after.strip_prefix(':') {
            let value = value.trim_start().trim_start_matches('"');
            let end = value
                .find(|c: char| c == ',' || c == '}' || c == '"' || c.is_whitespace())
                .unwrap_or(value.len());
            if end > 0 {
                found.push(value[..end].to_string());
            }
        }
    }
    found
}

/// The role names a schema declares: the `enum` of every `role` property it
/// carries, wherever that property sits in the document.
pub fn declared_roles(schema_path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(schema_path)
        .unwrap_or_else(|e| panic!("schema file {schema_path} unreadable: {e}"));
    let schema: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("schema file {schema_path} did not parse: {e}"));
    let mut roles = Vec::new();
    collect_role_enum(&schema, false, &mut roles);
    roles
}

fn collect_role_enum(node: &serde_json::Value, under_role: bool, roles: &mut Vec<String>) {
    match node {
        serde_json::Value::Object(map) => {
            if under_role && let Some(serde_json::Value::Array(values)) = map.get("enum") {
                for value in values {
                    if let Some(name) = value.as_str()
                        && !roles.iter().any(|carried| carried == name)
                    {
                        roles.push(name.to_string());
                    }
                }
            }
            for (key, value) in map {
                collect_role_enum(value, key == "role", roles);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_role_enum(item, under_role, roles);
            }
        }
        _ => {}
    }
}

/// The index of the line the tagline occupies in the help asset, found by the
/// placeholder the asset carries. The rendered help puts whatever fills the
/// tagline on that same line, so a scenario asserting on "the tagline line"
/// reads the rendered output at this index.
pub fn tagline_line_index(asset: &str, placeholder: &str) -> usize {
    asset
        .lines()
        .position(|line| line.contains(placeholder))
        .unwrap_or_else(|| panic!("the help asset carries no {placeholder} line"))
}

/// One match reported by the verification-conformance rule set.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceMatch {
    pub rule_id: String,
    pub file: String,
    pub lines: String,
    pub range: MatchRange,
}

#[derive(Debug, Deserialize)]
pub struct MatchRange {
    pub start: MatchPosition,
}

#[derive(Debug, Deserialize)]
pub struct MatchPosition {
    pub line: usize,
}

impl std::fmt::Display for ConformanceMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {} ({})",
            self.file,
            self.range.start.line + 1,
            self.lines.trim(),
            self.rule_id
        )
    }
}

/// Run the derived verification-conformance rule set over `scope` and return
/// every match it reports. The `conformance` command in RIGGING.md is
/// `ast-grep scan`; this runs that scanner over the named source scope and asks
/// for its machine-readable form, so a scenario can name the rule it attests.
/// The rule set itself is read from `sgconfig.yml`, exactly as the bare command
/// reads it. A match is an error-severity finding, so the scanner exits non-zero
/// when it finds one: the exit status carries no information the parsed matches
/// do not, and a scanner that failed to run at all is caught by the parse.
pub fn run_conformance_scan(scope: &str) -> Vec<ConformanceMatch> {
    let output = std::process::Command::new("ast-grep")
        .args(["scan", "--json=compact", scope])
        .output()
        .unwrap_or_else(|e| panic!("the conformance rule set could not be run: {e}"));
    let stdout = String::from_utf8(output.stdout).expect("the scanner emits UTF-8");
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "the conformance rule set emitted no parseable report: {e}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

/// Every scantling in the scantlings directory, as `(path, document)` pairs,
/// ordered by path, read once per run.
fn scantling_documents() -> &'static [(String, serde_json::Value)] {
    &SCANTLING_DOCUMENTS
}

/// Every scantling path under the scantlings directory, ordered by path.
pub fn scantling_paths() -> Vec<String> {
    scantling_documents()
        .iter()
        .map(|(path, _)| path.clone())
        .collect()
}

/// The scantling paths that no scenario and no step definition names.
///
/// A scantling creates no work until something references it, so an unreferenced
/// one is a contract nobody discharges while every attestation stays green. A
/// boundary contract reached through a path literal in a step definition is
/// reached as soundly as one named in a scenario, so both surfaces are read.
pub fn unreached_scantlings(paths: &[String]) -> Vec<String> {
    let mut texts: Vec<String> = SPEC_PATHS
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("source {} unreadable: {e}", path.display()))
        })
        .collect();
    texts.extend(SUPPORT_SOURCES.iter().map(|(_, text)| text.clone()));
    paths
        .iter()
        .filter(|scantling| !texts.iter().any(|text| text.contains(scantling.as_str())))
        .cloned()
        .collect()
}

/// Every scantling that declares a JSON Schema dialect, as `(path, document)`
/// pairs. A scantling carrying no `$schema` declares no dialect: it is a proof
/// contract discharged by its own checker rather than a schema, so it is not a
/// candidate for meta-schema validation.
pub fn dialect_scantlings() -> Vec<(String, serde_json::Value)> {
    scantling_documents()
        .iter()
        .filter(|(_, document)| document.get("$schema").is_some())
        .cloned()
        .collect()
}

/// Every scantling that declares no JSON Schema dialect, as `(path, document)`
/// pairs. These are the proof contracts: each is discharged by a checker in
/// this support module, so its own shape is checked against the proof-contract
/// meta-schema rather than by a dialect it declares.
pub fn nondialect_scantlings() -> Vec<(String, serde_json::Value)> {
    scantling_documents()
        .iter()
        .filter(|(_, document)| document.get("$schema").is_none())
        .cloned()
        .collect()
}

/// The property names the style object in the scantling at `path` requires.
/// The model defines its style once under `$defs` and the assistant contract
/// restates it inside the region it constrains, so the reader finds the object
/// by the key it hangs from rather than by a path one of the two files happens
/// to use. A region pointing at a shared definition carries a `$ref` and no
/// `required` of its own, so it contributes nothing and the definition it
/// points at contributes once.
pub fn required_style_properties(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("scantling {path} unreadable: {e}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("scantling {path} did not parse: {e}"));
    let mut found = std::collections::BTreeSet::new();
    collect_required_style(&document, &mut found);
    found.into_iter().collect()
}

/// Walk `value`, collecting the `required` entries of every object hanging from
/// a key named `style`.
fn collect_required_style(
    value: &serde_json::Value,
    found: &mut std::collections::BTreeSet<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if key == "style" {
                    let required = child.get("required").and_then(|r| r.as_array());
                    for name in required.into_iter().flatten() {
                        if let Some(name) = name.as_str() {
                            found.insert(name.to_string());
                        }
                    }
                }
                collect_required_style(child, found);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_required_style(item, found);
            }
        }
        _ => {}
    }
}

/// A property a JSON Schema declares, named by the schema pointer of the node
/// that declares it and the property's own name. The pointer is what keeps two
/// declarations of the same name apart: an expectation's `within` and a
/// locator's `within` are different declarations, and a check that compared
/// names alone would let a production type that carries one stand in for the
/// other.
pub type DeclaredProperty = (String, String);

/// Resolve a local `$ref` against `document`, answering the node it points at
/// and the pointer of that node. A node carrying no `$ref` is its own answer.
fn resolve_ref<'a>(
    document: &'a serde_json::Value,
    node: &'a serde_json::Value,
    pointer: &str,
) -> (&'a serde_json::Value, String) {
    let Some(target) = node.get("$ref").and_then(|r| r.as_str()) else {
        return (node, pointer.to_string());
    };
    let path = target
        .strip_prefix("#")
        .unwrap_or_else(|| panic!("only local references are resolved, not {target:?}"));
    let resolved = document
        .pointer(path)
        .unwrap_or_else(|| panic!("the schema carries no node at {path:?}"));
    (resolved, path.to_string())
}

/// Every property the JSON Schema at `path` declares, each named by the pointer
/// of the node declaring it.
pub fn declared_properties(path: &str) -> Vec<DeclaredProperty> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("scantling {path} unreadable: {e}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("scantling {path} did not parse: {e}"));
    let mut found = std::collections::BTreeSet::new();
    collect_declared(&document, "", &mut found);
    found.into_iter().collect()
}

/// Walk `node`, recording every property each schema node declares.
fn collect_declared(
    node: &serde_json::Value,
    pointer: &str,
    found: &mut std::collections::BTreeSet<DeclaredProperty>,
) {
    match node {
        serde_json::Value::Object(map) => {
            if let Some(properties) = map.get("properties").and_then(|p| p.as_object()) {
                for name in properties.keys() {
                    found.insert((pointer.to_string(), name.clone()));
                }
            }
            for (key, child) in map {
                collect_declared(child, &format!("{pointer}/{key}"), found);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_declared(item, &format!("{pointer}/{index}"), found);
            }
        }
        _ => {}
    }
}

/// Every property `instance` really carries, named by the pointer of the schema
/// node that governs it, so a property the instance carries is credited to the
/// declaration it satisfies rather than to its bare name.
pub fn carried_properties(
    schema_path: &str,
    instance: &serde_json::Value,
) -> std::collections::BTreeSet<DeclaredProperty> {
    let text = std::fs::read_to_string(schema_path)
        .unwrap_or_else(|e| panic!("scantling {schema_path} unreadable: {e}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("scantling {schema_path} did not parse: {e}"));
    let mut found = std::collections::BTreeSet::new();
    collect_carried(&document, &document, "", instance, &mut found);
    found
}

/// Walk a schema node and an instance node together, recording every declared
/// property the instance carries. A choice of forms is followed into whichever
/// branch the instance satisfies, so a bare-string form and a full-object form
/// each credit the declarations they really govern.
fn collect_carried(
    document: &serde_json::Value,
    node: &serde_json::Value,
    pointer: &str,
    instance: &serde_json::Value,
    found: &mut std::collections::BTreeSet<DeclaredProperty>,
) {
    let (node, pointer) = resolve_ref(document, node, pointer);
    let Some(map) = node.as_object() else { return };
    if let Some(branches) = map.get("oneOf").and_then(|b| b.as_array()) {
        for (index, branch) in branches.iter().enumerate() {
            let branch_pointer = format!("{pointer}/oneOf/{index}");
            if schema_counterexamples_for(document, branch, instance).is_empty() {
                collect_carried(document, branch, &branch_pointer, instance, found);
            }
        }
    }
    if let Some(properties) = map.get("properties").and_then(|p| p.as_object())
        && let Some(carried) = instance.as_object()
    {
        for (name, child) in properties {
            if let Some(value) = carried.get(name) {
                found.insert((pointer.clone(), name.clone()));
                collect_carried(
                    document,
                    child,
                    &format!("{pointer}/properties/{name}"),
                    value,
                    found,
                );
            }
        }
    }
    if let Some(items) = map.get("items")
        && let Some(elements) = instance.as_array()
    {
        for element in elements {
            collect_carried(document, items, &format!("{pointer}/items"), element, found);
        }
    }
}

/// Whether `instance` satisfies `branch`, read through the whole document so a
/// branch that references a definition still resolves it.
fn schema_counterexamples_for(
    document: &serde_json::Value,
    branch: &serde_json::Value,
    instance: &serde_json::Value,
) -> Vec<String> {
    let mut schema = branch.clone();
    if let (Some(map), Some(defs)) = (schema.as_object_mut(), document.get("$defs")) {
        map.entry("$defs").or_insert_with(|| defs.clone());
    }
    let Ok(validator) = jsonschema::validator_for(&schema) else {
        return vec!["the branch is not a valid schema".to_string()];
    };
    validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect()
}

/// Every published schema URI the repository serves to consumers, as
/// `(source path, uri)` pairs. A scantling publishes its own URI as `$id`; an
/// example plan publishes the one its language server reads from the
/// `$schema=` token in its leading comment. Both are fetched over the network
/// by consumers who never run this suite.
///
/// A scantling carrying no `$id` publishes no URI, so it contributes none: the
/// proof-contract meta-schema is read from a repository path by the checkers in
/// this module and is never fetched by URI. The named count in the scenario is
/// what pins the set, so a scantling that loses its `$id` drops the count and
/// fails there rather than being waved through here.
pub fn published_schema_uris() -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (display, document) in SCANTLING_DOCUMENTS.iter() {
        let Some(id) = document.get("$id").and_then(|v| v.as_str()) else {
            continue;
        };
        found.push((display.clone(), id.to_string()));
    }
    for (display, text) in EXAMPLE_PLANS.iter() {
        let uri = text
            .lines()
            .find_map(|line| line.split_once("$schema="))
            .map(|(_, uri)| uri.trim().to_string())
            .unwrap_or_else(|| panic!("example plan {display} names no $schema"));
        found.push((display.clone(), uri));
    }
    found.sort();
    found
}

/// The version the package declares in its manifest.
///
/// Two packaged manifests are read here, the Cargo manifest and the npm
/// manifest, so the reader dispatches on the manifest's own form rather than on
/// the one the crate happens to use. A JSON manifest read as TOML declares no
/// version by construction, which is the reading that cannot tell a manifest
/// that disagrees from one that was never parsed.
pub fn package_version(manifest_path: &str) -> String {
    let text = std::fs::read_to_string(manifest_path)
        .unwrap_or_else(|e| panic!("manifest {manifest_path} unreadable: {e}"));
    if manifest_path.ends_with(".json") {
        let document: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("manifest {manifest_path} did not parse: {e}"));
        return document
            .get("version")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| panic!("manifest {manifest_path} declares no package version"))
            .to_string();
    }
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            return value.trim().trim_matches('"').to_string();
        }
    }
    panic!("manifest {manifest_path} declares no package version");
}

/// Run one blocking real-service call on its own thread and return what it
/// produced, bounded by `budget`. A real-service step carries an explicit
/// failure ceiling, so a provider that accepts the connection and then never
/// answers fails loudly here, naming its budget, rather than hanging until the
/// whole run is killed. A hung call reports nothing and still bills for the
/// request, so the ceiling is what makes a paid tier honest. The waiting thread
/// is abandoned on expiry because the blocking client offers no cancellation;
/// it ends with the process at run end.
pub fn within_budget<T: Send + 'static>(
    what: &str,
    budget: std::time::Duration,
    call: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(call());
    });
    receiver.recv_timeout(budget).unwrap_or_else(|_| {
        panic!(
            "{what} did not answer within its {}s budget, so the step failed on \
             the budget rather than hanging the run",
            budget.as_secs()
        )
    })
}

/// What a real-service call produced, retried toward `deadline`. Each attempt
/// carries `attempt` as its own failure ceiling, so no attempt hangs the run.
/// An attempt that answers with nothing is a transient of a hosted service
/// rather than a verdict, so another attempt starts while the deadline stands.
/// Absent once the deadline passes with no answer, so the calling step reports
/// the absence in the terms its own scenario asserts. Every abandoned attempt
/// is named on stderr, because a step that retried silently would report a
/// degrading provider as an ordinary green.
pub fn within_deadline<T: Send + 'static>(
    what: &str,
    attempt: std::time::Duration,
    deadline: std::time::Duration,
    call: impl Fn() -> Option<T> + Send + Sync + 'static,
) -> Option<T> {
    let call = std::sync::Arc::new(call);
    let started = std::time::Instant::now();
    let mut attempts = 0usize;
    while started.elapsed() < deadline {
        attempts += 1;
        let call = std::sync::Arc::clone(&call);
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(call());
        });
        let observed = match receiver.recv_timeout(attempt) {
            Ok(Some(value)) => return Some(value),
            Ok(None) => "answered with nothing".to_string(),
            Err(_) => format!("did not answer within {}s", attempt.as_secs()),
        };
        eprintln!(
            "{what} {observed} on attempt {attempts}, {:.1}s into its {}s deadline",
            started.elapsed().as_secs_f64(),
            deadline.as_secs()
        );
    }
    None
}

/// Check the source a boundary policy governs against that policy. Returns
/// counterexamples; an empty list means the module carries no forbidden
/// reference and every required reference.
pub fn check_boundary(policy_path: &str) -> Vec<String> {
    let policy: BoundaryPolicy = read_policy(policy_path);
    let source = match std::fs::read_to_string(&policy.module) {
        Ok(s) => s,
        Err(e) => {
            return vec![format!("source {} unreadable: {e}", policy.module)];
        }
    };
    let mut bad = Vec::new();
    for reference in &policy.forbidden_references {
        if source.contains(reference) {
            bad.push(format!(
                "{} references forbidden token {reference}",
                policy.module
            ));
        }
    }
    for reference in &policy.required_references {
        if !source.contains(reference) {
            bad.push(format!(
                "{} is missing required reference {reference}",
                policy.module
            ));
        }
    }
    bad
}

/// The inline scan rule that finds a construction of `constructed_type`, in
/// either spelling a construction may carry.
///
/// A bare name and a qualified path name the same type, so both are the same
/// construction and the boundary binds them alike. Matching only the spelling
/// the contract carries is what let a qualified construction past while the
/// boundary scenario stayed green: the type's name sits in a different syntax
/// node once a path precedes it, so the bare pattern never reaches it.
fn construction_scan_rule(constructed_type: &str) -> String {
    format!(
        r#"{{id: construction, language: rust, rule: {{any: [{{pattern: "{constructed_type} {{ $$$FIELDS }}"}}, {{pattern: "$PATH::{constructed_type} {{ $$$FIELDS }}"}}]}}}}"#
    )
}

/// The type a construction boundary contract bounds.
pub fn bounded_construction_type(contract_path: &str) -> String {
    let contract: ConstructionBoundary = read_policy(contract_path);
    contract.constructed_type
}

/// How many constructions of the contract's bounded type the boundary checker's
/// own scan finds in `source`.
///
/// The floor asserting a qualified construction counts needs an input the
/// implementation cannot supply, because a qualified construction standing in
/// the tree is exactly the state the boundary forbids. Running the checker's own
/// rule over a source written for it makes the spelling observable without
/// planting a violation in the implementation.
pub fn constructions_found_in(contract_path: &str, source: &str) -> usize {
    let contract: ConstructionBoundary = read_policy(contract_path);
    let scratch = ScratchDir::new("construction-probe");
    let probe = scratch.path().join("probe.rs");
    std::fs::write(&probe, source).unwrap_or_else(|e| {
        panic!(
            "the probe source at {} is not written: {e}",
            probe.display()
        )
    });
    run_inline_scan(
        &construction_scan_rule(&contract.constructed_type),
        &scratch.path().display().to_string(),
    )
    .len()
}

/// Check every construction of the bounded type against the contract's
/// permitted set. Returns counterexamples; an empty list means every
/// construction of the type sits in a permitted module.
///
/// The scan matches a construction expression rather than the bare token, so the
/// type's own declaration is not a counterexample and neither is an import or a
/// mention in a comment. A scan that reached nothing would report a clean bill,
/// so the floor is asserted here: the search paths carry at least one
/// construction, and a contract that found none is itself a counterexample.
pub fn check_construction_boundary(contract_path: &str) -> Vec<String> {
    let contract: ConstructionBoundary = read_policy(contract_path);
    let rule = construction_scan_rule(&contract.constructed_type);
    let mut bad = Vec::new();
    let mut found = 0usize;
    for scope in &contract.search_paths {
        for hit in run_inline_scan(&rule, scope) {
            found += 1;
            if !contract.permitted_modules.contains(&hit.file) {
                bad.push(format!(
                    "{}:{} constructs {}, which only {} may construct",
                    hit.file,
                    hit.range.start.line + 1,
                    contract.constructed_type,
                    contract.permitted_modules.join(", ")
                ));
            }
        }
    }
    if found == 0 {
        bad.push(format!(
            "no construction of {} was found under {}, so the scan asserted nothing",
            contract.constructed_type,
            contract.search_paths.join(", ")
        ));
    }
    bad
}

/// Check every deserializable type under the contract's paths for the derive
/// attribute that refuses a field the model does not define. Returns
/// counterexamples; an empty list means every bound type is strict.
///
/// The requirement binds struct types and struct-variant enums. A unit-variant
/// enum is strict by construction, since an unrecognised value is already an
/// unknown-variant error, and a newtype-variant enum carries no fields of its
/// own. A scan that reached no bound type would report a clean bill, so the
/// floor is asserted here: the search paths carry at least one bound type, and a
/// contract that found none is itself a counterexample.
pub fn check_deserialization_strictness(contract_path: &str) -> Vec<String> {
    let contract: DeserializationStrictness = read_policy(contract_path);
    let mut bad = Vec::new();
    let mut bound = 0usize;
    for path in &contract.search_paths {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(e) => {
                bad.push(format!("search path {path} unreadable: {e}"));
                continue;
            }
        };
        for item in deserializable_items(&source) {
            if !item.bound || contract.exempt_types.contains(&item.name) {
                continue;
            }
            bound += 1;
            if !item
                .attributes
                .contains(&contract.required_derive_attribute)
            {
                bad.push(format!(
                    "{path}:{} {} deserializes an operator's plan without {}",
                    item.line, item.name, contract.required_derive_attribute
                ));
            }
        }
    }
    if bound == 0 {
        bad.push(format!(
            "no bound deserializable type was found under {}, so the scan asserted nothing",
            contract.search_paths.join(", ")
        ));
    }
    bad
}

/// One deserializable type a source declares: its name, the line it is declared
/// on, the attributes attached to it, and whether the strictness requirement
/// binds it.
struct DeserializableItem {
    name: String,
    line: usize,
    attributes: String,
    bound: bool,
}

/// Every type in `source` that derives `Deserialize`, with the attributes
/// attached to its declaration.
///
/// Attributes are siblings of the item they attach to rather than children, so
/// the reader accumulates them until a declaration arrives and clears them
/// after. A doc comment carries no attribute, so a comment naming the required
/// attribute in prose never stands in for the attribute itself.
fn deserializable_items(source: &str) -> Vec<DeserializableItem> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    let mut attributes = String::new();
    let mut open = 0usize;
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if open > 0 {
            attributes.push_str(line);
            open += line.matches('[').count();
            open = open.saturating_sub(line.matches(']').count());
            continue;
        }
        if line.starts_with("#[") {
            attributes.push_str(line);
            open = line
                .matches('[')
                .count()
                .saturating_sub(line.matches(']').count());
            continue;
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some((name, kind)) = declared_type(line)
            && attributes.contains("Deserialize")
        {
            let bound = match kind {
                TypeKind::Struct => true,
                TypeKind::Enum => enum_has_struct_variant(&lines[index..]),
            };
            found.push(DeserializableItem {
                name,
                line: index + 1,
                attributes: attributes.clone(),
                bound,
            });
        }
        attributes.clear();
    }
    found
}

/// The verification support sources, the scope the mark census reads.
const VERIFICATION_SUPPORT_SOURCES: [&str; 2] = ["tests/cucumber.rs", "tests/cucumber/support.rs"];

/// The ordinals by which the Verification agreement's three permitted conditions
/// are named. The agreement enumerates them, so the ordinal is the handle a mark
/// has for saying which one applies.
const PERMITTED_CONDITIONS: [&str; 3] = ["first", "second", "third"];

/// One `@exceptional-double` mark a verification support source carries: where it
/// sits, the text it carries, and the condition that text names.
#[derive(Debug, Clone)]
pub struct ExceptionalDoubleMark {
    pub file: String,
    pub line: usize,
    pub text: String,
    pub condition: Option<String>,
}

/// Every `@exceptional-double` mark the verification support sources carry.
///
/// A mark opens its own documentation line, so prose that mentions the tag inside
/// a sentence is writing about marks rather than being one. The mark then runs to
/// the end of its documentation block, because the condition it names lives in the
/// justification below the tag rather than on the tag line alone.
pub fn exceptional_double_marks() -> Vec<ExceptionalDoubleMark> {
    let mut marks = Vec::new();
    for file in VERIFICATION_SUPPORT_SOURCES {
        let source = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("the verification support at {file} is unreadable: {e}"));
        let mut open: Option<(usize, String)> = None;
        for (index, raw) in source.lines().enumerate() {
            let line = raw.trim();
            let documentation = line.strip_prefix("///").map(str::trim);
            match (documentation, open.as_mut()) {
                (Some(content), Some((_, text))) => {
                    text.push(' ');
                    text.push_str(content);
                }
                (Some(content), None) if content.starts_with("@exceptional-double") => {
                    open = Some((index + 1, content.to_string()));
                }
                (Some(_), None) => {}
                (None, _) => {
                    if let Some((line_number, text)) = open.take() {
                        marks.push(ExceptionalDoubleMark {
                            file: file.to_string(),
                            line: line_number,
                            condition: condition_named(&text),
                            text,
                        });
                    }
                }
            }
        }
        if let Some((line_number, text)) = open.take() {
            marks.push(ExceptionalDoubleMark {
                file: file.to_string(),
                line: line_number,
                condition: condition_named(&text),
                text,
            });
        }
    }
    marks
}

/// The permitted condition a mark's own text names, where it names exactly one.
///
/// The reader takes an ordinal qualifying the word `condition`, which is the form
/// the agreement's own enumeration gives a mark to point with, allowing a word
/// such as `named` between the two. A mark naming no condition is a gesture rather
/// than a justification; a mark naming two says which applies no more clearly than
/// a mark naming none, so both read as unnamed.
fn condition_named(text: &str) -> Option<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .collect();
    let mut found: Vec<&str> = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if word != "condition" && word != "conditions" {
            continue;
        }
        let opens = index.saturating_sub(2);
        for candidate in &words[opens..index] {
            if let Some(ordinal) = PERMITTED_CONDITIONS
                .iter()
                .find(|permitted| *permitted == candidate)
                && !found.contains(ordinal)
            {
                found.push(ordinal);
            }
        }
    }
    match found.as_slice() {
        [only] => Some((*only).to_string()),
        _ => None,
    }
}

/// The two declaration forms the strictness requirement reads differently.
enum TypeKind {
    Struct,
    Enum,
}

/// The type a declaration line declares, as its name and its form. A line
/// declaring neither a struct nor an enum answers nothing.
fn declared_type(line: &str) -> Option<(String, TypeKind)> {
    let body = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    let (kind, rest) = match body.strip_prefix("struct ") {
        Some(rest) => (TypeKind::Struct, rest),
        None => (TypeKind::Enum, body.strip_prefix("enum ")?),
    };
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some((name, kind))
}

/// Whether the enum declared on `lines[0]` carries a variant with a field
/// block. Such a variant deserializes named fields of its own, so the
/// strictness requirement binds it; a unit or newtype variant names none.
fn enum_has_struct_variant(lines: &[&str]) -> bool {
    let mut depth = 0usize;
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if index > 0
            && depth == 1
            && let Some(head) = line.split('{').next()
            && line.contains('{')
            && !head.trim().is_empty()
            && head.trim().chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return true;
        }
        depth += line.matches('{').count();
        depth = depth.saturating_sub(line.matches('}').count());
        if index > 0 && depth == 0 {
            return false;
        }
    }
    false
}

/// The backend identity that names no sandbox at all.
pub const UNSANDBOXED_BACKEND: &str = "none";

/// Every outcome the backend resolution seam can return, as the names it
/// constructs a resolved backend with, read from the seam's own source.
///
/// The seam is read rather than exercised because an outcome no caller reaches
/// is still an outcome the seam offers. The type's own declaration names its
/// field rather than a value, so a construction is read only where a string
/// literal follows the field name.
pub fn resolution_outcomes() -> Vec<String> {
    let path = "src/backend.rs";
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("the backend resolution seam {path} unreadable: {e}"));
    let mut found = Vec::new();
    for (index, _) in source.match_indices("ResolvedBackend {") {
        let mut end = (index + 400).min(source.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        let tail = &source[index..end];
        let Some(field) = tail.find("name:") else {
            continue;
        };
        let after = tail[field + "name:".len()..].trim_start();
        let Some(rest) = after.strip_prefix('"') else {
            continue;
        };
        let Some(close) = rest.find('"') else {
            continue;
        };
        found.push(rest[..close].to_string());
    }
    found.sort();
    found.dedup();
    found
}

/// Check every seam the contract names against the references it forbids.
/// Returns counterexamples; an empty list means no named seam carries a
/// construct that aborts the process.
///
/// A construct named in a comment aborts nothing, so comment lines are read
/// past. A search path that cannot be read is itself a counterexample, and so
/// is a contract naming no path at all: a scan that reached nothing would
/// otherwise report a clean bill for a set it never opened.
pub fn check_seam_references(contract_path: &str) -> Vec<String> {
    let contract: SeamReferenceContract = read_policy(contract_path);
    let mut bad = Vec::new();
    let mut roots = contract.search_paths.clone();
    roots.extend(contract.module.clone());
    if roots.is_empty() {
        bad.push(format!(
            "{contract_path} names neither a search path nor a module, so the scan asserted nothing"
        ));
    }
    // A contract may forbid references, require them, or do both, so neither
    // list is required on its own. Naming neither is the case that reads as a
    // clean bill while asserting nothing about the sources it opened.
    if contract.forbidden_references.is_empty() && contract.required_references.is_empty() {
        bad.push(format!(
            "{contract_path} neither forbids nor requires a reference, so the scan asserted nothing"
        ));
    }
    // Which required references were seen anywhere the contract scans. A
    // required reference is satisfied by the set rather than by each source,
    // because a contract guarding a tree requires the reference to be present
    // in it, not repeated in every file of it.
    let mut seen_required: Vec<&String> = Vec::new();
    for path in &roots {
        let sources = if std::path::Path::new(path).is_dir() {
            let found = rust_sources_under(path);
            if found.is_empty() {
                bad.push(format!(
                    "seam search path {path} holds no Rust source, so the scan asserted nothing"
                ));
            }
            found
        } else {
            vec![path.clone()]
        };
        for source_path in sources {
            let source = match std::fs::read_to_string(&source_path) {
                Ok(source) => source,
                Err(e) => {
                    bad.push(format!("seam source {source_path} unreadable: {e}"));
                    continue;
                }
            };
            for (index, line) in source.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for reference in &contract.forbidden_references {
                    if carries_token(line, reference) {
                        bad.push(format!(
                            "{source_path}:{} carries the forbidden construct {reference}",
                            index + 1
                        ));
                    }
                }
                for reference in &contract.required_references {
                    if carries_token(line, reference) && !seen_required.contains(&reference) {
                        seen_required.push(reference);
                    }
                }
            }
        }
    }
    for reference in &contract.required_references {
        if !seen_required.contains(&reference) {
            bad.push(format!(
                "no source the contract scans references {reference}, which it requires"
            ));
        }
    }
    bad
}

/// Whether `line` carries `reference` as a whole token rather than as a
/// fragment of a longer identifier.
///
/// A plain substring test reports `println!("{failure}")` inside
/// `eprintln!("{failure}")`, so a contract forbidding a write to the data stream
/// reddens on every correct write to the error stream, and the seams the
/// contract exists to protect are the ones it accuses. An occurrence counts only
/// where the characters flanking it are not identifier characters, checked at
/// whichever end of the reference is itself an identifier character.
fn carries_token(line: &str, reference: &str) -> bool {
    let identifier = |c: char| c.is_alphanumeric() || c == '_';
    let guard_start = reference.chars().next().is_some_and(identifier);
    let guard_end = reference.chars().next_back().is_some_and(identifier);
    line.match_indices(reference).any(|(start, _)| {
        let before_ok = !guard_start || !line[..start].chars().next_back().is_some_and(identifier);
        let after_ok = !guard_end
            || !line[start + reference.len()..]
                .chars()
                .next()
                .is_some_and(identifier);
        before_ok && after_ok
    })
}

/// Every Rust source under `root`, depth first and sorted. A search path naming
/// a tree rather than one file is expanded here, so a contract that guards a
/// whole implementation directory reaches a module added later that nobody
/// listed. A root that cannot be read yields nothing, and the caller's floor
/// reports that as a counterexample rather than a clean bill.
fn rust_sources_under(root: &str) -> Vec<String> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rust_sources_under(&path.to_string_lossy()));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path.to_string_lossy().into_owned());
        }
    }
    found.sort();
    found
}

/// The timed seams and their ceilings, as the latency budget contract declares
/// them.
pub fn latency_budgets(contract_path: &str) -> Vec<LatencyBudget> {
    let contract: LatencyBudgets = read_policy(contract_path);
    contract.budgets
}

/// One match an `ast-grep` scan reports, in the shape the derived
/// `plank-inventory` and `step-usage` commands emit.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanMatch {
    pub text: String,
    pub file: String,
    pub range: MatchRange,
    #[serde(default)]
    pub meta_variables: Option<MetaVariables>,
}

#[derive(Debug, Deserialize)]
pub struct MetaVariables {
    pub single: std::collections::BTreeMap<String, MetaVariable>,
}

#[derive(Debug, Deserialize)]
pub struct MetaVariable {
    pub text: String,
}

/// The inline rule the `plank-inventory` command in RIGGING.md carries.
const PLANK_INVENTORY_RULE: &str =
    r#"{id: planks, language: rust, rule: {kind: line_comment, regex: "@planks"}}"#;

/// The inline rule the `step-usage` command in RIGGING.md carries.
const STEP_USAGE_RULE: &str = r##"{id: step-usage, language: rust, rule: {any: [{pattern: "#[given(expr = $P)]"}, {pattern: "#[when(expr = $P)]"}, {pattern: "#[then(expr = $P)]"}, {pattern: "#[given($P)]"}, {pattern: "#[when($P)]"}, {pattern: "#[then($P)]"}]}}"##;

/// Run an `ast-grep` scan with an inline rule over `scope` and return every
/// match it reports. A scanner that failed to run at all is caught by the parse.
fn run_inline_scan(rule: &str, scope: &str) -> Vec<ScanMatch> {
    let output = std::process::Command::new("ast-grep")
        .args(["scan", "--inline-rules", rule, "--json=compact", scope])
        .output()
        .unwrap_or_else(|e| panic!("the inline scan over {scope} could not be run: {e}"));
    let stdout = String::from_utf8(output.stdout).expect("the scanner emits UTF-8");
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "the inline scan over {scope} emitted no parseable report: {e}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

/// Read a Rust string literal, quotes and escapes included, as the text it
/// carries.
fn read_string_literal(literal: &str) -> Option<String> {
    let body = literal.strip_prefix('"')?.strip_suffix('"')?;
    let mut text = String::new();
    let mut characters = body.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            text.push(character);
            continue;
        }
        match characters.next()? {
            'n' => text.push('\n'),
            't' => text.push('\t'),
            escaped => text.push(escaped),
        }
    }
    Some(text)
}

/// One plank the inventory reports: the seam it sits on, and the step
/// definition pattern it names.
#[derive(Debug, Clone)]
pub struct Plank {
    pub file: String,
    pub line: usize,
    pub pattern: String,
}

/// Every `@planks` annotation in the implementation, read through the
/// `plank-inventory` command in RIGGING.md. A `@planks-provisional` annotation
/// names a scenario rather than a pattern, so it is no member of this join. An
/// annotation carrying the token in neither form is malformed and fails here.
pub fn plank_inventory() -> Vec<Plank> {
    let mut planks = Vec::new();
    for reported in run_inline_scan(PLANK_INVENTORY_RULE, "src") {
        let text = reported.text.trim().to_string();
        let line = reported.range.start.line + 1;
        if text.contains("@planks-provisional(") {
            continue;
        }
        let literal = text
            .split_once("@planks(")
            .and_then(|(_, rest)| rest.strip_suffix(')'))
            .unwrap_or_else(|| {
                panic!(
                    "{}:{line} carries a malformed plank annotation: {text}",
                    reported.file
                )
            });
        let pattern = read_string_literal(literal).unwrap_or_else(|| {
            panic!("{}:{line} names no readable pattern: {text}", reported.file)
        });
        planks.push(Plank {
            file: reported.file,
            line,
            pattern,
        });
    }
    planks
}

/// One provisional plank the inventory reports: the seam it sits on, and the
/// scenario reference it names.
#[derive(Debug, Clone)]
pub struct ProvisionalPlank {
    pub file: String,
    pub line: usize,
    pub reference: String,
}

/// Every `@planks-provisional` annotation in the implementation, beside the size
/// of the plank inventory they were read from.
///
/// An empty provisional set is the healthy resting state, so the count that
/// guards this read is the inventory's own: an unreadable scan and a tree
/// carrying no provisional plank both report an empty provisional set, and only
/// the inventory size tells them apart.
#[derive(Debug, Clone)]
pub struct ProvisionalInventory {
    pub annotations: usize,
    pub provisional: Vec<ProvisionalPlank>,
}

/// The provisional planks the implementation carries, read through the same
/// `plank-inventory` command in RIGGING.md that reports the ordinary planks. A
/// provisional plank names a `@captain` scenario rather than a step-definition
/// pattern, so it is no member of the pattern join and is read here instead.
pub fn provisional_plank_inventory() -> ProvisionalInventory {
    let reported = run_inline_scan(PLANK_INVENTORY_RULE, "src");
    let annotations = reported.len();
    let mut provisional = Vec::new();
    for entry in reported {
        let text = entry.text.trim().to_string();
        let line = entry.range.start.line + 1;
        let Some((_, rest)) = text.split_once("@planks-provisional(") else {
            continue;
        };
        let literal = rest.strip_suffix(')').unwrap_or_else(|| {
            panic!(
                "{}:{line} carries a malformed provisional plank: {text}",
                entry.file
            )
        });
        let reference = read_string_literal(literal).unwrap_or_else(|| {
            panic!(
                "{}:{line} names no readable scenario reference: {text}",
                entry.file
            )
        });
        provisional.push(ProvisionalPlank {
            file: entry.file,
            line,
            reference,
        });
    }
    ProvisionalInventory {
        annotations,
        provisional,
    }
}

/// How a step definition matches a step, exactly as the runner matches it: a
/// cucumber expression through the expression crate the runner's own macros
/// use, a bare literal by equality.
#[derive(Debug)]
pub enum Matcher {
    Literal(String),
    Expression(cucumber::codegen::Regex),
}

impl Matcher {
    /// Whether this step definition binds the step text.
    pub fn binds(&self, step: &str) -> bool {
        match self {
            Matcher::Literal(literal) => literal == step,
            Matcher::Expression(regex) => regex.is_match(step),
        }
    }
}

/// One step definition the step-usage command reports: the pattern literal it
/// declares, and how that pattern matches a step.
#[derive(Debug)]
pub struct StepPattern {
    pub file: String,
    pub line: usize,
    pub pattern: String,
    pub matcher: Matcher,
}

/// The custom cucumber parameters this suite declares, as the runner's own
/// macros register them. The runner expands a pattern against the parameters in
/// scope, so a checker expanding against the built-in set alone would reject a
/// pattern the runner accepts. Each type deriving `cucumber::Parameter` is
/// registered here from its own `NAME` and `REGEX`, so the two never disagree.
/// A parameter added and left unregistered fails loudly at the expansion above,
/// naming the pattern, rather than passing quietly.
fn custom_parameters() -> std::collections::HashMap<String, String> {
    use cucumber::codegen::Parameter as _;
    let mut parameters = std::collections::HashMap::new();
    parameters.insert(
        crate::StandardAttribute::NAME.to_string(),
        crate::StandardAttribute::REGEX.to_string(),
    );
    parameters
}

/// Every step definition pattern the verification support declares, read
/// through the `step-usage` command in RIGGING.md. The pattern is carried
/// verbatim, so the plank join over it is exact string membership.
pub fn step_definition_patterns() -> Vec<StepPattern> {
    let mut patterns = Vec::new();
    for reported in run_inline_scan(STEP_USAGE_RULE, "tests") {
        let line = reported.range.start.line + 1;
        let literal = reported
            .meta_variables
            .as_ref()
            .and_then(|variables| variables.single.get("P"))
            .map(|variable| variable.text.clone())
            .unwrap_or_else(|| {
                panic!(
                    "{}:{line} reports no pattern literal: {}",
                    reported.file, reported.text
                )
            });
        let pattern = read_string_literal(&literal).unwrap_or_else(|| {
            panic!(
                "{}:{line} declares an unreadable pattern literal: {literal}",
                reported.file
            )
        });
        let matcher = if reported.text.contains("expr =") {
            let parameters = custom_parameters();
            Matcher::Expression(
                cucumber::codegen::Expression::regex_with_parameters(pattern.as_str(), &parameters)
                    .unwrap_or_else(|e| {
                        panic!(
                            "{}:{line} declares the cucumber expression {pattern}, which does not expand: {e}",
                            reported.file
                        )
                    }),
            )
        } else {
            Matcher::Literal(pattern.clone())
        };
        patterns.push(StepPattern {
            file: reported.file,
            line,
            pattern,
            matcher,
        });
    }
    patterns
}

/// One scenario a spec declares: the feature carrying it, its name, and every
/// step text it carries, background steps included.
#[derive(Debug, Clone)]
pub struct SpecScenario {
    pub feature: String,
    pub name: String,
    pub steps: Vec<String>,
    /// Every tag reaching this scenario, its feature's and its rule's included,
    /// each without its leading `@`, as the Gherkin parser reports them.
    pub tags: Vec<String>,
}

impl SpecScenario {
    /// This scenario's watchbill reference, the `<spec>.feature:<Scenario Name>`
    /// form a provisional plank names a scenario by.
    pub fn reference(&self) -> String {
        format!("{}:{}", self.feature, self.name)
    }

    /// Whether `tag` reaches this scenario, named with or without its `@`.
    pub fn carries_tag(&self, tag: &str) -> bool {
        let wanted = tag.trim_start_matches('@');
        self.tags.iter().any(|carried| carried == wanted)
    }
}

/// Tag names without their leading `@`, so a tag is compared one way wherever it
/// is read.
fn strip_tag_marks(tags: &[String]) -> Vec<String> {
    tags.iter()
        .map(|tag| tag.trim_start_matches('@').to_string())
        .collect()
}

/// Every scenario the specs declare, read with the Gherkin parser the runner
/// itself parses the specs with, so the set is the runner's own and not a
/// second reading of the same files.
pub fn spec_scenarios() -> Vec<SpecScenario> {
    SPEC_SCENARIOS.clone()
}

/// The parsed specs, read once per run. Parsing every spec is the cost this
/// corpus carries, so the parsed scenarios are what is shared rather than the
/// listing that reaches them.
static SPEC_SCENARIOS: std::sync::LazyLock<Vec<SpecScenario>> = std::sync::LazyLock::new(|| {
    let mut found = Vec::new();
    for path in SPEC_PATHS.iter() {
        let display = path.display().to_string();
        // The runner's own parser expands a Scenario Outline into one scenario
        // per example row before matching any step, so a checker reading the
        // unexpanded outline would see `<placeholder>` where the runner sees a
        // value, and would report every outline-only step definition as binding
        // nothing. Expanding here reads what the runner runs.
        use cucumber::feature::Ext as _;
        let feature =
            cucumber::gherkin::Feature::parse_path(path, cucumber::gherkin::GherkinEnv::default())
                .unwrap_or_else(|e| panic!("spec {display} did not parse: {e}"))
                .expand_examples()
                .unwrap_or_else(|e| panic!("spec {display} did not expand its examples: {e:?}"));
        let feature_background: Vec<String> = feature
            .background
            .iter()
            .flat_map(|background| background.steps.iter().map(|step| step.value.clone()))
            .collect();
        let feature_tags = strip_tag_marks(&feature.tags);
        let mut collect = |scenario: &cucumber::gherkin::Scenario,
                           ambient: &[String],
                           ambient_tags: &[String]| {
            let mut steps = ambient.to_vec();
            steps.extend(scenario.steps.iter().map(|step| step.value.clone()));
            let mut tags = ambient_tags.to_vec();
            tags.extend(strip_tag_marks(&scenario.tags));
            found.push(SpecScenario {
                feature: display.clone(),
                name: scenario.name.clone(),
                steps,
                tags,
            });
        };
        for scenario in &feature.scenarios {
            collect(scenario, &feature_background, &feature_tags);
        }
        for rule in &feature.rules {
            let mut ambient = feature_background.clone();
            ambient.extend(
                rule.background
                    .iter()
                    .flat_map(|background| background.steps.iter().map(|step| step.value.clone())),
            );
            let mut ambient_tags = feature_tags.clone();
            ambient_tags.extend(strip_tag_marks(&rule.tags));
            for scenario in &rule.scenarios {
                collect(scenario, &ambient, &ambient_tags);
            }
        }
    }
    found
});

/// One scenario reference a shipped document cites: the document carrying it,
/// the line it sits on, and the `<spec>.feature:<Scenario Name>` reference
/// itself.
#[derive(Debug, Clone)]
pub struct Citation {
    pub document: String,
    pub line: usize,
    pub reference: String,
}

/// The paths the packaged manifest excludes from the crate, each a repository
/// path prefix.
fn packaged_exclusions(manifest_path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(manifest_path)
        .unwrap_or_else(|e| panic!("manifest {manifest_path} unreadable: {e}"));
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("exclude")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            return value
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|entry| entry.trim().trim_matches('"').to_string())
                .filter(|entry| !entry.is_empty())
                .collect();
        }
    }
    Vec::new()
}

/// The Markdown documents the package ships.
///
/// The crate packages the tracked files its manifest does not exclude, so that
/// is what is read here rather than a list kept by hand: a document added to
/// the tree is surveyed the moment it is tracked, and one the manifest excludes
/// is never opened. Tracked is the load-bearing half. An untracked Markdown
/// file reaches no consumer, and reading one would survey the wake, such as the
/// README the npm ship command copies into `npm/`, whose citations are a second
/// copy of the README's own.
pub fn shipped_markdown_documents() -> Vec<String> {
    let listing = std::process::Command::new("git")
        .args(["ls-files"])
        .output()
        .expect("git ls-files ran");
    assert!(
        listing.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let excluded = packaged_exclusions("Cargo.toml");
    let mut found: Vec<String> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .map(str::to_string)
        .filter(|path| path.ends_with(".md"))
        .filter(|path| {
            !excluded
                .iter()
                .any(|entry| path == entry || path.starts_with(&format!("{entry}/")))
        })
        .collect();
    found.sort();
    found
}

/// Every scenario reference the shipped Markdown documents cite.
///
/// A citation is a backticked token carrying `.feature:`, the reference form
/// the watchbill already uses. Keying on that form is what lets a citation
/// announce itself: a bare backticked title cannot be told from any other
/// backticked string, and a survey that guesses reads a command line as a
/// scenario title. A backticked feature path carrying no title is a durable
/// pointer to the file rather than a citation of a scenario, so it names no
/// scenario for this join to resolve and is not read as one.
pub fn cited_scenario_references() -> Vec<Citation> {
    let mut found = Vec::new();
    for document in shipped_markdown_documents() {
        let text = std::fs::read_to_string(&document)
            .unwrap_or_else(|e| panic!("shipped document {document} unreadable: {e}"));
        for (offset, line) in text.lines().enumerate() {
            for (index, span) in line.split('`').enumerate() {
                // An odd index sits between a pair of backticks; an even one is
                // the prose around them.
                if index % 2 == 1 && span.contains(".feature:") {
                    found.push(Citation {
                        document: document.clone(),
                        line: offset + 1,
                        reference: span.to_string(),
                    });
                }
            }
        }
    }
    found
}

/// The membership constructions a watchbill claim is written with.
///
/// A claim is a membership construction, not the word `watchbill`. The rule
/// forbidding these claims, its reasoning, and a settled condemnation all name
/// the watchbill in the shipped prose, so a filter keyed on the word reddens
/// the sentences that define the fault. Each entry here is a preposition
/// against a watch noun, and each is a prefix of its `watchbill` form, so
/// `on the watch` also catches `on the watchbill`.
const WATCHBILL_MEMBERSHIP_CONSTRUCTIONS: [&str; 5] = [
    "on the watch",
    "on a watch",
    "on this watch",
    "in the watch",
    "in a watch",
];

/// One line of a shipped document claiming the scenario it names stands on a
/// watchbill: the document, the line it sits on, the construction that made it
/// a claim, and the line itself.
#[derive(Debug, Clone)]
pub struct WatchbillClaim {
    pub document: String,
    pub line: usize,
    pub construction: String,
    pub text: String,
}

/// What a survey of the shipped documents for watchbill membership claims read
/// and what it found.
#[derive(Debug, Clone)]
pub struct WatchbillSurvey {
    pub documents_read: Vec<String>,
    pub claims: Vec<WatchbillClaim>,
}

/// The shipped-document lines that claim a scenario they name is on a
/// watchbill.
///
/// The subject is given a findable form rather than a wider filter, the shape
/// the citation survey above already takes: a line names a scenario or a spec
/// when it carries a backticked span holding `.feature`. That is what separates
/// a claim from a definition. A claim names which scenario stands on the watch,
/// where the rule forbidding claims speaks of a scenario generically and cites
/// none, so the definition carries no reference for this filter to catch. A
/// line naming the reference form the watchbill uses carries a reference and no
/// membership construction, so it is not read as a claim either.
pub fn watchbill_membership_claims(documents: &[String]) -> WatchbillSurvey {
    let mut documents_read = Vec::new();
    let mut claims = Vec::new();
    for document in documents {
        let text = std::fs::read_to_string(document)
            .unwrap_or_else(|e| panic!("shipped document {document} unreadable: {e}"));
        documents_read.push(document.clone());
        for (offset, line) in text.lines().enumerate() {
            let names_a_spec = line
                .split('`')
                .enumerate()
                .any(|(index, span)| index % 2 == 1 && span.contains(".feature"));
            if !names_a_spec {
                continue;
            }
            let lowered = line.to_lowercase();
            if let Some(construction) = WATCHBILL_MEMBERSHIP_CONSTRUCTIONS
                .iter()
                .find(|construction| lowered.contains(*construction))
            {
                claims.push(WatchbillClaim {
                    document: document.clone(),
                    line: offset + 1,
                    construction: (*construction).to_string(),
                    text: line.trim().to_string(),
                });
            }
        }
    }
    WatchbillSurvey {
        documents_read,
        claims,
    }
}

/// The `ast-grep` project configuration, read for the rule directories the
/// derived verification-conformance rule set lives in.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScannerConfig {
    rule_dirs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ScannerRule {
    id: String,
}

/// The machine-readable summary the duplication scanner emits.
#[derive(Debug, Deserialize)]
struct DuplicationSummary {
    exact_duplicate_groups: usize,
}

/// What the duplication scanner reports over a source tree: the exact-duplicate
/// groups it found, with the members of each as it named them.
#[derive(Debug, Clone)]
pub struct DuplicationReport {
    pub groups: Vec<DuplicateGroup>,
}

/// One exact-duplicate group out of the scanner's listing: the fingerprint the
/// scanner uses as the group's identity, and the units it named as members.
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub fingerprint: String,
    pub members: Vec<String>,
}

/// One group the duplication allowance names. Only the fingerprint is read: it
/// is the scanner's own group identity and therefore the whole join. The reason
/// and the member list the allowance carries are written for a human reading the
/// file, and reading them here would assert nothing a run could fail on.
#[derive(Debug, Deserialize)]
struct AllowedDuplicateGroup {
    fingerprint: String,
}

/// One duplicated-constant group the allowance names. The declarations
/// themselves are the group's identity here, because the constant census runs no
/// scanner and so has no fingerprint of its own to join on. The members are read
/// and the reason is not, for the same cause as above.
#[derive(Debug, Deserialize)]
struct AllowedDuplicateConstants {
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DuplicationAllowance {
    #[serde(rename = "allowedGroups")]
    allowed_groups: Vec<AllowedDuplicateGroup>,
    #[serde(rename = "allowedConstants", default)]
    allowed_constants: Vec<AllowedDuplicateConstants>,
}

/// The duplication allowance at `path`, read once.
fn duplication_allowance(path: &str) -> DuplicationAllowance {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("the duplication allowance {path} is unreadable: {e}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("the duplication allowance {path} did not parse: {e}"))
}

/// The group fingerprints the duplication allowance at `path` names as
/// coincidence rather than as copied logic.
pub fn duplication_allowance_fingerprints(path: &str) -> Vec<String> {
    duplication_allowance(path)
        .allowed_groups
        .into_iter()
        .map(|group| group.fingerprint)
        .collect()
}

/// The duplicated-constant groups the allowance at `path` names as coincidence,
/// each as its sorted member list, which is the group's whole identity.
pub fn duplication_allowance_constants(path: &str) -> Vec<Vec<String>> {
    duplication_allowance(path)
        .allowed_constants
        .into_iter()
        .map(|group| {
            let mut members = group.members;
            members.sort();
            members
        })
        .collect()
}

/// The inline rule the constant census carries. A constant declares itself with
/// or without a visibility, and the two are different nodes, so both forms are
/// asked for.
const CONSTANT_CENSUS_RULE: &str = r#"{id: constants, language: rust, rule: {any: [{pattern: "const $N: $T = $V;"}, {pattern: "pub const $N: $T = $V;"}]}}"#;

/// One constant the census read: where it is declared, the name it carries, and
/// the declaration it gives that name.
#[derive(Debug, Clone)]
pub struct CensusConstant {
    pub file: String,
    pub name: String,
    /// The type and value the declaration gives, as the source writes them. Two
    /// constants agreeing here declare the same thing under two names.
    pub declaration: String,
}

impl CensusConstant {
    /// The form a member is named by, in the allowance and in a failure alike.
    pub fn member(&self) -> String {
        format!("{}:{}", self.file, self.name)
    }
}

/// One group the constant census reports: the constant two or more sources
/// declare identically, and where each copy of it sits.
#[derive(Debug, Clone)]
pub struct DuplicateConstants {
    /// The name and declaration the copies share, as the source writes them.
    pub declaration: String,
    pub members: Vec<String>,
}

/// Every constant the sources under `scope` declare, read with the same scanner
/// the rigging's own derived commands hand their rules to.
///
/// The duplication scanner is structurally blind to a constant: its unit census
/// names closures, methods, functions and trait impl blocks and carries no unit
/// kind for a constant at all, so no threshold and no flag reaches one. This is
/// the second census that covers the rest.
pub fn constant_census(scope: &str) -> Vec<CensusConstant> {
    let mut found = Vec::new();
    for reported in run_inline_scan(CONSTANT_CENSUS_RULE, scope) {
        let line = reported.range.start.line + 1;
        let read = |key: &str| -> String {
            reported
                .meta_variables
                .as_ref()
                .and_then(|variables| variables.single.get(key))
                .map(|variable| variable.text.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "{}:{line} was reported as a constant declaring no {key}",
                        reported.file
                    )
                })
        };
        found.push(CensusConstant {
            file: reported.file.clone(),
            name: read("N"),
            declaration: format!("{} = {}", read("T"), read("V")),
        });
    }
    found.sort_by_key(|constant| constant.member());
    found.dedup_by(|a, b| a.member() == b.member() && a.declaration == b.declaration);
    found
}

/// The constants the census reports as duplicated: every name two or more
/// sources declare identically.
///
/// The join is the name together with the declaration, because that pair is what
/// a copy is. A constant encoding a decision that was copied rather than shared
/// is two places to change and one place to forget, and the copy stays green
/// while it drifts because nothing joins the two declarations.
///
/// Joining on the declaration alone would report every constant that merely
/// agrees in value with an unrelated one, such as six independently chosen
/// deadlines that all happen to be two seconds. Those are not one decision
/// written twice, so nothing will forget to change both, and excusing them would
/// put permanent entries in the allowance for duplication that never existed. A
/// census the project owns can tell them apart, where the third-party scanner
/// beside it cannot and so must defer to a named allowance.
pub fn duplicate_constants(constants: &[CensusConstant]) -> Vec<DuplicateConstants> {
    let mut by_declaration: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for constant in constants {
        by_declaration
            .entry(format!("{} : {}", constant.name, constant.declaration))
            .or_default()
            .push(constant.member());
    }
    by_declaration
        .into_iter()
        .filter(|(_, members)| members.len() > 1)
        .map(|(declaration, mut members)| {
            members.sort();
            DuplicateConstants {
                declaration,
                members,
            }
        })
        .collect()
}

/// The inline rule the process-wait census carries for the sites that construct
/// a child process.
///
/// A syntax query sees the shape of a call and not the type of its receiver, so
/// a wait on a child process is indistinguishable by shape from a wait on
/// anything else: both are a method call on a plain binding. The census is
/// therefore anchored on the construction sites, and asks what bounds each wait
/// on the binding a construction site produced.
const PROCESS_SPAWN_RULE: &str = r#"{id: spawns, language: rust, rule: {any: [{pattern: "$X.spawn()"}, {pattern: "$X.spawn_command($C)"}]}}"#;

/// The inline rule the process-wait census carries for the waits themselves.
const PROCESS_WAIT_RULE: &str = r#"{id: waits, language: rust, rule: {any: [{pattern: "$X.wait()"}, {pattern: "$X.try_wait()"}]}}"#;

/// The constructs that repeat a call, so a non-blocking ask becomes a wait.
const REPEAT_FORMS: [&str; 3] = ["loop {", "while ", "exited_within("];

/// The constructs that bound a wait by a deadline.
const DEADLINE_FORMS: [&str; 3] = ["exited_within(", "Instant::now() +", "recv_timeout("];

/// One wait on a spawned child the census read.
#[derive(Debug, Clone)]
pub struct ProcessWait {
    pub file: String,
    pub line: usize,
    /// The call as the source makes it, on one line.
    pub text: String,
    /// The deadline the wait reaches, named as the source writes it. `None` when
    /// nothing bounds the wait, which is the fault this census exists to find.
    pub bound: Option<String>,
}

impl ProcessWait {
    /// The form a wait is named by in a failure.
    pub fn site(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

/// The text of one line of a verification support source, counted from one.
fn support_source_line(file: &str, line: usize) -> String {
    let (_, text) = SUPPORT_SOURCES
        .iter()
        .find(|(path, _)| path == file)
        .unwrap_or_else(|| panic!("{file} is no verification support source"));
    text.lines()
        .nth(line - 1)
        .unwrap_or_else(|| panic!("{file} carries no line {line}"))
        .to_string()
}

/// The name a `let` binds on `line`, when it binds one.
fn binding_name(line: &str) -> Option<String> {
    let after = line.trim_start().strip_prefix("let ")?;
    let after = after.strip_prefix("mut ").unwrap_or(after);
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The text of the function enclosing `line` in a verification support source.
///
/// A wait is judged by what its own function establishes, so the reader takes
/// the whole function rather than a window around the call.
fn enclosing_function(file: &str, line: usize) -> String {
    let (_, source) = SUPPORT_SOURCES
        .iter()
        .find(|(path, _)| path == file)
        .unwrap_or_else(|| panic!("{file} is no verification support source"));
    let lines: Vec<&str> = source.lines().collect();
    let index = line - 1;
    let start = (0..=index)
        .rev()
        .find(|candidate| {
            let trimmed = lines[*candidate].trim_start();
            trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
        })
        .unwrap_or_else(|| panic!("{file}:{line} sits in no function this census can read"));
    let mut depth: usize = 0;
    let mut opened = false;
    let mut end = lines.len() - 1;
    for (offset, text) in lines[start..].iter().enumerate() {
        for character in text.chars() {
            match character {
                '{' => {
                    depth += 1;
                    opened = true;
                }
                '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        if opened && depth == 0 {
            end = start + offset;
            break;
        }
    }
    lines[start..=end].join("\n")
}

/// Every wait on a spawned child the verification support sources make, each
/// with the deadline that bounds it.
///
/// A wait that carries no deadline hangs the whole suite rather than failing one
/// scenario, and it fails in the worst shape a run has: no output, no red and no
/// weather line, because the sweep never finishes.
///
/// A `try_wait` asks once and returns, so it can hang nothing on its own. It is
/// a wait only where its function repeats it, and then the repeat is what a
/// deadline must bound.
pub fn process_wait_census() -> Vec<ProcessWait> {
    let mut bindings: Vec<(String, String)> = Vec::new();
    for reported in run_inline_scan(PROCESS_SPAWN_RULE, "tests") {
        let line = reported.range.start.line + 1;
        let text = support_source_line(&reported.file, line);
        let binding = binding_name(&text).unwrap_or_else(|| {
            panic!(
                "{}:{line} spawns a child into no named binding: {text}",
                reported.file
            )
        });
        bindings.push((reported.file.clone(), binding));
    }
    bindings.sort();
    bindings.dedup();

    let mut found = Vec::new();
    for reported in run_inline_scan(PROCESS_WAIT_RULE, "tests") {
        let line = reported.range.start.line + 1;
        let text = reported
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let Some((receiver, _)) = text.rsplit_once('.') else {
            continue;
        };
        let receiver = receiver.trim();
        // A wait counts only where its receiver is a binding a construction site
        // in the same source spawned a child into, whether it is that binding or
        // the field it was moved into.
        let on_child = bindings.iter().any(|(file, name)| {
            file == &reported.file && (receiver == name || receiver.ends_with(&format!(".{name}")))
        });
        if !on_child {
            continue;
        }
        let function = enclosing_function(&reported.file, line);
        let blocking = text.ends_with(".wait()");
        let repeated = REPEAT_FORMS.iter().any(|form| function.contains(form));
        if !blocking && !repeated {
            continue;
        }
        let bound = DEADLINE_FORMS
            .iter()
            .find(|form| function.contains(**form))
            .map(|form| form.to_string());
        found.push(ProcessWait {
            file: reported.file.clone(),
            line,
            text,
            bound,
        });
    }
    found.sort_by_key(|wait| wait.site());
    found
}

/// The exact-duplicate groups in the scanner's printed listing, read as the
/// scanner writes them: a `Group N (fingerprint: H, K members):` heading, then
/// one `- name (kind) at file:lines` line per member. The member name is taken
/// up to the kind that follows it, which is how the scanner names the unit.
fn parse_duplicate_groups(section: &str) -> Vec<DuplicateGroup> {
    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Group ") {
            let Some(marker) = trimmed.find("fingerprint: ") else {
                continue;
            };
            let after = &trimmed[marker + "fingerprint: ".len()..];
            let end = after.find([',', ')']).unwrap_or(after.len());
            groups.push(DuplicateGroup {
                fingerprint: after[..end].trim().to_string(),
                members: Vec::new(),
            });
        } else if let Some(member) = trimmed.strip_prefix("- ")
            && let Some(group) = groups.last_mut()
        {
            let name = member.split(" (").next().unwrap_or(member);
            group.members.push(name.trim().to_string());
        }
    }
    groups
}

/// Run the duplication scanner over `path` and read what it reports.
///
/// The count comes from the scanner's own JSON field rather than from its
/// printed prose, so a wording change cannot quietly turn a red green. The
/// printed report is read beside it only when a group was found, because the
/// summary carries counts alone and a failing assertion needs the members. The
/// scanner prints a closing line after its JSON, so the object is taken as the
/// span between its outermost braces.
pub fn duplication_report(path: &str) -> DuplicationReport {
    let run = |arguments: &[&str]| -> String {
        let output = std::process::Command::new("cargo")
            .arg("dupes")
            .args(arguments)
            .output()
            .unwrap_or_else(|e| {
                panic!("the duplication scanner over {path} could not be run: {e}")
            });
        String::from_utf8(output.stdout).expect("the duplication scanner emits UTF-8")
    };
    let summary = run(&["check", "--path", path, "--format", "json"]);
    let (Some(start), Some(end)) = (summary.find('{'), summary.rfind('}')) else {
        panic!("the duplication scanner over {path} emitted no summary object: {summary}");
    };
    let parsed: DuplicationSummary = serde_json::from_str(&summary[start..=end])
        .unwrap_or_else(|e| panic!("the duplication summary over {path} is unreadable: {e}"));
    // The scanner lists the members of each group only when a threshold it was
    // given is exceeded, so the detail run carries the ceiling the rigging's own
    // conformance command carries. A listing read off a passing run is empty,
    // and a failing assertion would then name a count with no members.
    let detail = if parsed.exact_duplicate_groups == 0 {
        String::new()
    } else {
        exact_duplicate_section(&run(&["check", "--path", path, "--max-exact", "0"]))
    };
    let groups = parse_duplicate_groups(&detail);
    // The scanner counts in its JSON and lists in its prose, and the join reads
    // the prose. A parse that drifted from the listing's shape would yield fewer
    // groups than the scanner found and every one of them allowed, which reads
    // as a clean bill. Holding the two counts together makes that drift loud.
    assert_eq!(
        groups.len(),
        parsed.exact_duplicate_groups,
        "the duplication scanner over {path} counts {} exact duplicate group(s) but its listing \
         parsed as {}, so the listing was not read as the scanner writes it:\n{detail}",
        parsed.exact_duplicate_groups,
        groups.len()
    );
    DuplicationReport { groups }
}

/// The exact-duplicate listing out of the duplication scanner's printed report:
/// the section from its own heading to the next one, or to the end.
fn exact_duplicate_section(report: &str) -> String {
    let mut inside = false;
    let mut section = Vec::new();
    for line in report.lines() {
        if line.trim() == "Exact Duplicates" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.trim() == "Near Duplicates" {
            break;
        }
        section.push(line);
    }
    section.join("\n").trim().to_string()
}

/// The rule ids the derived verification-conformance rule set carries, read from
/// the rule directories `sgconfig.yml` names, exactly as the scanner reads them.
pub fn conformance_rule_ids() -> Vec<String> {
    let config: ScannerConfig = read_policy("sgconfig.yml");
    let mut ids = Vec::new();
    for directory in &config.rule_dirs {
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(directory)
            .unwrap_or_else(|e| panic!("rule directory {directory} is unreadable: {e}"))
            .map(|entry| entry.expect("a rule directory entry").path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("yml"))
            .collect();
        paths.sort();
        for path in paths {
            let display = path.display().to_string();
            let rule: ScannerRule = read_policy(&display);
            ids.push(rule.id);
        }
    }
    ids.sort();
    ids
}

/// The characters a scenario name may not carry, because the focused command
/// passes the name to the runner as a regex. This is the set the runner's own
/// expression crate escapes when it builds a regex from a literal.
pub const REGEX_METACHARACTERS: &str = "^$[]()\\{}.|?*+";

/// The `- key: value` items one `## Section` of the rigging declares, in the
/// fixed Markdown shape the Rigging read contract states. A key repeats once per
/// value, so a multi-value key is read whole, and only the first colon separates
/// a key from its value, so a command value carrying colons survives.
fn rigging_section(path: &str, section: &str) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("the rigging at {path} is unreadable: {e}"));
    let mut items = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            inside = heading.trim() == section;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };
        items.push((
            key.trim().to_string(),
            value.trim().trim_matches('`').to_string(),
        ));
    }
    items
}

/// A wall-clock value the rigging declares, such as `90s` or `500ms`.
fn read_duration(value: &str) -> Option<std::time::Duration> {
    if let Some(millis) = value.strip_suffix("ms") {
        return Some(std::time::Duration::from_millis(
            millis.trim().parse().ok()?,
        ));
    }
    let seconds = value.strip_suffix('s')?;
    Some(std::time::Duration::from_secs(seconds.trim().parse().ok()?))
}

/// One tier ceiling the rigging declares: the key declaring it, the tier tag it
/// bounds, the wall clock it allows, and the sweep command that records that
/// tier's wall clock.
#[derive(Debug, Clone)]
pub struct TierBudget {
    pub key: String,
    pub tier: String,
    pub ceiling: std::time::Duration,
    pub sweep: Option<String>,
}

/// Every tier budget the rigging declares, joined to the tier tag it bounds and
/// to the sweep command that records that tier's wall clock. A `budget` key
/// bounds the default tier and a `budget-<tier>` key bounds the tier that suffix
/// names, so the join follows the rigging's own suffix convention rather than a
/// list kept here: a tier added to the rigging is read without an edit.
pub fn tier_budgets(rigging: &str) -> Vec<TierBudget> {
    let tiers = rigging_section(rigging, "Tiers");
    let commands = rigging_section(rigging, "Commands");
    let mut budgets = Vec::new();
    for (key, value) in &tiers {
        let suffix = if key == "budget" {
            "default"
        } else {
            match key.strip_prefix("budget-") {
                Some(suffix) => suffix,
                None => continue,
            }
        };
        let tier = tiers
            .iter()
            .find(|(named, _)| named == suffix)
            .map(|(_, tag)| tag.clone())
            .unwrap_or_else(|| panic!("the rigging declares {key} but names no {suffix} tier"));
        let ceiling = read_duration(value)
            .unwrap_or_else(|| panic!("the rigging declares {key}: {value}, which is no duration"));
        let sweep_key = if suffix == "default" {
            "broad".to_string()
        } else {
            format!("broad-{suffix}")
        };
        budgets.push(TierBudget {
            key: key.clone(),
            tier,
            ceiling,
            sweep: commands
                .iter()
                .find(|(named, _)| *named == sweep_key)
                .map(|(_, command)| command.clone()),
        });
    }
    budgets
}

/// The path the rigging keeps the weather record at.
pub fn weather_record_path(rigging: &str) -> String {
    rigging_section(rigging, "Tiers")
        .into_iter()
        .find(|(key, _)| key == "weather")
        .map(|(_, path)| path)
        .unwrap_or_else(|| panic!("the rigging at {rigging} names no weather record"))
}

/// One outbound target the rigging declares: the channel it names, and the ship
/// and verify commands declared under it.
#[derive(Debug, Clone)]
pub struct OutboundTarget {
    pub name: String,
    pub ship: Option<String>,
    pub verify: Option<String>,
}

/// Every outbound target the rigging declares, each joined to the ship and
/// verify commands that follow it.
///
/// The section is a flat item list, so an `outbound` key opens a target and the
/// `ship` and `verify` keys under it belong to that target until the next one
/// opens. Reading it positionally is what lets a target declaring only one of
/// the two be seen at all: a key-indexed read would find both keys present in
/// the section and report nothing missing.
pub fn outbound_targets(rigging: &str) -> Vec<OutboundTarget> {
    let mut targets: Vec<OutboundTarget> = Vec::new();
    for (key, value) in rigging_section(rigging, "Outbound") {
        match key.as_str() {
            "outbound" => targets.push(OutboundTarget {
                name: value,
                ship: None,
                verify: None,
            }),
            "ship" => {
                if let Some(target) = targets.last_mut() {
                    target.ship = Some(value);
                }
            }
            "verify" => {
                if let Some(target) = targets.last_mut() {
                    target.verify = Some(value);
                }
            }
            _ => {}
        }
    }
    targets
}

/// Whether an outbound command reports the exit status of the work it ran.
///
/// A release command runs its real work and then says how it went, so the three
/// parts are read together: the status is captured off the work, printed where
/// an operator reads it, and carried out as the command's own status. A command
/// that only ends in the work's status reports nothing an operator watching a
/// release can see, and one that only prints exits clean over a failure.
pub fn reports_its_own_exit_status(command: &str) -> bool {
    command.contains("=$?") && command.contains("exit=") && command.contains("exit $")
}

/// One tier sweep the weather record carries: the tier it swept and the wall
/// clock it took.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordedSweep {
    pub tier: String,
    pub ms: u64,
}

/// Every sweep the weather record carries. The record is the wake, so it is
/// absent on a fresh clone where no sweep has run yet: an absent record reads as
/// no sweep rather than as a fault, and the producer is what the check verifies
/// structurally there.
pub fn recorded_sweeps(rigging: &str) -> Vec<RecordedSweep> {
    let path = weather_record_path(rigging);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).unwrap_or_else(|e| {
                panic!("the weather record {path} carries a line that is no sweep: {e}\n{line}")
            })
        })
        .collect()
}

/// The directory the specs live in, which is the root every tier sweeps.
///
/// It is named here because the durations record carries the root each run
/// swept, and that root is what tells a tier sweep from a generated fixture run
/// through the same recorder.
pub const SPECS_DIRECTORY: &str = "features";

/// The environment variable naming the directory or file the suite runs its
/// features from. The tiers leave it unset and sweep the specs directory; the
/// concurrency measurement sets it to a generated fixture so it can run the
/// suite against four scenarios of its own choosing.
pub const FEATURE_ROOT_VARIABLE: &str = "TINMAN_FEATURES";

/// The command-line flags that narrow a run to part of a tier: a glob over the
/// feature files and a regex over the scenario names.
///
/// A tag filter is not among them, because a tag is what names a tier rather
/// than what narrows one, and every sweep command exports one.
const NARROWING_FLAGS: [&str; 5] = ["-i", "--input", "-n", "--name", "--scenario-name"];

/// Whether the command line narrowed this run to part of a tier.
///
/// The focused command selects one scenario by feature path and name, and such
/// a run declares the specs directory as its root exactly as a sweep does. A
/// reader taking the root alone would attest one scenario while reporting on a
/// tier, which is the same substitution a generated fixture makes.
pub fn run_is_narrowed() -> bool {
    std::env::args().any(|argument| {
        NARROWING_FLAGS.contains(&argument.as_str())
            || NARROWING_FLAGS
                .iter()
                .filter(|flag| flag.starts_with("--"))
                .any(|flag| argument.starts_with(&format!("{flag}=")))
    })
}

/// How long each fixture scenario's sandboxed process sleeps.
///
/// The measurement needs scenarios whose wall clock is dominated by waiting on
/// a real process rather than by starting one. Two seconds is long enough that
/// the launch cost is a small fraction of it and short enough that the serial
/// arm of the comparison stays inside the tier's budget.
pub const FIXTURE_SLEEP_SECONDS: u64 = 2;

/// What the fixture asserts about each sandboxed run.
///
/// The exit status is asserted rather than the screen, because inspect reads a
/// program two ways and which way it reads this one is a race the measurement
/// must not inherit. A program that exits inside `inspect`'s exit deadline has
/// its whole output stream read, where a line of plain text is a region; one
/// still running at that deadline has its drawn screen read instead, where the
/// same plain text is no region at all. A sandboxed sleep sits near that
/// deadline by construction, and concurrency moves it across, so an assertion
/// on the output would redden precisely when four workers started overlapping.
/// The exit status is zero down both paths.
const FIXTURE_ASSERTION: &str = "the command exits with a zero status";

/// How many scenarios the concurrency fixture carries. The scenario naming this
/// count in its own text is what the fixture is written to satisfy, so the two
/// are kept in one place.
pub const FIXTURE_SCENARIO_COUNT: usize = 4;

/// Write the concurrency fixture into `dir` and report the file written.
///
/// Every step the fixture uses is one the specs already bind, so the fixture
/// adds no step definition and reaches the same support seam a sandboxed
/// scenario reaches. A fixture with a private step of its own could be made to
/// yield while the real tier still blocked, and the measurement would then
/// describe the fixture rather than the suite.
///
/// It is generated rather than kept in the tree because a `.feature` file under
/// the specs directory is a durable product contract, and this is neither: it
/// is verification support that exists for the length of one scenario.
pub fn write_concurrency_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    let mut text = String::from(
        "Feature: runner concurrency fixture\n\
         \n\
           Rule: each scenario below blocks on a real sandboxed process, so the\n\
         wall clock of a run over them reports what the runner overlaps.\n\
         \n",
    );
    for ordinal in ["first", "second", "third", "fourth"] {
        text.push_str(&format!(
            "  Scenario: the {ordinal} scenario blocks on a sandboxed process\n    \
             When the operator inspects the command \"sh -c 'sleep {FIXTURE_SLEEP_SECONDS}'\"\n    \
             Then {FIXTURE_ASSERTION}\n\n"
        ));
    }
    let path = dir.join("concurrency.feature");
    std::fs::write(&path, text).unwrap_or_else(|e| {
        panic!(
            "the concurrency fixture {} was not written: {e}",
            path.display()
        )
    });
    path
}

/// What one run of the suite over a feature root observed: the wall clock it
/// took and everything it wrote.
#[derive(Debug)]
pub struct FixtureRun {
    pub elapsed: std::time::Duration,
    pub output: String,
}

/// Run this same suite binary over `features` at `workers` concurrent
/// scenarios, and report the wall clock it took.
///
/// The binary already built is re-executed rather than a fresh `cargo test`
/// invocation, so the measurement carries no build and no contention on the
/// build lock the running suite already holds.
///
/// The tag filter is cleared for the child. A sweep exports one, the fixture
/// scenarios carry no tag, and a filter inherited from the parent would select
/// none of them: both arms would then measure an empty run and their ratio
/// would report concurrency nobody exercised.
pub fn run_suite_over(features: &std::path::Path, workers: usize) -> FixtureRun {
    let binary = std::env::current_exe().expect("the running suite binary is named");
    let mut command = std::process::Command::new(binary);
    command
        .arg("-c")
        .arg(workers.to_string())
        .env(FEATURE_ROOT_VARIABLE, features)
        .env_remove("CUCUMBER_FILTER_TAGS");
    let started = std::time::Instant::now();
    let output = command
        .output()
        .expect("the suite binary runs over the concurrency fixture");
    let elapsed = started.elapsed();
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "the concurrency fixture did not pass at {workers} worker(s); it reported:\n{text}"
    );
    FixtureRun {
        elapsed,
        output: text,
    }
}

/// Run this same suite binary with a filter that matches no feature, and report
/// the run identifier the durations record gained.
///
/// The runner reports no feature, no scenario and exits clean for such a run, so
/// the wake is the only place it announces itself at all. The identifier is
/// taken from the child's own pid rather than parsed from its output, because
/// the child prints nothing naming its run, and it is joined against the runs
/// already recorded so a pid the machine has reused names the child alone.
///
/// The binary already built is re-executed rather than a fresh `cargo test`
/// invocation, for the reason `run_suite_over` gives. The feature root is
/// cleared, so the child sweeps the specs directory as a tier does, and the
/// filter is a tag no scenario carries rather than a narrowing flag: the run
/// this describes is sweep-shaped in every respect but the set it matched, so
/// the emptiness is the only thing that can keep it from being offered.
pub fn run_suite_matching_no_feature() -> String {
    /// A tag no scenario in the specs carries.
    const NO_FEATURE_TAG: &str = "@matches-no-scenario";
    /// How long the child has to exit. A run that reads no feature does no work
    /// at all, so this is a failure ceiling far above the real cost rather than
    /// an expectation of it.
    const EXIT_BUDGET: std::time::Duration = std::time::Duration::from_secs(60);

    let already_recorded: std::collections::HashSet<String> = recorded_durations()
        .into_iter()
        .map(|run| run.run)
        .collect();
    let binary = std::env::current_exe().expect("the running suite binary is named");
    let mut child = std::process::Command::new(binary)
        .env_remove(FEATURE_ROOT_VARIABLE)
        .env("CUCUMBER_FILTER_TAGS", NO_FEATURE_TAG)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the suite binary runs with a filter matching no feature");
    let pid = child.id();
    let exited = exited_within(EXIT_BUDGET, || matches!(child.try_wait(), Ok(Some(_))));
    assert!(
        exited,
        "the run whose filter matched no feature did not exit within {EXIT_BUDGET:?}"
    );
    let output = child
        .wait_with_output()
        .expect("the run whose filter matched no feature was read");
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "a run whose filter matches no feature exits clean, which is what makes it \
         indistinguishable from a sweep; this one did not:\n{text}"
    );
    recorded_durations()
        .into_iter()
        .map(|run| run.run)
        .find(|run| run.starts_with(&format!("{pid}-")) && !already_recorded.contains(run))
        .unwrap_or_else(|| {
            panic!(
                "the run whose filter matched no feature left no entry under pid {pid} in the \
                 wake at {DURATIONS_RECORD}; it reported:\n{text}"
            )
        })
}

/// The wake record the runner appends each scenario's own wall clock to.
///
/// The weather record carries one line per tier sweep, so it says a tier outran
/// its ceiling and never says which scenarios spent it. This record carries the
/// attribution. It is named here rather than in the rigging because the rigging
/// names the wake records a role outside verification reads, and Shipwright
/// reads this one at the harbour economy audit, so a `durations` key under
/// `## Tiers` is owed there before that harbour opens.
pub const DURATIONS_RECORD: &str = "target/tinman-durations.jsonl";

/// This process's run identifier, so the entries of one sweep are told from
/// another's in an append-only record. A pid repeats on a long-lived machine,
/// so the moment the process started is carried beside it.
pub fn current_run_id() -> &'static str {
    static RUN_ID: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let started = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_millis())
            .unwrap_or_default();
        format!("{}-{started}", std::process::id())
    });
    &RUN_ID
}

/// The key a scenario is recorded under: the spec carrying it, the line it sits
/// on, and its name. The line is carried because a `Scenario Outline` expands to
/// one scenario per example row and every row shares the outline's name, so the
/// name alone names several scenarios.
pub fn scenario_key(
    feature: &cucumber::gherkin::Feature,
    scenario: &cucumber::gherkin::Scenario,
) -> String {
    let spec = feature
        .path
        .as_ref()
        .map_or_else(|| feature.name.clone(), |path| path.display().to_string());
    format!("{spec}:{}:{}", scenario.position.line, scenario.name)
}

/// The scenario starts this run has seen and not yet timed. Concurrent
/// scenarios sharing a key are paired last in first out: which instance a
/// duration is attributed to is arbitrary there, and the set of durations the
/// record carries is the same either way.
static SCENARIO_STARTS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Vec<std::time::Instant>>>,
> = std::sync::LazyLock::new(std::sync::Mutex::default);

/// Append one line to the durations record, whole.
///
/// A failure to write is left to the check that reads the record rather than
/// raised here: a wake write that cannot land empties the record, and the
/// scenario watching the record is what reports it. Failing the scenario that
/// happened to be running would name the wrong subject and take the suite with
/// it.
fn append_duration_line(line: &str) {
    use std::io::Write as _;
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _held = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(parent) = std::path::Path::new(DURATIONS_RECORD).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(DURATIONS_RECORD)
    else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

/// Record that a scenario started, before it runs. A scenario the runner started
/// and never timed is a gap a reader can see; an unrecorded start is an absence
/// nothing can.
pub fn record_scenario_started(key: &str) {
    SCENARIO_STARTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(key.to_string())
        .or_default()
        .push(std::time::Instant::now());
    append_duration_line(
        &serde_json::json!({ "run": current_run_id(), "scenario": key, "started": true })
            .to_string(),
    );
}

/// Record how long a scenario took, in the wall clock the run itself observed.
pub fn record_scenario_finished(key: &str) {
    let started = SCENARIO_STARTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_mut(key)
        .and_then(Vec::pop);
    let Some(started) = started else {
        return;
    };
    let ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    append_duration_line(
        &serde_json::json!({ "run": current_run_id(), "scenario": key, "ms": ms }).to_string(),
    );
}

/// Record that the run itself reached its end, so a reader tells a sweep that
/// finished from one that was killed part way. A killed run leaves scenarios it
/// never reached, and judging it would redden the attribution check for a fault
/// that is the kill rather than the record.
///
/// The line carries the root the run swept and whether the command line
/// narrowed it, so a run is offered as a tier sweep on what it covered rather
/// than on having merely finished. Both are recorded because both substitutions
/// have been observed: the concurrency measurement runs generated fixtures
/// through this same recorder, and a focused run declares the specs directory
/// exactly as a sweep does while running one scenario.
pub fn record_run_complete(features: &str, narrowed: bool) {
    append_duration_line(
        &serde_json::json!({
            "run": current_run_id(),
            "complete": true,
            "features": features,
            "narrowed": narrowed,
        })
        .to_string(),
    );
}

/// One run the durations record carries: the scenarios it started, the wall
/// clock each of them took, whether the run reached its end, and the root it
/// swept and the narrowing it carried when it did.
#[derive(Debug, Clone, Default)]
pub struct RecordedDurations {
    pub run: String,
    pub complete: bool,
    pub features: Option<String>,
    /// Whether the command line narrowed the run, as the completion line
    /// reports it. `None` where the line carries no such report, which a run
    /// recorded before the field existed does.
    pub narrowed: Option<bool>,
    pub started: Vec<String>,
    pub timed: Vec<(String, u64)>,
}

impl RecordedDurations {
    /// Whether the record offers this run as a tier sweep.
    ///
    /// Four conditions, and each closes a run that reads as a sweep and is not
    /// one. A run that never reached its end left scenarios it never started. A
    /// run whose filter matched no feature reports no feature, no scenario and
    /// exits clean, so completion alone would offer an empty set to a reader
    /// that attests over it. A run whose root is a generated fixture measured
    /// four scenarios of its own. A narrowed run declares the specs directory
    /// exactly as a sweep does and ran the part of it a filter selected.
    pub fn is_sweep(&self) -> bool {
        self.complete
            && !self.started.is_empty()
            && self.features.as_deref() == Some(SPECS_DIRECTORY)
            && self.narrowed == Some(false)
    }

    /// The scenarios this run started that name a spec outside the specs
    /// directory, each named once.
    ///
    /// A run is offered as a sweep on the root it declared, and this reads the
    /// specs it actually ran, so the two are different evidence: a recorder
    /// declaring one root while running another is caught rather than trusted.
    /// The parent directory is compared by component, because the runner
    /// reports a spec path as the feature root gave it and that root is
    /// absolute for one invocation and relative for another.
    pub fn scenarios_outside_the_specs(&self) -> Vec<String> {
        let mut outside: Vec<String> = self
            .started
            .iter()
            .filter(|key| {
                let spec = std::path::Path::new(key.split(':').next().unwrap_or_default());
                !spec
                    .parent()
                    .is_some_and(|directory| directory.ends_with(SPECS_DIRECTORY))
            })
            .cloned()
            .collect();
        outside.sort();
        outside.dedup();
        outside
    }

    /// The scenarios this run started and recorded no duration for, each named
    /// once per start the record left unpaired.
    pub fn missing_durations(&self) -> Vec<String> {
        let mut outstanding: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for key in &self.started {
            *outstanding.entry(key.as_str()).or_default() += 1;
        }
        for (key, _) in &self.timed {
            if let Some(count) = outstanding.get_mut(key.as_str()) {
                *count = count.saturating_sub(1);
            }
        }
        let mut missing: Vec<String> = outstanding
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .flat_map(|(key, count)| std::iter::repeat_n(key.to_string(), count))
            .collect();
        missing.sort();
        missing
    }
}

/// Every run the durations record carries, in the order the record names them.
/// The record is the wake, so it is absent on a fresh clone where no sweep has
/// run yet, and an absent record reads as no run rather than as a fault.
pub fn recorded_durations() -> Vec<RecordedDurations> {
    let Ok(text) = std::fs::read_to_string(DURATIONS_RECORD) else {
        return Vec::new();
    };
    let mut runs: Vec<RecordedDurations> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
            panic!("the durations record {DURATIONS_RECORD} carries a line that is no entry: {e}\n{line}")
        });
        let Some(run) = entry.get("run").and_then(serde_json::Value::as_str) else {
            panic!(
                "the durations record {DURATIONS_RECORD} carries an entry naming no run:\n{line}"
            );
        };
        let index = runs.iter().position(|carried| carried.run == run);
        let index = index.unwrap_or_else(|| {
            runs.push(RecordedDurations {
                run: run.to_string(),
                ..RecordedDurations::default()
            });
            runs.len() - 1
        });
        let carried = &mut runs[index];
        let scenario = entry.get("scenario").and_then(serde_json::Value::as_str);
        if entry.get("complete").and_then(serde_json::Value::as_bool) == Some(true) {
            carried.complete = true;
            carried.features = entry
                .get("features")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            carried.narrowed = entry.get("narrowed").and_then(serde_json::Value::as_bool);
        } else if let (Some(key), Some(ms)) = (
            scenario,
            entry.get("ms").and_then(serde_json::Value::as_u64),
        ) {
            carried.timed.push((key.to_string(), ms));
        } else if let Some(key) = scenario {
            carried.started.push(key.to_string());
        }
    }
    runs
}

/// One enumeration a scantling declares: the scantling declaring it, the JSON
/// pointer it sits at, and the values it names.
#[derive(Debug, Clone)]
pub struct ScantlingEnumeration {
    pub scantling: String,
    pub pointer: String,
    pub values: Vec<serde_json::Value>,
}

/// Every `enum` a scantling declares, found by walking each document rather
/// than by a hand-kept list, so an enumeration added to a scantling is read
/// without an edit here.
pub fn scantling_enumerations() -> Vec<ScantlingEnumeration> {
    fn walk(
        scantling: &str,
        node: &serde_json::Value,
        pointer: &str,
        found: &mut Vec<ScantlingEnumeration>,
    ) {
        match node {
            serde_json::Value::Object(members) => {
                if let Some(serde_json::Value::Array(values)) = members.get("enum") {
                    found.push(ScantlingEnumeration {
                        scantling: scantling.to_string(),
                        pointer: pointer.to_string(),
                        values: values.clone(),
                    });
                }
                for (key, value) in members {
                    walk(scantling, value, &format!("{pointer}/{key}"), found);
                }
            }
            serde_json::Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    walk(scantling, item, &format!("{pointer}/{index}"), found);
                }
            }
            _ => {}
        }
    }

    let mut found = Vec::new();
    for (path, document) in scantling_documents() {
        walk(path, document, "", &mut found);
    }
    found.sort_by(|a, b| (&a.scantling, &a.pointer).cmp(&(&b.scantling, &b.pointer)));
    found
}

/// A scantling enumeration joined to the production enumeration it constrains:
/// the values the scantling declares, the variant names the production type
/// accepts, and the round trip that parses a name and reports how the type
/// serializes it.
pub struct EnumerationPair {
    pub scantling: String,
    pub pointer: String,
    pub production: &'static str,
    pub declared: Vec<String>,
    pub accepted: Vec<String>,
    pub serialized: fn(&str) -> Result<String, String>,
}

impl std::fmt::Debug for EnumerationPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumerationPair")
            .field("scantling", &self.scantling)
            .field("pointer", &self.pointer)
            .field("production", &self.production)
            .field("declared", &self.declared)
            .field("accepted", &self.accepted)
            .finish()
    }
}

/// A value no production enumeration can name, used to make a type report the
/// variant names it accepts.
const VARIANT_PROBE: &str = "tinman-variant-probe";

/// The variant names a type accepts, read from the type itself: an unknown
/// variant makes the deserializer name every variant it knows, so the set comes
/// from the production enumeration rather than from a list kept here that could
/// fall behind it.
fn accepted_variants<T: for<'de> Deserialize<'de>>() -> Vec<String> {
    let probe = serde_json::Value::String(VARIANT_PROBE.to_string());
    let report = match serde_json::from_value::<T>(probe) {
        Ok(_) => panic!("the probe value {VARIANT_PROBE} was accepted as a variant name"),
        Err(e) => e.to_string(),
    };
    let mut names = Vec::new();
    let mut rest = report.as_str();
    while let Some((_, after)) = rest.split_once('`') {
        let Some((name, tail)) = after.split_once('`') else {
            break;
        };
        names.push(name.to_string());
        rest = tail;
    }
    names.retain(|name| name != VARIANT_PROBE);
    assert!(
        !names.is_empty(),
        "no variant names were read from the deserializer's report: {report}"
    );
    names
}

/// Parse a name into the production enumeration and report how that
/// enumeration serializes what it parsed.
fn serialized_variant<T>(name: &str) -> Result<String, String>
where
    T: for<'de> Deserialize<'de> + serde::Serialize,
{
    let parsed = serde_json::from_value::<T>(serde_json::Value::String(name.to_string()))
        .map_err(|e| e.to_string())?;
    let serialized = serde_json::to_value(&parsed).map_err(|e| e.to_string())?;
    serialized
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("serialized to {serialized} rather than to a name"))
}

/// Scantling enumerations that constrain no production enumeration, each with
/// the reason it is out of the join's reach. A scantling enumeration named
/// neither here nor in the join below is unclassified, and reading the pairs
/// fails rather than passing over it.
const UNJOINED_ENUMERATIONS: [(&str, &str, &str); 8] = [
    (
        "scantlings/inference-request.schema.json",
        "/properties/body/properties/messages/items/properties/role",
        "the wire message carries the role as a string, so no production enumeration carries the set",
    ),
    (
        "scantlings/driver-protocol.schema.json",
        "/$defs/request/properties/method",
        "the driver dispatches on the method string, so no production enumeration carries the set",
    ),
    (
        "scantlings/driver-protocol.schema.json",
        "/$defs/params/properties/scope",
        "the driver reads the scope as a string, so no production enumeration carries the set",
    ),
    (
        "scantlings/driver-protocol.schema.json",
        "/$defs/error/properties/code",
        "the codes are integers the driver writes directly, not an enumeration's names",
    ),
    (
        "scantlings/harness-plan.schema.json",
        "/$defs/step/properties/capture/properties/scope",
        "the plan carries the capture scope as a string, so no production enumeration carries the set",
    ),
    (
        "scantlings/harness-plan.schema.json",
        "/$defs/locator/properties/binding",
        "the plan carries the binding as a string; tinman::tom::Binding names it through as_str rather than through serialization",
    ),
    (
        "scantlings/tom.schema.json",
        "/$defs/colour/oneOf/1",
        "the screen carries a cell colour as a string, so no production enumeration carries the set",
    ),
    (
        "scantlings/tom.schema.json",
        "/$defs/region/allOf/0/if/properties/role",
        "the clause selects the subset of roles WAI-ARIA requires a name on rather than declaring a set of its own; the model's role enumeration is joined at /$defs/region/properties/role, and the subset is read from this scantling by the accessible-name sweep",
    ),
];

/// What the join knows of a production enumeration: its path, the variant names
/// it accepts, and its serialization round trip.
type ProductionEnumeration = (
    &'static str,
    Vec<String>,
    fn(&str) -> Result<String, String>,
);

/// Every scantling enumeration joined to the production enumeration it
/// constrains. An enumeration this join does not recognize fails here, so a new
/// scantling enumeration is classified rather than silently uncovered.
pub fn enumeration_pairs() -> Vec<EnumerationPair> {
    let mut pairs = Vec::new();
    for enumeration in scantling_enumerations() {
        let scantling = enumeration.scantling.as_str();
        let pointer = enumeration.pointer.as_str();
        if UNJOINED_ENUMERATIONS
            .iter()
            .any(|(s, p, _)| *s == scantling && *p == pointer)
        {
            continue;
        }
        let (production, accepted, serialized): ProductionEnumeration = match (scantling, pointer) {
            ("scantlings/sandbox-spec.schema.json", "/properties/backend") => (
                "tinman::sandbox::Backend",
                accepted_variants::<tinman::sandbox::Backend>(),
                serialized_variant::<tinman::sandbox::Backend>,
            ),
            ("scantlings/sandbox-spec.schema.json", "/properties/home") => (
                "tinman::sandbox::Home",
                accepted_variants::<tinman::sandbox::Home>(),
                serialized_variant::<tinman::sandbox::Home>,
            ),
            ("scantlings/sandbox-spec.schema.json", "/properties/network") => (
                "tinman::sandbox::Network",
                accepted_variants::<tinman::sandbox::Network>(),
                serialized_variant::<tinman::sandbox::Network>,
            ),
            ("scantlings/sandbox-spec.schema.json", "/properties/mounts/items/properties/mode") => {
                (
                    "tinman::sandbox::MountMode",
                    accepted_variants::<tinman::sandbox::MountMode>(),
                    serialized_variant::<tinman::sandbox::MountMode>,
                )
            }
            (
                "scantlings/sandbox-spec.schema.json",
                "/properties/env/additionalProperties/oneOf/1/properties/from",
            ) => (
                "tinman::sandbox::EnvOrigin",
                accepted_variants::<tinman::sandbox::EnvOrigin>(),
                serialized_variant::<tinman::sandbox::EnvOrigin>,
            ),
            ("scantlings/tom.schema.json", "/$defs/region/properties/role") => (
                "tinman::tom::Role",
                accepted_variants::<tinman::tom::Role>(),
                serialized_variant::<tinman::tom::Role>,
            ),
            _ => panic!(
                "the enumeration at {pointer} in {scantling} names neither a production \
                 enumeration this join carries nor a reason it carries none"
            ),
        };
        let declared = enumeration
            .values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .unwrap_or_else(|| {
                        panic!("the enumeration at {pointer} in {scantling} declares the non-string value {value}")
                    })
                    .to_string()
            })
            .collect();
        pairs.push(EnumerationPair {
            scantling: enumeration.scantling.clone(),
            pointer: enumeration.pointer.clone(),
            production,
            declared,
            accepted,
            serialized,
        });
    }
    pairs
}

// ---------------------------------------------------------------------------
// the recorded upstream the proxied-egress scenarios cross a proxy to reach
// ---------------------------------------------------------------------------

/// One request the upstream received, as it read it off the wire.
#[derive(Debug, Clone)]
pub struct ReceivedRequest {
    pub method: String,
    pub path: String,
    /// The header lines the upstream read off the wire, so a scenario asserting
    /// that a credential travelled reads what arrived rather than what the
    /// client reports about itself.
    pub headers: Vec<String>,
}

/// A real upstream answering on a real loopback port.
///
/// The scenarios assert both that an exchange reached the upstream and that one
/// did not, so the upstream records what it received rather than the run
/// inferring it from the client's side. A proxy that answered from a recording,
/// injected a status or refused the connection leaves this log empty, and that
/// emptiness is the assertion rather than an absence nobody checked.
#[derive(Debug)]
pub struct Upstream {
    base_url: String,
    /// The body the upstream currently answers with. It is shared with the
    /// serving thread so a scenario can change the live answer, which is what
    /// lets a replay scenario prove the recording was served rather than the
    /// upstream's own current reply.
    body: std::sync::Arc<std::sync::Mutex<String>>,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    received: std::sync::Arc<std::sync::Mutex<Vec<ReceivedRequest>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Upstream {
    /// Start an upstream answering every request with `body`, and wait until it
    /// observably serves before reporting it ready.
    pub fn answering(body: &str) -> Upstream {
        use std::io::{BufRead, BufReader, Write};
        use std::sync::atomic::Ordering;

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("the upstream binds a loopback port");
        let addr = listener
            .local_addr()
            .expect("the upstream reports its address");
        listener
            .set_nonblocking(true)
            .expect("the upstream listener is non-blocking");

        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop = std::sync::Arc::clone(&shutdown);
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let arrived = std::sync::Arc::clone(&received);
        let answer = std::sync::Arc::new(std::sync::Mutex::new(body.to_string()));
        let served = std::sync::Arc::clone(&answer);

        let handle = std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("the accepted connection is blocking");
                        let mut reader =
                            BufReader::new(stream.try_clone().expect("the connection is cloned"));
                        let mut request_line = String::new();
                        if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
                            continue;
                        }
                        // The readiness probe and a real request both arrive as a
                        // request line, so the line is what names the exchange.
                        let mut parts = request_line.split_whitespace();
                        let method = parts.next().unwrap_or_default().to_string();
                        let path = parts.next().unwrap_or_default().to_string();
                        let mut headers = Vec::new();
                        loop {
                            let mut line = String::new();
                            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                                break;
                            }
                            if line == "\r\n" || line == "\n" {
                                break;
                            }
                            headers.push(line.trim_end().to_string());
                        }
                        arrived
                            .lock()
                            .expect("the upstream request log is readable")
                            .push(ReceivedRequest {
                                method,
                                path,
                                headers,
                            });
                        let current = served
                            .lock()
                            .expect("the upstream answer is readable")
                            .clone();
                        let mut stream = stream;
                        let _ = stream.write_all(http_response(200, "OK", &current).as_bytes());
                        let _ = stream.flush();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        let upstream = Upstream {
            base_url: format!("http://{addr}"),
            body: answer,
            shutdown,
            received,
            handle: Some(handle),
        };
        upstream.await_ready();
        // The readiness probe is an exchange the upstream recorded, and no
        // scenario asked for it, so the log starts where the scenario does.
        upstream.forget_received();
        upstream
    }

    /// Poll the upstream until it observably answers, bounded by a deadline, so
    /// a scenario never races the listener into being ready.
    fn await_ready(&self) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if std::net::TcpStream::connect_timeout(
                &self
                    .base_url
                    .trim_start_matches("http://")
                    .parse()
                    .expect("the upstream address parses"),
                std::time::Duration::from_millis(200),
            )
            .is_ok()
            {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the upstream at {} did not answer within 5s",
                self.base_url
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// Where the upstream answers.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The body the upstream answers every request with at this moment.
    pub fn body(&self) -> String {
        self.body
            .lock()
            .expect("the upstream answer is readable")
            .clone()
    }

    /// Change the body the upstream answers with from here on, so a scenario
    /// can tell a recorded reply apart from the upstream's current one.
    pub fn answer_with(&self, body: &str) {
        *self.body.lock().expect("the upstream answer is writable") = body.to_string();
    }

    /// Every request the upstream received.
    pub fn received(&self) -> Vec<ReceivedRequest> {
        self.received
            .lock()
            .expect("the upstream request log is readable")
            .clone()
    }

    /// Discard the requests recorded so far, so a scenario whose setup had to
    /// reach the upstream for real asserts over the action alone.
    pub fn forget_received(&self) {
        self.received
            .lock()
            .expect("the upstream request log is readable")
            .clear();
    }

    /// Stop the upstream and wait for its thread to leave, so a scenario running
    /// "with the upstream stopped" has an upstream that observably went.
    pub fn stop(&mut self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Upstream {
    fn drop(&mut self) {
        self.stop();
    }
}

/// What a request through the proxy answered.
#[derive(Debug, Clone)]
pub struct ProxiedResponse {
    pub status: u16,
    pub body: String,
    /// How long the exchange took, so a scenario asserting an injected delay
    /// measures the response rather than trusting the configuration.
    pub elapsed: std::time::Duration,
}

/// Make `url` through the HTTP proxy listening at `proxy`, as a client
/// configured to use a proxy does: the request line carries the absolute URI and
/// the connection is opened to the proxy rather than to the host.
///
/// A refused connection and a proxy error are answers a scenario asserts on, so
/// both are returned rather than panicked.
pub fn request_through_proxy(proxy: &str, url: &str) -> Result<ProxiedResponse, String> {
    use std::io::{BufRead, BufReader, Read, Write};

    let started = std::time::Instant::now();
    let address: std::net::SocketAddr = proxy
        .trim_start_matches("http://")
        .parse()
        .map_err(|e| format!("the proxy address {proxy} does not parse: {e}"))?;
    let host = url.trim_start_matches("http://");
    let host = host.split('/').next().unwrap_or(host);
    let mut stream =
        std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_secs(10))
            .map_err(|e| format!("the connection to the proxy at {proxy} failed: {e}"))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .map_err(|e| format!("the read budget could not be set: {e}"))?;
    write!(
        stream,
        "GET {url} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| format!("the request to the proxy failed: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("the request to the proxy was not flushed: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .map_err(|e| format!("the proxy sent no status line: {e}"))?;
    if status_line.is_empty() {
        return Err("the proxy closed the connection without answering".to_string());
    }
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("the proxy answered an unreadable status line: {status_line}"))?;
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .map_err(|e| format!("the proxy sent an unreadable header: {e}"))?
            == 0
        {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
    }
    let mut body = String::new();
    if length > 0 {
        let mut buffer = vec![0u8; length];
        reader
            .read_exact(&mut buffer)
            .map_err(|e| format!("the proxy sent a short body: {e}"))?;
        body = String::from_utf8_lossy(&buffer).into_owned();
    } else {
        let _ = reader.read_to_string(&mut body);
    }
    Ok(ProxiedResponse {
        status,
        body,
        elapsed: started.elapsed(),
    })
}

/// One exchange a HAR recording carries, as a scenario reads it.
#[derive(Debug, Clone)]
pub struct HarEntry {
    pub method: String,
    pub url: String,
    pub status: u16,
}

/// The entries a HAR recording carries, read where HAR puts them: `log.entries`,
/// each carrying the request it recorded and the response that answered it.
///
/// A recording that does not parse is a failure rather than an empty reading: a
/// scenario asserting one entry would otherwise pass an unwritten file off as a
/// recording that carries nothing.
pub fn har_entries(path: &str) -> Vec<HarEntry> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("the HAR recording {path} is unreadable: {e}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("the HAR recording {path} did not parse: {e}"));
    let entries = document
        .get("log")
        .and_then(|log| log.get("entries"))
        .and_then(|entries| entries.as_array())
        .unwrap_or_else(|| panic!("the HAR recording {path} carries no log.entries array"));
    entries
        .iter()
        .map(|entry| {
            let request = entry
                .get("request")
                .unwrap_or_else(|| panic!("a HAR entry in {path} carries no request"));
            let response = entry
                .get("response")
                .unwrap_or_else(|| panic!("a HAR entry in {path} carries no response"));
            HarEntry {
                method: request
                    .get("method")
                    .and_then(|value| value.as_str())
                    .unwrap_or_else(|| panic!("a HAR request in {path} records no method"))
                    .to_string(),
                url: request
                    .get("url")
                    .and_then(|value| value.as_str())
                    .unwrap_or_else(|| panic!("a HAR request in {path} records no url"))
                    .to_string(),
                status: response
                    .get("status")
                    .and_then(|value| value.as_u64())
                    .unwrap_or_else(|| panic!("a HAR response in {path} records no status"))
                    as u16,
            }
        })
        .collect()
}
