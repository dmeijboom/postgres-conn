use std::collections::HashMap;
use std::io;

use crate::backend::auth::{AuthMethod, AuthResult};
use crate::backend::{Auth, Conn};

use crate::proto::messages::{
    AuthenticationCleartextPassword, AuthenticationOk, CommandComplete, ErrorResponse, Handshake,
    IncomingMessage, ReadyForQuery, SSLResponse, Severity, TransactionStatus,
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

pub struct Manager<A: Auth> {
    conn: Conn,
    state: State,
    authenticator: A,
    postgres_version: i32,
}

impl<A: Auth> Manager<A> {
    pub fn new(conn: Conn, authenticator: A) -> io::Result<Self> {
        Ok(Self {
            conn,
            state: State::default(),
            authenticator,
            postgres_version: 0,
        })
    }

    // In the startup phase we optionally setup SSL encryption (not implemented yet) and parse the
    // startup message which contains the initial state
    // @TODO: support cancel-request
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

    pub fn auth(&mut self, method: AuthMethod) -> io::Result<AuthResult> {
        Ok(match method {
            AuthMethod::CleartextPassword => {
                self.conn.send(AuthenticationCleartextPassword {})?;
                self.authenticator
                    .clear_text_password(&self.state, self.conn.recv()?)
            }
            AuthMethod::None => AuthResult::Ok,
        })
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
                Severity::Error,
                "P0001".to_string(),
                "the 'user' option is mandatory".to_string(),
            ))?;
        }

        let method = self.authenticator.method(&self.state);
        log::debug!("selecting auth method: {:?}", method);

        match self.auth(method)? {
            AuthResult::Ok => self.conn.send(AuthenticationOk {})?,
            AuthResult::Err(msg) => {
                log::debug!("auth failed: {}", msg);

                return self.conn.send(ErrorResponse::new(
                    Severity::Error,
                    "28P01".to_string(),
                    msg,
                ));
            }
        };

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
