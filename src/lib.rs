//! The Tinman library crate.
//!
//! The sandbox abstraction, backend selection, Bubblewrap backend, prepared
//! process, and PTY runner live here. The public seams are exercised by the
//! cucumber suite in `tests/`.

pub mod backend;
pub mod bwrap;
pub mod process;
pub mod pty;
pub mod record;
pub mod sandbox;
pub mod screen;
pub mod view;
