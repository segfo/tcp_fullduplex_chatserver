use crate::structs::*;
#[derive(Debug, PartialEq)]
pub enum ServerStateKind {
    MessageRequest(Vec<u8>, std::net::SocketAddr),
    MessageResponse(Vec<u8>),
    ConnectEstablished(SocketInfo),
    BeforeClosing(std::net::SocketAddr),
}