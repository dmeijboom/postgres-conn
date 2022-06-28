use std::io;
use std::net::TcpStream;

use crate::proto::{Decode, Encode, Reader, Writer};

pub struct Conn {
    reader: Reader<TcpStream>,
    writer: Writer<TcpStream>,
}

impl Conn {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: Reader::new(stream.try_clone()?),
            writer: Writer::new(stream.try_clone()?),
        })
    }

    #[inline]
    pub fn recv<T: Decode>(&mut self) -> io::Result<T> {
        T::decode(&mut self.reader)
    }

    #[inline]
    pub fn send<T: Encode>(&mut self, msg: T) -> io::Result<()> {
        msg.encode(&mut self.writer)
    }
}
