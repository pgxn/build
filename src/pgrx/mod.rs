//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

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

    /// Determines the confidence that the Pgrx pipeline can build the
    /// contents of `dir`. Returns 255 if it contains a file named
    /// `Cargo.toml` and lists pgrx as a dependency. Otherwise returns 1 if
    /// `Cargo.toml` exists and 0 if it does not.
    fn confidence(dir: &Path) -> u8 {
        let file = dir.join("Cargo.toml");
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
