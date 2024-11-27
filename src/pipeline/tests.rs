use super::*;
use crate::tests::compile_mock;
use assertables::*;
use std::env;
use tempfile::tempdir;

struct TestPipeline<P: AsRef<Path>> {
    dir: P,
}

// Create a mock version of the trait.
#[cfg(test)]
impl<P: AsRef<Path>> Pipeline<P> for TestPipeline<P> {
    fn new(dir: P, _: bool) -> Self {
        TestPipeline { dir }
    }
    fn dir(&self) -> &P {
        &self.dir
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

    // Test basic success.
    let pipe = TestPipeline::new(&tmp, false);
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
