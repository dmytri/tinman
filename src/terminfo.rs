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

/// Write the curated entry into the shared database directory and return its
/// root, laid out the way a `TERMINFO` root is read: one subdirectory named
/// after the entry's first character, holding the entry itself.
///
/// The database is a constant, so one directory named after the entry serves
/// every launch, every run and every process. A directory named after the run
/// that made it would be a resource nothing can reclaim: the sandbox binds it
/// for the life of the launched program, and the paths that launch one do not
/// all reclaim what the launch staged, so a per-launch directory outlives its
/// process and they accumulate until the disk answers for them.
///
/// The entry is written under a name of this process's own and renamed onto the
/// entry path, which is atomic, so a process writing the database never shows a
/// concurrently launching process a half-written entry.
///
/// @planks("the operator inspects a command that asks terminfo for the terminal width")
/// @planks("the operator inspects {string}")
pub fn materialize() -> PathBuf {
    let root = std::env::temp_dir().join(format!("tinman-terminfo-{TERM}"));
    let entry_dir = root.join("x");
    std::fs::create_dir_all(&entry_dir).unwrap_or_else(|e| {
        panic!(
            "curated terminfo directory {} not created: {e}",
            entry_dir.display()
        )
    });
    let staged = entry_dir.join(format!("{TERM}.{}", std::process::id()));
    std::fs::write(&staged, XTERM_256COLOR).unwrap_or_else(|e| {
        panic!(
            "curated terminfo entry {} not written: {e}",
            staged.display()
        )
    });
    let entry_path = entry_dir.join(TERM);
    std::fs::rename(&staged, &entry_path).unwrap_or_else(|e| {
        panic!(
            "curated terminfo entry {} not renamed into place: {e}",
            entry_path.display()
        )
    });
    root
}
