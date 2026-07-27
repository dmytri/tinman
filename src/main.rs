//! The `tinman` binary.

use clap::Parser;
use std::io::IsTerminal;
use tinman::cli::{Cli, Command};

/// Parse the command line and run the command it names. The help flag and the
/// help subcommand render the help, one behaviour reached two ways: an operator
/// on a terminal sees the tagline line filled; anything
/// else sees the conventional help, which needs no model, credential or network.
/// A configured credential opens the assistant beneath the help, so a provider
/// that withholds its answer still gets a prompt rather than a bare screen.
/// A plan whose step fails names the step and leaves a failure status.
///
/// @planks("the operator runs {string} with stdout redirected to a file")
/// @planks("the operator runs {string} in an interactive terminal")
/// @planks("the operator tests that plan")
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the fixture terminal program as JSON")
/// @planks("the operator inspects the command {string}")
/// @planks("the operator inspects a command that writes to the sentinel path and prints {string}")
/// @planks("the operator records a command that writes to the sentinel path and prints {string}")
/// @planks("the operator records a command that prints {string} and the value of {string}")
/// @planks("the operator records the command {string} and presses {string}")
/// @planks("the operator records the command {string}")
/// @planks("the operator records the command {string} with {string}")
/// @planks("the operator records the fixture terminal program")
/// @planks("the operator records that program")
/// @planks("the Tinman driver is running")
fn main() {
    let cli = Cli::parse();
    if cli.help || matches!(cli.command, Some(Command::Help)) {
        if std::io::stdout().is_terminal() {
            let settings = tinman::inference::Settings::from_process();
            let expansion = tinman::inference::tagline_expansion(&settings);
            println!("{}", tinman::help::interactive(expansion.as_deref()));
            if settings.api_key.is_some() {
                tinman::assistant::converse(&settings);
            }
        } else {
            println!("{}", tinman::help::conventional());
        }
    }
    match cli.command {
        Some(Command::Record { output, command }) => {
            let mut tokens = command.into_iter();
            let spec = tinman::sandbox::CommandSpec {
                program: tokens.next().expect("the record command names a program"),
                args: tokens.collect(),
            };
            let workspace = std::env::current_dir().expect("the working directory is read");
            if let Err(failure) = tinman::record::record(&spec, &workspace, output.as_deref()) {
                println!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Test { plan }) => {
            let source = std::fs::read_to_string(&plan)
                .unwrap_or_else(|e| panic!("the plan {} was not read: {e}", plan.display()));
            let plan = tinman::plan::parse(&source)
                .unwrap_or_else(|e| panic!("the plan did not parse: {e}"));
            let workspace = std::env::current_dir().expect("the working directory is read");
            if let Err(failure) = tinman::flow::execute(&plan, &workspace, None) {
                println!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Inspect { command, json }) => {
            let workspace = std::env::current_dir().expect("the working directory is read");
            let model = tinman::inspect::model(&command, &workspace)
                .unwrap_or_else(|e| panic!("the inspected command did not run: {e}"));
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&model).expect("the model is written as JSON")
                );
            } else {
                println!("{}", tinman::inspect::render(&model));
            }
        }
        Some(Command::Driver) => tinman::driver::serve(),
        Some(Command::Help) | None => {}
    }
}
