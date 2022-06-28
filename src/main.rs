use std::io;
use std::net::{TcpListener, TcpStream};

use crate::backend::{Conn, Manager};

mod backend;
mod proto;

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:5432")?;

    for stream in listener.incoming() {
        handle(stream?);
    }

    Ok(())
}

fn handle(stream: TcpStream) {
    log::debug!("new connection");

    match Conn::new(stream)
        .and_then(|c| Manager::new(c))
        .and_then(|mut b| b.handle())
    {
        Ok(_) => log::debug!("connection closed"),
        Err(e) => log::error!("failed to handle connection: {}", e),
    }
}
