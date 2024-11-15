/*!

Interface to local and remote PGXN mirrors and the PGXN API.

*/
mod dist;
pub use dist::{Dist, Release, Releases};

use crate::error::BuildError;
use iri_string::spec;
use iri_string::template::{simple_context::SimpleContext, UriTemplateStr, UriTemplateString};
use log::{debug, info, trace};
use semver::Version;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

/// Returns a string representation of the final segment of a Path.
macro_rules! filename {
    ( $x:expr ) => {{
        $x.as_ref()
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("UNKNOWN"))
            .to_string_lossy()
    }};
}

/// Interface to the PGXN API.
pub struct Api {
    url: url::Url,
    agent: ureq::Agent,
    templates: HashMap<String, UriTemplateString>,
}

impl Api {
    /// Creates a new Api to access the PGXN API at `url`. Supports `file:`
    /// and `https:` URLs. Pass `proxy` to proxy requests. Returns a
    /// BuildError::Http if the Proxy URL is invalid. The `url` and `proxy`
    /// values are borrowed only for the duration of this function.
    pub fn new(url: &str, proxy: Option<&str>) -> Result<Api, BuildError> {
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        let mut builder = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(true)
            .user_agent(APP_USER_AGENT);

        if let Some(p) = proxy {
            builder = builder.proxy(ureq::Proxy::new(p)?);
        }

        let url = parse_base_url(url)?;
        let agent = builder.build();
        let idx = url.join("index.json")?;
        let templates = fetch_templates(&agent, &idx)?;

        Ok(Api {
            url,
            agent,
            templates,
        })
    }

    /// Fetch the distribution release data for distribution `name`.
    pub fn dist(&self, name: &str) -> Result<Dist, BuildError> {
        let mut ctx = SimpleContext::new();
        ctx.insert("dist", name);
        let url = self.url_for("dist", ctx)?;
        let read = fetch_reader(&self.agent, &url)?;
        Dist::from_reader(read)
    }

    /// Fetch the distribution release metadata for distribution `name`
    /// version `version`.
    pub fn meta(
        &self,
        name: &str,
        version: &Version,
    ) -> Result<pgxn_meta::release::Release, BuildError> {
        let mut ctx = SimpleContext::new();
        ctx.insert("dist", name);
        ctx.insert("version", version.to_string());
        let url = self.url_for("meta", ctx)?;
        let mut val = fetch_json(&self.agent, &url)?;
        debug!(url:display; "parsing");
        if val.get("meta-spec").is_none() {
            // PGXN v1 stripped meta-spec out of this API :-/.
            let val_type = type_of(&val);
            val.as_object_mut()
                .ok_or_else(|| BuildError::Type(url.to_string(), "object", val_type))?
                .insert("meta-spec".to_string(), json!({"version": "1.0.0"}));
        }
        let rel = pgxn_meta::release::Release::try_from(val)?;
        Ok(rel)
    }

    /// Unpack download `file` in directory `into` and return the path to the
    /// unpacked directory.
    pub fn unpack<P: AsRef<Path>>(&self, into: P, file: P) -> Result<PathBuf, BuildError> {
        info!(file:display = filename!(file); "unpacking");
        let zip = File::open(file)?;
        let mut archive = zip::ZipArchive::new(zip)?;
        archive.extract(&into)?;
        let first = archive
            .by_index(0)?
            .enclosed_name()
            .ok_or_else(|| zip::result::ZipError::FileNotFound)?;
        Ok(into.as_ref().join(first))
    }

    /// url_for finds the `name` template, evaluates with `ctx`, and returns a
    /// [url::Url] relative to the base URL passed to new().
    fn url_for(&self, name: &str, ctx: SimpleContext) -> Result<url::Url, BuildError> {
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| BuildError::UnknownTemplate(name.to_string()))?;
        let path = template.expand::<spec::UriSpec, _>(&ctx)?;
        let url = self.url.join(&path.to_string())?;
        trace!(url:display; "resolved URL");
        Ok(url)
    }

    /// Download the archive for release `meta` to `dir` and validate it
    /// against the digests in `meta`. Returns the full path to the file.
    pub fn download_to<P: AsRef<Path>>(
        &self,
        dir: P,
        meta: &pgxn_meta::release::Release,
    ) -> Result<PathBuf, BuildError> {
        let mut ctx = SimpleContext::new();
        ctx.insert("dist", meta.name());
        ctx.insert("version", meta.version().to_string());
        let url = self.url_for("download", ctx)?;
        info!(url:display; "downloading");
        let file = self.download_url_to(dir, url)?;
        info!(file:display = file.display(); "validating");
        meta.release().digests().validate(&file)?;
        Ok(file)
    }

    /// Download `url` to `dir`. The file name must be the last segment of the
    /// URL. Returns the full path to the file.
    fn download_url_to<P: AsRef<Path>>(
        &self,
        dir: P,
        url: url::Url,
    ) -> Result<PathBuf, BuildError> {
        trace!( url:display, dir:display = dir.as_ref().display(); "downloading");
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
                        // Copy the file. Eschew std::fs::copy for better
                        // error messages.
                        let mut input = get_file(&url)?;
                        return match File::create(&dst) {
                            Err(e) => Err(BuildError::File(
                                "creating",
                                dst.display().to_string(),
                                e.kind(),
                            )),
                            Ok(mut out) => match io::copy(&mut input, &mut out) {
                                Ok(_) => Ok(dst),
                                Err(e) => Err(BuildError::File(
                                    "copying",
                                    format!(
                                        "from {} to {}",
                                        url.to_file_path().unwrap().display(),
                                        dst.display()
                                    ),
                                    e.kind(),
                                )),
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
                            Ok(_) => Ok(dst),
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

/// parse_base_url parses `url` into a [`url::Url`], ensuring that it always
/// ends in a slash, so that it can be properly used as a base URL.
fn parse_base_url(url: &str) -> Result<url::Url, url::ParseError> {
    if url.ends_with("/") {
        Url::parse(url)
    } else {
        let url = format!("{url}/");
        Url::parse(&url)
    }
}

/// type_of returns a the type of `v`.
fn type_of(v: &Value) -> &'static str {
    match v {
        Value::Array(_) => "array",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
        Value::Number(_) => "number",
        Value::Object(_) => "object",
        Value::String(_) => "string",
    }
}

/// Fetches the JSON at URL and converts it to a serde_json::Value.
fn fetch_json(agent: &ureq::Agent, url: &url::Url) -> Result<Value, BuildError> {
    debug!(url:display; "fetching");
    match url.scheme() {
        "file" => Ok(serde_json::from_reader(get_file(url)?)?),
        // Avoid .into_json(); it returns IO errors.
        "http" | "https" => Ok(serde_json::from_reader(
            agent.request_url("GET", url).call()?.into_reader(),
        )?),
        s => Err(BuildError::Scheme(s.to_string())),
    }
}

/// Fetches the JSON at URL and converts it to a serde_json::Value.
fn fetch_reader(
    agent: &ureq::Agent,
    url: &url::Url,
) -> Result<Box<dyn io::Read + Send + Sync + 'static>, BuildError> {
    debug!(url:display; "fetching");
    match url.scheme() {
        "file" => Ok(Box::new(get_file(url)?)),
        // Avoid .into_json(); it returns IO errors.
        "http" | "https" => Ok(agent.request_url("GET", url).call()?.into_reader()),
        s => Err(BuildError::Scheme(s.to_string())),
    }
}

/// Opens a the file on disk that `url` points to. The scheme in `url` must be
/// `file`.
fn get_file(url: &url::Url) -> Result<File, BuildError> {
    let src = match url.to_file_path() {
        Err(_) => Err(BuildError::NoUrlFile(url.clone()))?,
        Ok(s) => s,
    };
    // if src.is_dir() {
    //     return Err(BuildError::File(
    //         "opening",
    //         src.display().to_string(),
    //         io::ErrorKind::IsADirectory,
    //     ));
    // }
    match File::open(&src) {
        Ok(fh) => Ok(fh),
        Err(e) => Err(BuildError::File(
            "opening",
            src.display().to_string(),
            e.kind(),
        )),
    }
}

/// Fetches and loads the templates file from `url`, returning a HashMap with
/// template names pointing to UriTemplateString values.
fn fetch_templates(
    agent: &ureq::Agent,
    url: &url::Url,
) -> Result<HashMap<String, UriTemplateString>, BuildError> {
    let val = fetch_json(agent, url)?;
    let obj = val
        .as_object()
        .ok_or_else(|| BuildError::Type(url.to_string(), "object", type_of(&val)))?;

    let mut map: HashMap<String, UriTemplateString> = HashMap::with_capacity(obj.len());
    for (k, v) in obj.into_iter() {
        let str = v.as_str().ok_or_else(|| {
            BuildError::Type(
                format!("template {} in {}", json!(k), url),
                "string",
                type_of(&val),
            )
        })?;

        // Remove leading / if present and parse it into a template.
        let str = str.strip_prefix("/").unwrap_or(str);
        map.insert(k.to_string(), UriTemplateStr::new(str)?.to_owned());
    }

    Ok(map)
}

#[cfg(test)]
mod tests;
