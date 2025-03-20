use super::*;
use crate::line::LineWriter;
use std::{collections::HashMap, fs::File, io::Write};
use tempfile::tempdir;

#[test]
fn confidence() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    // let mut out = Vec::new();
    // let mut err = Vec::new();
    // // Test basic success.
    // let exec = Executor::new(&tmp, LineWriter::new(&mut out), LineWriter::new(&mut err));

    // No Cargo.toml.
    assert_eq!(0, Pgrx::confidence(tmp.as_ref()));

    // Create a Cargo.toml.
    let mut file = File::create(tmp.as_ref().join("Cargo.toml"))?;
    assert_eq!(1, Pgrx::confidence(tmp.as_ref()));

    // Add a pgrx dependency.
    writeln!(&file, "[dependencies]\npgrx = \"0.12.6\"")?;
    file.flush().unwrap();
    assert_eq!(255, Pgrx::confidence(tmp.as_ref()));

    // Add another dependency (to be ignored).
    writeln!(&file, "serde_json = \"1.0\"")?;
    assert_eq!(255, Pgrx::confidence(tmp.as_ref()));

    Ok(())
}

#[test]
fn new() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cfg = PgConfig::from_map(HashMap::new());

    let exec = Executor::new(dir, LineWriter::new(vec![]), LineWriter::new(vec![]))
    let pipe = Pgrx::new(dir, cfg.clone());
    assert_eq!(dir, pipe.dir);
    assert_eq!(&dir, pipe.dir());
    assert_eq!(&cfg, pipe.pg_config());

    let dir2 = dir.join("corpus");
    let cfg2 = PgConfig::from_map(HashMap::from([("bindir".to_string(), "bin".to_string())]));
    let pipe = Pgrx::new(dir2.as_path(), cfg2.clone());
    assert_eq!(dir2, pipe.dir);
    assert_eq!(&dir2, pipe.dir());
    assert_eq!(&cfg2, pipe.pg_config());
}

#[test]
fn configure_et_al() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgrx::new(dir, PgConfig::from_map(HashMap::new()));
    assert!(pipe.configure().is_ok());
    assert!(pipe.compile().is_ok());
    assert!(pipe.test().is_ok());
    assert!(pipe.install().is_ok());
}
