//! The command-line session: a session the operator names, launches a program
//! in, addresses with a verb, and closes. A session outlives the command that
//! launched it, so the program is held by a Tinman process of its own that keeps
//! the sandbox standing and writes the screen its program draws where the next
//! command reads it. Closing removes the session's directory, which is both the
//! reclaim and the signal that ends the holding process.

use crate::backend::resolve;
use crate::driver::EXPECT_DEADLINE;
use crate::plan::{Action, Expectation, FlowStep, Plan, TuiProcess};
use crate::pty::capture_interactive;
use crate::sandbox::{CommandSpec, SandboxSpec};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The environment variable naming the session directory a Tinman process
/// holds. An operator never sets it: `tinman launch` sets it on the process it
/// starts, which is what tells that process it is the session rather than the
/// command line asking for one.
const HELD_SESSION: &str = "TINMAN_SESSION_DIR";

/// The file the holding process writes the session's screen to, so a verb
/// running in another process reads what the program has drawn.
const SCREEN_FILE: &str = "screen";

/// The file carrying the identifier of the process holding the session, so
/// closing waits for that process to end rather than for a clock.
const HOLDER_FILE: &str = "holder";

/// The file naming the program the session drives, so a plan written from the
/// session names that program rather than the session.
const COMMAND_FILE: &str = "command";

/// The file the steps the session performed are journalled to, one step per
/// line, appended by each verb that held. A verb that failed appends nothing,
/// so the plan carries the steps that held and no others.
const STEPS_FILE: &str = "steps";

/// The file a key waiting to be forwarded is written to. The holding process
/// removes it once the program has been sent the key, which is what tells the
/// command line the press landed.
const KEY_FILE: &str = "key";

/// Where a key is written before it is renamed into place, so the holding
/// process reads a whole key rather than a file it caught half-written.
const PENDING_KEY_FILE: &str = "pending-key";

/// How often the holding process writes the screen its program has drawn, and
/// how often a wait reads the state it is waiting on.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// How long a launch gives the session to draw its first screen, so a command
/// following a launch reads a drawn screen rather than a blank one.
const LAUNCH_DEADLINE: Duration = Duration::from_secs(10);

/// How long a pressed key gives the holding process to forward it.
const PRESS_DEADLINE: Duration = Duration::from_secs(5);

/// How long closing gives the holding process to end.
const CLOSE_DEADLINE: Duration = Duration::from_secs(5);

/// What every session directory is named for, so a sweep reads the sessions a
/// temporary directory holds by the same word that creates them.
const SESSION_PREFIX: &str = "tinman-session-";

/// Where the session `name` keeps what launching it created. The name the
/// operator gave is in the directory's own name, so the resources a session owns
/// are found by the word the operator used on every command that touches it.
///
/// @planks("the operator executes {string}")
fn directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{SESSION_PREFIX}{name}"))
}

/// The screen the session's holding process last wrote, empty where it has yet
/// to write one.
fn screen_of(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(SCREEN_FILE)).unwrap_or_default()
}

/// Wait until `ready` answers, giving it `budget`, and report `failure` where
/// the budget runs out. Every wait here ends on the state it names rather than
/// on a sleep, so a session that is up resolves at once.
fn awaited(
    budget: Duration,
    ready: impl Fn() -> bool,
    failure: impl Fn() -> String,
) -> Result<(), String> {
    let deadline = Instant::now() + budget;
    while !ready() {
        if Instant::now() >= deadline {
            return Err(failure());
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    Ok(())
}

/// Launch `command` as the session `name`, over `workspace`. A process carrying
/// the held-session variable is the session itself and holds the program; every
/// other is the operator's command line, which starts one and returns once the
/// session has drawn.
///
/// @planks("the operator executes {string}")
/// @planks("a session named {string} running {string}")
/// @planks("the operator launches a session against a target that prints whether it read {string}")
pub fn launch(command: &[String], name: &str, workspace: &Path) -> Result<(), String> {
    match std::env::var(HELD_SESSION) {
        Ok(held) => hold(command, workspace, Path::new(&held)),
        Err(_) => start(command, name, workspace),
    }
}

/// Reclaim every session standing with nobody holding it. A holding process
/// ends when its program does, and an operator who walks away never closes what
/// it left, so a session nobody holds is a sandbox and a staging directory
/// waiting on a close that is not coming. Launching is where they are reclaimed,
/// on the same reasoning that reclaims at the start of a run rather than
/// trusting a killed one to have tidied up after itself.
///
/// A session whose holder is not recorded yet is another launch in progress, so
/// it is left to the launch that is creating it.
///
/// @planks("the operator executes {string}")
fn reclaim_unheld() -> Result<(), String> {
    let temp = std::env::temp_dir();
    let entries = std::fs::read_dir(&temp).map_err(|e| {
        format!(
            "the sessions standing in {} were not read: {e}",
            temp.display()
        )
    })?;
    for entry in entries {
        let session = entry
            .map_err(|e| format!("a session standing in {} was not read: {e}", temp.display()))?
            .path();
        let named = session
            .file_name()
            .and_then(|entry| entry.to_str())
            .is_some_and(|entry| entry.starts_with(SESSION_PREFIX));
        if !named {
            continue;
        }
        let record = session.join(HOLDER_FILE);
        if !record.exists() {
            continue;
        }
        let holder = std::fs::read_to_string(&record).map_err(|e| {
            format!(
                "what holds the session at {} was not read: {e}",
                session.display()
            )
        })?;
        if Path::new("/proc").join(holder.trim()).exists() {
            continue;
        }
        std::fs::remove_dir_all(&session).map_err(|e| {
            format!(
                "the unheld session at {} was not reclaimed: {e}",
                session.display()
            )
        })?;
    }
    Ok(())
}

/// Start the session `name`: reclaim what nobody is holding, create the
/// directory this session owns, record the program it drives and the empty
/// journal its verbs append to, start the Tinman process that holds it, and
/// return once that process has drawn a screen, so the verb the operator runs
/// next reads what the program shows.
///
/// @planks("a session named {string} running {string}")
/// @planks("the operator executes {string}")
fn start(command: &[String], name: &str, workspace: &Path) -> Result<(), String> {
    reclaim_unheld()?;
    let dir = directory(name);
    // A session of this name standing from an earlier run is closed first, so
    // the operator addresses the session they just launched rather than the
    // screen and the sandbox a previous one left behind.
    if dir.join(HOLDER_FILE).exists() {
        reclaim(name)?;
    }
    std::fs::create_dir_all(&dir).map_err(|e| {
        format!(
            "the session directory {} was not created: {e}",
            dir.display()
        )
    })?;
    std::fs::write(dir.join(COMMAND_FILE), command.join(" "))
        .map_err(|e| format!("the session {name:?} did not record the program it drives: {e}"))?;
    std::fs::write(dir.join(STEPS_FILE), "")
        .map_err(|e| format!("the session {name:?} did not open its journal: {e}"))?;
    let tinman = std::env::current_exe()
        .map_err(|e| format!("the Tinman binary holding the session was not found: {e}"))?;
    let holder = std::process::Command::new(tinman)
        .arg("launch")
        .args(command)
        .arg("--session")
        .arg(name)
        .current_dir(workspace)
        .env(HELD_SESSION, &dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("the session {name:?} was not started: {e}"))?;
    std::fs::write(dir.join(HOLDER_FILE), holder.id().to_string())
        .map_err(|e| format!("the session {name:?} did not record what holds it: {e}"))?;
    awaited(
        LAUNCH_DEADLINE,
        || !screen_of(&dir).trim().is_empty(),
        || format!("the session {name:?} drew nothing before its launch deadline"),
    )
}

/// Hold the session: launch `command` inside the sandbox over `workspace` and
/// keep writing the screen it draws into `dir`. The write is also what reads the
/// session's life, since closing removes the directory: a write that no longer
/// lands is the session ending, and this process ends with it, taking the
/// sandbox that dies with it. A key the command line left in `dir` is forwarded
/// to the program on the same pass, because this process holds the terminal the
/// program reads and no other process can reach it.
///
/// The program going is the other end of the session: this process holds a
/// sandbox for a program that is no longer there, so it writes the screen the
/// program left and ends, leaving a session nobody holds for the next launch to
/// reclaim.
///
/// @planks("the operator has expected {string} and pressed {string} in that session")
/// @planks("a session named {string} whose program has exited")
fn hold(command: &[String], workspace: &Path, dir: &Path) -> Result<(), String> {
    let resolved = resolve(std::env::consts::OS).map_err(|e| format!("{e:?}"))?;
    let prepared = resolved.backend().prepare_over_tree(
        &SandboxSpec::default(),
        &CommandSpec {
            program: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), command.join(" ")],
        },
        workspace,
        None,
    )?;
    let mut capture = capture_interactive(&prepared)?;
    let screen = dir.join(SCREEN_FILE);
    let key = dir.join(KEY_FILE);
    while std::fs::write(&screen, capture.screen().contents()).is_ok() {
        if capture.finished() {
            break;
        }
        if key.exists() {
            let pressed = std::fs::read_to_string(&key)
                .map_err(|e| format!("the key waiting for the program was not read: {e}"))?;
            capture.press_key(&pressed);
            std::fs::remove_file(&key)
                .map_err(|e| format!("the key {pressed:?} was not cleared: {e}"))?;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    Ok(())
}

/// Append `action` to the journal of steps the session in `dir` performed. Only
/// a verb that held appends, so the journal is the record a plan is written
/// from rather than a transcript of what was attempted.
///
/// @planks("the operator has expected {string} in that session")
/// @planks("the operator has expected {string} and pressed {string} in that session")
/// @planks("the operator has expected {string} and then expected {string} in that session")
fn performed(dir: &Path, action: &Action) -> Result<(), String> {
    let line = serde_json::to_string(action)
        .map_err(|e| format!("the step {action:?} was not written: {e}"))?;
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join(STEPS_FILE))
        .map_err(|e| format!("the session's journal was not opened: {e}"))?;
    writeln!(journal, "{line}").map_err(|e| format!("the step {line} was not recorded: {e}"))
}

/// State the expectation `expectation` names against the screen the session
/// `name` is showing, waiting for the program to draw it. A session the operator
/// never launched is reported as that rather than launched on their behalf,
/// because which program they meant is a guess.
///
/// An expectation that held is journalled as a step the session performed, so
/// the plan written from the session carries it.
///
/// @planks("the operator executes {string}")
/// @planks("the operator launches a session against a target that prints whether it read {string}")
/// @planks("the operator has expected {string} in that session")
/// @planks("the operator has expected {string} and then expected {string} in that session")
pub fn expect_text(name: &str, expectation: &[String]) -> Result<(), String> {
    let dir = directory(name);
    if !dir.exists() {
        return Err(format!("no session named {name:?} is running"));
    }
    let text = expectation
        .first()
        .ok_or_else(|| "the expectation names nothing".to_string())?;
    awaited(
        EXPECT_DEADLINE,
        || screen_of(&dir).contains(text.as_str()),
        || {
            format!(
                "the expectation of {text:?} found this screen instead:\n{}",
                screen_of(&dir)
            )
        },
    )?;
    performed(
        &dir,
        &Action::Expect(Expectation {
            text: text.clone(),
            within: None,
            locator: None,
        }),
    )
}

/// Send `key` to the program the session `name` is running. The terminal the
/// program reads is held by the session's own process, so the key is left where
/// that process takes it and the press ends when the key has been taken rather
/// than when it was written.
///
/// @planks("the operator executes {string}")
/// @planks("the operator has expected {string} and pressed {string} in that session")
pub fn press(name: &str, key: &str) -> Result<(), String> {
    let dir = directory(name);
    if !dir.exists() {
        return Err(format!("no session named {name:?} is running"));
    }
    let pending = dir.join(PENDING_KEY_FILE);
    std::fs::write(&pending, key).map_err(|e| format!("the key {key:?} was not staged: {e}"))?;
    std::fs::rename(&pending, dir.join(KEY_FILE))
        .map_err(|e| format!("the key {key:?} was not sent: {e}"))?;
    awaited(
        PRESS_DEADLINE,
        || !dir.join(KEY_FILE).exists(),
        || format!("the key {key:?} was not taken before its deadline"),
    )?;
    performed(&dir, &Action::Press(key.to_string()))
}

/// End the session `name`, writing the plan it performed at `output` where the
/// operator asked for one. The plan is read before the reclaim, which removes
/// the directory the journal sits in.
///
/// @planks("the operator executes {string}")
/// @planks("a plan written from a session that expected {string} against {string}")
pub fn close(name: &str, output: Option<&str>, workspace: &Path) -> Result<(), String> {
    let Some(output) = output else {
        return reclaim(name);
    };
    let plan = performed_plan(&directory(name))?;
    reclaim(name)?;
    crate::examples::write_plan(&plan, &workspace.join(output))
}

/// The plan the session in `dir` performed: the program it drove, under the
/// isolation it ran under, carrying the steps that held in the order they held.
/// The session's own name is how the command line addressed it and means
/// nothing to a plan, so no part of the plan carries it.
///
/// @planks("the operator executes {string}")
/// @planks("a plan written from a session that expected {string} against {string}")
fn performed_plan(dir: &Path) -> Result<Plan, String> {
    let command = std::fs::read_to_string(dir.join(COMMAND_FILE))
        .map_err(|e| format!("the session's program was not read: {e}"))?;
    let journal = std::fs::read_to_string(dir.join(STEPS_FILE))
        .map_err(|e| format!("the session's journal was not read: {e}"))?;
    let steps = journal
        .lines()
        .map(|line| {
            serde_json::from_str(line).map_err(|e| format!("the step {line:?} was not read: {e}"))
        })
        .collect::<Result<Vec<Action>, String>>()?;
    Ok(Plan {
        sandbox: SandboxSpec::default(),
        flow: vec![FlowStep::Tui(TuiProcess { command, steps })],
        sources: Vec::new(),
    })
}

/// The sessions standing now, each with the program it drives. A session's
/// directory is named for the session and records the program at launch, so what
/// is running is read off the same files launching a session created.
///
/// @planks("the operator executes {string}")
fn standing() -> Result<Vec<(String, String)>, String> {
    let temp = std::env::temp_dir();
    let entries = std::fs::read_dir(&temp).map_err(|e| {
        format!(
            "the sessions standing in {} were not read: {e}",
            temp.display()
        )
    })?;
    let mut sessions = Vec::new();
    for entry in entries {
        let session = entry
            .map_err(|e| format!("a session standing in {} was not read: {e}", temp.display()))?
            .path();
        let named = session
            .file_name()
            .and_then(|entry| entry.to_str())
            .and_then(|entry| entry.strip_prefix(SESSION_PREFIX))
            .map(str::to_string);
        let Some(name) = named else {
            continue;
        };
        let command = std::fs::read_to_string(session.join(COMMAND_FILE))
            .map_err(|e| format!("the program the session {name:?} drives was not read: {e}"))?;
        sessions.push((name, command));
    }
    Ok(sessions)
}

/// What is running, one session per line against the program it drives. A
/// session outlives the command that launched it, so an operator who has been
/// exploring has no other way to see what they are still holding, and an empty
/// listing says so rather than saying nothing.
///
/// @planks("the operator executes {string}")
/// @planks("the listing names {string} and {string}")
/// @planks("each is listed against the program it drives")
/// @planks("the listing reports no session is running")
pub fn listing() -> Result<String, String> {
    let sessions = standing()?;
    if sessions.is_empty() {
        return Ok("no session is running".to_string());
    }
    Ok(sessions
        .into_iter()
        .map(|(name, command)| format!("{name}\t{command}"))
        .collect::<Vec<String>>()
        .join("\n"))
}

/// End every session standing, reclaiming what each of them created. Listing is
/// what makes purging a decision rather than a guess, so the two read the same
/// sessions.
///
/// @planks("the operator executes {string}")
pub fn close_all() -> Result<(), String> {
    for (name, _) in standing()? {
        reclaim(&name)?;
    }
    Ok(())
}

/// Reclaim what launching the session `name` created. Removing the directory is
/// the reclaim and the signal together: the process holding the session ends
/// when its screen no longer lands, and the sandbox dies with that process, so
/// this returns once the holder is gone.
///
/// @planks("the operator executes {string}")
fn reclaim(name: &str) -> Result<(), String> {
    let dir = directory(name);
    let holder = std::fs::read_to_string(dir.join(HOLDER_FILE))
        .map_err(|_| format!("no session named {name:?} is running"))?;
    std::fs::remove_dir_all(&dir)
        .map_err(|e| format!("the session {name:?} was not reclaimed: {e}"))?;
    let record = Path::new("/proc").join(holder.trim());
    awaited(
        CLOSE_DEADLINE,
        || !record.exists(),
        || format!("the session {name:?} was still held at its closing deadline"),
    )
}
