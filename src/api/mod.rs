/*!

Interface to local and remote PGXN mirrors and the PGXN API.

*/
use crate::error::BuildError;
use std::{fs::File, io, path::Path, time::Duration};
use url::Url;

/// Interface to the PGXN API.
pub struct Api {
    url: String,
    agent: ureq::Agent,
}

impl Api {
    /// Creates a new Api to access the PGXN API at `url`. Supports `file:`
    /// and `https:` URLs. Pass `proxy` to proxy requests. Returns a
    /// BuildError::Http if the Proxy URL is invalid.
    pub fn new(url: String, proxy: Option<String>) -> Result<Api, BuildError> {
        let mut builder = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(true)
            .user_agent("pgxn-http/1.0");

        if let Some(p) = proxy {
            builder = builder.proxy(ureq::Proxy::new(&p)?);
        }

        Ok(Api {
            url,
            agent: builder.build(),
        })
    }

    /// Download version `version` of `dist` to `dir`.
    pub fn download_to<P: AsRef<Path>>(
        &self,
        dir: P,
        dist: &str,
        version: &str,
    ) -> Result<(), BuildError> {
        // TODO: use URI templates.
        let url = format!("{}/dist/{dist}/{version}/{dist}-{version}.zip", self.url);
        let url = Url::parse(&url)?;
        self.download_url_to(dir, url)
    }

    /// Download `url` to `dir`. The file name must be the last segment of the URL.
    fn download_url_to<P: AsRef<Path>>(&self, dir: P, url: url::Url) -> Result<(), BuildError> {
        // Extract the file name from the URL.
        match url.path_segments() {
            None => Err(BuildError::NoUrlFile(url))?,
            Some(segments) => match segments.last() {
                None => Err(BuildError::NoUrlFile(url))?,
                Some(filename) => {
                    if filename.is_empty() {
                        return Err(BuildError::NoUrlFile(url));
                    }
                    let dst = dir.as_ref().join(filename);

                    if url.scheme() == "file" {
                        // Just copy the file.
                        let src = match url.to_file_path() {
                            Err(_) => Err(BuildError::NoUrlFile(url.clone()))?,
                            Ok(s) => s,
                        };

                        // Eschew std::fs::copy for better error messages.
                        return match File::open(&src) {
                            Err(e) => Err(BuildError::File(
                                "opening",
                                src.display().to_string(),
                                e.kind(),
                            )),
                            Ok(mut input) => match File::create(&dst) {
                                Err(e) => Err(BuildError::File(
                                    "creating",
                                    dst.display().to_string(),
                                    e.kind(),
                                )),
                                Ok(mut out) => match io::copy(&mut input, &mut out) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(BuildError::File(
                                        "copying",
                                        format!("from {} to {}", src.display(), dst.display()),
                                        e.kind(),
                                    )),
                                },
                            },
                        };
                    }

                    // Download the file over HTTP.
                    let res = self.agent.request_url("GET", &url).call()?;
                    match File::create(&dst) {
                        Err(e) => Err(BuildError::File(
                            "create",
                            dst.display().to_string(),
                            e.kind(),
                        )),
                        Ok(mut out) => match io::copy(&mut res.into_reader(), &mut out) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(BuildError::File(
                                "copying",
                                format!("from request to {}", dst.display()),
                                e.kind(),
                            )),
                        },
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
