//! Build Pipeline interface definition.

use crate::{error::BuildError, pg_config::PgConfig};
use log::debug;
use std::{
    io::{self, BufRead, BufReader, IsTerminal, Write},
    path::Path,
    process::{Command, Stdio},
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
    O: io::Write + IsTerminal,
    E: io::Write + IsTerminal,
{
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    debug!(command:? = cmd; "Executing");

    // Spawn the child process.
    let mut child = cmd
        .spawn()
        .map_err(|e| BuildError::Command(format!("{:?}", cmd), e.kind().to_string()))?;

    let grey = ansi_term::Color::Fixed(244).dimmed();
    let red = ansi_term::Color::Red;
    {
        // https://stackoverflow.com/a/41024767/79202
        let child_out = child
            .stdout
            .take()
            .ok_or_else(|| BuildError::Command(format!("{:?}", cmd), "no stdout".to_string()))?;
        let child_err = child
            .stderr
            .take()
            .ok_or_else(|| BuildError::Command(format!("{:?}", cmd), "no stderr".to_string()))?;

        let mut child_out = BufReader::new(child_out);
        let mut child_err = BufReader::new(child_err);

        loop {
            let (stdout_len, stderr_len) = match (child_out.fill_buf(), child_err.fill_buf()) {
                (Ok(child_out), Ok(child_err)) => {
                    if out.is_terminal() {
                        write!(out, "{}", grey.prefix())?;
                        out.write_all(child_out)?;
                        write!(out, "{}", grey.suffix())?;
                    } else {
                        out.write_all(child_out)?;
                    }
                    if err.is_terminal() {
                        write!(err, "{}", red.prefix())?;
                        err.write_all(child_err)?;
                        write!(err, "{}", red.suffix())?;
                    } else {
                        err.write_all(child_err)?;
                    }

                    (child_out.len(), child_err.len())
                }
                other => panic!("Some better error handling here... {:?}", other),
            };

            if stdout_len == 0 && stderr_len == 0 {
                // if let Ok(Some(_)) = child.try_wait() {
                break;
            }

            child_out.consume(stdout_len);
            child_err.consume(stderr_len);
        }
    }

    // Execute the command.
    match child.wait() {
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
