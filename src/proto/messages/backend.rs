use std::io;
use std::io::Write;

use crate::proto::{Encode, Writer};

macro_rules! sizeof {
    (i32) => { sizeof!(int i32) };
    (u8) => { 1 };

    (int $kind:ty) => {
        (<$kind>::BITS / 8) as i32
    }
}

#[allow(dead_code)]
pub enum SSLResponse {
    Ssl,
    NoSsl,
}

impl Encode for SSLResponse {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        match self {
            Self::Ssl => writer.write_byte(b'S'),
            Self::NoSsl => writer.write_byte(b'N'),
        }
    }
}

macro_rules! impl_auth_msg {
    ($(($ty:ident, $kind:expr)),+) => {
        $(impl_auth_msg!{$ty, $kind})+
    };

    ($ty:ident, $kind:expr) => {
        pub struct $ty {}

        impl Encode for $ty {
            fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
                writer.write_byte(b'R')?;
                writer.write_i32(sizeof!(i32) + sizeof!(i32))?;
                writer.write_i32($kind)
            }
        }
    };
}
impl_auth_msg!(
    (AuthenticationOk, 0),
    (AuthenticationCleartextPassword, 3)
);

#[allow(dead_code)]
pub enum Field {
    SeverityI18n,
    Severity,
    Code,
    Message,
    Detail,
    Hint,
    Position,
    InternalPosition,
    Query,
    Where,
    Schema,
    Table,
    Column,
    DataType,
    Constraint,
    File,
    Line,
    Routine,
}

impl Encode for Field {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.write_byte(match self {
            Self::SeverityI18n => b'S',
            Self::Severity => b'V',
            Self::Code => b'C',
            Self::Message => b'M',
            Self::Detail => b'D',
            Self::Hint => b'H',
            Self::Position => b'P',
            Self::InternalPosition => b'p',
            Self::Query => b'q',
            Self::Where => b'W',
            Self::Schema => b's',
            Self::Table => b't',
            Self::Column => b'c',
            Self::DataType => b'd',
            Self::Constraint => b'n',
            Self::File => b'F',
            Self::Line => b'L',
            Self::Routine => b'R',
        })
    }
}

pub struct ErrorResponse {
    pub fields: Vec<(Field, String)>,
}

impl ErrorResponse {
    pub fn new(severity: String, code: String, message: String) -> Self {
        Self {
            fields: vec![
                (Field::Severity, severity),
                (Field::Code, code),
                (Field::Message, message),
            ],
        }
    }
}

impl Encode for ErrorResponse {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.write_byte(b'E')?;
        writer.write_i32(
            sizeof!(i32)
                + self
                    .fields
                    .iter()
                    .map(|(_, value)| (value.len() + sizeof!(u8) + sizeof!(u8)) as i32)
                    .sum::<i32>()
                + sizeof!(u8),
        )?;

        for (field, value) in self.fields.iter() {
            field.encode(writer)?;
            writer.write_str(value)?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub enum TransactionStatus {
    Idle,
    InTransaction,
    Failed,
}

impl Encode for TransactionStatus {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.write_byte(match self {
            Self::Idle => b'r',
            Self::InTransaction => b'T',
            Self::Failed => b'E',
        })
    }
}

pub struct ReadyForQuery {
    pub transaction_status: TransactionStatus,
}

impl ReadyForQuery {
    pub fn new(transaction_status: TransactionStatus) -> Self {
        Self { transaction_status }
    }
}

impl Encode for ReadyForQuery {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.write_byte(b'Z')?;
        writer.write_i32(sizeof!(i32) + sizeof!(u8))?;
        self.transaction_status.encode(writer)
    }
}

pub struct CommandComplete {
    pub command_tag: String,
}

impl CommandComplete {
    pub fn new(command_tag: String) -> Self {
        Self { command_tag }
    }
}

impl Encode for CommandComplete {
    fn encode<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.write_byte(b'C')?;
        writer.write_i32(sizeof!(i32) + self.command_tag.len() as i32 + sizeof!(u8))?;
        writer.write_str(&self.command_tag)
    }
}
