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

    /// URL scheme Error.
    #[error("unsupported URL scheme: {0}")]
    Scheme(String),

    /// HTTP error.
    #[error(transparent)]
    Http(#[from] Box<ureq::Error>),

    /// Serde JSON error.
    #[error("invalid JSON: {0}")]
    Serde(#[from] serde_json::Error),

    /// Invalid type.
    #[error("invalid type: {0} expected to be {1} but got {2}")]
    Type(String, &'static str, &'static str),

    /// URI Template error.
    #[error(transparent)]
    TemplateError(#[from] iri_string::template::Error),

    /// UnknownURI Template.
    #[error("unknown URI template: {0}")]
    UnknownTemplate(String),

    /// Unexpected data error.
    #[error("{0}")]
    Invalid(&'static str),
}

impl From<ureq::Error> for BuildError {
    fn from(value: ureq::Error) -> Self {
        Self::Http(Box::new(value))
    }
}
