//! The backend-neutral sandbox vocabulary: the command to launch, the requested
//! backend identity, the network policy, and the portable sandbox
//! specification.

/// A backend-neutral network policy for the sandbox.
///
/// @planks("a sandbox specification that denies network access")
/// @planks("the parsed sandbox specification denies network access")
/// @planks("a sandbox specification setting network to {string}")
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Allow,
    #[default]
    Deny,
    Proxy,
}

/// The egress policy read only where a sandbox specification's network is
/// `proxy`: the HAR file the proxy records to or replays from, and the fault
/// it injects into every exchange.
///
/// @planks("the proxy is recording to a HAR file")
/// @planks("a HAR file carrying one recorded exchange")
/// @planks("the same request is made")
/// @planks("a request the recording does not carry is made")
#[derive(Debug, Clone)]
pub struct Egress {
    pub har: Option<String>,
    pub fault: Option<Fault>,
}

/// A fault the proxy injects into every exchange instead of reaching the
/// upstream, so a condition the real provider will not produce on demand can
/// be tested.
///
/// @planks("the proxy is injecting the status {int}")
/// @planks("the proxy is injecting a latency of {int} seconds")
/// @planks("the proxy is injecting the offline fault")
#[derive(Debug, Clone)]
pub struct Fault {
    pub status: Option<u16>,
    pub latency: Option<String>,
    pub offline: bool,
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
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Auto,
    Bubblewrap,
    Mac,
}

/// How the sandbox home directory is provisioned.
///
/// @planks("the specification is serialized")
/// @planks("the parsed sandbox specification provisions an empty home")
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Home {
    #[default]
    Empty,
}

/// How an explicitly mounted fixture tree is exposed to the target.
///
/// @planks("the mount's mode is {string}")
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountMode {
    #[default]
    Readonly,
    Copy,
    Writable,
}

impl MountMode {
    /// The mode's name, as the plan writes it.
    ///
    /// @planks("the mount's mode is {string}")
    pub fn as_name(&self) -> &'static str {
        match self {
            MountMode::Readonly => "readonly",
            MountMode::Copy => "copy",
            MountMode::Writable => "writable",
        }
    }
}

/// An explicitly mounted fixture tree. A mount the plan gives no mode is
/// read-only, so a grant is never wider than the plan states.
///
/// @planks("the mount's mode is {string}")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub mode: MountMode,
}

/// Where a sandboxed environment variable's value comes from.
///
/// @planks("a process is prepared and launched")
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvOrigin {
    Host,
}

/// A named environment variable the plan grants the sandboxed process, and
/// where its value comes from. A variable the plan never names never reaches
/// the target, however the operator's own environment carries it.
///
/// @planks("a process is prepared and launched")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvGrant {
    pub from: EnvOrigin,
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
            _ => None,
        }
    }
}

/// The portable, backend-neutral sandbox specification.
///
/// @planks("a sandbox specification that denies network access")
/// @planks("the specification is serialized")
/// @planks("the parsed sandbox specification names no Bubblewrap flag")
/// @planks("the parsed sandbox specification denies network access")
/// @planks("the parsed sandbox specification provisions an empty home")
/// @planks("a process is prepared and launched")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SandboxSpec {
    pub home: Home,
    pub network: Network,
    pub mounts: Vec<Mount>,
    pub env: std::collections::BTreeMap<String, EnvGrant>,
    pub path: Vec<String>,
}

impl SandboxSpec {
    /// The default sandbox specification for a recorded run: an empty home and
    /// network denied, so isolation is the starting point. The backend is
    /// Tinman's choice, so the specification carries none.
    ///
    /// @planks("a sandbox specification that denies network access")
    /// @planks("the default sandbox specification for {string}")
    pub fn default_for_record() -> SandboxSpec {
        SandboxSpec {
            home: Home::Empty,
            network: Network::Deny,
            mounts: Vec::new(),
            env: std::collections::BTreeMap::new(),
            path: Vec::new(),
        }
    }
}
