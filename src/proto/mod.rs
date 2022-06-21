pub mod messages;
mod reader;
mod writer;

use std::io;
use std::io::{Read, Write};

pub use reader::Reader;
pub use writer::Writer;

pub trait Encode {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()>;
}

pub trait Decode {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized;
}
