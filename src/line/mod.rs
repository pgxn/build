use owo_colors::Style;
use std::io::{Result, Write};

pub trait WriteLine: Write {
    fn write_line(&mut self, line: &str) -> Result<()>;
}

pub struct LineWriter<T: Write>(T);

impl<T: Write> LineWriter<T> {
    pub fn new(writer: T) -> Self {
        Self(writer)
    }
}

impl<T: Write> Write for LineWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

impl<T: Write> WriteLine for LineWriter<T> {
    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.0, "{}", line)
    }
}

impl<W: WriteLine + ?Sized> WriteLine for Box<W> {
    fn write_line(&mut self, line: &str) -> Result<()> {
        (**self).write_line(line)
    }
}

pub struct ColorLine<T: Write> {
    inner: T,
    style: Style,
}

impl<T: Write> ColorLine<T> {
    pub fn new(writer: T, style: Style) -> Self {
        Self {
            inner: writer,
            style,
        }
    }
}

impl<T: Write> Write for ColorLine<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<T: Write> WriteLine for ColorLine<T> {
    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.inner, "{}", self.style.style(line))
    }
}
