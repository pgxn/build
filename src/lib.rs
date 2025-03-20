#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
Build PGXN distributions.

This crate builds PGXN distributions for a variety of platforms and Postgres
versions.

*/
pub mod api;
pub mod error;
pub mod pg_config;
mod pgrx;
mod pgxs;
mod pipeline;

use crate::{error::BuildError, pgrx::Pgrx, pgxs::Pgxs, pipeline::Pipeline};
use pg_config::PgConfig;
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

impl Build {
    /// Returns a build pipeline identified by `pipe`, or an error if `pipe`
    /// is unknown.
    fn new(
        pipe: &dist::Pipeline,
        dir: impl AsRef<Path>,
        cfg: PgConfig,
    ) -> Result<Build, BuildError> {
        match pipe {
            dist::Pipeline::Pgxs => Ok(Build::Pgxs(Pgxs::new(dir, cfg))),
            dist::Pipeline::Pgrx => Ok(Build::Pgrx(Pgrx::new(dir, cfg))),
            _ => Err(BuildError::UnknownPipeline(pipe.to_string())),
        }
    }

    /// Attempts to detect and return the appropriate build pipeline to build
    /// the contents of `dir`. Returns an error if no pipeline can do so.
    fn detect(dir: impl AsRef<Path>, cfg: PgConfig) -> Result<Build, BuildError> {
        // Start with PGXS.
        let mut score = Pgxs::confidence(dir.as_ref());
        let mut pipe = dist::Pipeline::Pgxs;

        // Does pgrx have a higher score?
        let c = Pgrx::confidence(dir.as_ref());
        if c > score {
            score = c;
            pipe = dist::Pipeline::Pgrx;
        }

        // Try each of the others as they're added.
        // Return an error if no confidence.
        if score == 0 {
            return Err(BuildError::NoPipeline());
        }

        // Construct the winner.
        match pipe {
            dist::Pipeline::Pgrx => Ok(Build::Pgrx(Pgrx::new(dir, cfg))),
            dist::Pipeline::Pgxs => Ok(Build::Pgxs(Pgxs::new(dir, cfg))),
            _ => unreachable!("unknown pipelines {pipe}"),
        }
    }
}

/// Builder builds PGXN releases.
#[derive(Debug, PartialEq)]
pub struct Builder {
    pipeline: Build,
    meta: Release,
}

impl Builder {
    /// Creates and returns a new builder using the appropriate pipeline.
    pub fn new(dir: impl AsRef<Path>, meta: Release, cfg: PgConfig) -> Result<Self, BuildError> {
        let pipeline = if let Some(deps) = meta.dependencies() {
            if let Some(pipe) = deps.pipeline() {
                Build::new(pipe, dir, cfg)?
            } else {
                Build::detect(dir, cfg)?
            }
        } else {
            Build::detect(dir, cfg)?
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

    /// Installs a distribution on a particular platform and Postgres version.
    pub fn install(&self) -> Result<(), BuildError> {
        match &self.pipeline {
            Build::Pgxs(pgxs) => pgxs.install(),
            Build::Pgrx(pgrx) => pgrx.install(),
        }
    }
}

/// Returns a string representation of `path`.
pub(crate) fn filename<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("UNKNOWN"))
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests;
