use std::fmt;
use std::io;

use crate::line::WriteLine;

pub struct Writer {
    stdout: Box<dyn WriteLine>,
    stderr: Box<dyn WriteLine>,
}

impl Writer {
    pub fn new(stdout: impl WriteLine + 'static, stderr: impl WriteLine + 'static) -> Self {
        Self {
            stdout: Box::new(stdout),
            stderr: Box::new(stderr),
        }
    }

    /// Write a line to standard output
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.stdout.write_all(line.as_bytes())?;
        self.stdout.write_all(b"\n")?;
        Ok(())
    }

    /// Write a line to standard error
    pub fn write_error(&mut self, line: &str) -> io::Result<()> {
        self.stderr.write_all(line.as_bytes())?;
        self.stderr.write_all(b"\n")?;
        Ok(())
    }
}

impl PartialEq for Writer {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer").finish()
    }
}
