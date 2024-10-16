//! Builder implementation for [PGXS] Pipelines.
//!
//! [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use pgxn_meta::release::Release;

/// Builder implementation for [PGXS] Pipelines.
///
/// [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html
#[derive(Debug, PartialEq)]
pub(crate) struct Pgxs {
    meta: Release,
}

impl Pipeline for Pgxs {
    fn new(meta: Release) -> Self {
        Pgxs { meta }
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
