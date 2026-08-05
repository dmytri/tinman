//! The setup form: what an assistant with no credential has to offer, which is
//! how to give it one.
//!
//! The form needs no model to run. It shows the endpoint and the model a run
//! would address, names the environment variable it reads, and takes the
//! credential masked, writing it where a secret belongs rather than beside the
//! operator's work.

use crate::assistant::{
    BACKSPACE, BOTTOM_LEFT, BOTTOM_RIGHT, END_OF_INPUT, ESCAPE, MAX_COLUMNS, TOP_LEFT, TOP_RIGHT,
};
use crate::inference::Settings;
use crate::tom::{HORIZONTAL, VERTICAL};
use std::io::{Read, Write};

/// The setup asset, inlined at build time: its first line titles the form, its
/// second names the keys that save and leave, and its third names the
/// environment variable the form also reads the credential from. The form reads
/// its copy from this asset each time it draws, so an edited asset is a live
/// reading rather than a hand-kept copy.
const FORM: &str = include_str!("../assets/help/setup-form.txt");

/// The label naming the field the credential is typed into. The colon before
/// the field is what says the underscores after it are an input rather than a
/// rule. Its value ends in a space the field's width calculation counts, so it
/// stays a constant rather than a line of the asset, where a trailing space is
/// invisible in every editor that trims on save.
const LABEL: &str = "key: ";

/// The form's own copy, read from the setup asset: the title on its first line,
/// the key hints on its second, and the environment sentence on its third, the
/// order assets/help/setup-form.txt carries them in.
///
/// @planks("the form title is the title the setup asset carries")
fn form_copy() -> (String, String, String) {
    let mut lines = FORM.trim().lines();
    let title = lines.next().unwrap_or_default().to_string();
    let hint = lines.next().unwrap_or_default().to_string();
    let environment = lines.next().unwrap_or_default().to_string();
    (title, hint, environment)
}

/// The cell the credential field is drawn with. The field shows its own width
/// and never the key, so the characters on the screen carry no secret.
const MASK: &str = "_";

/// The escapes that draw the credential field concealed and put the terminal's
/// own presentation back after it. Concealed is what a terminal calls a
/// password field, so the field says what it is rather than looking ordinary.
///
/// @planks("the credential field is hidden")
const HIDDEN: &str = "\u{1b}[8m";
const PLAIN: &str = "\u{1b}[0m";

/// The file the saved credential is written to, under the configuration
/// directory the operator's environment names. The project directory is the
/// wrong home for a secret, since a file written beside the operator's work is
/// a file their next commit can carry.
///
/// @planks("the credential is written under the configuration directory")
/// @planks("no credential is written to the operator's working directory")
fn credential_path() -> std::path::PathBuf {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(configured) => std::path::PathBuf::from(configured),
        None => std::path::PathBuf::from(
            std::env::var_os("HOME").expect("the operator's home directory is named"),
        )
        .join(".config"),
    };
    base.join("tinman").join(".env")
}

/// Write `key` under the configuration directory, in the dotenv form the other
/// route already reads. The file is created readable by its owner alone, at
/// creation rather than after it, so the secret is never on disk unprotected.
///
/// @planks("the operator saves a key through the form")
/// @planks("the credential is written under the configuration directory")
/// @planks("that file is readable only by its owner")
/// @planks("no credential is written to the operator's working directory")
fn save(key: &str) {
    use std::os::unix::fs::OpenOptionsExt;

    let path = credential_path();
    let directory = path
        .parent()
        .expect("the credential path names a directory");
    std::fs::create_dir_all(directory).unwrap_or_else(|e| {
        panic!(
            "the configuration directory {} was not created: {e}",
            directory.display()
        )
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
        .unwrap_or_else(|e| panic!("the credential file {} was not opened: {e}", path.display()));
    writeln!(file, "TINMAN_API_KEY={key}").expect("the credential reaches its file");
}

/// The lines the form is drawn from, each at most `width` cells wide: a box
/// carrying the endpoint and model a run would address and the environment
/// variable it reads, and beneath it the field the credential is typed into.
/// The field sits below the box because the cursor rests in it, and a box
/// holding the cursor is the whole field to a reader rather than the panel it
/// is.
///
/// @planks("a region titled {string} is drawn")
/// @planks("the form offers {string} as the endpoint")
/// @planks("the form offers {string} as the model")
/// @planks("the form names {string} as an environment variable it reads")
/// @planks("the credential field is hidden")
fn form_lines(width: usize, settings: &Settings) -> Vec<String> {
    let (title, hint, environment) = form_copy();
    let inner = width - 2;
    let mut lines = Vec::new();
    let rule = HORIZONTAL.repeat(inner - title.chars().count());
    lines.push(format!("{TOP_LEFT}{title}{rule}{TOP_RIGHT}"));
    for body in [
        format!("endpoint  {}", settings.base_url),
        format!("model     {}", settings.model),
        environment,
    ] {
        let shown: String = body.chars().take(inner).collect();
        let padding = " ".repeat(inner - shown.chars().count());
        lines.push(format!("{VERTICAL}{shown}{padding}{VERTICAL}"));
    }
    let rule = HORIZONTAL.repeat(inner - hint.chars().count());
    lines.push(format!("{BOTTOM_LEFT}{hint}{rule}{BOTTOM_RIGHT}"));
    let field = MASK.repeat(width - LABEL.chars().count());
    lines.push(format!("{LABEL}{HIDDEN}{field}{PLAIN}"));
    lines
}

/// Draw `lines` where the cursor stands and leave the cursor at the head of the
/// credential field, which is where the next character the operator types
/// belongs.
///
/// @planks("a region titled {string} is drawn")
/// @planks("the operator has opened the setup form")
fn draw(out: &mut impl Write, lines: &[String]) -> std::io::Result<()> {
    write!(
        out,
        "{}\r\u{1b}[{}C",
        lines.join("\r\n"),
        LABEL.chars().count()
    )?;
    out.flush()
}

/// Draw the setup form beneath the help and take the credential the operator
/// types, until they send it or leave. The terminal is held in raw mode while
/// the form stands, so the key the operator types is never echoed and the
/// screen carries the field alone. Enter saves the key and ends the form;
/// escape and the end of the input leave it unsaved.
///
/// @planks("a region titled {string} is drawn")
/// @planks("no region titled {string} is drawn")
/// @planks("the form offers {string} as the endpoint")
/// @planks("the form offers {string} as the model")
/// @planks("the form names {string} as an environment variable it reads")
/// @planks("the operator has opened the setup form")
/// @planks("the operator types a key into the credential field")
/// @planks("the credential field is hidden")
/// @planks("the operator saves a key through the form")
pub fn form(settings: &Settings) -> std::io::Result<()> {
    let columns = crossterm::terminal::size()?.0 as usize;
    let width = columns.min(MAX_COLUMNS);
    let mut out = std::io::stdout();
    let raw = crate::rawmode::RawMode::enter()?;
    draw(&mut out, &form_lines(width, settings))?;
    let mut input = std::io::stdin();
    let mut byte = [0u8; 1];
    let mut key: Vec<u8> = Vec::new();
    loop {
        let read = input.read(&mut byte)?;
        if read == 0 || END_OF_INPUT.contains(&byte[0]) || byte[0] == ESCAPE {
            break;
        }
        if byte[0] == b'\r' {
            let typed = std::mem::take(&mut key);
            save(&String::from_utf8(typed).expect("the credential is text"));
            break;
        }
        if byte[0] == BACKSPACE {
            key.pop();
            continue;
        }
        key.push(byte[0]);
    }
    drop(raw);
    println!();
    Ok(())
}
