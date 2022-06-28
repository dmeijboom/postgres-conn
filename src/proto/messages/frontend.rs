use std::io;
use std::io::Read;

use secstr::SecStr;

use crate::proto::{Decode, Reader};

const SSL_REQUEST_CODE: i32 = 80877103;

pub enum Handshake {
    SSLRequest(SSLRequest),
    StartupMessage(StartupMessage),
}

impl Decode for Handshake {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;
        let version = reader.read_i32()?;

        if version == SSL_REQUEST_CODE {
            return Ok(Handshake::SSLRequest(SSLRequest { len, code: version }));
        }

        let params = Params::decode(reader)?;

        Ok(Handshake::StartupMessage(StartupMessage {
            len,
            version,
            params,
        }))
    }
}

pub struct SSLRequest {
    pub len: i32,
    pub code: i32,
}

impl Decode for SSLRequest {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;
        let code = reader.read_i32()?;

        if code != SSL_REQUEST_CODE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid SSLRequest code",
            ));
        }

        Ok(Self { len, code })
    }
}

#[derive(Debug)]
pub struct Params(Vec<(String, String)>);

impl Params {
    pub fn into_iter(self) -> impl Iterator<Item = (String, String)> {
        self.0.into_iter()
    }
}

impl Decode for Params {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut params = vec![];

        loop {
            let name = reader.read_string()?;
            let value = reader.read_string()?;

            params.push((name, value));

            if let Some(b'\0') = reader.peek()? {
                break;
            }
        }

        // Read the null byte that we peeked but didn't consume
        reader.read_byte()?;

        Ok(Self(params))
    }
}

#[derive(Debug)]
pub struct StartupMessage {
    pub len: i32,
    pub version: i32,
    pub params: Params,
}

impl Decode for StartupMessage {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;
        let version = reader.read_i32()?;
        let params = Params::decode(reader)?;

        Ok(Self {
            len,
            version,
            params,
        })
    }
}

pub enum IncomingMessage {
    Query(Query),
    Terminate(Terminate),
}

impl Decode for IncomingMessage {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let id = reader.read_byte()?;

        match id {
            b'Q' => Ok(IncomingMessage::Query(Query::decode(reader)?)),
            b'X' => Ok(IncomingMessage::Terminate(Terminate::decode(reader)?)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid message type: {:?}", id),
            )),
        }
    }
}

pub struct Terminate {
    pub len: i32,
}

impl Decode for Terminate {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;

        Ok(Self { len })
    }
}

#[derive(Debug)]
pub struct Query {
    pub len: i32,
    pub query: String,
}

impl Decode for Query {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;
        let query = reader.read_string()?;

        Ok(Self { len, query })
    }
}

pub struct PasswordMessage {
    pub len: i32,
    pub password: SecStr,
}

impl Decode for PasswordMessage {
    fn decode<R: Read>(reader: &mut Reader<R>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_i32()?;
        let password = reader.read_string_bytes()?;

        Ok(Self {
            len,
            password: SecStr::new(password),
        })
    }
}
