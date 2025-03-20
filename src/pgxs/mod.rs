//! Builder implementation for [PGXS] Pipelines.
//!
//! [PGXS]: https://www.postgresql.org/docs/current/extend-pgxs.html

use crate::{
    error::BuildError,
    exec::Executor,
    line::{self, WriteLine},
    pg_config::PgConfig,
    pipeline::Pipeline,
};
use log::{debug, info};
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
pub(crate) struct Pgxs<
    O: WriteLine = line::LineWriter<std::io::Stdout>,
    E: WriteLine = line::LineWriter<std::io::Stdout>,
> {
    exec: Executor<O, E>,
    cfg: PgConfig,
}

impl<O, E> Pipeline<O, E> for Pgxs<O, E>
where
    O: WriteLine,
    E: WriteLine,
{
    fn new(exec: Executor<O, E>, cfg: PgConfig) -> Self {
        Pgxs { exec, cfg }
    }

    /// Determines the confidence that the Pgxs pipeline can build the
    /// contents of `dir`. Returns 0 unless the directory contains a Makefile.
    /// Otherwise it returns a score as follows;
    ///
    /// *   Returns 255 if it declares a variable named `PG_CONFIG`.
    /// *   Returns 200 if it declares variables named `MODULES`,
    ///     `MODULE_big`, `PROGRAM`, `EXTENSION`, `DATA`, or `DATA_built`
    /// *   Otherwise returns 127
    fn confidence(dir: impl AsRef<Path>) -> u8 {
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

    /// Returns the Executor passed to [`Self::new`].
    fn executor(&mut self) -> &mut Executor<O, E> {
        &mut self.exec
    }

    /// Returns the PgConfig passed to [`Self::new`].
    fn pg_config(&self) -> &PgConfig {
        &self.cfg
    }

    fn configure(&mut self) -> Result<(), BuildError> {
        // Run configure if it exists.
        if let Ok(ok) = fs::exists(self.exec.dir().join("configure")) {
            if ok {
                info!("running configure");
                // "." will not work on VMS or MacOS Classic.
                let cmd = Path::new(".").join("configure").display().to_string();
                return self.run(&cmd, [""; 0], false);
            }
        }

        Ok(())
    }

    fn compile(&mut self) -> Result<(), BuildError> {
        info!("building extension");
        self.run("make", ["all"], false)?;
        Ok(())
    }

    fn test(&mut self) -> Result<(), BuildError> {
        info!("testing extension");
        self.run("make", ["installcheck"], false)?;
        Ok(())
    }

    fn install(&mut self) -> Result<(), BuildError> {
        info!("installing extension");
        self.run("make", ["install"], true)?;
        Ok(())
    }
}

/// Returns the path to a Makefile in `dir`, or [`None`] if no Makefile
/// exists.
fn makefile(dir: &Path) -> Option<PathBuf> {
    for makefile in ["GNUmakefile", "makefile", "Makefile"] {
        let file = dir.join(makefile);
        if file.exists() {
            debug!("Found {:?}", file);
            return Some(file);
        }
    }
    None
}

#[cfg(test)]
mod tests;
