//! Builder implementation for [PGXS] Pipelines.
//!
//! [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use std::path::PathBuf;

/// Builder implementation for [PGXS] Pipelines.
///
/// [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html
#[derive(Debug, PartialEq)]
pub(crate) struct Pgxs {
    dir: PathBuf,
    sudo: bool,
}

impl Pipeline for Pgxs {
    fn new(dir: PathBuf, sudo: bool) -> Self {
        Pgxs { dir, sudo }
    }

    fn configure(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn compile(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn test(&self) -> Result<(), BuildError> {
        Ok(())
    }
}
