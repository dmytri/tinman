//! The `tinman` binary.

use clap::{CommandFactory, Parser};
use std::io::{IsTerminal, Write};
use tinman::cli::{Cli, Command};

/// Parse the command line and run the command it names. The help flag and the
/// help subcommand render the help, one behaviour reached two ways: an operator
/// on a terminal sees the tagline line filled; anything
/// else sees the conventional help, which needs no model, credential or network.
/// A configured credential opens the assistant beneath the help, so a provider
/// that withholds its answer still gets a prompt rather than a bare screen.
/// A command line naming nothing at all is a question, and the assistant answers
/// it, on the two streams that say an operator is standing there: a terminal on
/// output and a terminal on input, with a credential to answer through. A file
/// on either stream is a program being fed rather than an operator waiting, and
/// that gets the conventional help. An operator waiting with no credential gets
/// the conventional help and the setup form beneath it, which is the one useful
/// thing an assistant that cannot reach a model has to say.
/// A plan whose step fails names the step and leaves a failure status.
///
/// @planks("the operator has opened the setup form")
/// @planks("the operator runs {string} with stdout redirected to a file")
/// @planks("it carries the one-line description from the asset at {string}")
/// @planks("the DESCRIPTION section carries the closing paragraph of the asset at {string}")
/// @planks("each is requested from {string}")
/// @planks("the operator runs {string} in an interactive terminal")
/// @planks("the operator runs {string} in an interactive terminal with stdin redirected from a file")
/// @planks("the operator executes {string}")
/// @planks("a session named {string} running {string}")
/// @planks("the operator launches a session against a target that prints whether it read {string}")
/// @planks("the operator states an expectation against a target that prints whether it read {string}")
/// @planks("the operator has expected {string} and pressed {string} in that session")
/// @planks("a plan written from a session that expected {string} against {string}")
/// @planks("the operator tests that plan")
/// @planks("the operator tests that plan with the streams captured separately")
/// @planks("the operator tests the plan {string}")
/// @planks("the operator runs that plan")
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the fixture terminal program as JSON")
/// @planks("the operator inspects the command {string}")
/// @planks("the operator inspects {string}")
/// @planks("the operator inspects that command with the streams captured separately")
/// @planks("the operator inspects {string} with its documented examples")
/// @planks("the operator inspects {string} with its documented examples and writes a plan")
/// @planks("the operator inspects the fixture terminal program with its documented examples")
/// @planks("the operator inspects the fixture terminal program with its documented examples and writes a plan")
/// @planks("the operator inspects a command that writes to the sentinel path and prints {string}")
/// @planks("the operator records a command that writes to the sentinel path and prints {string}")
/// @planks("the operator records a command that prints {string} and the value of {string}")
/// @planks("the operator records the command {string} and presses {string}")
/// @planks("the operator records the command {string}")
/// @planks("the operator records the command {string} with {string}")
/// @planks("the operator records the fixture terminal program")
/// @planks("the operator records that program")
/// @planks("the operator records {string} to that file with the streams captured separately")
/// @planks("the Tinman driver is running")
/// @planks("the operator starts the driver")
/// @planks("the error stream reports Tinman has no {string} command")
/// @planks("each is asked for help")
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Some(page) = tinman::cli::command_help(&arguments) {
        print!("{page}");
        return;
    }
    let cli = Cli::parse();
    if cli.help || matches!(cli.command, Some(Command::Help)) {
        if std::io::stdout().is_terminal() {
            let settings = tinman::inference::Settings::from_process();
            // The generation runs while the help is drawn, and the drawn help
            // is written over once the tagline settles, so the Commands block
            // is on the screen whatever the provider is doing.
            let tagline = tinman::inference::pending_tagline(&settings);
            let columns = crossterm::terminal::size()
                .expect("the terminal reports its size")
                .0;
            print!("{}", tinman::help::drawn_ahead(columns));
            std::io::stdout()
                .flush()
                .expect("the drawn help reaches the terminal");
            println!(
                "{}",
                tinman::help::interactive(tagline.settled().as_deref())
            );
            if settings.api_key.is_some() {
                tinman::assistant::converse(&settings).expect("the terminal takes the assistant");
            }
        } else {
            println!("{}", tinman::help::conventional());
        }
    } else if cli.command.is_none() {
        let settings = tinman::inference::Settings::from_process();
        let waiting = std::io::stdout().is_terminal() && std::io::stdin().is_terminal();
        if waiting && settings.api_key.is_some() {
            tinman::assistant::converse(&settings).expect("the terminal takes the assistant");
        } else {
            println!("{}", tinman::help::conventional());
            if waiting {
                tinman::setup::form(&settings).expect("the terminal takes the setup form");
            }
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
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Test { plan }) => {
            let source = match std::fs::read_to_string(&plan) {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("the plan {} was not read: {e}", plan.display());
                    std::process::exit(1);
                }
            };
            let plan = match tinman::plan::parse(&source) {
                Ok(plan) => plan,
                Err(e) => {
                    eprintln!("the plan did not parse: {e}");
                    std::process::exit(1);
                }
            };
            let workspace = std::env::current_dir().expect("the working directory is read");
            if let Err(failure) = tinman::flow::execute_over_tree(&plan, &workspace, None) {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Inspect {
            command,
            json,
            examples,
            output,
        }) => {
            if let Err(failure) = tinman::bwrap::require_available() {
                eprintln!("{failure}");
                std::process::exit(1);
            }
            let workspace = std::env::current_dir().expect("the working directory is read");
            if examples {
                // The page source is configuration, so a probe reads it the same
                // way every other configured endpoint is read.
                let settings = tinman::inference::Settings::from_process();
                match tinman::examples::probe(&command, &workspace, &settings.tldr_base_url) {
                    Ok(probe) => {
                        println!("{}", probe.listing);
                        if let Some(path) = output
                            && let Err(failure) =
                                tinman::examples::write_plan(&probe.plan, &workspace.join(path))
                        {
                            eprintln!("{failure}");
                            std::process::exit(1);
                        }
                    }
                    Err(failure) => {
                        eprintln!("{failure}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            // Inspection is capture time, where the configured engine names
            // what the deterministic reading cannot, so the command resolves
            // the credential the same way every other configured endpoint is
            // resolved.
            let settings = tinman::inference::Settings::from_process();
            match tinman::inspect::inspected(&command, &workspace, &settings) {
                Ok(launched) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&launched.model)
                                .expect("the model is written as JSON")
                        );
                    } else {
                        println!("{}", tinman::inspect::render(&launched.model));
                    }
                }
                Err(failure) => {
                    eprintln!("{failure}");
                    std::process::exit(1);
                }
            }
        }
        Some(Command::Expect {
            role,
            name,
            session,
            output,
            after,
            conforms,
            expectation,
        }) => {
            let workspace = std::env::current_dir().expect("the working directory is read");
            // A named session is the program the expectation is stated against,
            // so the verb reads the screen that session is showing rather than
            // launching a program of its own.
            let stated = match &session {
                Some(session) => tinman::session::expect_text(session, &expectation),
                None => tinman::expect::state(
                    &expectation,
                    role.as_deref(),
                    name.as_deref(),
                    output.as_deref(),
                    after.as_deref(),
                    conforms.as_deref(),
                    &workspace,
                ),
            };
            if let Err(failure) = stated {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Launch { session, command }) => {
            let workspace = std::env::current_dir().expect("the working directory is read");
            if let Err(failure) = tinman::session::launch(&command, &session, &workspace) {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Sessions) => match tinman::session::listing() {
            Ok(listing) => println!("{listing}"),
            Err(failure) => {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        },
        Some(Command::Close {
            session,
            all: _,
            output,
        }) => {
            let workspace = std::env::current_dir().expect("the working directory is read");
            // The parser takes a named session or every session and refuses
            // both, so a close naming no session is the purge.
            let closed = match session {
                Some(session) => tinman::session::close(&session, output.as_deref(), &workspace),
                None => tinman::session::close_all(),
            };
            if let Err(failure) = closed {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Press { session, key }) => {
            if let Err(failure) = tinman::session::press(&session, &key) {
                eprintln!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Driver) => {
            if let Err(failure) = tinman::bwrap::require_available() {
                eprintln!("{failure}");
                std::process::exit(1);
            }
            tinman::driver::serve().expect("the driver answers its client")
        }
        Some(Command::Man { command }) => {
            // Tinman's own page is always rendered, so a name is the only thing
            // that can leave the parser without a page to write.
            let Some(page) = tinman::cli::manual_page(command.as_ref()) else {
                let name = command.expect("only a named command has no page");
                eprintln!("tinman has no {name:?} command");
                std::process::exit(1);
            };
            print!("{page}");
        }
        Some(Command::Completions { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "tinman", &mut std::io::stdout());
        }
        Some(Command::Help) | None => {}
    }
}
