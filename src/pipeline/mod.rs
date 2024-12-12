//! Build Pipeline interface definition.

use crate::error::BuildError;
use log::debug;
use std::{io::Write, path::Path, process::Command};

/// Defines the interface for build pipelines to configure, compile, and test
/// PGXN distributions.
pub(crate) trait Pipeline<P: AsRef<Path>> {
    /// Creates an instance of a Pipeline.
    fn new(dir: P, sudo: bool) -> Self;

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

    /// Attempts to write a temporary file to `dir` and returns `true` on
    /// success and `false` on failure. The temporary file will be deleted.
    fn is_writeable<D: AsRef<Path>>(&self, dir: D) -> bool {
        debug!(dir:display = crate::filename(&dir); "testing write access");
        match tempfile::Builder::new()
            .prefix("pgxn-")
            .suffix(".test")
            .tempfile_in(dir)
        {
            Ok(f) => write!(&f, "ok").is_ok(),
            Err(_) => false,
        }
    }

    /// Run a command. Runs it with elevated privileges using `sudo` unless
    /// it's on Windows.
    fn run<S, I>(&self, cmd: &str, args: I, sudo: bool) -> Result<(), BuildError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        // Use `sudo` if the param is set.
        let mut cmd = if sudo {
            let mut c = Command::new("sudo");
            c.arg(cmd);
            c
        } else {
            Command::new(cmd)
        };

        cmd.args(args);
        cmd.current_dir(self.dir());
        match cmd.output() {
            Ok(out) => {
                if !out.status.success() {
                    return Err(BuildError::Command(
                        format!("{:?}", cmd),
                        String::from_utf8_lossy(&out.stderr).to_string(),
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

#[cfg(test)]
mod tests;
