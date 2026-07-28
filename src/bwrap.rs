//! The Bubblewrap backend: translate a portable sandbox specification into
//! Bubblewrap arguments, and prepare a process for the PTY runner. An
//! unavailable Bubblewrap is a hard failure, never a silent unsandboxed run.

use crate::process::PreparedProcess;
use crate::sandbox::{CommandSpec, EnvOrigin, MountMode, Network, SandboxSpec};
use crate::terminfo;
use std::path::Path;

/// Where the sandboxed program's home directory sits inside the sandbox.
const SANDBOX_HOME: &str = "/sandbox";

/// Where Tinman's curated terminfo database sits inside the sandbox, so a
/// full-screen program can resolve its terminal type without the host's own
/// database, which the sandbox never mounts.
const SANDBOX_TERMINFO: &str = "/terminfo";

/// How a host directory becomes the sandboxed program's home. A workspace the
/// caller provisioned and owns is bound writable, so the caller collects what
/// the program wrote. The operator's own tree is overlaid instead: the program
/// reads it through the overlay's lower layer and every write it makes lands in
/// an invisible tmpfs that goes when the sandbox goes.
#[derive(Debug, Clone, Copy)]
enum HomeMode {
    Writable,
    Overlay,
}

/// The Bubblewrap backend. It holds the name of the `bwrap` executable so the
/// availability check can be exercised against a name that is off PATH.
///
/// @planks("the Bubblewrap executable is absent")
#[derive(Debug, Clone)]
pub struct BubblewrapBackend {
    executable: String,
}

impl Default for BubblewrapBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl BubblewrapBackend {
    /// The Bubblewrap backend using the standard `bwrap` executable.
    ///
    /// @planks("the Bubblewrap backend generates its arguments")
    pub fn new() -> Self {
        Self {
            executable: "bwrap".to_string(),
        }
    }

    /// The Bubblewrap backend pointed at a specific executable name.
    ///
    /// @planks("the Bubblewrap executable is absent")
    pub fn with_executable(executable: String) -> Self {
        Self { executable }
    }

    /// Generate the Bubblewrap argument vector that enforces isolation for the
    /// given specification and command.
    ///
    /// @planks("the Bubblewrap backend generates its arguments")
    /// @planks("a Bubblewrap-prepared process that prints its home directory and the value of {string}")
    /// @planks("a Bubblewrap-prepared process that probes for a network route")
    pub fn generate_args(&self, spec: &SandboxSpec, command: &CommandSpec) -> Vec<String> {
        self.args_with_home(spec, command, None, None)
    }

    /// The same argument vector, with `home` mounted at the sandbox home when
    /// the caller provides one, writable for a workspace the caller owns and
    /// reclaims, overlaid for the operator's own tree. The program starts in
    /// `cwd` under that home when the caller names one, so a command writing a
    /// relative path lands in the directory the plan named.
    ///
    /// @planks("the Tinman driver has a session running {string}")
    /// @planks("a flow whose only step runs {string} in the directory {string}")
    /// @planks("the operator inspects a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator inspects a command that prints the contents of {string}")
    /// @planks("the operator records a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator runs that plan")
    /// @planks("the operator inspects a command that asks terminfo for the terminal width")
    /// @planks("the operator inspects {string}")
    fn args_with_home(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
        home: Option<(&Path, HomeMode)>,
        cwd: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![
            "--unshare-all".to_string(),
            "--clearenv".to_string(),
            "--die-with-parent".to_string(),
        ];
        if spec.network == Network::Deny {
            args.push("--unshare-net".to_string());
        }
        // A fresh procfs, isolated from the host, so the sandboxed program can
        // observe its own namespace. Under an unshared network namespace its
        // routing table is genuinely empty.
        args.push("--proc".to_string());
        args.push("/proc".to_string());
        // Read-only system directories so the sandboxed command and its dynamic
        // loader resolve. Each is mounted read-only, and the host root is never
        // mounted wholesale, so isolation holds.
        for system_path in ["/bin", "/lib", "/lib64"] {
            args.push("--ro-bind".to_string());
            args.push(system_path.to_string());
            args.push(system_path.to_string());
        }
        // A temporary HOME, never the operator's real home. The sandboxed
        // process starts there, so relative paths it writes land in the host
        // directory the caller bound, or in the overlay standing over the
        // operator's tree, rather than in a directory the new mount namespace
        // cannot reach.
        if let Some((home, mode)) = home {
            match mode {
                HomeMode::Writable => {
                    args.push("--bind".to_string());
                    args.push(home.display().to_string());
                    args.push(SANDBOX_HOME.to_string());
                }
                HomeMode::Overlay => {
                    args.push("--overlay-src".to_string());
                    args.push(home.display().to_string());
                    args.push("--tmp-overlay".to_string());
                    args.push(SANDBOX_HOME.to_string());
                }
            }
            args.push("--chdir".to_string());
            args.push(match cwd {
                Some(cwd) => format!("{SANDBOX_HOME}/{cwd}"),
                None => SANDBOX_HOME.to_string(),
            });
        }
        args.push("--setenv".to_string());
        args.push("HOME".to_string());
        args.push(SANDBOX_HOME.to_string());
        // Tinman's own curated terminfo database, bound read-only, so a
        // full-screen program can resolve its terminal type without the host's
        // database, which sits outside the directories the sandbox binds.
        let terminfo_dir = terminfo::materialize();
        args.push("--ro-bind".to_string());
        args.push(terminfo_dir.display().to_string());
        args.push(SANDBOX_TERMINFO.to_string());
        args.push("--setenv".to_string());
        args.push("TERM".to_string());
        args.push(terminfo::TERM.to_string());
        args.push("--setenv".to_string());
        args.push("TERMINFO".to_string());
        args.push(SANDBOX_TERMINFO.to_string());
        // The PATH the plan lists, and nothing else: an unlisted plan grants no
        // PATH beyond what --clearenv already leaves cleared.
        if !spec.path.is_empty() {
            args.push("--setenv".to_string());
            args.push("PATH".to_string());
            args.push(spec.path.join(":"));
        }
        // Named environment variables the plan grants from the host, read fresh
        // so a grant never outlives what the operator's own environment
        // currently holds.
        for (name, grant) in &spec.env {
            match grant.from {
                EnvOrigin::Host => {
                    if let Ok(value) = std::env::var(name) {
                        args.push("--setenv".to_string());
                        args.push(name.clone());
                        args.push(value);
                    }
                }
            }
        }
        // Explicitly mounted fixture trees: read-only unless the plan grants a
        // writable bind or a private writable copy.
        for mount in &spec.mounts {
            let source = match mount.mode {
                MountMode::Copy => writable_copy_of(&mount.source),
                MountMode::Readonly | MountMode::Writable => {
                    std::path::PathBuf::from(&mount.source)
                }
            };
            let flag = match mount.mode {
                MountMode::Readonly => "--ro-bind",
                MountMode::Writable | MountMode::Copy => "--bind",
            };
            args.push(flag.to_string());
            args.push(source.display().to_string());
            args.push(mount.target.clone());
        }
        // The command to run inside the sandbox.
        args.push(command.program.clone());
        args.extend(command.args.iter().cloned());
        args
    }

    /// Prepare a process for the PTY runner. An unavailable Bubblewrap is a hard
    /// failure, so no unsandboxed process is ever prepared.
    ///
    /// @planks("a process is prepared and launched")
    /// @planks("launching fails and reports Bubblewrap is unavailable")
    /// @planks("the Bubblewrap backend prepares the process")
    pub fn prepare(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
    ) -> Result<PreparedProcess, String> {
        self.prepare_with_home(spec, command, None, None)
    }

    /// Prepare a process the same way, giving the sandbox the host directory
    /// `home` as its home, and starting the program in `cwd` under that home
    /// when the caller names one. The caller created that directory and reclaims
    /// it, so the sandbox writes somewhere real and leaves nothing behind.
    ///
    /// @planks("the Tinman driver has a session running {string}")
    /// @planks("the session's temporary sandbox directories no longer exist")
    /// @planks("the fixture program reports a home directory other than the operator's home")
    /// @planks("the step reports a home directory other than the operator's home")
    /// @planks("a flow whose only step runs {string} in the directory {string}")
    pub fn prepare_with_home(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
        home: Option<&Path>,
        cwd: Option<&str>,
    ) -> Result<PreparedProcess, String> {
        self.prepare_home(
            spec,
            command,
            home.map(|home| (home, HomeMode::Writable)),
            cwd,
        )
    }

    /// Prepare a process whose home is an overlay over `tree`: the program reads
    /// the real directory through the overlay's lower layer and every write it
    /// makes lands in an invisible tmpfs that goes when the sandbox goes. This is
    /// what an unfamiliar program gets when the home it is given is the
    /// directory the operator is standing in.
    ///
    /// @planks("the operator inspects a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator inspects a command that prints the contents of {string}")
    /// @planks("the operator records a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator runs that plan")
    pub fn prepare_over_tree(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
        tree: &Path,
        cwd: Option<&str>,
    ) -> Result<PreparedProcess, String> {
        self.prepare_home(spec, command, Some((tree, HomeMode::Overlay)), cwd)
    }

    /// Prepare a process with the home mount the caller chose.
    ///
    /// @planks("a process is prepared and launched")
    /// @planks("the operator inspects a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator records a command that writes {string} into its working directory and prints {string}")
    /// @planks("the operator runs that plan")
    fn prepare_home(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
        home: Option<(&Path, HomeMode)>,
        cwd: Option<&str>,
    ) -> Result<PreparedProcess, String> {
        if !executable_available(&self.executable) {
            return Err("Bubblewrap is unavailable".to_string());
        }
        let args = self.args_with_home(spec, command, home, cwd);
        Ok(PreparedProcess {
            program: self.executable.clone(),
            args,
            env: Vec::new(),
            cwd: None,
            cleanup: Vec::new(),
        })
    }
}

/// A fresh, private directory seeded with `source`'s entries, so a copy mount
/// gives the target a writable copy the original tree never sees a write
/// through.
///
/// @planks("the fixture terminal program writes {string} into {string}")
fn writable_copy_of(source: &str) -> std::path::PathBuf {
    let staging = std::env::temp_dir().join(format!(
        "tinman-copy-mount-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&staging).unwrap_or_else(|e| {
        panic!(
            "copy mount staging directory {} not created: {e}",
            staging.display()
        )
    });
    if let Ok(entries) = std::fs::read_dir(source) {
        for entry in entries.flatten() {
            let _ = std::fs::copy(entry.path(), staging.join(entry.file_name()));
        }
    }
    staging
}

/// Whether an executable name resolves to a real file, either as an absolute
/// path or on PATH.
fn executable_available(executable: &str) -> bool {
    if executable.contains('/') {
        return std::path::Path::new(executable).exists();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(executable).exists())
}
