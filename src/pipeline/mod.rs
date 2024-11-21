//! Build Pipeline interface definition.

use crate::error::BuildError;
use std::path::{Path, PathBuf};

/// Defines the interface for build pipelines to configure, compile, and test
/// PGXN distributions.
pub(crate) trait Pipeline {
    /// Creates an instance of a Pipeline.
    fn new(dir: PathBuf, sudo: bool) -> Self;

    /// Returns a score for the confidence that this pipeline can build the
    /// contents of `dir`. A score of 0 means no confidence and 255 means the
    /// highest confidence.
    fn confidence(dir: &Path) -> u8;

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    fn configure(&self) -> Result<(), BuildError>;

    /// Compiles a distribution on a particular platform and Postgres version.
    fn compile(&self) -> Result<(), BuildError>;

    /// Installs a distribution on a particular platform and Postgres version.
    fn install(&self) -> Result<(), BuildError>;

    /// Tests a distribution a particular platform and Postgres version.
    fn test(&self) -> Result<(), BuildError>;
}
