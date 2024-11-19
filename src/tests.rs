use super::*;
use serde_json::{json, Value};

fn release_meta(pipeline: &str) -> Value {
    json!({
      "name": "pair",
      "abstract": "A key/value pair data type",
      "version": "0.1.8",
      "maintainers": [
        { "name": "David E. Wheeler", "email": "david@justatheory.com" }
      ],
      "license": "PostgreSQL",
      "contents": {
        "extensions": {
          "pair": {
            "sql": "sql/pair.sql",
            "control": "pair.control"
          }
        }
      },
      "dependencies": { "pipeline": pipeline },
      "meta-spec": { "version": "2.0.0" },
      "certs": {
        "pgxn": {
          "payload": "eyJ1c2VyIjoidGhlb3J5IiwiZGF0ZSI6IjIwMjQtMDktMTNUMTc6MzI6NTVaIiwidXJpIjoiZGlzdC9wYWlyLzAuMS43L3BhaXItMC4xLjcuemlwIiwiZGlnZXN0cyI6eyJzaGE1MTIiOiJiMzUzYjVhODJiM2I1NGU5NWY0YTI4NTllN2EyYmQwNjQ4YWJjYjM1YTdjMzYxMmIxMjZjMmM3NTQzOGZjMmY4ZThlZTFmMTllNjFmMzBmYTU0ZDdiYjY0YmNmMjE3ZWQxMjY0NzIyYjQ5N2JjYjYxM2Y4MmQ3ODc1MTUxNWI2NyJ9fQ",
          "signature": "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q",
        },
      },
    })
}

#[test]
fn pgxs() {
    // Test pgxs pipeline.
    let meta = release_meta("pgxs");
    let dir = Path::new("dir");
    let rel = Release::try_from(meta.clone()).unwrap();
    let builder = Builder::new(dir, rel).unwrap();
    let rel = Release::try_from(meta).unwrap();
    let exp = Builder {
        pipeline: Build::Pgxs(Pgxs::new(dir.to_path_buf(), true)),
        meta: rel,
    };
    assert_eq!(exp, builder, "pgxs");
    assert!(builder.configure().is_ok());
    assert!(builder.compile().is_ok());
    assert!(builder.test().is_ok());
}

#[test]
fn pgrx() {
    // Test pgrx pipeline.
    let meta = release_meta("pgrx");
    let dir = Path::new("dir");
    let rel = Release::try_from(meta.clone()).unwrap();
    let builder = Builder::new(dir, rel).unwrap();
    let rel = Release::try_from(meta).unwrap();
    let exp = Builder {
        pipeline: Build::Pgrx(Pgrx::new(dir.to_path_buf(), true)),
        meta: rel,
    };
    assert_eq!(exp, builder, "pgrx");
    assert!(builder.configure().is_ok());
    assert!(builder.compile().is_ok());
    assert!(builder.test().is_ok());
}

#[test]
fn unsupported_pipeline() {
    // Test unsupported pipeline.
    let meta = release_meta("meson");
    let rel = Release::try_from(meta).unwrap();
    assert_eq!(
        BuildError::UnknownPipeline("meson".to_string()).to_string(),
        Builder::new("dir", rel).unwrap_err().to_string(),
    );
}

#[test]
#[should_panic(expected = "Detect pipeline")]
fn detect_pipeline() {
    // Test unspecified pipeline.
    let mut meta = release_meta("");
    meta.as_object_mut().unwrap().remove("dependencies");
    let rel = Release::try_from(meta).unwrap();
    _ = Builder::new("dir", rel);
}

#[test]
#[should_panic(expected = "Detect pipeline")]
fn no_pipeline() {
    // Test unspecified pipeline.
    let mut meta = release_meta("");
    let deps = meta
        .as_object_mut()
        .unwrap()
        .get_mut("dependencies")
        .unwrap()
        .as_object_mut()
        .unwrap();

    deps.remove("pipeline");
    deps.insert("postgres".to_string(), json!({"version": "14"}));
    let rel = Release::try_from(meta).unwrap();
    _ = Builder::new("dir", rel);
}
