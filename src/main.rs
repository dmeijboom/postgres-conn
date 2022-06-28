use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use clap::Parser;

use crate::backend::{Conn, Manager, NoopAuth, NoopQueryExec};

mod backend;
mod proto;

#[derive(Parser)]
struct Opts {
    address: Option<SocketAddr>,
}

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let addr = opts
        .address
        .unwrap_or_else(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5432));

    log::info!("starting postgres-conn on {}", addr);

    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        handle(stream?);
    }

    Ok(())
}

fn handle(stream: TcpStream) {
    log::info!("new connection");

    match Conn::new(stream)
        .and_then(|c| Manager::new(c, NoopAuth::new(), NoopQueryExec::new()))
        .and_then(|mut b| b.handle())
    {
        Ok(_) => log::info!("connection closed"),
        Err(e) => log::info!("failed to handle connection: {}", e),
    }
}
