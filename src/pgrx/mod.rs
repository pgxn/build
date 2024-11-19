//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use std::path::PathBuf;

/// Builder implementation for [pgrx] Pipelines.
///
/// [pgrx]: https://github.com/pgcentralfoundation/pgrx
#[derive(Debug, PartialEq)]
pub(crate) struct Pgrx {
    dir: PathBuf,
    sudo: bool,
}

impl Pipeline for Pgrx {
    fn new(dir: PathBuf, sudo: bool) -> Self {
        Pgrx { dir, sudo }
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
