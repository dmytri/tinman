//! The backend-neutral sandbox vocabulary: the command to launch, the requested
//! backend identity, the network policy, and the portable sandbox
//! specification.

/// A backend-neutral network policy for the sandbox.
///
/// @planks("a sandbox specification that denies network access")
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Allow,
    Deny,
}

/// The program a harness launches, with its arguments.
///
/// @planks("the capture target program is {string}")
/// @planks("the capture target arguments are {string} and {string}")
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

/// A backend identity the operator can request.
///
/// @planks("the requested backend is {string}")
#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Auto,
    Bubblewrap,
    Mac,
    None,
}

/// How the sandbox home directory is provisioned.
///
/// @planks("the specification is serialized")
#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Home {
    Empty,
}

impl Backend {
    /// Parse a backend name from the harness or CLI.
    ///
    /// @planks("the requested backend is {string}")
    pub fn from_name(name: &str) -> Option<Backend> {
        match name {
            "auto" => Some(Backend::Auto),
            "bubblewrap" => Some(Backend::Bubblewrap),
            "mac" => Some(Backend::Mac),
            "none" => Some(Backend::None),
            _ => None,
        }
    }
}

/// The portable, backend-neutral sandbox specification.
///
/// @planks("a sandbox specification that denies network access")
/// @planks("the specification is serialized")
#[derive(Debug, Clone, serde::Serialize)]
pub struct SandboxSpec {
    pub backend: Backend,
    pub home: Home,
    pub network: Network,
}

impl SandboxSpec {
    /// The default sandbox specification for a recorded run: auto backend, an
    /// empty home, and network denied, so isolation is the starting point.
    ///
    /// @planks("a sandbox specification that denies network access")
    /// @planks("the default sandbox specification for {string}")
    pub fn default_for_record() -> SandboxSpec {
        SandboxSpec {
            backend: Backend::Auto,
            home: Home::Empty,
            network: Network::Deny,
        }
    }
}
