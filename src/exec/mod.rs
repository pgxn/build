//! Command execution context.
use crate::error::BuildError;
use color_print::cwriteln;
use log::debug;
use std::{
    clone::Clone,
    io::{self, BufRead, BufReader, IsTerminal, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

// Define a structure fo capturing output.
struct Output {
    line: String,
    is_err: bool,
}
impl Output {
    fn new(line: String, is_err: bool) -> Self {
        Self { line, is_err }
    }
}

/// Command execution context.
pub(crate) struct Executor<P, O, E>
where
    P: AsRef<Path>,
    O: Write + IsTerminal,
    E: Write + IsTerminal,
{
    dir: P,
    out: O,
    err: E,
    color: bool,
}

impl<P, O, E> Executor<P, O, E>
where
    P: AsRef<Path>,
    O: Write + IsTerminal,
    E: Write + IsTerminal,
{
    /// Creates a new command execution context. Commands passed to
    /// [`execute`] will have their current directory set to `dir`. STDOUT
    /// lines will be sent to `out` and STDERR lines will be sent to err. If
    /// `color` is true, lines sent to STDOUT will be a dimmed grey if it
    /// refers to a terminal, while lines sent to STDERR will be red if it
    /// refers to a terminal.
    pub fn new(dir: P, out: O, err: E, color: bool) -> Self {
        println!("OUT IS? {}", out.is_terminal());
        Self {
            dir,
            out,
            err,
            color,
        }
    }

    /// Sets `cmd`'s `current_dir` to `self.dir`, pipes output to `self.out`
    /// and `self.err`, and executes `cmd`.
    pub fn execute(&mut self, mut cmd: Command) -> Result<(), BuildError> {
        // Execute from self.dir and create pipes from the child's stdout and stderr.
        cmd.current_dir(&self.dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the child process.
        debug!(command:? = cmd; "Executing");
        let mut child = cmd
            .spawn()
            .map_err(|e| BuildError::Command(format!("{:?}", cmd), e.kind().to_string()))?;

        // Grab the stdout and stderr pipes.
        let child_out = child
            .stdout
            .take()
            .ok_or_else(|| BuildError::Command(format!("{:?}", cmd), "no stdout".to_string()))?;
        let child_err = child
            .stderr
            .take()
            .ok_or_else(|| BuildError::Command(format!("{:?}", cmd), "no stderr".to_string()))?;

        // Setup a channel to send stdout and stderr lines.
        let (otx, rx) = mpsc::channel();
        let etx = otx.clone();

        // Spawn a thread to stream STDOUT lines back to the main thread.
        let stdout_thread = thread::spawn(move || -> Result<(), io::Error> {
            let buf = BufReader::new(child_out);
            for line in buf.lines() {
                otx.send(Output::new(line?, false)).unwrap()
            }
            Ok(())
        });

        // Spawn a thread to stream STDERR lines back to the main thread.
        let stderr_thread = thread::spawn(move || -> Result<(), io::Error> {
            let stderr_lines = BufReader::new(child_err).lines();
            for line in stderr_lines {
                etx.send(Output::new(line?, true)).unwrap();
            }
            Ok(())
        });

        // Read the lines from the spawned threads and format them as appropriate.
        if !self.color || (!self.out.is_terminal() && !self.err.is_terminal()) {
            // No color wanted or allowed. Just write the unmodified output.
            for output in rx {
                if output.is_err {
                    writeln!(self.err, "{}", output.line)?;
                } else {
                    writeln!(self.out, "{}", output.line)?;
                }
            }
        } else if self.out.is_terminal() && self.err.is_terminal() {
            // Write colored output to both outputs.
            for output in rx {
                if output.is_err {
                    cwriteln!(self.err, "<red>{}</red>", output.line)?;
                } else {
                    cwriteln!(self.out, "<dim><244>{}</244></dim>", output.line)?;
                }
            }
        } else if self.out.is_terminal() {
            // Write colored output only to self.out.
            for output in rx {
                if output.is_err {
                    writeln!(self.err, "{}", output.line)?;
                } else {
                    cwriteln!(self.out, "<dim><244>{}</244></dim>", output.line)?;
                }
            }
        } else {
            // Write colored output only to self.err.
            for output in rx {
                if output.is_err {
                    cwriteln!(self.err, "<red>{}</red>", output.line)?;
                } else {
                    writeln!(self.out, "{}", output.line)?;
                }
            }
        }

        // Wait for the child and output threads to finish.
        let res = child.wait();
        stdout_thread.join().unwrap()?;
        stderr_thread.join().unwrap()?;

        // Determine how the command finished.
        match res {
            Ok(status) => {
                if !status.success() {
                    return Err(BuildError::Command(
                        format!("{:?}", cmd),
                        match status.code() {
                            Some(code) => format!("exited with status code: {code}"),
                            None => "process terminated by signal".to_string(),
                        },
                    ));
                }
                Ok(())
            }
            Err(e) => Err(BuildError::Command(
                format!("{:?}", cmd),
                e.kind().to_string(),
            )),
        }
    }
}
