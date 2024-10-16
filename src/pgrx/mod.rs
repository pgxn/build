//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use pgxn_meta::release::Release;

/// Builder implementation for [pgrx] Pipelines.
///
/// [pgrx]: https://github.com/pgcentralfoundation/pgrx
#[derive(Debug, PartialEq)]
pub(crate) struct Pgrx {
    meta: Release,
}

impl Pipeline for Pgrx {
    fn new(meta: Release) -> Self {
        Pgrx { meta }
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
