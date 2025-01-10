use super::*;
use crate::tests::compile_mock;
use assertables::*;
use std::{collections::HashMap, env};
use tempfile::tempdir;

struct TestPipeline<P: AsRef<Path>> {
    dir: P,
    cfg: PgConfig,
}

// Create a mock version of the trait.
#[cfg(test)]
impl<P: AsRef<Path>> Pipeline<P> for TestPipeline<P> {
    fn new(dir: P, cfg: PgConfig) -> Self {
        TestPipeline { dir, cfg }
    }

    fn dir(&self) -> &P {
        &self.dir
    }

    fn pg_config(&self) -> &PgConfig {
        &self.cfg
    }

    fn confidence(_: P) -> u8 {
        0
    }
    fn configure(&self) -> Result<(), BuildError> {
        Ok(())
    }
    fn compile(&self) -> Result<(), BuildError> {
        Ok(())
    }
    fn install(&self) -> Result<(), BuildError> {
        Ok(())
    }
    fn test(&self) -> Result<(), BuildError> {
        Ok(())
    }
}

#[test]
fn run() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let cfg = PgConfig::from_map(HashMap::new());

    // Test basic success.
    let pipe = TestPipeline::new(&tmp, cfg);
    if let Err(e) = pipe.run("echo", ["hello"], false) {
        panic!("echo hello failed: {e}");
    }

    // Test nonexistent file.
    match pipe.run("__nonesuch_nope__", [""], false) {
        Ok(_) => panic!("Nonexistent file unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing ");
            assert_ends_with!(
                e.to_string(),
                "\"__nonesuch_nope__\" \"\"`: entity not found"
            )
        }
    }

    // Test an executable that returns an error.
    let path = tmp.path().join("exit_err").display().to_string();
    compile_mock("exit_err", &path);
    match pipe.run(&path, ["hi"], false) {
        Ok(_) => panic!("exit_err unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing");
            assert_ends_with!(e.to_string(), " DED: hi\n");
        }
    }

    // Build a mock `sudo` that echos output.
    let dest = tmp
        .path()
        .join(if cfg!(windows) { "sudo.exe" } else { "sudo" })
        .display()
        .to_string();
    compile_mock("echo", &dest);

    // Create a PATH variable that searches tmp first.
    let path = env::var("PATH").unwrap();
    let path = [tmp.path().to_path_buf()]
        .into_iter()
        .chain(env::split_paths(&path));

    // Run sudo echo with the path set.
    temp_env::with_var("PATH", Some(env::join_paths(path).unwrap()), || {
        if let Err(e) = pipe.run("echo", ["hello"], true) {
            panic!("echo hello failed: {e}");
        }
    });

    Ok(())
}

#[test]
fn is_writeable() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let cfg = PgConfig::from_map(HashMap::new());

    let pipe = TestPipeline::new(&tmp, cfg);
    assert!(pipe.is_writeable(&tmp));
    assert!(!pipe.is_writeable(tmp.path().join(" nonesuch")));

    Ok(())
}

#[test]
fn maybe_sudo() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let cfg = PgConfig::from_map(HashMap::from([(
        "pkglibdir".to_string(),
        tmp.as_ref().display().to_string(),
    )]));
    let pipe = TestPipeline::new(&tmp, cfg);

    // Never use sudo when param is false.
    let cmd = pipe.maybe_sudo("foo", false);
    assert_eq!("foo", cmd.get_program().to_str().unwrap());

    // Never use sudo when directory is writeable.
    let cmd = pipe.maybe_sudo("foo", true);
    assert_eq!("foo", cmd.get_program().to_str().unwrap());

    // Use sudo when the directory is not writeable.
    let cfg = PgConfig::from_map(HashMap::from([(
        "pkglibdir".to_string(),
        tmp.path().join("nonesuch").display().to_string(),
    )]));
    let pipe = TestPipeline::new(&tmp, cfg);
    let cmd = pipe.maybe_sudo("foo", true);
    assert_eq!("sudo", cmd.get_program().to_str().unwrap());
    let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
    assert_eq!(args, &["foo"]);

    // Never use sudo when param is false.
    let cmd = pipe.maybe_sudo("foo", false);
    assert_eq!("foo", cmd.get_program().to_str().unwrap());

    Ok(())
}
