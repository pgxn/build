use super::*;
use crate::error::BuildError;
use crate::line;
use crate::tests::compile_mock;
use assertables::*;
// use std::str;
use tempfile::tempdir;

#[test]
fn output() {
    let output = Output::new("hello".to_string(), false);
    assert_eq!("hello", output.line);
    assert!(!output.is_err);
    let output = Output::new("🥏 Flying Disk!".to_string(), true);
    assert_eq!("🥏 Flying Disk!", output.line);
    assert!(output.is_err);
}

#[test]
fn execute() -> Result<(), BuildError> {
    let tmp = tempdir()?;
    // Build an app that echos output.
    let dest = tmp
        .path()
        .join(if cfg!(windows) { "sudo.exe" } else { "sudo" })
        .display()
        .to_string();
    compile_mock("emit", &dest);

    // Set up buffers for output.
    let out = Vec::new();
    let err = Vec::new();
    {
        let stdout = line::LineWriter::new(out);
        let stderr = line::LineWriter::new(err);
        let mut exec = Executor::new(tmp.as_ref(), stdout, stderr);

        // Run the app.
        let mut cmd = Command::new(&dest);
        cmd.arg("this is standard output")
            .arg("this is error output");
        if let Err(e) = exec.execute(cmd) {
            panic!("emit execution failed: {e}");
        }

        // Run it again.
        let mut cmd = Command::new(&dest);
        cmd.arg("more standard output").arg("more error output");
        if let Err(e) = exec.execute(cmd) {
            panic!("emit execution failed: {e}");
        }
    }

    // Check the output.
    // let res = str::from_utf8(out.as_slice()).unwrap();
    // assert_eq!("this is standard output\nmore standard output\n", res);
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("this is error output\nmore error output\n", res);

    // Test nonexistent file.
    let out = Vec::new();
    let err = Vec::new();
    let stdout = line::LineWriter::new(out);
    let stderr = line::LineWriter::new(err);
    let mut exec = Executor::new(tmp.as_ref(), stdout, stderr);
    match exec.execute(Command::new("__nonesuch_nope__")) {
        Ok(_) => panic!("Nonexistent file unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing ");
            assert_ends_with!(e.to_string(), "\"__nonesuch_nope__\"`: entity not found")
        }
    }
    // assert_eq!(0, out.len());
    // assert_eq!(0, err.len());

    // Test an executable that returns an error.
    let out = Vec::new();
    let err = Vec::new();
    let stdout = line::LineWriter::new(out);
    let stderr = line::LineWriter::new(err);
    let mut exec = Executor::new(tmp.as_ref(), stdout, stderr);
    let path = tmp.path().join("exit_err").display().to_string();
    compile_mock("exit_err", &path);
    match exec.execute(Command::new(&path)) {
        Ok(_) => panic!("exit_err unexpectedly succeeded"),
        Err(e) => {
            assert_starts_with!(e.to_string(), "executing");
            assert_ends_with!(e.to_string(), " exited with status code: 2");
        }
    }
    // assert_eq!(0, out.len());
    // let res = str::from_utf8(err.as_slice()).unwrap();
    // assert_eq!("DED: \n", res);

    Ok(())
}
