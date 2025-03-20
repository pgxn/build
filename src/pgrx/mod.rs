//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::pg_config::PgConfig;
use crate::pipeline::Pipeline;
use crate::{error::BuildError, writer::Writer};
use std::{
    io,
    path::{Path, PathBuf},
};

/// Builder implementation for [pgrx] Pipelines.
///
/// [pgrx]: https://github.com/pgcentralfoundation/pgrx
#[derive(Debug, PartialEq)]
pub(crate) struct Pgrx {
    cfg: PgConfig,
    dir: PathBuf,
    writer: Writer,
}

impl Pipeline for Pgrx {
    fn new(writer: Writer, dir: impl AsRef<Path>, cfg: PgConfig) -> Self {
        Pgrx {
            cfg,
            dir: dir.as_ref().to_path_buf(),
            writer,
        }
    }

    /// Returns the directory passed to [`Self::new`].
    fn dir(&self) -> impl AsRef<Path> {
        &self.dir
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
    fn configure(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo build`.
    fn compile(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo test`.
    fn test(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Runs `cargo install`.
    fn install(&self) -> Result<(), BuildError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
