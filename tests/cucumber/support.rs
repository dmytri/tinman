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
    pub forbid: Forbid,
    pub system_mount_flag: String,
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

/// What the local provider answers a chat-completions request with.
#[derive(Debug, Clone)]
pub enum ProviderReply {
    /// A successful completion carrying this message content.
    Content(String),
    /// A rejected credential.
    Unauthorized,
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

        let handle = std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("the accepted connection is blocking");
                        let mut reader =
                            BufReader::new(stream.try_clone().expect("the connection is cloned"));
                        let mut length = 0usize;
                        loop {
                            let mut line = String::new();
                            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                                break;
                            }
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
            handle: Some(handle),
        };
        provider.await_ready();
        provider
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
#[derive(Debug)]
pub struct RunOutcome {
    pub stdout: String,
    pub status: i32,
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
            let status = command.stdout(std::process::Stdio::from(file)).status()?;
            Ok(RunOutcome {
                stdout: std::fs::read_to_string(path)?,
                status: status.code().unwrap_or(-1),
            })
        }
        None => {
            let output = command.output()?;
            Ok(RunOutcome {
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                status: output.status.code().unwrap_or(-1),
            })
        }
    }
}

/// A running `tinman driver` process, spoken to over its real stdin and stdout
/// with one JSON message per line, exactly as a test runner in any language
/// would drive it. Every message exchanged is kept, so a scenario can attest
/// the whole conversation against the protocol scantling.
pub struct DriverProcess {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
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
        use std::io::{BufRead, BufReader};

        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_tinman"))
            .arg("driver")
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
            stdin,
            replies,
            exchanged: Vec::new(),
        }
    }

    /// Send one request line and read the driver's reply, bounded by a budget
    /// generous enough for a real sandboxed launch.
    pub fn send_line(&mut self, line: &str) -> serde_json::Value {
        use std::io::Write;

        let request: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("the request line is not JSON: {e}\n{line}"));
        self.exchanged.push(request);
        writeln!(self.stdin, "{line}").expect("the request reaches the driver");
        self.stdin.flush().expect("the request is flushed");
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

impl Drop for DriverProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
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

/// Draw a screen whose top row carries `text`.
pub fn top_line_screen(text: &str) -> tinman::screen::VirtualScreen {
    tinman::screen::VirtualScreen::from_text(&format!("\x1b[1;1H{text}"))
}

/// Run the real `tinman` binary on a real pseudo-terminal, so the run sees a
/// terminal rather than a pipe, with `dir` as its working directory and only
/// the `TINMAN_*` configuration named in `env`. Everything the program wrote is
/// returned together with its exit status. The terminal is opened large enough
/// to hold the whole help output, so no line the scenario asserts on scrolls
/// away before the program exits.
pub fn run_tinman_on_a_terminal(
    dir: &std::path::Path,
    args: &[&str],
    env: &[(String, String)],
) -> Result<RunOutcome, String> {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};
    use std::io::Read;

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 60,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_tinman"));
    for arg in args {
        command.arg(arg);
    }
    command.cwd(dir);
    for (key, value) in std::env::vars() {
        if !key.starts_with("TINMAN_") {
            command.env(key, value);
        }
    }
    for (key, value) in env {
        command.env(key, value);
    }

    let mut child = pair
        .slave
        .spawn_command(command)
        .map_err(|e| e.to_string())?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    drop(pair.slave);
    let mut output = Vec::new();
    reader.read_to_end(&mut output).map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    Ok(RunOutcome {
        stdout: String::from_utf8_lossy(&output).replace('\r', ""),
        status: status.exit_code() as i32,
    })
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
