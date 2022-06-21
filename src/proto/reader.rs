use std::io;
use std::io::{BufRead, BufReader, Read};

pub struct Reader<R: Read> {
    buf_reader: BufReader<R>,
}

impl<R: Read> Reader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            buf_reader: BufReader::new(inner),
        }
    }

    pub fn peek(&mut self) -> io::Result<Option<&u8>> {
        let buf = self.buf_reader.fill_buf()?;

        if !buf.is_empty() {
            return Ok(Some(&buf[0]));
        }

        Ok(None)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.buf_reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.buf_reader.read_exact(&mut buf)?;

        Ok(i32::from_be_bytes(buf))
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let mut buf = vec![];
        self.buf_reader.read_until(b'\0', &mut buf)?;

        // Remove the null byte
        buf.pop();

        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
