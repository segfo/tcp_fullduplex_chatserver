pub mod global;
pub mod whisper;
pub mod connection_list;

use crate::define::ServerStateKind;
use regex::Regex;
use crate::structs::SocketInfo;
type ClientList = std::collections::HashMap<std::net::SocketAddr, SocketInfo>;

pub type Container<T> = Box<T>;

pub trait ParserHandler{
    fn set_next(&mut self,next: Container<dyn ParserHandler>);
    fn do_parse(&self,msg:String,sender_peer:std::net::SocketAddr,client_list:&ClientList);
}
