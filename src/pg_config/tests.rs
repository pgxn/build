use super::*;
use assertables::*;
use tempfile::tempdir;

#[test]
fn pg_config() -> Result<(), BuildError> {
    // Build a mock pg_config.
    let tmp = tempdir()?;
    let path = tmp.path().join("pg_config").display().to_string();
    compile_mock("pg_config", &path);

    let exp = HashMap::from([
        ("bindir".to_string(), "/opt/data/pgsql-17.2/bin".to_string()),
        (
            "mandir".to_string(),
            "/opt/data/pgsql-17.2/share/man".to_string(),
        ),
        (
            "pgxs".to_string(),
            "/opt/data/pgsql-17.2/lib/pgxs/src/makefiles/pgxs.mk".to_string(),
        ),
        ("cflags_sl".to_string(), "".to_string()),
        (
            "libs".to_string(),
            "-lpgcommon -lpgport -lxml2 -lssl -lcrypto -lz -lreadline -lm ".to_string(),
        ),
        ("version".to_string(), "PostgreSQL 17.2".to_string()),
    ]);

    // Parse its output.
    let mut cfg = PgConfig::new(&path)?;
    assert_eq!(&exp, &cfg.0);

    // Get lowercase.
    assert_eq!(
        cfg.get("bindir"),
        Some("/opt/data/pgsql-17.2/bin"),
        "bindir"
    );
    assert_eq!(
        cfg.get("mandir"),
        Some("/opt/data/pgsql-17.2/share/man"),
        "mandir"
    );
    assert_eq!(
        cfg.get("pgxs"),
        Some("/opt/data/pgsql-17.2/lib/pgxs/src/makefiles/pgxs.mk"),
        "pgxs",
    );
    assert_eq!(cfg.get("cflags_sl"), Some(""));
    assert_eq!(
        cfg.get("libs"),
        Some("-lpgcommon -lpgport -lxml2 -lssl -lcrypto -lz -lreadline -lm "),
        "libs",
    );
    assert_eq!(cfg.get("version"), Some("PostgreSQL 17.2"), "version");

    // Uppercase and unknown keys ignored.
    for name in [
        "BINDIR",
        "MANDIR",
        "PGXS",
        "CFLAGS_SL",
        "LIBS",
        "VERSION",
        "nonesuch",
    ] {
        assert_eq!(cfg.get(name), None, "{name}");
    }

    // Test iter.
    let mut all = HashMap::new();
    for (k, v) in cfg.iter() {
        all.insert(k.to_string(), v.to_string());
    }
    assert_eq!(&exp, &all);

    // Test into_iter.
    let mut all = HashMap::new();
    for (k, v) in &cfg {
        all.insert(k.to_string(), v.to_string());
    }
    assert_eq!(&exp, &all);

    Ok(())
}

#[test]
fn pg_config_err() {
    // Build a mock pg_config that exits with an error.
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("exit_err").display().to_string();
    compile_mock("exit_err", &path);

    // Get the error.
    match PgConfig::new(&path) {
        Ok(_) => panic!("exit_err unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing");
            assert_ends_with!(e.to_string(), " DED: \n");
        }
    }

    // Try executing a nonexistent file.
    let path = tmp.path().join("nonesuch").display().to_string();
    match PgConfig::new(&path) {
        Ok(_) => panic!("nonesuch unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing");
            assert_ends_with!(e.to_string(), "nonesuch\"`: entity not found");
        }
    }
}

fn compile_mock(name: &str, dest: &str) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mocks")
        .join(format!("{name}.rs"))
        .display()
        .to_string();
    Command::new("rustc")
        .args([&src, "-o", dest])
        .output()
        .unwrap();
}
