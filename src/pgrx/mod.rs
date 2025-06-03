//! Builder implementation for [pgrx] Pipelines.
//!
//! [pgrx]: https://github.com/pgcentralfoundation/pgrx

use crate::pipeline::{Context, Pipeline};
use crate::{error::BuildError, exec::Executor, pg_config::PgConfig};
use cargo_toml::Manifest;
use std::collections::HashMap;
use std::path::Path;

/// Builder implementation for [pgrx] Pipelines.
///
/// [pgrx]: https://github.com/pgcentralfoundation/pgrx
#[derive(Debug, PartialEq)]
pub(crate) struct Pgrx {
    exec: Executor,
    cfg: PgConfig,
    pkg: String,
}

impl Pipeline for Pgrx {
    fn new(exec: Executor, cfg: PgConfig, ctx: Context) -> Self {
        Pgrx {
            exec,
            cfg,
            pkg: ctx.config.get("package").unwrap().to_string(),
        }
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
    fn evaluate(dir: impl AsRef<Path>) -> Context {
        let file = dir.as_ref().join("Cargo.toml");
        if !file.exists() {
            return Context {
                score: 0,
                config: HashMap::with_capacity(0),
                err: None,
            };
        }

        // Load cargo.toml.
        if let Ok(cargo) = cargo_toml::Manifest::from_path(file) {
            // Determine the score
            let score = get_score(&cargo);

            if score > 0 {
                // Determine the package.
                if let Some(pkg) = cargo.package {
                    return Context {
                        score,
                        config: HashMap::from([("package".to_string(), pkg.name)]),
                        err: None,
                    };
                }

                // Is it a workspace?
                if let Some(work) = cargo.workspace {
                    if !work.members.is_empty() {
                        // XXX Look for pgrx in each member?
                        return Context {
                            score,
                            config: HashMap::with_capacity(0),
                            err: Some(BuildError::SelectPackage(work.members)),
                        };
                    }
                }
                return Context {
                    score,
                    config: HashMap::with_capacity(0),
                    err: Some(BuildError::NoPackage()),
                };
            }
        }

        // Have Cargo.toml but no dependence on pgrx. Weak confidence.
        Context {
            score: 1,
            config: HashMap::with_capacity(0),
            err: None,
        }
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

fn get_score(cargo: &Manifest) -> u8 {
    if cargo.dependencies.contains_key("pgrx") {
        // Full confidence
        return 255;
    }

    if let Some(work) = &cargo.workspace {
        if work.dependencies.contains_key("pgrx") {
            // Full confidence
            return 255;
        }
    }

    return 0;
}

#[cfg(test)]
mod tests;
