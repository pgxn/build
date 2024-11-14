use super::*;
use httpmock::prelude::*;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

#[test]
fn constructor() -> Result<(), BuildError> {
    let url = String::from("https://api.pgxn.org");
    let api = Api::new(url.clone(), None)?;
    assert_eq!(url, api.url);
    let cfg = format!("{:?}", api.agent);
    assert!(cfg.contains("timeout_read: Some(5s)"));
    assert!(cfg.contains("timeout_write: Some(5s)"));
    assert!(cfg.contains("https_only: true"));
    assert!(cfg.contains("user_agent: \"pgxn-http/1.0\""));
    assert!(cfg.contains("proxy: None"));

    Ok(())
}

#[test]
fn constructor_proxy() -> Result<(), BuildError> {
    let url = "https://root.pgxn.org";
    let proxy = "socks5://john:smith@socks.google.com";
    let api = Api::new(url.to_string(), Some(proxy.to_string()))?;
    assert_eq!(url, api.url);
    let cfg = format!("{:?}", api.agent);
    assert!(cfg.contains("timeout_read: Some(5s)"));
    assert!(cfg.contains("timeout_write: Some(5s)"));
    assert!(cfg.contains("https_only: true"));
    assert!(cfg.contains("user_agent: \"pgxn-http/1.0\""));
    assert!(cfg.contains("Some(Proxy { server: \"socks.google.com\", port: 1080, user: Some(\"john\"), password: Some(\"smith\"), proto: SOCKS5 })"));

    Ok(())
}

#[test]
fn download_file() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus");
    let url = format!("file://{}", dir.display());
    let tmp_dir = tempdir()?;
    let exp_path = tmp_dir.as_ref().join("pair-0.1.7.zip");

    // Download the file.
    assert!(!exp_path.exists());
    let api = Api::new(url.to_string(), None)?;
    api.download_to(tmp_dir.as_ref(), "pair", "0.1.7")?;
    assert!(exp_path.exists());

    // Make sure it's the same file.
    let src_path = dir
        .join("dist")
        .join("pair")
        .join("0.1.7")
        .join("pair-0.1.7.zip");
    assert!(src_path.exists());
    files_eq(src_path, exp_path)?;

    Ok(())
}

#[test]
fn download_http() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus");
    let src_path = dir
        .join("dist")
        .join("pair")
        .join("0.1.7")
        .join("pair-0.1.7.zip");

    // Start a lightweight mock server.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dist/pair/0.1.7/pair-0.1.7.zip");
        then.status(200)
            .header("content-type", "application/zip")
            .body_from_file(src_path.display().to_string());
    });

    // Create a client and disable TLS.
    let api = Api {
        url: server.base_url(),
        agent: ureq::agent(),
    };

    // Download the file.
    let tmp_dir = tempdir()?;
    let exp_path = tmp_dir.as_ref().join("pair-0.1.7.zip");
    assert!(!exp_path.exists());
    api.download_to(tmp_dir.as_ref(), "pair", "0.1.7")?;
    assert!(exp_path.exists());
    mock.assert();

    Ok(())
}

#[test]
fn download_errors() -> Result<(), BuildError> {
    let api = Api::new("x:y".to_string(), None)?;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dst = dir.join("nope");

    for (name, dir, url, err) in [
        (
            "no segments",
            &dst,
            "data:text/plain,HelloWorld".to_string(),
            "missing file name segment from data:text/plain,HelloWorld".to_string(),
        ),
        (
            "empty segments",
            &dst,
            "http://example.com".to_string(),
            "missing file name segment from http://example.com/".to_string(),
        ),
        (
            "not tls",
            &dst,
            "http://example.com/foo.text".to_string(),
            "http://example.com/foo.text: Insecure request attempted with https_only set: can't perform non https request with https_only set".to_string(),
        ),
        (
            "nonexistent file",
            &dst,
            format!("file://{}", dir.join("nope.txt").display()),
            format!("opening {}: {}", dir.join("nope.txt").display(), io::ErrorKind::NotFound),
        ),
        (
            "nonexistent destination",
            &dst,
            format!("file://{}", dir.join("Cargo.toml").display()),
            format!(
                "creating {}: {}",
                dst.join("Cargo.toml").display(),
                io::ErrorKind::NotFound
            ),
        ),
    ] {
        match api.download_url_to(dir.clone(), Url::parse(&url)?) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

fn files_eq<P: AsRef<Path>>(left: P, right: P) -> Result<(), io::Error> {
    let left = std::fs::read(left)?;
    let right = std::fs::read(right)?;
    let left = Sha256::digest(left);
    let right = Sha256::digest(right);
    assert_eq!(left, right);
    Ok(())
}
