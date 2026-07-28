//! The curated terminfo database Tinman carries so a sandboxed full-screen
//! program can resolve its own terminal type without reading the host's
//! terminal database, which sits outside the directories the sandbox binds.
//! The whole database is far too large to carry, so what is carried is a
//! curated entry: enough for a program to look up its terminal's capabilities
//! and draw on it.

use std::path::PathBuf;

/// The terminal type name the sandbox exports as `TERM`, and the name the
/// curated entry below is filed under.
pub const TERM: &str = "xterm-256color";

/// The compiled terminfo entry for `TERM`, embedded so the database survives
/// wherever Tinman itself is installed, rather than read from the host.
const XTERM_256COLOR: &[u8] = include_bytes!("terminfo_db/x/xterm-256color");

/// Write the curated entry into a fresh directory and return its root, laid
/// out the way a `TERMINFO` root is read: one subdirectory named after the
/// entry's first character, holding the entry itself.
///
/// @planks("the operator inspects a command that asks terminfo for the terminal width")
/// @planks("the operator inspects {string}")
pub fn materialize() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tinman-terminfo-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    let entry_dir = root.join("x");
    std::fs::create_dir_all(&entry_dir).unwrap_or_else(|e| {
        panic!(
            "curated terminfo directory {} not created: {e}",
            entry_dir.display()
        )
    });
    let entry_path = entry_dir.join(TERM);
    std::fs::write(&entry_path, XTERM_256COLOR).unwrap_or_else(|e| {
        panic!(
            "curated terminfo entry {} not written: {e}",
            entry_path.display()
        )
    });
    root
}
