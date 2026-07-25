//! Backend selection: resolve a requested backend identity into the backend
//! that will run the process, or a clear error.

use crate::sandbox::Backend;

/// The backend selected to run a process.
///
/// @planks("the resolved backend is {string}")
#[derive(Debug)]
pub struct ResolvedBackend {
    name: String,
}

impl ResolvedBackend {
    /// The resolved backend's stable name.
    ///
    /// @planks("the resolved backend is {string}")
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// Why backend resolution failed.
///
/// @planks("resolution fails with an unsupported-backend error")
/// @planks("resolution fails and reports that the unsafe option is required")
#[derive(Debug)]
pub enum ResolveError {
    UnsupportedBackend {},
    UnsafeRequired,
}

/// Resolve a requested backend into the backend that will run the process.
/// Sandboxed execution is the default; the unsafe local backend requires an
/// explicit opt-in.
///
/// @planks("the backend is resolved on Linux")
/// @planks("the backend is resolved")
/// @planks("the backend is resolved without the unsafe option")
pub fn resolve(requested: Backend, allow_unsafe: bool) -> Result<ResolvedBackend, ResolveError> {
    match requested {
        Backend::Auto | Backend::Bubblewrap => Ok(ResolvedBackend {
            name: "bubblewrap".to_string(),
        }),
        Backend::Mac => Err(ResolveError::UnsupportedBackend {}),
        Backend::None => {
            if allow_unsafe {
                Ok(ResolvedBackend {
                    name: "none".to_string(),
                })
            } else {
                Err(ResolveError::UnsafeRequired)
            }
        }
    }
}
