use super::*;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{env, fs::File, io::Write};
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

    let pipe = TestPipeline::new(&tmp, false);
    if let Err(e) = pipe.run("echo", ["hello"], false) {
        panic!("echo hello failed: {e}");
    }

    // Mock up sudo command as a simple shell script.
    #[cfg(target_family = "windows")]
    {
        // This should work but does not, even though it's put into the path
        // below, because Command only supports `.exe` files:
        //
        // > Note on Windows: For executable files with the .exe extension, it
        // > can be omitted when specifying the program for this Command.
        // > However, if the file has a different extension, a filename
        // > including the extension needs to be provided, otherwise the file
        // > won’t be found.
        //
        // https://doc.rust-lang.org/std/process/struct.Command.html#platform-specific-behavior
        //
        // For now run() ignores the `sudo` param, since it's not clear it's
        // the right command anyway.
        let sudo = tmp.path().join("sudo.bat");
        let file = File::create(&sudo)?;
        writeln!(&file, "@echo off\r\necho %*\r\n")?;
    }
    #[cfg(not(target_family = "windows"))]
    {
        let sudo = tmp.path().join("sudo");
        let file = File::create(&sudo)?;
        writeln!(&file, "#! /bin/sh\n\necho \"$@\"\n")?;
        let mut perms = std::fs::metadata(&sudo)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&sudo, perms)?;
    }

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
