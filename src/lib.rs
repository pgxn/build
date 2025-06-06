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

use crate::{error::BuildError, exec::Executor, pgrx::Pgrx, pgxs::Pgxs, pipeline::Pipeline};
pub use api::Api;
use line::WriteLine;
use owo_colors::Style;
pub use pg_config::PgConfig;
use pgxn_meta::{dist, release::Release};
use std::{
    io,
    path::{Path, PathBuf},
};
use supports_color::Stream;

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
    fn new(pipe: &dist::Pipeline, exec: Executor, cfg: PgConfig) -> Result<Build, BuildError> {
        match pipe {
            dist::Pipeline::Pgxs => Ok(Build::Pgxs(Pgxs::new(exec, cfg))),
            dist::Pipeline::Pgrx => Ok(Build::Pgrx(Pgrx::new(exec, cfg))),
            _ => Err(BuildError::UnknownPipeline(pipe.to_string())),
        }
    }

    /// Attempts to detect and return the appropriate build pipeline to build
    /// the contents of `dir`. Returns an error if no pipeline can do so.
    fn detect(exec: Executor, cfg: PgConfig) -> Result<Build, BuildError> {
        // Start with PGXS.
        let mut score = Pgxs::confidence(exec.dir());
        let mut pipe = dist::Pipeline::Pgxs;

        // Does pgrx have a higher score?
        let c = Pgrx::confidence(exec.dir());
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
#[derive(Debug, PartialEq)]
pub struct Builder {
    pipeline: Build,
    meta: Release,
}

impl Builder {
    /// Creates and returns a new builder using the appropriate pipeline.
    pub fn new(dir: impl Into<PathBuf>, meta: Release, cfg: PgConfig) -> Result<Self, BuildError> {
        let exec = Executor::new(dir.into(), stdout(), stderr());
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

/// Returns a WriteLine implementation that prints to STDOUT. Returns a
/// [line::ColorLine] that styles text dimmed grey if STDOUT supports color.
/// Otherwise returns a [line::LineWriter].
fn stdout() -> Box<dyn WriteLine> {
    if cfg!(test) {
        return Box::new(line::LineWriter::new(vec![]));
    }
    _styled_stdout()
}

fn _styled_stdout() -> Box<dyn WriteLine> {
    if supports_color::on(Stream::Stdout).is_some() {
        return Box::new(line::ColorLine::new(
            io::stdout(),
            Style::new().white().dimmed(),
        ));
    }
    Box::new(line::LineWriter::new(io::stdout()))
}

/// Returns a WriteLine implementation that prints to STDERR. Returns a
/// [line::ColorLine] that styles text red if STDERR supports color. Otherwise
/// returns a [line::LineWriter].
fn stderr() -> Box<dyn WriteLine> {
    if cfg!(test) {
        return Box::new(line::LineWriter::new(vec![]));
    }
    _styled_stderr()
}

fn _styled_stderr() -> Box<dyn WriteLine> {
    if supports_color::on(Stream::Stderr).is_some() {
        return Box::new(line::ColorLine::new(io::stderr(), Style::new().red()));
    }
    Box::new(line::LineWriter::new(io::stderr()))
}

#[cfg(test)]
mod tests;
