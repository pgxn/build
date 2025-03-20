//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::pipeline::Pipeline;
use crate::{error::BuildError, exec::Executor, line::WriteLine, pg_config::PgConfig};
use std::path::Path;

/// Builder implementation for [pgrx] Pipelines.
///
/// [pgrx]: https://github.com/pgcentralfoundation/pgrx
#[derive(PartialEq)]
pub(crate) struct Pgrx {
    exec: Executor,
    cfg: PgConfig,
}

impl Pipeline for Pgrx {
    fn new(exec: Executor, cfg: PgConfig) -> Self {
        Pgrx { exec, cfg }
    }

    /// Returns the Executor passed to [`Self::new`].
    fn executor(&mut self) -> &mut Executor {
        &mut self.exec
    }

    /// Returns the PgConfig passed to [`Self::new`].
    fn pg_config(&self) -> &PgConfig {
        &self.cfg
    }

    /// Determines the confidence that the Pgrx pipeline can build the
    /// contents of `dir`. Returns 255 if it contains a file named
    /// `Cargo.toml` and lists pgrx as a dependency. Otherwise returns 1 if
    /// `Cargo.toml` exists and 0 if it does not.
    fn confidence(dir: impl AsRef<Path>) -> u8 {
        let file = dir.as_ref().join("Cargo.toml");
        if !file.exists() {
            return 0;
        }

        // Does Cargo.toml mention pgrx?
        if let Ok(cargo) = cargo_toml::Manifest::from_path(file) {
            if cargo.dependencies.contains_key("pgrx") {
                // Full confidence
                return 255;
            }
        }

        // Have Cargo.toml but no dependence on pgrx. Weak confidence.
        1
    }

    /// Runs `cargo init`.
    fn configure(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo build`.
    fn compile(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo test`.
    fn test(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo install`.
    fn install(&mut self) -> Result<(), BuildError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
