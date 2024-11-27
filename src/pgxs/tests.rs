use super::*;
use assertables::*;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
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
    let pipe = Pgxs::new(dir, false);
    assert_eq!(dir, pipe.dir);
    assert_eq!(&dir, pipe.dir());
    assert!(!pipe.sudo);

    let dir2 = dir.join("corpus");
    let pipe = Pgxs::new(dir2.as_path(), true);
    assert_eq!(dir2, pipe.dir);
    assert_eq!(&dir2, pipe.dir());
    assert!(pipe.sudo);
}

#[test]
fn configure() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let pipe = Pgxs::new(&tmp, false);

    // Try with no Configure file.
    if let Err(e) = pipe.configure() {
        panic!("configure with no file: {e}");
    }

    // Now try with a configure file.
    let path = tmp.path().join("configure");
    {
        let cfg = File::create(&path)?;
        #[cfg(target_family = "windows")]
        writeln!(&cfg, "@echo off\r\necho configuring something...\r\n")?;
        #[cfg(not(target_family = "windows"))]
        writeln!(&cfg, "#! /bin/sh\n\necho configuring something...\n")?;
    }
    match pipe.configure() {
        Ok(_) => panic!("configure unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing ");
            assert_ends_with!(
                e.to_string(),
                if cfg!(windows) {
                    "`\".\\\\configure\"`: entity not found"
                } else {
                    "\"./configure\"`: permission denied"
                },
            )
        }
    }

    // Make it executable.
    #[cfg(target_family = "windows")]
    // Turn it into a batch file.
    std::fs::rename(path, tmp.path().join("configure.bat"))?;
    #[cfg(not(target_family = "windows"))]
    {
        // Make it executable.
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
    }
    if let Err(e) = pipe.configure() {
        panic!("Configure failed: {e}");
    }

    Ok(())
}

#[test]
fn compile() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgxs::new(dir, false);
    assert!(pipe.compile().is_err());
    Ok(())
}

#[test]
fn test() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgxs::new(dir, false);
    assert!(pipe.test().is_err());
    Ok(())
}

#[test]
fn install() -> Result<(), BuildError> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pipe = Pgxs::new(dir, false);
    assert!(pipe.install().is_err());
    Ok(())
}
