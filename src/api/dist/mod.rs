//! PGXN [Dist API].
//!
//! [Dist API]: https://github.com/pgxn/pgxn-api/wiki/dist-api

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, io};

use crate::error::BuildError;

#[cfg(test)]
mod tests;

/// Represents a single distribution release in [`Release`].
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
}
