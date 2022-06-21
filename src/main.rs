use std::io;
use std::net::{TcpListener, TcpStream};

use crate::backend::Backend;

mod backend;
mod proto;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5432")?;

    for stream in listener.incoming() {
        handle(stream?);
    }

    Ok(())
}

fn handle(conn: TcpStream) {
    println!("incoming connection");

    match Backend::new(conn).and_then(|mut b| b.handle()) {
        Ok(_) => println!("connection closed"),
        Err(e) => eprintln!("failed to handle connection: {}", e),
    }
}
