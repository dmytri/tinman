//! The `tinman` binary.

use clap::Parser;
use std::io::IsTerminal;
use tinman::cli::Cli;

/// Parse the command line and render the help when the operator asks for it. An
/// operator on a terminal sees the tagline line filled; anything else sees the
/// conventional help, which needs no model, credential or network.
///
/// @planks("the operator runs {string} with stdout redirected to a file")
/// @planks("the operator runs {string} in an interactive terminal")
fn main() {
    let cli = Cli::parse();
    if cli.help {
        if std::io::stdout().is_terminal() {
            let settings = tinman::inference::Settings::from_process();
            println!("{}", tinman::help::interactive(&settings));
        } else {
            println!("{}", tinman::help::conventional());
        }
    }
}
