#[macro_use]
mod structs;
mod config;
mod define;
use crate::config::TomlConfigDeserializer;
use crate::structs::ServerConf;
use std::{
    net::TcpListener,
};

fn main() {
    server();
}
mod server;
use server::Server;
fn server() {
    let config: ServerConf =
        TomlConfigDeserializer::from_file("server.toml").unwrap_or_else(|_| ServerConf::default());
    let addr = (|| {
        let listen_if = config.interface.unwrap_or("127.0.0.1".to_owned());
        let port = config.port.unwrap_or(8080);
        format!("{}:{}", listen_if, port)
    })();
    let server = Server::new(TcpListener::bind(addr).unwrap());
    server.serve();
}
