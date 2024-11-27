use super::*;
use serde_json::{json, Value};
use std::{fs::File, io::Write, process::Command};
use tempfile::tempdir;

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
      "dependencies": {
        "pipeline": pipeline,
        "postgres": { "version": "14.0" }
      },
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
    let tmp = tempdir().unwrap();
    let rel = Release::try_from(meta.clone()).unwrap();
    let builder = Builder::new(tmp.as_ref(), rel, false).unwrap();
    let rel = Release::try_from(meta).unwrap();
    let exp = Builder {
        pipeline: Build::Pgxs(Pgxs::new(tmp.as_ref(), false)),
        meta: rel,
    };
    assert_eq!(exp, builder, "pgxs");
    assert!(builder.configure().is_ok());
    assert!(builder.compile().is_err());
    assert!(builder.test().is_err());
    assert!(builder.test().is_err());
    assert!(builder.install().is_err());
}

#[test]
fn pgrx() {
    // Test pgrx pipeline.
    let meta = release_meta("pgrx");
    let tmp = tempdir().unwrap();
    let rel = Release::try_from(meta.clone()).unwrap();
    let builder = Builder::new(tmp.as_ref(), rel, false).unwrap();
    let rel = Release::try_from(meta).unwrap();
    let exp = Builder {
        pipeline: Build::Pgrx(Pgrx::new(tmp.as_ref(), false)),
        meta: rel,
    };
    assert_eq!(exp, builder, "pgrx");
    assert!(builder.configure().is_ok());
    assert!(builder.compile().is_ok());
    assert!(builder.test().is_ok());
    assert!(builder.install().is_ok());
}

#[test]
fn unsupported_pipeline() {
    // Test unsupported pipeline.
    let meta = release_meta("meson");
    let rel = Release::try_from(meta).unwrap();
    assert_eq!(
        BuildError::UnknownPipeline("meson".to_string()).to_string(),
        Builder::new("dir", rel, true).unwrap_err().to_string(),
    );
}

#[test]
fn detect_pipeline() -> Result<(), BuildError> {
    let mut metas = [release_meta(""), release_meta("")];
    // Remove pipeline specification from the first item.
    metas[0]
        .as_object_mut()
        .unwrap()
        .get_mut("dependencies")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("pipeline");

    // Remove dependencies from the second item.
    metas[1].as_object_mut().unwrap().remove("dependencies");

    fn no_pipe(m: &Value) -> Release {
        Release::try_from(m.clone()).unwrap()
    }

    // With empty directory should find no pipeline.
    let tmp = tempdir()?;
    let dir = tmp.as_ref();
    match Build::detect(dir, true) {
        Ok(_) => panic!("detect unexpectedly succeeded with empty dir"),
        Err(e) => assert_eq!(
            "cannot detect build pipeline and none specified",
            e.to_string()
        ),
    }
    for meta in &metas {
        match Builder::new(dir, no_pipe(meta), true) {
            Ok(_) => panic!("detect unexpectedly succeeded with empty dir"),
            Err(e) => assert_eq!(
                "cannot detect build pipeline and none specified",
                e.to_string()
            ),
        }
    }

    // Add an empty Makefile, PGXS should win.
    let mut makefile = File::create(dir.join("Makefile"))?;
    match Build::detect(dir, true) {
        Ok(p) => assert_eq!(Build::Pgxs(Pgxs::new(dir, true)), p),
        Err(e) => panic!("Unexpectedly errored with Makefile: {e}"),
    }
    for meta in &metas {
        match Builder::new(dir, no_pipe(meta), true) {
            Ok(b) => assert_eq!(Build::Pgxs(Pgxs::new(dir, true)), b.pipeline),
            Err(e) => panic!("Unexpectedly errored with Makefile: {e}"),
        }
    }
    // Add an empty cargo.toml, PGXS should still win.
    let mut cargo_toml = File::create(dir.join("Cargo.toml"))?;
    match Build::detect(dir, false) {
        Ok(p) => assert_eq!(Build::Pgxs(Pgxs::new(dir, false)), p),
        Err(e) => panic!("Unexpectedly errored with Cargo.toml: {e}"),
    }
    for meta in &metas {
        match Builder::new(dir, no_pipe(meta), true) {
            Ok(b) => assert_eq!(Build::Pgxs(Pgxs::new(dir, true)), b.pipeline),
            Err(e) => panic!("Unexpectedly errored with Cargo.toml: {e}"),
        }
    }

    // Add pgrx to Cargo.toml; now pgrx should win.
    writeln!(&cargo_toml, "[dependencies]\npgrx = \"0.12.6\"")?;
    cargo_toml.flush()?;
    match Build::detect(dir, true) {
        Ok(p) => assert_eq!(Build::Pgrx(Pgrx::new(dir, true)), p),
        Err(e) => panic!("Unexpectedly errored with pgrx dependency: {e}"),
    }
    for meta in &metas {
        match Builder::new(dir, no_pipe(meta), false) {
            Ok(b) => assert_eq!(Build::Pgrx(Pgrx::new(dir, false)), b.pipeline),
            Err(e) => panic!("Unexpectedly errored with pgrx dependency: {e}"),
        }
    }

    // Add PG_CONFIG to the Makefile, PGXS should win again.
    writeln!(&makefile, "PG_CONFIG ?= pg_config")?;
    makefile.flush()?;
    match Build::detect(dir, false) {
        Ok(p) => assert_eq!(Build::Pgxs(Pgxs::new(dir, false)), p),
        Err(e) => panic!("Unexpectedly errored with PG_CONFIG var: {e}"),
    }
    for meta in &metas {
        match Builder::new(dir, no_pipe(meta), false) {
            Ok(b) => assert_eq!(Build::Pgxs(Pgxs::new(dir, false)), b.pipeline),
            Err(e) => panic!("Unexpectedly errored with PG_CONFIG var: {e}"),
        }
    }

    Ok(())
}

/// Utility function for compiling `mocks/{name}.rs` into `dest`. Used to
/// provide consistent execution and output for testing across OSes.
pub fn compile_mock(name: &str, dest: &str) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mocks")
        .join(format!("{name}.rs"))
        .display()
        .to_string();
    let out = Command::new("rustc")
        .args([&src, "-o", dest])
        .output()
        .unwrap();
    if !out.status.success() {
        panic!(
            "Failed to build {name}.rs: {}",
            String::from_utf8_lossy(&out.stderr),
        )
    }
}
