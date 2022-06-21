use std::io;
use std::net::{TcpListener, TcpStream};

use crate::backend::Backend;

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

fn handle(conn: TcpStream) {
    log::debug!("new connection");

    match Backend::new(conn).and_then(|mut b| b.handle()) {
        Ok(_) => log::debug!("connection closed"),
        Err(e) => log::error!("failed to handle connection: {}", e),
    }
}
