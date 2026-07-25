//! The prepared process handed to the PTY runner.

/// Everything the PTY runner needs to launch a process. A sandbox backend
/// produces it; the PTY runner consumes it without knowing which backend
/// prepared it.
///
/// @planks("a process is prepared and launched")
/// @planks("the Bubblewrap backend prepares the process")
#[derive(Debug, Clone, serde::Serialize)]
pub struct PreparedProcess {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cleanup: Vec<String>,
}
