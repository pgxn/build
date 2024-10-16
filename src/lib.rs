#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
Build PGXN distributions.

This crate builds PGXN distributions for a variety of platforms and Postgres
versions.

*/
pub mod error;
mod pgrx;
mod pgxs;
mod pipeline;

use crate::{error::BuildError, pgrx::Pgrx, pgxs::Pgxs, pipeline::Pipeline};
use pgxn_meta::{dist, release::Release};

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
pub struct Builder(Build);

impl Builder {
    /// Creates and returns a new builder using the appropriate pipeline.
    pub fn new(meta: Release) -> Result<Self, BuildError> {
        if let Some(deps) = meta.dependencies() {
            if let Some(pipeline) = deps.pipeline() {
                return match pipeline {
                    dist::Pipeline::Pgxs => Ok(Builder(Build::Pgxs(Pgxs::new(meta)))),
                    dist::Pipeline::Pgrx => Ok(Builder(Build::Pgrx(Pgrx::new(meta)))),
                    _ => Err(BuildError::UnknownPipeline(pipeline.to_string())),
                };
            }
        }
        println!("HERE");
        todo!("Detect pipeline");
    }

    /// Downloads a release.
    pub fn download(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Unpacks a release.
    pub fn unpack(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    pub fn configure(&self) -> Result<(), BuildError> {
        match &self.0 {
            Build::Pgxs(pgxs) => pgxs.configure(),
            Build::Pgrx(pgrx) => pgrx.configure(),
        }
    }

    /// Compiles a distribution on a particular platform and Postgres version.
    pub fn compile(&self) -> Result<(), BuildError> {
        match &self.0 {
            Build::Pgxs(pgxs) => pgxs.compile(),
            Build::Pgrx(pgrx) => pgrx.compile(),
        }
    }

    /// Tests a distribution a particular platform and Postgres version.
    pub fn test(&self) -> Result<(), BuildError> {
        match &self.0 {
            Build::Pgxs(pgxs) => pgxs.test(),
            Build::Pgrx(pgrx) => pgrx.test(),
        }
    }
}

#[cfg(test)]
mod tests;
