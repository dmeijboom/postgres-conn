use std::io;
use std::net::TcpStream;

use crate::proto::messages::{
    AuthenticationOk, CommandComplete, ErrorResponse, Handshake, IncomingMessage, ReadyForQuery,
    SSLResponse, TransactionStatus,
};
use crate::proto::{Decode, Encode, Reader, Writer};

pub enum Replication {
    Enabled,
    Disabled,
    Database,
}

pub struct BackendState {
    user: String,
    database: String,
    replication: Replication,
}

impl Default for BackendState {
    fn default() -> Self {
        Self {
            user: String::new(),
            database: String::new(),
            replication: Replication::Disabled,
        }
    }
}

struct Conn {
    reader: Reader<TcpStream>,
    writer: Writer<TcpStream>,
}

pub struct Backend {
    conn: Conn,
    state: BackendState,
    postgres_version: i32,
}

impl Backend {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            conn: Conn {
                reader: Reader::new(stream.try_clone()?),
                writer: Writer::new(stream.try_clone()?),
            },
            state: BackendState::default(),
            postgres_version: 0,
        })
    }

    pub fn recv<T: Decode>(&mut self) -> io::Result<T> {
        T::decode(&mut self.conn.reader)
    }

    pub fn send<T: Encode>(&mut self, msg: T) -> io::Result<()> {
        msg.encode(&mut self.conn.writer)
    }

    // In the startup phase we optionally setup SSL encryption (not implemented yet) and parse the
    // startup message which contains the initial state
    fn handle_startup(&mut self) -> io::Result<()> {
        let handshake: Handshake = self.recv()?;
        let startup_msg = match handshake {
            // @TODO: currently we don't support SSL encryption
            Handshake::SSLRequest(_) => {
                self.send(SSLResponse::NoSsl)?;
                self.recv()?
            }
            Handshake::StartupMessage(msg) => msg,
        };

        self.postgres_version = startup_msg.version;

        for (name, value) in startup_msg.params.into_iter() {
            match name.as_str() {
                "user" => self.state.user = value,
                "database" => self.state.database = value,
                "replication" => {
                    self.state.replication = match value.as_str() {
                        "database" => Replication::Database,
                        "disabled" => Replication::Disabled,
                        "enabled" => Replication::Enabled,
                        _ => unreachable!(),
                    }
                }
                _ => (),
            }
        }

        if self.state.database.is_empty() {
            self.state.database = self.state.user.clone();
        }

        Ok(())
    }

    pub fn handle(&mut self) -> io::Result<()> {
        log::debug!("entering startup phase");

        loop {
            self.handle_startup()?;

            if !self.state.user.is_empty() {
                break;
            }

            log::error!("no user specified, retrying startup");

            self.send(ErrorResponse::new(
                "ERROR".to_string(),
                "P0001".to_string(),
                "the 'user' option is mandatory".to_string(),
            ))?;
        }

        // @TODO: we don't support authentication yet
        self.send(AuthenticationOk {})?;
        self.send(ReadyForQuery::new(TransactionStatus::Idle))?;

        log::debug!("waiting for queries");

        loop {
            let msg: IncomingMessage = self.recv()?;

            match msg {
                IncomingMessage::Query(query) => {
                    log::debug!("received query: {}", query.query);

                    self.send(CommandComplete::new("SELECT 0".to_string()))?;
                    self.send(ReadyForQuery::new(TransactionStatus::Idle))?;
                }
                IncomingMessage::Terminate(_) => return Ok(()),
            }
        }
    }
}
