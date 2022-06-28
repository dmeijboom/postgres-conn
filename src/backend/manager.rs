use std::collections::HashMap;
use std::io;

use crate::backend::Conn;

use crate::proto::messages::{
    AuthenticationOk, CommandComplete, ErrorResponse, Handshake, IncomingMessage, ReadyForQuery,
    SSLResponse, TransactionStatus,
};

pub enum Replication {
    Enabled,
    Disabled,
    Database,
}

pub struct State {
    user: String,
    database: String,
    replication: Replication,
    extra_params: HashMap<String, String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            user: String::new(),
            database: String::new(),
            replication: Replication::Disabled,
            extra_params: HashMap::new(),
        }
    }
}

pub struct Manager {
    conn: Conn,
    state: State,
    postgres_version: i32,
}

impl Manager {
    pub fn new(conn: Conn) -> io::Result<Self> {
        Ok(Self {
            conn,
            state: State::default(),
            postgres_version: 0,
        })
    }

    // In the startup phase we optionally setup SSL encryption (not implemented yet) and parse the
    // startup message which contains the initial state
    fn handle_startup(&mut self) -> io::Result<()> {
        let handshake: Handshake = self.conn.recv()?;
        let startup_msg = match handshake {
            // @TODO: currently we don't support SSL encryption
            Handshake::SSLRequest(_) => {
                self.conn.send(SSLResponse::NoSsl)?;
                self.conn.recv()?
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
                _ => {
                    self.state.extra_params.insert(name, value);
                }
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

            self.conn.send(ErrorResponse::new(
                "ERROR".to_string(),
                "P0001".to_string(),
                "the 'user' option is mandatory".to_string(),
            ))?;
        }

        // @TODO: we don't support authentication yet
        self.conn.send(AuthenticationOk {})?;
        self.conn
            .send(ReadyForQuery::new(TransactionStatus::Idle))?;

        log::debug!("waiting for queries");

        loop {
            let msg: IncomingMessage = self.conn.recv()?;

            match msg {
                IncomingMessage::Query(query) => {
                    log::debug!("received query: {}", query.query);

                    self.conn
                        .send(CommandComplete::new("SELECT 0".to_string()))?;
                    self.conn
                        .send(ReadyForQuery::new(TransactionStatus::Idle))?;
                }
                IncomingMessage::Terminate(_) => return Ok(()),
            }
        }
    }
}
