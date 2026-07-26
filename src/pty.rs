//! The PTY runner. It launches a prepared process on a pseudo-terminal and
//! waits for it to finish. It knows only about a prepared process; it never
//! constructs sandbox arguments itself, so new sandbox backends never change
//! this layer.

use crate::process::PreparedProcess;
use crate::screen::VirtualScreen;
use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

/// Launch a prepared process on a PTY and wait for it to exit.
///
/// @planks("a process is prepared and launched")
pub fn launch(prepared: &PreparedProcess) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize::default())
        .map_err(|e| e.to_string())?;
    let mut cmd = CommandBuilder::new(prepared.program.clone());
    for arg in &prepared.args {
        cmd.arg(arg.clone());
    }
    let mut child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "launched process exited unsuccessfully: {status:?}"
        ))
    }
}

/// Launch a prepared process on a PTY, read its output until it exits, and
/// parse that output into a virtual screen.
///
/// @planks("the process is captured through a PTY")
pub fn capture(prepared: &PreparedProcess) -> Result<VirtualScreen, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize::default())
        .map_err(|e| e.to_string())?;
    let mut cmd = CommandBuilder::new(prepared.program.clone());
    for arg in &prepared.args {
        cmd.arg(arg.clone());
    }
    let mut child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    // Drop the parent's slave handle so the reader sees EOF once the child,
    // which holds its own slave, exits.
    drop(pair.slave);
    let mut output = Vec::new();
    reader.read_to_end(&mut output).map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    Ok(VirtualScreen::from_pty_output(&output))
}

/// A live interactive capture of a running program: keys pressed on it are
/// forwarded to the program through the PTY, and its output is accumulated in
/// the background so the current virtual screen can be read at any time.
///
/// @planks("the process is captured through a PTY")
pub struct InteractiveCapture {
    writer: Box<dyn Write + Send>,
    output: Arc<Mutex<Vec<u8>>>,
    child: Box<dyn Child + Send + Sync>,
    reader: Option<std::thread::JoinHandle<()>>,
    columns: u16,
}

impl std::fmt::Debug for InteractiveCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InteractiveCapture").finish_non_exhaustive()
    }
}

/// Launch a prepared process on a PTY and keep it running, reading its output
/// in the background, so keys can be forwarded to it and its screen read while
/// it runs.
///
/// @planks("the process is captured through a PTY")
pub fn capture_interactive(prepared: &PreparedProcess) -> Result<InteractiveCapture, String> {
    capture_interactive_at(prepared, None)
}

/// Launch a prepared process the same way, on a terminal `columns` wide. Size is
/// a property of the run, so the caller states it, and an unstated width leaves
/// the program on the operator's own terminal size. The width reaches the PTY
/// and the virtual screen together, so the program draws and is read at one
/// width.
///
/// The working directory comes from the prepared process, so a relative program
/// name resolves against the directory the backend that prepared it means
/// rather than against the operator's own home, which is where an unset working
/// directory otherwise lands the child.
///
/// @planks("the process is captured through a PTY")
/// @planks("that plan is replayed at {int} columns")
/// @planks("the operator records the fixture terminal program")
pub fn capture_interactive_at(
    prepared: &PreparedProcess,
    columns: Option<u16>,
) -> Result<InteractiveCapture, String> {
    let pty_system = native_pty_system();
    let columns = columns.unwrap_or(PtySize::default().cols);
    let pair = pty_system
        .openpty(PtySize {
            cols: columns,
            ..PtySize::default()
        })
        .map_err(|e| e.to_string())?;
    let mut cmd = CommandBuilder::new(prepared.program.clone());
    for arg in &prepared.args {
        cmd.arg(arg.clone());
    }
    if let Some(cwd) = &prepared.cwd {
        cmd.cwd(cwd);
    }
    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    // The child holds its own slave handle; drop the parent's so only the child
    // keeps the slave side open.
    drop(pair.slave);
    let output = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&output);
    let drain = std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while let Ok(read) = reader.read(&mut buf) {
            if read == 0 {
                break;
            }
            sink.lock().unwrap().extend_from_slice(&buf[..read]);
        }
    });
    Ok(InteractiveCapture {
        writer,
        output,
        child,
        reader: Some(drain),
        columns,
    })
}

impl InteractiveCapture {
    /// Forward a key press to the running program. Enter is sent as a carriage
    /// return, as a terminal sends it; any other key is sent as its own text.
    ///
    /// @planks("the operator presses the keys {string}, {string}, and Enter")
    pub fn press_key(&mut self, key: &str) {
        let bytes = if key == "Enter" { "\r" } else { key };
        self.writer
            .write_all(bytes.as_bytes())
            .expect("forward key press to the PTY");
        self.writer.flush().expect("flush the PTY");
    }

    /// Whether the running program has already exited, so a caller sends it
    /// nothing it can no longer read.
    ///
    /// @planks("the operator records the command {string} and presses {string}")
    pub fn finished(&mut self) -> bool {
        self.child
            .try_wait()
            .expect("read the launched program's status")
            .is_some()
    }

    /// End the running program's input and wait for it to finish, so the screen
    /// read afterwards is the screen the program left.
    ///
    /// The terminal's own end-of-transmission character is how a program
    /// reading a terminal sees its input end, and it answers one read. A closed
    /// input answers every read, so it is sent again for each read the program
    /// makes until the program exits. Joining the reader is what completes the
    /// screen: the exit is observed before the last bytes the program wrote
    /// have necessarily been read.
    ///
    /// @planks("the operator records the fixture terminal program")
    /// @planks("the operator records that program")
    pub fn end_session(&mut self) {
        while !self.finished() {
            self.writer.write_all(&[0x04]).expect("end the PTY input");
            self.writer.flush().expect("flush the PTY");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        if let Some(drain) = self.reader.take() {
            drain.join().expect("join the PTY output reader");
        }
    }

    /// The current virtual screen, parsed from all output read so far, at the
    /// width the program was given.
    ///
    /// @planks("the virtual screen contains the text {string}")
    /// @planks("that plan is replayed at {int} columns")
    pub fn screen(&self) -> VirtualScreen {
        let output = self.output.lock().unwrap();
        VirtualScreen::from_pty_output_at(&output, self.columns)
    }
}
