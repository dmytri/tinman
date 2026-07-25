//! Cucumber-rs runner for Clanker's dev rigging.
//!
//! Shipshape dev-rigging note: our own verification is real-by-default. Steps
//! drive real Clanker seams (real PTY, real ANSI parse, real render). This is
//! distinct from Clanker's product mandate, which is itself to drive *real*
//! TUIs (real coding agents) with no mocks. Both layers exercise real behaviour.

use cucumber::World;

/// Shared scenario state. Fields are added by the Quartermaster as step
/// definitions require them. Kept empty at bootstrap so the runner compiles
/// and executes before any behaviour is specified.
#[derive(Debug, Default, World)]
struct ClankerWorld;

#[tokio::main]
async fn main() {
    // `fail_on_skipped` makes undefined or unimplemented steps redden, so a
    // missing step definition is a failing verification target the QM can see.
    ClankerWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("features")
        .await;
}
