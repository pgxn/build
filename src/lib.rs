#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
Build PGXN distributions.

This crate builds PGXN distributions for a variety of platforms and Postgres
versions.

*/
use thiserror::Error;

/// Build errors.
#[derive(Error, Debug)]
pub enum BuildError {
    /// Errors configuring a build.
    #[error("configuration failure")]
    Configuration(),
}

/// Defines the interface for downloading, configuring, building, and testing
/// PGXN distributions.
pub trait Builder {
    /// Downloads a distribution.
    fn download() -> Result<(), BuildError>;

    /// Unpacks a downloaded distribution.
    fn unpack() -> Result<(), BuildError>;

    /// Patches a distribution.
    fn patch() -> Result<(), BuildError>;

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    fn configure() -> Result<(), BuildError>;

    /// Compiles a distribution on a particular platform and Postgres version.
    fn compile() -> Result<(), BuildError>;

    /// Tests a distribution a particular platform and Postgres version.
    fn test() -> Result<(), BuildError>;
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert!(true == true);
    }
}
