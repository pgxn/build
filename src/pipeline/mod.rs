//! Build Pipeline interface definition.

use crate::error::BuildError;
use pgxn_meta::release::Release;

/// Defines the interface for build pipelines to configure, compile, and test
/// PGXN distributions.
pub(crate) trait Pipeline {
    /// Creates an instance of a Builder.
    fn new(meta: Release) -> Self;

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    fn configure(&self) -> Result<(), BuildError>;

    /// Compiles a distribution on a particular platform and Postgres version.
    fn compile(&self) -> Result<(), BuildError>;

    /// Tests a distribution a particular platform and Postgres version.
    fn test(&self) -> Result<(), BuildError>;
}
