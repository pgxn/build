//! Build Pipeline interface definition.

use crate::{error::BuildError, pg_config::PgConfig};
use color_print::cwriteln;
use log::debug;
use std::{
    io::{self, BufRead, BufReader, IsTerminal, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// Defines the interface for build pipelines to configure, compile, and test
/// PGXN distributions.
pub(crate) trait Pipeline<P: AsRef<Path>> {
    /// Creates an instance of a Pipeline.
    fn new(dir: P, pg_config: PgConfig) -> Self;

    /// Returns a score for the confidence that this pipeline can build the
    /// contents of `dir`. A score of 0 means no confidence and 255 means the
    /// highest confidence.
    fn confidence(dir: P) -> u8;

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    fn configure(&self) -> Result<(), BuildError>;

    /// Compiles a distribution on a particular platform and Postgres version.
    fn compile(&self) -> Result<(), BuildError>;

    /// Installs a distribution on a particular platform and Postgres version.
    fn install(&self) -> Result<(), BuildError>;

    /// Tests a distribution a particular platform and Postgres version.
    fn test(&self) -> Result<(), BuildError>;

    /// Returns the directory passed to [`new`].
    fn dir(&self) -> &P;

    /// Returns the PgConfig passed to [`new`].
    fn pg_config(&self) -> &PgConfig;

    // maybe_sudo returns a Command that starts with the sudo command if
    // `sudo` is true and the `pkglibdir` returned by pg_config isn't
    // writeable by the current user.
    fn maybe_sudo(&self, program: &str, sudo: bool) -> Command {
        if sudo {
            if let Some(dir) = self.pg_config().get("pkglibdir") {
                if !self.is_writeable(dir) {
                    let mut c = Command::new("sudo");
                    c.arg(program);
                    return c;
                }
            }
        }
        Command::new(program)
    }

    /// Attempts to write a temporary file to `dir` and returns `true` on
    /// success and `false` on failure. The temporary file will be deleted.
    fn is_writeable<D: AsRef<Path>>(&self, dir: D) -> bool {
        debug!(dir:? = crate::filename(&dir); "testing write access");
        match tempfile::Builder::new()
            .prefix("pgxn-")
            .suffix(".test")
            .tempfile_in(dir)
        {
            Ok(f) => write!(&f, "ok").is_ok(),
            Err(_) => false,
        }
    }

    /// Run a command. Runs it with elevated privileges when `sudo` is true
    /// and `pg_config --pkglibdir` isn't writeable by the current user.
    fn run<S, I>(&self, program: &str, args: I, sudo: bool) -> Result<(), BuildError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        // Use `sudo` if the param is set.
        let mut cmd = self.maybe_sudo(program, sudo);
        cmd.args(args).current_dir(self.dir());
        pipe_command(cmd, io::stdout(), io::stderr())
    }
}

fn pipe_command<O, E>(mut cmd: Command, mut out: O, mut err: E) -> Result<(), BuildError>
where
    O: io::Write + IsTerminal + std::marker::Send + 'static,
    E: io::Write + IsTerminal + std::marker::Send + 'static,
{
    // Create pipes from the child's stdout and stderr.
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

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

    // Read from the pipes and write to final output in separate threads.
    // https://stackoverflow.com/a/72831067/79202
    let stdout_thread = thread::spawn(move || -> Result<(), io::Error> {
        let stdout_lines = BufReader::new(child_out).lines();
        for line in stdout_lines {
            cwriteln!(out, "<dim><244>{}</244></dim>", line.unwrap())?;
        }
        Ok(())
    });

    let stderr_thread = thread::spawn(move || -> Result<(), io::Error> {
        let stderr_lines = BufReader::new(child_err).lines();
        for line in stderr_lines {
            cwriteln!(err, "<red>{}</red>", line.unwrap())?;
        }
        Ok(())
    });

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

#[cfg(test)]
mod tests;
