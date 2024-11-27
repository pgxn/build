use std::{
    collections::{self, HashMap},
    io::{BufRead, BufReader},
    path::Path,
    process::Command,
};

use crate::error::BuildError;

pub(crate) struct PgConfig(HashMap<String, String>);

impl PgConfig {
    /// Executes `pg_config`, parses the output, and returns a `PgConfig`
    /// containing its key/value pairs.
    pub fn new<P: AsRef<Path>>(pg_config: P) -> Result<Self, BuildError> {
        // Execute pg_config.
        let mut cmd = Command::new(pg_config.as_ref().as_os_str());
        let out = cmd
            .output()
            .map_err(|e| BuildError::Command(format!("{:?}", cmd), e.kind().to_string()))?;
        if !out.status.success() {
            return Err(BuildError::Command(
                format!("{:?}", cmd),
                String::from_utf8_lossy(&out.stdout).to_string(),
            ));
        }

        // Parse each line, splitting on " = ".
        let reader = BufReader::new(out.stdout.as_slice());
        let mut cfg = HashMap::new();
        for line in reader.lines().map_while(Result::ok) {
            let mut split = line.splitn(2, " = ");
            if let Some(key) = split.nth(0) {
                if let Some(val) = split.last() {
                    cfg.insert(key.to_ascii_lowercase(), val.to_string());
                }
            }
        }

        Ok(PgConfig(cfg))
    }

    /// Returns the `pg_config` value for `cfg`, which should be a lowercase
    /// string.
    pub fn get(&mut self, cfg: &str) -> Option<&str> {
        match self.0.get(cfg) {
            Some(c) => Some(c.as_str()),
            None => None,
        }
    }

    /// An iterator visiting all `pg_config` key-value pairs in arbitrary
    /// order. Keys are lowercase. The iterator element type is
    /// `(&'a str, &'a str)`.
    pub fn iter(&self) -> collections::hash_map::Iter<'_, String, String> {
        self.0.iter()
    }
}

impl<'h> IntoIterator for &'h PgConfig {
    type Item = <&'h HashMap<String, String> as IntoIterator>::Item;
    type IntoIter = <&'h HashMap<String, String> as IntoIterator>::IntoIter;

    /// Convert into an iterator visiting all `pg_config` key-value pairs in
    /// arbitrary order. Keys are lowercase. The iterator element type is
    /// `(&'a str, &'a str)`.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests;
