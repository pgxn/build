//! Build Pipeline interface definition.

use crate::{error::BuildError, exec, pg_config::PgConfig};
use log::debug;
use std::{io::Write, path::Path, process::Command};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        stderr.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Ansi256(244)))
                .set_dimmed(true),
        )?;
        let mut exec = exec::Executor::new(self.dir(), &mut stdout, &mut stderr);
        let ret = exec.execute(cmd);
        stdout.reset()?;
        stderr.reset()?;
        ret
    }
}

#[cfg(test)]
mod tests;
