//! Build Pipeline interface definition.

use crate::error::BuildError;
use std::{path::Path, process::Command};

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

    /// Run a command. Runs it with elevated privileges using `sudo` unless
    /// it's on Windows.
    fn run<S, I>(&self, cmd: &str, args: I, sudo: bool) -> Result<(), BuildError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        // Ignore the `sudo` param on Windows, since it is not currently
        // possible to mock it on Windows (see notes in tests.rs), and though
        // it [exists](https://github.com/microsoft/sudo), it's not clear
        // whether it's the right thing to require, or if elevated privileges
        // will be required at all in Windows. Revisit once all the
        // dependencies for building extensions on Windows are recognized and
        // put to use to formally support building and installing extensions
        // on Windows.
        let mut cmd = if cfg!(not(windows)) && sudo {
            let mut c = Command::new("sudo");
            c.arg(cmd);
            c
        } else {
            Command::new(cmd)
        };

        cmd.args(args);
        cmd.current_dir(self.dir());
        match cmd.output() {
            Ok(_) => Ok(()),
            Err(e) => Err(BuildError::Command(cmd, e.kind())),
        }
    }
}

#[cfg(test)]
mod tests;
