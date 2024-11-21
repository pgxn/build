//! Builder implementation for [PGXS] Pipelines.
//!
//! [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

/// Builder implementation for [PGXS] Pipelines.
///
/// [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html
#[derive(Debug, PartialEq)]
pub(crate) struct Pgxs {
    dir: PathBuf,
    sudo: bool,
}

impl Pipeline for Pgxs {
    fn new(dir: PathBuf, sudo: bool) -> Self {
        Pgxs { dir, sudo }
    }

    /// Determines the confidence that the Pgxs pipeline can build the
    /// contents of `dir`. Returns 0 unless the directory contains a Makefile.
    /// Otherwise it returns a score as follows;
    ///
    /// *   Returns 255 if it declares a variable named `PG_CONFIG`.
    /// *   Returns 200 if it declares variables named `MODULES`,
    ///     `MODULE_big`, `PROGRAM`, `EXTENSION`, `DATA`, or `DATA_built`
    /// *   Otherwise returns 127
    fn confidence(dir: &Path) -> u8 {
        let file = match makefile(dir) {
            Some(f) => f,
            None => return 0,
        };

        // https://www.postgresql.org/docs/current/extend-pgxs.html
        // https://github.com/postgres/postgres/blob/master/src/makefiles/pgxs.mk
        let mut score: u8 = 127;
        if let Ok(file) = File::open(file) {
            let reader = BufReader::new(file);
            let pgc_rx = Regex::new(r"^PG_CONFIG\s*[:?]?=\s*").unwrap();
            let var_rx =
                Regex::new(r"^(MODULE(?:S|_big)|PROGRAM|EXTENSION|DATA(?:_built)?)\s*[:?]?=")
                    .unwrap();
            for line in reader.lines().map_while(Result::ok) {
                if pgc_rx.is_match(&line) {
                    // Full confidence
                    return 255;
                }
                if var_rx.is_match(&line) {
                    // Probably
                    score = 200;
                }
            }
        }

        // Probably can do `make all && make install`, probably not `installcheck`.
        score
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

    fn install(&self) -> Result<(), BuildError> {
        Ok(())
    }
}

fn makefile(dir: &Path) -> Option<PathBuf> {
    for makefile in ["GNUmakefile", "makefile", "Makefile"] {
        let file = dir.join(makefile);
        if file.exists() {
            return Some(file);
        }
    }
    None
}
