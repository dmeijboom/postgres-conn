use std::io;
use std::io::Write;

pub struct Writer<W: Write> {
    inner: W,
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.inner.write_all(&[byte])
    }

    pub fn write_i32(&mut self, value: i32) -> io::Result<()> {
        self.inner.write_all(&value.to_be_bytes())
    }

    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_all(s.as_bytes())?;
        self.inner.write_all(&[0])
    }
}
