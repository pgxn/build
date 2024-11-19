#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
Build PGXN distributions.

This crate builds PGXN distributions for a variety of platforms and Postgres
versions.

*/
pub mod api;
pub mod error;
mod pgrx;
mod pgxs;
mod pipeline;

use crate::{error::BuildError, pgrx::Pgrx, pgxs::Pgxs, pipeline::Pipeline};
use pgxn_meta::{dist, release::Release};
use std::path::Path;

/// Defines the types of builders.
#[derive(Debug, PartialEq)]
enum Build {
    /// Defines a builder using the PGXS pipeline.
    Pgxs(Pgxs),

    /// Defines a builder using the pgrx pipeline.
    Pgrx(Pgrx),
}

/// Builder builds PGXN releases.
#[derive(Debug, PartialEq)]
pub struct Builder {
    pipeline: Build,
    meta: Release,
}

impl Builder {
    /// Creates and returns a new builder using the appropriate pipeline.
    pub fn new<P: AsRef<Path>>(dir: P, meta: Release) -> Result<Self, BuildError> {
        let pipeline = if let Some(deps) = meta.dependencies() {
            if let Some(pipe) = deps.pipeline() {
                let dir = dir.as_ref().to_path_buf();
                match pipe {
                    dist::Pipeline::Pgxs => Build::Pgxs(Pgxs::new(dir, true)),
                    dist::Pipeline::Pgrx => Build::Pgrx(Pgrx::new(dir, true)),
                    _ => return Err(BuildError::UnknownPipeline(pipe.to_string())),
                }
            } else {
                todo!("Detect pipeline");
            }
        } else {
            todo!("Detect pipeline");
        };

        Ok(Builder { pipeline, meta })
    }

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    pub fn configure(&self) -> Result<(), BuildError> {
        match &self.pipeline {
            Build::Pgxs(pgxs) => pgxs.configure(),
            Build::Pgrx(pgrx) => pgrx.configure(),
        }
    }

    /// Compiles a distribution on a particular platform and Postgres version.
    pub fn compile(&self) -> Result<(), BuildError> {
        match &self.pipeline {
            Build::Pgxs(pgxs) => pgxs.compile(),
            Build::Pgrx(pgrx) => pgrx.compile(),
        }
    }

    /// Tests a distribution a particular platform and Postgres version.
    pub fn test(&self) -> Result<(), BuildError> {
        match &self.pipeline {
            Build::Pgxs(pgxs) => pgxs.test(),
            Build::Pgrx(pgrx) => pgrx.test(),
        }
    }
}

#[cfg(test)]
mod tests;
