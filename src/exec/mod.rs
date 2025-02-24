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
    pub fn new(dir: P, out: O, err: E, color: bool) -> Self {
        Self {
            dir,
            out,
            err,
            color,
        }
    }

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

        struct Output {
            line: String,
            is_err: bool,
        }

        // Setup a channel to send stdout and stderr lines.
        let (otx, rx) = mpsc::channel();
        let etx = otx.clone();

        // Read from the pipes and write to final output in separate threads.
        // https://stackoverflow.com/a/72831067/79202
        // (See also https://stackoverflow.com/a/41024767/79202)
        let stdout_thread = thread::spawn(move || -> Result<(), io::Error> {
            let buf = BufReader::new(child_out);
            for line in buf.lines() {
                otx.send(Output {
                    line: line?,
                    is_err: false,
                })
                .unwrap()
            }
            Ok(())
        });

        let stderr_thread = thread::spawn(move || -> Result<(), io::Error> {
            let stderr_lines = BufReader::new(child_err).lines();
            for line in stderr_lines {
                etx.send(Output {
                    line: line?,
                    is_err: true,
                })
                .unwrap()
            }
            Ok(())
        });

        if !self.color || (!self.out.is_terminal() && !self.err.is_terminal()) {
            for output in rx {
                if output.is_err {
                    writeln!(self.err, "{}", output.line)?;
                } else {
                    writeln!(self.out, "{}", output.line)?;
                }
            }
        } else if self.out.is_terminal() && self.err.is_terminal() {
            for output in rx {
                if output.is_err {
                    cwriteln!(self.err, "<red>{}</red>", output.line)?;
                } else {
                    cwriteln!(self.out, "<dim><244>{}</244></dim>", output.line)?;
                }
            }
        } else if self.out.is_terminal() {
            for output in rx {
                if output.is_err {
                    writeln!(self.err, "{}", output.line)?;
                } else {
                    cwriteln!(self.out, "<dim><244>{}</244></dim>", output.line)?;
                }
            }
        } else {
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
