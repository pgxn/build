use owo_colors::Style;
use std::{
    fmt,
    io::{Result, Write},
};

// WriteLine extends [io::Write] to add a function for writing a line of text.
pub trait WriteLine: Write {
    fn write_line(&mut self, line: &str) -> Result<()>;
}

// LineWriter implements WriteLine to write a line of text to an internal
// [std::io::Write] implementation.
pub struct LineWriter<T: Write>(T);

impl<T: Write> LineWriter<T> {
    // Create a new LineWriter that writes lines of text to `writer`.
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
    // Write `line` to the underlying writer.
    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.0, "{}", line)
    }
}

impl<W: WriteLine + ?Sized> WriteLine for Box<W> {
    // Write `line` to the underlying writer.
    fn write_line(&mut self, line: &str) -> Result<()> {
        (**self).write_line(line)
    }
}

// ColorLine implements WriteLine to write a colored line of text to an
// internal [std::io::Write] implementation.
pub struct ColorLine<T: Write> {
    inner: T,
    style: Style,
}

impl<T: Write> ColorLine<T> {
    // Create a new ColorLine that writes lines of text styled with `style` to
    // `writer`.
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
    /// Styles and write `line` to the underlying writer.
    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.inner, "{}", self.style.style(line))
    }
}

#[cfg(test)]
mod tests;
