//! The Tinman library crate.
//!
//! The sandbox abstraction, backend selection, Bubblewrap backend, prepared
//! process, and PTY runner live here. The public seams are exercised by the
//! cucumber suite in `tests/`.

pub mod assistant;
pub mod backend;
pub mod bwrap;
pub mod cli;
pub mod driver;
pub mod examples;
pub mod flow;
pub mod help;
pub mod inference;
pub mod inspect;
pub mod plan;
pub mod process;
pub mod pty;
pub mod record;
pub mod sandbox;
pub mod screen;
pub mod setup;
pub mod skill;
pub mod terminfo;
pub mod tom;
pub mod view;

/// The wall-clock ceiling the named timed seam enforces, absent where no seam
/// of that name enforces one. The latency contract declares a ceiling per seam
/// and this reports the ceiling production holds that seam to, so the two are
/// compared rather than a wait being driven out to measure it.
///
/// @planks("the ceiling each seam enforces is read")
pub fn enforced_ceiling(seam: &str) -> Option<std::time::Duration> {
    match seam {
        "tagline generation" => Some(inference::TAGLINE_CEILING),
        "inference availability probe" => Some(inference::PROBE_CEILING),
        "assistant answer generation" => Some(inference::GENERATION_CEILING),
        "inspect exit deadline" => Some(inspect::EXIT_DEADLINE),
        _ => None,
    }
}
