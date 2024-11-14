//! Build Errors.
use std::io;
use thiserror::Error;

/// Build errors.
#[derive(Error, Debug)]
pub enum BuildError {
    /// Errors configuring a build.
    #[error("configuration failure")]
    Configuration(),

    /// Unknown pipeline error.
    #[error("unknown build pipeline `{0}`")]
    UnknownPipeline(String),

    /// IO error.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// File error.
    #[error("{0} {1}: {2}")]
    File(&'static str, String, io::ErrorKind),

    /// URL Error.
    #[error(transparent)]
    Url(#[from] url::ParseError),

    /// URL lacks a file name segment.
    #[error("missing file name segment from {0}")]
    NoUrlFile(url::Url),

    /// HTTP error.
    #[error(transparent)]
    Http(#[from] Box<ureq::Error>),
}

impl From<ureq::Error> for BuildError {
    fn from(value: ureq::Error) -> Self {
        Self::Http(Box::new(value))
    }
}
