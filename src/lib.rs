#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
Build PGXN distributions.

This crate builds PGXN distributions for a variety of platforms and Postgres
versions.

*/
mod api;
pub mod error;
mod exec;
mod line;
mod pg_config;
mod pgrx;
mod pgxs;
mod pipeline;

use crate::{error::BuildError, pgrx::Pgrx, pgxs::Pgxs, pipeline::Pipeline};
pub use api::Api;
use exec::Executor;
use line::WriteLine;
pub use pg_config::PgConfig;
use pgxn_meta::{dist, release::Release};
use std::path::Path;

/// Defines the types of builders.
#[derive(PartialEq)]
enum Build<O: WriteLine, E: WriteLine> {
    /// Defines a builder using the PGXS pipeline.
    Pgxs(Pgxs<O, E>),

    /// Defines a builder using the pgrx pipeline.
    Pgrx(Pgrx<O, E>),
}

impl<O: WriteLine, E: WriteLine> Build<O, E> {
    /// Returns a build pipeline identified by `pipe`, or an error if `pipe`
    /// is unknown.
    fn new(
        pipe: &dist::Pipeline,
        exec: Executor<O, E>,
        cfg: PgConfig,
    ) -> Result<Build<O, E>, BuildError> {
        match pipe {
            dist::Pipeline::Pgxs => Ok(Build::Pgxs(Pgxs::new(exec, cfg))),
            dist::Pipeline::Pgrx => Ok(Build::Pgrx(Pgrx::new(exec, cfg))),
            _ => Err(BuildError::UnknownPipeline(pipe.to_string())),
        }
    }

    /// Attempts to detect and return the appropriate build pipeline to build
    /// the contents of `dir`. Returns an error if no pipeline can do so.
    fn detect(exec: Executor<O, E>, cfg: PgConfig) -> Result<Build<O, E>, BuildError> {
        // Start with PGXS.
        let mut score = Pgxs::<O, E>::confidence(exec.dir());
        let mut pipe = dist::Pipeline::Pgxs;

        // Does pgrx have a higher score?
        let c = Pgrx::<O, E>::confidence(exec.dir());
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
            dist::Pipeline::Pgrx => Ok(Build::Pgrx(Pgrx::new(exec, cfg))),
            dist::Pipeline::Pgxs => Ok(Build::Pgxs(Pgxs::new(exec, cfg))),
            _ => unreachable!("unknown pipelines {pipe}"),
        }
    }
}

/// Builder builds PGXN releases.
// TODO #[derive(Debug, PartialEq)]
pub(crate) struct Builder<O: WriteLine, E: WriteLine> {
    pipeline: Build<O, E>,
    meta: Release,
}

impl<O: WriteLine, E: WriteLine> Builder<O, E> {
    /// Creates and returns a new builder using the appropriate pipeline.
    pub fn new(exec: Executor<O, E>, meta: Release, cfg: PgConfig) -> Result<Self, BuildError> {
        let pipeline = if let Some(deps) = meta.dependencies() {
            if let Some(pipe) = deps.pipeline() {
                Build::new(pipe, exec, cfg)?
            } else {
                Build::detect(exec, cfg)?
            }
        } else {
            Build::detect(exec, cfg)?
        };

        Ok(Builder { pipeline, meta })
    }

    /// Configures a distribution to build on a particular platform and
    /// Postgres version.
    pub fn configure(&mut self) -> Result<(), BuildError> {
        match &mut self.pipeline {
            Build::Pgxs(pgxs) => pgxs.configure(),
            Build::Pgrx(pgrx) => pgrx.configure(),
        }
    }

    /// Compiles a distribution on a particular platform and Postgres version.
    pub fn compile(&mut self) -> Result<(), BuildError> {
        match &mut self.pipeline {
            Build::Pgxs(pgxs) => pgxs.compile(),
            Build::Pgrx(pgrx) => pgrx.compile(),
        }
    }

    /// Tests a distribution a particular platform and Postgres version.
    pub fn test(&mut self) -> Result<(), BuildError> {
        match &mut self.pipeline {
            Build::Pgxs(pgxs) => pgxs.test(),
            Build::Pgrx(pgrx) => pgrx.test(),
        }
    }

    /// Installs a distribution on a particular platform and Postgres version.
    pub fn install(&mut self) -> Result<(), BuildError> {
        match &mut self.pipeline {
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
