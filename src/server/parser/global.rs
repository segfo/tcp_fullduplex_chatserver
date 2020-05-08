use crate::server::parser::*;

// 1:n通知:全体通知のパーサ
pub struct GlobalParser;
impl GlobalParser {
    pub fn new()->Self {
        GlobalParser{}
    }
}
impl ParserHandler for GlobalParser {
    fn set_next(&mut self, _next: Container<dyn ParserHandler>) {
        unimplemented!();
    }
    fn do_parse(&self,msg:String,sender_peer:std::net::SocketAddr,client_list:&ClientList) {
        let s = format!("[{}:{}#public] {}", sender_peer.ip(), sender_peer.port(), msg);
        for client in client_list.values() {
            let (_reader, writer) = &client.channel;
            writer.send(ServerStateKind::MessageResponse(s.as_bytes().to_vec()));
        }
    }
}