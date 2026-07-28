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
/// @planks("the operator tests that plan")
/// @planks("the operator runs that plan")
/// @planks("the operator inspects the fixture terminal program")
/// @planks("the operator inspects the fixture terminal program as JSON")
/// @planks("the operator inspects the command {string}")
/// @planks("the operator inspects {string}")
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
            if let Err(failure) = tinman::flow::execute_over_tree(&plan, &workspace, None) {
                println!("{failure}");
                std::process::exit(1);
            }
        }
        Some(Command::Inspect { command, json }) => {
            let workspace = std::env::current_dir().expect("the working directory is read");
            match tinman::inspect::model(&command, &workspace) {
                Ok(model) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&model)
                                .expect("the model is written as JSON")
                        );
                    } else {
                        println!("{}", tinman::inspect::render(&model));
                    }
                }
                Err(failure) => {
                    println!("{failure}");
                    std::process::exit(1);
                }
            }
        }
        Some(Command::Driver) => tinman::driver::serve().expect("the driver answers its client"),
        Some(Command::Man { command }) => {
            // A doc comment on a command type is written for a developer, and
            // the operator reading the page wants the copy the help asset
            // carries, so the page's own prose is joined to the asset rather
            // than to whatever the compiler happens to read.
            let mut root = Cli::command()
                .about(tinman::help::one_line_description())
                .long_about(tinman::help::closing_paragraph());
            root.build();
            // A cross-reference is a claim that the page exists, so every
            // `tinman-<command>(1)` the page prints is one the binary writes.
            let man = match &command {
                Some(name) => clap_mangen::Man::new(
                    root.find_subcommand(name)
                        .unwrap_or_else(|| panic!("tinman has no {name:?} command"))
                        .clone(),
                )
                .title(format!("tinman-{name}")),
                None => clap_mangen::Man::new(root),
            };
            let mut page = Vec::new();
            man.render(&mut page).expect("the manual page is rendered");
            let page = String::from_utf8(page).expect("the manual page is text");
            // The doc comments clap renders into this page are where the trace
            // annotations live, so an annotation line is dropped along with the
            // paragraph macro that opens it, leaving the operator the prose.
            let mut kept: Vec<&str> = Vec::new();
            for line in page.lines() {
                if line.contains("@planks") {
                    if kept.last() == Some(&".PP") {
                        kept.pop();
                    }
                    continue;
                }
                kept.push(line);
            }
            let page = format!("{}\n", kept.join("\n"));
            // A man page opens with its title macro, and the roff writer opens
            // with its own apostrophe preamble, which the page still needs for
            // the apostrophes it carries. The title leads and the preamble
            // follows it.
            let title = page
                .find(".TH")
                .expect("the rendered page carries a title macro");
            let (preamble, body) = page.split_at(title);
            let (title_line, rest) = body
                .split_once('\n')
                .expect("the title macro ends its line");
            print!("{title_line}\n{preamble}{rest}");
        }
        Some(Command::Completions { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "tinman", &mut std::io::stdout());
        }
        Some(Command::Help) | None => {}
    }
}
