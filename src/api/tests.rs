use super::*;
use httpmock::prelude::*;
use sha2::{Digest, Sha256};
use std::io::Read;
use tempfile::tempdir;
use ureq::json;

fn corpus_dir() -> Box<std::path::PathBuf> {
    Box::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus"))
}

fn ua() -> String {
    format!(
        "user_agent: \"{}\"",
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
    )
}

#[test]
fn constructor() -> Result<(), BuildError> {
    let url = format!("file://{}", corpus_dir().display());
    let exp_url = format!("{url}/");
    let exp_url = Url::parse(&exp_url)?;
    let api = Api::new(&url, None)?;
    assert_eq!(exp_url, api.url);
    let idx = exp_url.join("index.json")?;
    assert_eq!(fetch_templates(&api.agent, &idx)?, api.templates);
    let cfg = format!("{:?}", api.agent);
    assert!(cfg.contains("timeout_read: Some(5s)"));
    assert!(cfg.contains("timeout_write: Some(5s)"));
    assert!(cfg.contains("https_only: true"));
    let ua = ua();
    assert!(cfg.contains(&ua));
    assert!(cfg.contains("proxy: None"));

    Ok(())
}

#[test]
fn constructor_proxy() -> Result<(), BuildError> {
    let url = format!("file://{}/", corpus_dir().display());
    let exp_url = Url::parse(&url)?;
    let proxy = "socks5://john:smith@socks.google.com";
    let api = Api::new(&url, Some(proxy))?;
    assert_eq!(exp_url, api.url);
    let idx = exp_url.join("index.json")?;
    assert_eq!(fetch_templates(&api.agent, &idx)?, api.templates);
    let cfg = format!("{:?}", api.agent);
    assert!(cfg.contains("timeout_read: Some(5s)"));
    assert!(cfg.contains("timeout_write: Some(5s)"));
    assert!(cfg.contains("https_only: true"));
    let ua = ua();
    assert!(cfg.contains(&ua));
    assert!(cfg.contains("Some(Proxy { server: \"socks.google.com\", port: 1080, user: Some(\"john\"), password: Some(\"smith\"), proto: SOCKS5 })"));

    Ok(())
}

#[test]
fn download_file() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let url = format!("file://{}", dir.display());

    // Load the distribution release meta.
    let api = Api::new(&url, None)?;
    let v = Version::new(0, 1, 7);
    let meta = api.meta("pair", &v)?;

    // Download the file.
    let tmp_dir = tempdir()?;
    let exp_path = tmp_dir.as_ref().join("pair-0.1.7.zip");
    assert!(!exp_path.exists());
    assert_eq!(
        tmp_dir.path().join("pair-0.1.7.zip"),
        api.download_to(tmp_dir.as_ref(), &meta)?
    );
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
    let dir = corpus_dir();
    let src_path = dir.join("dist").join("pair").join("0.1.7");

    // Start a lightweight mock server.
    let server = MockServer::start();
    let idx_url = format!("file://{}/index.json", dir.display());
    let idx_url = Url::parse(&idx_url)?;
    let agent = ureq::agent();
    let templates = fetch_templates(&agent, &idx_url)?;

    // Create a client and disable TLS.
    let api = Api {
        url: Url::parse(&server.url("/"))?,
        agent,
        templates,
    };

    // Load the distribution release meta.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dist/pair/0.1.7/META.json");
        then.status(200)
            .header("content-type", "application/json")
            .body_from_file(src_path.join("META.json").display().to_string());
    });
    let v = Version::new(0, 1, 7);
    let meta = api.meta("pair", &v)?;
    mock.assert();

    // Download the file.
    let mut mock = server.mock(|when, then| {
        when.method(GET).path("/dist/pair/0.1.7/pair-0.1.7.zip");
        then.status(200)
            .header("content-type", "application/zip")
            .body_from_file(src_path.join("pair-0.1.7.zip").display().to_string());
    });
    let tmp_dir = tempdir()?;
    let exp_path = tmp_dir.as_ref().join("pair-0.1.7.zip");
    assert!(!exp_path.exists());
    assert_eq!(exp_path, api.download_to(tmp_dir.as_ref(), &meta)?);
    assert!(exp_path.exists());
    mock.assert();
    mock.delete();

    // Try a validation failure.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dist/pair/0.1.7/pair-0.1.7.zip");
        then.status(200)
            .header("content-type", "application/zip")
            .body_from_file(src_path.join("META.json").display().to_string());
    });
    let res = api.download_to(tmp_dir.as_ref(), &meta);
    mock.assert();
    assert!(res.is_err());
    assert_eq!("SHA-1 digest cafa55f06cdc9861b23de72687024b02322ad21c does not match 5b9e3ba948b18703227e4dea17696c0f1d971759", res.unwrap_err().to_string());

    Ok(())
}

#[test]
fn download_file_errors() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let url = format!("file://{}", corpus_dir().display());
    let api = Api::new(&url, None)?;
    let dst = dir.join("nope");
    let tmp = tempdir()?;

    for (name, dir, url, err) in [
        (
            "no segments",
            dst.as_path(),
            "data:text/plain,HelloWorld".to_string(),
            "missing file name segment from data:text/plain,HelloWorld".to_string(),
        ),
        (
            "empty segments",
            dst.as_path(),
            "http://example.com".to_string(),
            "missing file name segment from http://example.com/".to_string(),
        ),
        (
            "not tls",
            dst.as_path(),
            "http://example.com/foo.text".to_string(),
            "http://example.com/foo.text: Insecure request attempted with https_only set: can't perform non https request with https_only set".to_string(),
        ),
        (
            "nonexistent file",
            dst.as_path(),
            format!("file://{}", dir.join("nope.txt").display()),
            format!("opening {}: {}", dir.join("nope.txt").display(), io::ErrorKind::NotFound),
        ),
        (
            "nonexistent destination",
            dst.as_path(),
            format!("file://{}", dir.join("index.json").display()),
            format!(
                "creating {}: {}",
                dst.join("index.json").display(),
                io::ErrorKind::NotFound
            ),
        ),
        (
            "directory source",
            tmp.as_ref(),
            format!("file://{}", dir.join("dist").display()),
            if cfg!(windows) {
                format!(
                    "opening {}: {}",
                    dir.join("dist").display(),
                    io::ErrorKind::PermissionDenied,
                )
            } else {
                format!(
                    "copying from {} to {}: {}",
                    dir.join("dist").display(),
                    tmp.as_ref().join("dist").display(),
                    "is a directory", // io::ErrorKind::IsADirectory,
                )
            },
        ),
    ] {
        match api.download_url_to(dir, Url::parse(&url)?) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn download_http_errors() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let dst = dir.join("nope");
    // let tmp = tempdir()?;

    // Start a lightweight mock server.
    let server = MockServer::start();
    let base_url = Url::parse(&server.url("/"))?;
    let idx_url = format!("file://{}/index.json", dir.display());
    let idx_url = Url::parse(&idx_url)?;
    let agent = ureq::agent();
    let templates = fetch_templates(&agent, &idx_url)?;

    // Create a client and disable TLS.
    let api = Api {
        url: Url::parse(&server.url("/"))?,
        agent,
        templates,
    };

    for (name, dir, url, mock, err) in [
        (
            "nonexistent destination",
            dst.as_path(),
            base_url.join("index.txt")?,
            server.mock(|when, then| {
                when.method(GET).path("/index.txt");
                then.status(200).body("hello");
            }),
            format!(
                "creating {}: {}",
                dst.join("index.txt").display(),
                io::ErrorKind::NotFound,
            ),
        ),
        // No way to get ureq.Response.into_reader to return a useless reader.
        // (
        //     "nonexistent source",
        //     tmp.as_ref(),
        //     base_url.join("empty.txt")?,
        //     server.mock(|when, then| {
        //         when.method(GET).path("/empty.txt");
        //         then.status(200);
        //     }),
        //     format!(
        //         "copying {} to {}: {}",
        //         base_url.join("index.txt")?,
        //         tmp.as_ref().join("empty.txt").display(),
        //         io::ErrorKind::NotFound,
        //     ),
        // ),
    ] {
        match api.download_url_to(dir, url) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => {
                assert_eq!(err, e.to_string(), "{name}");
                mock.assert();
            }
        }
    }

    Ok(())
}

#[test]
fn type_of_fn() {
    for (exp, val) in [
        ("string", json!("hi")),
        ("boolean", json!(true)),
        ("number", json!(42)),
        ("null", json!(null)),
        ("object", json!({})),
        ("array", json!([])),
    ] {
        assert_eq!(exp, type_of!(val), "{exp}");
    }
}
#[test]
fn get_file_fn() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        dir.join("README.md"),
        dir.join("Cargo.toml"),
        dir.join("LICENSE.md"),
    ] {
        let url = format!("file://{}", file.display());
        let url = Url::parse(&url)?;
        let mut fh = get_file(&url)?;
        let mut exp = File::open(file)?;
        read_eq(&mut exp, &mut fh)?;
    }
    Ok(())
}

#[test]
fn get_file_err() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let nonesuch = dir.join("nonesuch.txt");
    let nonesuch = nonesuch.display();

    for (name, url, err) in [
        (
            "not a file",
            "http://x.y".to_string(),
            "missing file name segment from http://x.y/".to_string(),
        ),
        (
            "nonexistent file",
            format!("file://{nonesuch}"),
            format!("opening {nonesuch}: entity not found"),
        ),
        // Due in next release?
        // https://github.com/rust-lang/rust/pull/128316/files
        // https://github.com/rust-lang/rust/issues/86442
        // (
        //     "directory",
        //     format!("file://{dir}"),
        //     format!("opening {dir}: not a file"),
        // ),
    ] {
        let url = Url::parse(&url)?;
        match get_file(&url) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn fetch_json_file() -> Result<(), BuildError> {
    // Test with index.json.
    let dir = corpus_dir();
    let url = format!("file://{}/index.json", dir.display());
    let url = Url::parse(&url)?;

    let agent = ureq::agent();
    let json = fetch_json(&agent, &url)?;
    assert_eq!(index_json(), json);

    Ok(())
}

#[test]
fn fetch_reader_fn() -> Result<(), BuildError> {
    // Fetch via file://.
    let dir = corpus_dir();
    let url = format!("file://{}/index.json", dir.display());
    let url = Url::parse(&url)?;
    let agent = ureq::agent();
    let json = fetch_reader(&agent, &url)?;
    let json: Value = serde_json::from_reader(json)?;
    assert_eq!(index_json(), json);

    // Fail fetch via file://.
    let url = format!("file://{}/nonesuch.txt", dir.display());
    let url = Url::parse(&url)?;
    match fetch_reader(&agent, &url) {
        Ok(_) => panic!("404 unexpectedly succeeded"),
        Err(e) => assert_eq!(
            format!(
                "opening {}: {}",
                dir.join("nonesuch.txt").display(),
                io::ErrorKind::NotFound
            ),
            e.to_string(),
            "404"
        ),
    }

    // Fetch via http://.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/some.json");
        then.status(200)
            .header("content-type", "text/plain")
            .body("greetings");
    });

    let url = Url::parse(&server.url("/some.json"))?;
    let read = fetch_reader(&agent, &url)?;
    assert_eq!("greetings", std::io::read_to_string(read)?);
    mock.assert();

    // Fail fetch via http://
    let mock = server.mock(|when, then| {
        when.method(GET).path("/nonesuch.json");
        then.status(404)
            .header("content-type", "text/plain")
            .body("not found");
    });
    let url = Url::parse(&server.url("/nonesuch.json"))?;
    match fetch_reader(&agent, &url) {
        Ok(_) => panic!("404 unexpectedly succeeded"),
        Err(e) => assert_eq!(format!("{url}: status code 404"), e.to_string(), "404"),
    }
    mock.assert();

    // Try unsupported scheme.
    let url = Url::parse("ftp://hi")?;
    match fetch_reader(&agent, &url) {
        Ok(_) => panic!("ftp unexpectedly succeeded"),
        Err(e) => assert_eq!("unsupported URL scheme: ftp", e.to_string(), "ftp"),
    }

    Ok(())
}

#[test]
fn fetch_json_http() -> Result<(), BuildError> {
    // Start a lightweight mock server.
    let server = MockServer::start();
    let agent = ureq::agent();
    let base_url = Url::parse(&server.base_url())?;

    // Try a successful request.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/xyz/some.json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"a": true, "x": null}));
    });

    let url = base_url.join("/xyz/some.json")?;
    let json = fetch_json(&agent, &url)?;
    mock.assert();
    assert_eq!(json!({"a": true, "x": null}), json, "json ok");

    // Try a 404 error
    let mock = server.mock(|when, then| {
        when.method(GET).path("/xyz/nonesuch.json");
        then.status(404).body("not found");
    });

    let url = base_url.join("/xyz/nonesuch.json")?;
    let exp = format!("{url}: status code 404");
    match fetch_json(&agent, &url) {
        Ok(_) => panic!("404 unexpectedly succeeded"),
        Err(e) => assert_eq!(exp, e.to_string(), "404"),
    }
    mock.assert();

    // Try invalid JSON.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/xyz/readme.md");
        then.status(200)
            .header("content-type", "text/plain; charset=UTF-8")
            .body("PGXN FTW!");
    });

    let url = base_url.join("/xyz/readme.md")?;
    let exp = "invalid JSON: expected value at line 1 column 1";
    match fetch_json(&agent, &url) {
        Ok(_) => panic!("bad JSON unexpectedly succeeded"),
        Err(e) => assert_eq!(exp, e.to_string(), "404"),
    }
    mock.assert();

    Ok(())
}

#[test]
fn fetch_json_err() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let agent = ureq::agent();
    let nonesuch = dir.join("nonesuch.txt");
    let html = dir.join("index.html");

    for (name, url, err) in [
        (
            "unsupported scheme",
            "data:text/plain,HelloWorld".to_string(),
            "unsupported URL scheme: data".to_string(),
        ),
        (
            "nonexistent file",
            format!("file://{}", nonesuch.display()),
            format!("opening {}: entity not found", nonesuch.display()),
        ),
        (
            "not JSON",
            format!("file://{}", html.display()),
            "invalid JSON: expected value at line 1 column 1".to_string(),
        ),
    ] {
        let url = Url::parse(&url)?;
        match fetch_json(&agent, &url) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn fetch_templates_fn() -> Result<(), BuildError> {
    // Construct expected HashMap.
    let idx = index_json();
    let mut exp: HashMap<String, UriTemplateString> = HashMap::new();
    for (k, v) in idx.as_object().unwrap().into_iter() {
        let v = v.as_str().unwrap();
        let v = v.strip_prefix("/").unwrap();
        exp.insert(k.to_string(), UriTemplateStr::new(v)?.to_owned());
    }

    // Fetch and compare.
    let dir = corpus_dir();
    let url = format!("file://{}/index.json", dir.display());
    let url = Url::parse(&url)?;
    let agent = ureq::agent();
    let templates = fetch_templates(&agent, &url)?;
    assert_eq!(exp, templates);
    Ok(())
}

#[test]
fn fetch_templates_err() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let agent = ureq::agent();

    // Set up an invalid index.json.
    let tmp_dir = tempdir()?;
    let array_path = tmp_dir.path().join("array.json");
    let array_url = format!("file://{}", array_path.display());
    let array_url = Url::parse(&array_url)?;
    let tmp_file = File::create(&array_path)?;
    serde_json::to_writer(&tmp_file, &json!(["not an object"]))?;
    tmp_file.sync_all()?;

    // Set up an object with non-string value.
    let bad_obj_path = tmp_dir.path().join("bad_obj.json");
    let bad_obj_url = format!("file://{}", bad_obj_path.display());
    let bad_obj_url = Url::parse(&bad_obj_url)?;
    let tmp_file = File::create(&bad_obj_path)?;
    serde_json::to_writer(&tmp_file, &json!({"thing": ["oops"]}))?;
    tmp_file.sync_all()?;

    // Set up an object with an invalid URI path.
    let bad_plate_path = tmp_dir.path().join("bad_template.json");
    let tmp_file = File::create(&bad_plate_path)?;
    serde_json::to_writer(&tmp_file, &json!({"thing": "/foo/{xyz/"}))?;
    tmp_file.sync_all()?;

    for (name, url, err) in [
        (
            "unsupported scheme",
            "data:text/plain,HelloWorld".to_string(),
            "unsupported URL scheme: data".to_string(),
        ),
        (
            "not JSON",
            format!("file://{}", dir.join("index.html").display()),
            "invalid JSON: expected value at line 1 column 1".to_string(),
        ),
        (
            "not an object",
            format!("file://{}", array_path.display()),
            format!(
                "invalid type: {} expected to be object but got array",
                array_url
            ),
        ),
        (
            "value not a string",
            format!("file://{}", bad_obj_path.display()),
            format!(
                "invalid type: template \"thing\" in {} expected to be string but got object",
                bad_obj_url
            ),
        ),
        (
            "bad template",
            format!("file://{}", bad_plate_path.display()),
            "invalid URI template: expression not closed (at 4-th byte)".to_string(),
        ),
    ] {
        let url = Url::parse(&url)?;
        match fetch_templates(&agent, &url) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn parse_base_url_fn() -> Result<(), BuildError> {
    for (name, url, exp, err) in [
        (
            "invalid URL",
            "not a url",
            "",
            Some(BuildError::Url(url::ParseError::RelativeUrlWithoutBase)),
        ),
        (
            "invalid URL slash",
            "not a url/",
            "",
            Some(BuildError::Url(url::ParseError::RelativeUrlWithoutBase)),
        ),
        ("file", "file://foo", "file://foo/", None),
        ("file slash", "file://foo/", "file://foo/", None),
        ("http", "http://pgxn.org", "http://pgxn.org/", None),
        ("http slash", "http://pgxn.org/", "http://pgxn.org/", None),
        ("https", "https://xyz.org", "https://xyz.org/", None),
        ("https slash", "https://xyz.org/", "https://xyz.org/", None),
    ] {
        let res = parse_base_url(url);
        match err {
            Some(e) => assert_eq!(e.to_string(), res.unwrap_err().to_string(), "{name}"),
            None => {
                let exp = Url::parse(exp)?;
                assert_eq!(exp, res.unwrap(), "{name}");
            }
        }
    }

    Ok(())
}

#[test]
fn url_for() -> Result<(), BuildError> {
    // Setup.
    let agent = ureq::agent();
    let dir = corpus_dir();
    let index = format!("file://{}", dir.join("index.json").display());
    let index = Url::parse(&index)?;
    let templates = fetch_templates(&agent, &index)?;

    for (base, prefix) in [
        ("file://foo/bar", "file://foo/bar/"),
        ("file://foo/bar/", "file://foo/bar/"),
        ("http://example.com", "http://example.com/"),
        ("https://api.pgxn.org/", "https://api.pgxn.org/"),
    ] {
        let api = Api {
            agent: ureq::agent(),
            templates: templates.clone(),
            url: parse_base_url(base)?,
        };
        for (name, template, vars, exp) in [
            // (
            //     "unknown template",
            //     "nonesuch",
            //     [("x", "y")],
            //     "",
            //     Some(BuildError::UnknownTemplate("nonesuch".to_string())),
            // ),
            (
                "dist pair",
                "dist",
                vec![("dist", "pair")],
                "dist/pair.json",
            ),
            ("dist foo", "dist", vec![("dist", "foo")], "dist/foo.json"),
            ("mirrors", "mirrors", vec![], "meta/mirrors.json"),
            ("tag hi", "tag", vec![("tag", "hi")], "tag/hi.json"),
            ("tag 😍", "tag", vec![("tag", "😍")], "tag/😍.json"),
            ("user hi", "user", vec![("user", "hi")], "user/hi.json"),
            ("user 😍", "user", vec![("user", "😍")], "user/😍.json"),
            (
                "extension hi",
                "extension",
                vec![("extension", "hi")],
                "extension/hi.json",
            ),
            (
                "extension 😍",
                "extension",
                vec![("extension", "😍")],
                "extension/😍.json",
            ),
            (
                "tag space",
                "tag",
                vec![("tag", "hi there")],
                "tag/hi there.json",
            ),
            (
                "stats users",
                "stats",
                vec![("stats", "users")],
                "stats/users.json",
            ),
            (
                "spec html",
                "spec",
                vec![("format", "html")],
                "meta/spec.html",
            ),
            (
                "spec text",
                "spec",
                vec![("format", "txt")],
                "meta/spec.txt",
            ),
            (
                "meta pair",
                "meta",
                vec![("dist", "pair"), ("version", "0.1.7")],
                "dist/pair/0.1.7/META.json",
            ),
            (
                "meta fooBar",
                "meta",
                vec![("dist", "fooBar"), ("version", "1.2.3")],
                "dist/fooBar/1.2.3/META.json",
            ),
            (
                "readme pair",
                "readme",
                vec![("dist", "pair"), ("version", "0.1.7")],
                "dist/pair/0.1.7/README.txt",
            ),
            (
                "readme fooBar",
                "readme",
                vec![("dist", "fooBar"), ("version", "1.2.3")],
                "dist/fooBar/1.2.3/README.txt",
            ),
            (
                "download pair",
                "download",
                vec![("dist", "pair"), ("version", "0.1.7")],
                "dist/pair/0.1.7/pair-0.1.7.zip",
            ),
            (
                "download Block",
                "download",
                vec![("dist", "Block"), ("version", "0.1.7")],
                "dist/Block/0.1.7/Block-0.1.7.zip",
            ),
        ] {
            let mut ctx = SimpleContext::new();
            for (k, v) in vars {
                ctx.insert(k, v);
            }
            let exp = format!("{}{}", prefix, exp);
            let exp = Url::parse(&exp)?;

            match api.url_for(template, ctx) {
                Err(e) => panic!("Unexpected error for {name}: {e}"),
                Ok(url) => assert_eq!(exp, url, "{base} {name}"),
            };
        }
    }

    Ok(())
}

#[test]
fn url_for_err() -> Result<(), BuildError> {
    use iri_string::template::simple_context::Value;

    // Set up an index.json with some issues.
    let tmp_dir = tempdir()?;
    let path = tmp_dir.path().join("index.json");
    let url = format!("file://{}/", tmp_dir.path().display());
    let url = Url::parse(&url)?;
    let tmp_file = File::create(&path)?;
    serde_json::to_writer(&tmp_file, &json!({"badVar": "foo{list:4}"}))?;
    tmp_file.sync_all()?;

    let agent = ureq::agent();
    let idx_url = url.join("index.json")?;
    let templates = fetch_templates(&agent, &idx_url)?;

    let api = Api {
        agent: ureq::agent(),
        templates: templates.clone(),
        url,
    };

    for (name, template, vars, err) in [
        (
            "unknown template",
            "nonesuch",
            vec![],
            "unknown URI template: nonesuch",
        ),
        // URI templates forbid list variables used with prefix modifiers (:4
        // in this template). We don't ever use that combination, but it's
        // best to test the error condition.
        (
            "bad variable",
            "badVar",
            vec![(
                "list",
                Value::List(vec!["one".to_string(), "two".to_string()]),
            )],
            "invalid URI template: unexpected value type for the variable (at 4-th byte)",
        ),
    ] {
        let mut ctx = SimpleContext::new();
        for (k, v) in vars {
            ctx.insert(k, v);
        }

        match api.url_for(template, ctx) {
            Err(e) => assert_eq!(err.to_string(), e.to_string(), "{name}"),
            Ok(_) => panic!("Unexpected success for {name}"),
        };
    }

    Ok(())
}

#[test]
fn dist() -> Result<(), BuildError> {
    let url = format!("file://{}/", corpus_dir().display());
    let api = Api::new(&url, None)?;
    let dist = api.dist("pair")?;
    assert_eq!("pair", dist.name());
    assert_eq!(8, dist.releases().stable().unwrap().len());

    match api.dist("nonesuch") {
        Ok(_) => panic!("dist unexpectedly succeeded"),
        Err(e) => assert!(e.to_string().contains("nonesuch.json: entity not found")),
    }

    Ok(())
}

#[test]
fn meta() -> Result<(), BuildError> {
    let url = format!("file://{}/", corpus_dir().display());
    let api = Api::new(&url, None)?;
    let v = Version::parse("0.1.7").unwrap();
    let dist = api.meta("pair", &v)?;
    assert_eq!("pair", dist.name());
    assert_eq!(&v, dist.version());
    let sha = dist.release().digests().sha1().unwrap();
    assert_eq!("5b9e3ba948b18703227e4dea17696c0f1d971759", hex::encode(sha));

    Ok(())
}

#[test]
fn meta_err() -> Result<(), BuildError> {
    // Start a lightweight mock server.
    let server = MockServer::start();
    let base_url = Url::parse(&server.base_url())?;

    // Load the URL templates.
    let idx_url = format!("file://{}/index.json", corpus_dir().display());
    let idx_url = Url::parse(&idx_url)?;
    let agent = ureq::agent();
    let templates = fetch_templates(&agent, &idx_url)?;

    // Set up an Api.
    let api = Api {
        url: base_url.clone(),
        agent,
        templates,
    };

    // Test an invalid META file json value.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dist/bad_meta/0.0.1/META.json");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });
    let v = Version::parse("0.0.1").unwrap();
    let meta = api.meta("bad_meta", &v);
    mock.assert();
    assert!(meta.is_err());
    assert_eq!(
        format!(
            "invalid type: {} expected to be object but got array",
            base_url.join("dist/bad_meta/0.0.1/META.json")?,
        ),
        meta.unwrap_err().to_string()
    );

    // Test an invalid META file json value.
    let mock = server.mock(|when, then| {
        when.method(GET).path("/dist/invalid_meta/0.0.1/META.json");
        then.status(200)
            .header("content-type", "application/json")
            .body("{}");
    });
    let meta = api.meta("invalid_meta", &v);
    mock.assert();
    assert!(meta.is_err());
    assert!(meta
        .unwrap_err()
        .to_string()
        .contains("missing properties 'name', 'version', 'abstract'"));

    Ok(())
}

#[test]
fn unpack() -> Result<(), BuildError> {
    let dir = corpus_dir();
    let url = format!("file://{}/", dir.display());
    let api = Api::new(&url, None)?;
    let tmp_dir = tempdir()?;
    let zip = dir
        .join("dist")
        .join("pair")
        .join("0.1.7")
        .join("pair-0.1.7.zip");

    // Test unpack.
    let dir = api.unpack(tmp_dir.as_ref(), &zip)?;
    let dst = tmp_dir.as_ref().join("pair-0.1.7");
    assert_eq!(&dir, &dst);

    // Check the contents.
    for file in [
        dst.join("README.md"),
        dst.join("META.json"),
        dst.join("Changes"),
        dst.join("Makefile"),
        dst.join("pair.control"),
        dst.join("META.json"),
        dst.join("doc").join("pair.md"),
        dst.join("sql").join("pair.sql"),
        dst.join("sql").join("pair--unpackaged--0.1.2.sql"),
        dst.join("test").join("sql").join("base.sql"),
        dst.join("test").join("expected").join("base.out"),
    ] {
        assert!(file.exists(), "{}", file.display());
    }

    // Test an invalid zip file.
    let idx = corpus_dir().join("index.json");
    let res = api.unpack(tmp_dir.as_ref(), &idx);
    assert!(res.is_err());
    assert_eq!(
        "invalid Zip archive: Could not find EOCD",
        res.unwrap_err().to_string()
    );

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

fn read_eq(left: &mut File, right: &mut File) -> Result<(), io::Error> {
    let mut left_buf = Vec::new();
    left.read_to_end(&mut left_buf)?;
    let mut right_buf = Vec::new();
    right.read_to_end(&mut right_buf)?;
    let left = Sha256::digest(left_buf);
    let right = Sha256::digest(right_buf);
    assert_eq!(left, right);
    Ok(())
}

fn index_json() -> Value {
    json!({
      "download": "/dist/{dist}/{version}/{dist}-{version}.zip",
      "readme": "/dist/{dist}/{version}/README.txt",
      "meta": "/dist/{dist}/{version}/META.json",
      "dist": "/dist/{dist}.json",
      "extension": "/extension/{extension}.json",
      "user": "/user/{user}.json",
      "tag": "/tag/{tag}.json",
      "stats": "/stats/{stats}.json",
      "mirrors": "/meta/mirrors.json",
      "spec": "/meta/spec.{format}"
    })
}
