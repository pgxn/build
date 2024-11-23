//! PGXN [Dist API].
//!
//! [Dist API]: https://github.com/pgxn/pgxn-api/wiki/dist-api

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, io};

use crate::error::BuildError;

/// Represents a single distribution release in [`Releases`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Release {
    version: Version,
    date: DateTime<Utc>,
}

impl Release {
    /// Borrows the Release version.
    pub fn version(&self) -> &Version {
        self.version.borrow()
    }

    /// Borrows the release date.
    pub fn date(&self) -> &DateTime<Utc> {
        self.date.borrow()
    }
}

/// Represents all the releases for a [`Dist`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Releases {
    #[serde(skip_serializing_if = "Option::is_none")]
    stable: Option<Vec<Release>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unstable: Option<Vec<Release>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    testing: Option<Vec<Release>>,
}

impl Releases {
    /// Borrows the stable releases.
    pub fn stable(&self) -> Option<&[Release]> {
        self.stable.as_deref()
    }

    /// Borrows the unstable releases.
    pub fn unstable(&self) -> Option<&[Release]> {
        self.unstable.as_deref()
    }

    /// Borrows the testing releases.
    pub fn testing(&self) -> Option<&[Release]> {
        self.testing.as_deref()
    }
}

/// Represents the release data for a distribution name. Loaded from the PGXN
/// [Dist API].
///
///  [Dist API]: https://github.com/pgxn/pgxn-api/wiki/dist-api
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Dist {
    name: String,
    releases: Releases,
}

impl Dist {
    /// Loads a [`Dist`] from an [`std::io::Read`].
    pub fn from_reader<R: io::Read>(rdr: R) -> Result<Dist, BuildError> {
        let dist: Dist = serde_json::from_reader(rdr)?;
        Ok(dist)
    }

    /// Borrows the Dist name
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Borrows the Dist releases.
    pub fn releases(&self) -> &Releases {
        self.releases.borrow()
    }

    /// Finds and returns the best version to install, preferring the latest
    /// stable version. If there are no stable versions, it tries to return
    /// the latest testing version. If there are no testing versions, it
    /// returns the latest unstable versions. Returns an error if there are no
    /// versions at all.
    pub fn best_version(&self) -> Result<&Version, BuildError> {
        if let Some(v) = self.latest_stable_version() {
            return Ok(v);
        }
        if let Some(v) = self.latest_testing_version() {
            return Ok(v);
        }
        if let Some(v) = self.latest_unstable_version() {
            return Ok(v);
        }

        Err(BuildError::Invalid("missing release data"))
    }

    /// Finds and returns the latest stable version.
    pub fn latest_stable_version(&self) -> Option<&Version> {
        latest_version(self.releases.stable())
    }

    /// Finds and returns the latest unstable version.
    pub fn latest_unstable_version(&self) -> Option<&Version> {
        latest_version(self.releases.unstable())
    }

    /// Finds and returns the latest testing version.
    pub fn latest_testing_version(&self) -> Option<&Version> {
        latest_version(self.releases.testing())
    }
}

fn latest_version(releases: Option<&[Release]>) -> Option<&Version> {
    match releases {
        None => None,
        Some(list) => Some(list[0].version()),
    }
}

#[cfg(test)]
mod tests;
