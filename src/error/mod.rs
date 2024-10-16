//! Build Errors.
use thiserror::Error;

/// Build errors.
#[derive(Error, Debug, PartialEq)]
pub enum BuildError {
    /// Errors configuring a build.
    #[error("configuration failure")]
    Configuration(),

    /// Unknown pipeline error.
    #[error("unknown build pipeline `{0}`")]
    UnknownPipeline(String),
}
