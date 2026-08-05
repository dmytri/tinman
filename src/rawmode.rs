//! The terminal's raw mode, held by a guard that gives it back.
//!
//! A program that puts the terminal in raw mode owns giving it back however it
//! leaves. Written as a pair of calls around the work, the restoring call runs
//! on the ordinary path only, and every fallible operation between the two is an
//! early return that skips it: the operator meets a terminal with no prompt, no
//! cursor and no echo. Acquisition lives here instead, in a guard whose drop
//! restores, so every path out restores by construction.

/// The terminal held in raw mode for as long as this guard stands. Dropping it
/// gives the terminal back, on the ordinary path and on an early return alike.
///
/// @planks("the terminal is out of raw mode")
pub struct RawMode;

impl RawMode {
    /// Put the terminal in raw mode and report the guard that gives it back.
    ///
    /// @planks("the terminal is out of raw mode")
    pub fn enter() -> std::io::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
