use super::*;
use std::{
    fs::{self, File},
    io::Write,
};
use tempfile::tempdir;

#[test]
fn confidence() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    // No Makefile.
    assert_eq!(0, Pgxs::confidence(tmp.as_ref()));

    // Create each variant of a makefile.
    for name in ["GNUmakefile", "makefile", "Makefile"] {
        let makefile = tmp.as_ref().join(name);
        let _file = File::create(&makefile)?;
        assert_eq!(127, Pgxs::confidence(tmp.as_ref()), "{name} exists");

        // With variables.
        for var in [
            "MODULES",
            "MODULE_big",
            "PROGRAM",
            "EXTENSION",
            "DATA",
            "DATA_built",
        ] {
            for (i, op) in ["=", "=", "?="].into_iter().enumerate() {
                let mut file = File::create(&makefile)?;
                writeln!(&file, "{var:<width$}{op:<width$}whatever", width = i + 1)?;
                file.flush()?;
                assert_eq!(200, Pgxs::confidence(tmp.as_ref()), "{name} {var}");

                // Append PG_CONFIG, should get full confidence.
                let var = "PG_CONFIG";
                writeln!(&file, "{var:<width$}{op:<width$}whatever", width = i + 1)?;
                file.flush()?;
                assert_eq!(
                    255,
                    Pgxs::confidence(tmp.as_ref()),
                    "{name} {var} PG_CONFIG"
                );
            }
        }
        fs::remove_file(&makefile)?;
    }

    Ok(())
}

#[test]
fn new() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgxs::new(dir.to_path_buf(), false);
    assert_eq!(dir, pipe.dir);
    assert!(!pipe.sudo);

    let dir2 = dir.join("corpus");
    let pipe = Pgxs::new(dir2.to_path_buf(), true);
    assert_eq!(dir2, pipe.dir);
    assert!(pipe.sudo);
}

#[test]
fn configure_et_al() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgxs::new(dir.to_path_buf(), false);
    assert!(pipe.configure().is_ok());
    assert!(pipe.compile().is_ok());
    assert!(pipe.test().is_ok());
    assert!(pipe.install().is_ok());
}
