use super::*;
use std::{fs::File, io::Write};
use tempfile::tempdir;

#[test]
fn confidence() -> Result<(), BuildError> {
    let tmp = tempdir()?;
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
    let pipe = Pgrx::new(dir, false);
    assert_eq!(dir, pipe.dir);
    assert_eq!(&dir, pipe.dir());
    assert!(!pipe.sudo);

    let dir2 = dir.join("corpus");
    let pipe = Pgrx::new(dir2.as_path(), true);
    assert_eq!(dir2, pipe.dir);
    assert_eq!(&dir2, pipe.dir());
    assert!(pipe.sudo);
}

#[test]
fn configure_et_al() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgrx::new(dir, false);
    assert!(pipe.configure().is_ok());
    assert!(pipe.compile().is_ok());
    assert!(pipe.test().is_ok());
    assert!(pipe.install().is_ok());
}
