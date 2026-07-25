//! The Bubblewrap backend: translate a portable sandbox specification into
//! Bubblewrap arguments, and prepare a process for the PTY runner. An
//! unavailable Bubblewrap is a hard failure, never a silent unsandboxed run.

use crate::process::PreparedProcess;
use crate::sandbox::{CommandSpec, Network, SandboxSpec};

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
        // A temporary HOME, never the operator's real home.
        args.push("--setenv".to_string());
        args.push("HOME".to_string());
        args.push("/sandbox".to_string());
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
        if !executable_available(&self.executable) {
            return Err("Bubblewrap is unavailable".to_string());
        }
        let args = self.generate_args(spec, command);
        Ok(PreparedProcess {
            program: self.executable.clone(),
            args,
            env: Vec::new(),
            cleanup: Vec::new(),
        })
    }
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
