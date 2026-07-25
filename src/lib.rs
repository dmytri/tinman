//! The Tinman library crate.
//!
//! The sandbox abstraction, backend selection, Bubblewrap backend, prepared
//! process, and PTY runner live here. The public seams are exercised by the
//! cucumber suite in `tests/`.

pub mod assistant;
pub mod backend;
pub mod bwrap;
pub mod cli;
pub mod flow;
pub mod help;
pub mod inference;
pub mod plan;
pub mod process;
pub mod pty;
pub mod record;
pub mod sandbox;
pub mod screen;
pub mod skill;
pub mod tom;
pub mod view;
