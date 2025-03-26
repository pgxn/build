use super::*;
use crate::line::LineWriter;
use crate::tests::compile_mock;
use assertables::*;
use std::{collections::HashMap, env};
use tempfile::tempdir;

struct TestPipeline {
    exec: Executor,
    cfg: PgConfig,
}

// Create a mock version of the trait.
impl Pipeline for TestPipeline {
    fn new(exec: Executor, cfg: PgConfig) -> Self {
        TestPipeline { exec, cfg }
    }

    fn executor(&mut self) -> &mut Executor {
        &mut self.exec
    }

    fn pg_config(&self) -> &PgConfig {
        &self.cfg
    }

    fn confidence(_: impl AsRef<Path>) -> u8 {
        0
    }

    fn configure(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    fn compile(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    fn install(&mut self) -> Result<(), BuildError> {
        Ok(())
    }

    fn test(&mut self) -> Result<(), BuildError> {
        Ok(())
    }
}

#[test]
fn trait_functions() {
    assert_eq!(0, TestPipeline::confidence("some dir"));

    let exec = Executor::new(
        env!("CARGO_MANIFEST_DIR"),
        LineWriter::new(vec![]),
        LineWriter::new(vec![]),
    );
    let mut pipe = TestPipeline::new(exec, PgConfig::from_map(HashMap::new()));
    assert!(pipe.configure().is_ok());
    assert!(pipe.compile().is_ok());
    assert!(pipe.install().is_ok());
    assert!(pipe.test().is_ok());
}

#[test]
fn run() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let out = Vec::new();
    let err = Vec::new();
    {
        // Test basic success.
        let exec = Executor::new(tmp.as_ref(), LineWriter::new(out), LineWriter::new(err));
        let mut pipe = TestPipeline::new(exec, PgConfig::from_map(HashMap::new()));
        if let Err(e) = pipe.run("echo", ["hello"], false) {
            panic!("echo hello failed: {e}");
        }
    }

    // Check the output.
    // let res = str::from_utf8(out.as_slice()).unwrap();
    // assert_eq!("hello\n", res);
    // out.clear();
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("", res);

    // Test nonexistent file.
    {
        let exec = Executor::new(
            tmp.as_ref(),
            LineWriter::new(vec![]),
            LineWriter::new(vec![]),
        );
        let mut pipe = TestPipeline::new(exec, PgConfig::from_map(HashMap::new()));
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
    }

    // Check the output.
    // let res = str::from_utf8(out.as_slice()).unwrap();
    // assert_eq!("", res);
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("\"__nonesuch_nope__\" \"\"`: entity not found", res);
    // err.clear();

    // Test an executable that returns an error.
    {
        let exec = Executor::new(
            tmp.as_ref(),
            LineWriter::new(vec![]),
            LineWriter::new(vec![]),
        );
        let mut pipe = TestPipeline::new(exec, PgConfig::from_map(HashMap::new()));
        let path = tmp.path().join("exit_err").display().to_string();
        compile_mock("exit_err", &path);
        match pipe.run(&path, ["hi"], false) {
            Ok(_) => panic!("exit_err unexpectedly succeeded"),
            Err(e) => {
                assert_starts_with!(e.to_string(), "executing");
                assert_ends_with!(e.to_string(), " exited with status code: 2");
            }
        }
    }

    // Check the output.
    // let res = str::from_utf8(out.as_slice()).unwrap();
    // assert_eq!("", res);
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("exited with status code: 2", res);
    // err.clear();

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
        let exec = Executor::new(
            tmp.as_ref(),
            LineWriter::new(vec![]),
            LineWriter::new(vec![]),
        );
        let mut pipe = TestPipeline::new(exec, PgConfig::from_map(HashMap::new()));
        if let Err(e) = pipe.run("echo", ["hello"], true) {
            panic!("echo hello failed: {e}");
        }
    });

    // Check the output.
    // let res = str::from_utf8(out.as_slice()).unwrap();
    // assert_eq!("hello\n", res);
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("", res);
    // out.clear();

    Ok(())
}

#[test]
fn is_writeable() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    let cfg = PgConfig::from_map(HashMap::new());
    let exec = Executor::new(
        tmp.as_ref(),
        LineWriter::new(vec![]),
        LineWriter::new(vec![]),
    );
    let pipe = TestPipeline::new(exec, cfg);
    assert!(pipe.is_writeable(tmp.as_ref()));
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
    let exec = Executor::new(
        tmp.as_ref(),
        LineWriter::new(vec![]),
        LineWriter::new(vec![]),
    );
    let pipe = TestPipeline::new(exec, cfg);

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

    let exec = Executor::new(
        tmp.as_ref(),
        LineWriter::new(vec![]),
        LineWriter::new(vec![]),
    );
    let pipe = TestPipeline::new(exec, cfg);
    let cmd = pipe.maybe_sudo("foo", true);
    assert_eq!("sudo", cmd.get_program().to_str().unwrap());
    let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
    assert_eq!(args, &["foo"]);

    // Never use sudo when param is false.
    let cmd = pipe.maybe_sudo("foo", false);
    assert_eq!("foo", cmd.get_program().to_str().unwrap());

    Ok(())
}
