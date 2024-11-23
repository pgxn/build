//! Builder implementation for [PGXS] Pipelines.
//!
//! [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html

use crate::error::BuildError;
use crate::pipeline::Pipeline;
use log::info;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

/// Builder implementation for [PGXS] Pipelines.
///
/// [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html
#[derive(Debug, PartialEq)]
pub(crate) struct Pgxs<P: AsRef<Path>> {
    sudo: bool,
    dir: P,
}

impl<P: AsRef<Path>> Pipeline<P> for Pgxs<P> {
    fn new(dir: P, sudo: bool) -> Self {
        Pgxs { sudo, dir }
    }

    /// Determines the confidence that the Pgxs pipeline can build the
    /// contents of `dir`. Returns 0 unless the directory contains a Makefile.
    /// Otherwise it returns a score as follows;
    ///
    /// *   Returns 255 if it declares a variable named `PG_CONFIG`.
    /// *   Returns 200 if it declares variables named `MODULES`,
    ///     `MODULE_big`, `PROGRAM`, `EXTENSION`, `DATA`, or `DATA_built`
    /// *   Otherwise returns 127
    fn confidence(dir: P) -> u8 {
        let file = match makefile(dir.as_ref()) {
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

    /// Returns the directory passed to [`Self::new`].
    fn dir(&self) -> &P {
        &self.dir
    }

    fn configure(&self) -> Result<(), BuildError> {
        // Run configure if it exists.
        if let Ok(ok) = fs::exists(self.dir().as_ref().join("configure")) {
            if ok {
                info!("running configure");
                // "." will not work on VMS or MacOS Classic.
                let cmd = Path::new(".").join("configure").display().to_string();
                return self.run(&cmd, [""; 0], false);
            }
        }

        Ok(())
    }

    fn compile(&self) -> Result<(), BuildError> {
        info!("building extension");
        self.run("make", ["all"], self.sudo)?;
        Ok(())
    }

    fn test(&self) -> Result<(), BuildError> {
        info!("testing extension");
        self.run("make", ["installcheck"], self.sudo)?;
        Ok(())
    }

    fn install(&self) -> Result<(), BuildError> {
        info!("installing extension");
        self.run("make", ["install"], self.sudo)?;
        Ok(())
    }
}

/// Returns the path to a Makefile in `dir`, or [`None`] if no Makefile
/// exists.
fn makefile(dir: &Path) -> Option<PathBuf> {
    for makefile in ["GNUmakefile", "makefile", "Makefile"] {
        let file = dir.join(makefile);
        if file.exists() {
            return Some(file);
        }
    }
    None
}

#[cfg(test)]
mod tests;
