//! Backend selection: resolve the platform Tinman runs on into the backend that
//! will run the process, or a clear error.

use crate::bwrap::BubblewrapBackend;

/// The backend selected to run a process, and the backend itself, so a launch
/// path takes what isolates its program from resolution rather than naming one.
///
/// @planks("the resolved backend is {string}")
/// @planks("the verifier checks the backend construction boundary")
#[derive(Debug)]
pub struct ResolvedBackend {
    name: String,
    backend: BubblewrapBackend,
}

impl ResolvedBackend {
    /// The resolved backend's stable name.
    ///
    /// @planks("the resolved backend is {string}")
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// The backend that prepares the process.
    ///
    /// @planks("the verifier checks the backend construction boundary")
    pub fn backend(&self) -> &BubblewrapBackend {
        &self.backend
    }
}

/// Why backend resolution failed. The platform is carried so the failure names
/// what it could not serve.
///
/// @planks("resolution fails with an unsupported-backend error")
/// @planks("the failure names the platform it could not serve")
#[derive(Debug)]
pub enum ResolveError {
    UnsupportedBackend { platform: String },
}

/// Resolve the platform Tinman runs on into the backend that will run the
/// process. Linux is served by Bubblewrap; every other platform is refused,
/// because there is no unsandboxed route.
///
/// @planks("the backend is resolved for that platform")
/// @planks("the verifier checks the backend construction boundary")
pub fn resolve(platform: &str) -> Result<ResolvedBackend, ResolveError> {
    match platform {
        "linux" => Ok(ResolvedBackend {
            name: "bubblewrap".to_string(),
            backend: BubblewrapBackend {
                executable: "bwrap".to_string(),
                environment: std::env::vars().collect(),
            },
        }),
        other => Err(ResolveError::UnsupportedBackend {
            platform: other.to_string(),
        }),
    }
}
