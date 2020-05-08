use crate::server::parser::*;

// 接続者リストの取得
pub struct ConnectionListParser {
    next: Option<Container<dyn ParserHandler>>,
}
impl ConnectionListParser {
    pub fn new()->Self {
        ConnectionListParser{
            next:None,
        }
    }
}
impl ParserHandler for ConnectionListParser {
    fn set_next(&mut self, next: Container<dyn ParserHandler>) {
        self.next = Some(next);
    }
    fn do_parse(&self,msg: String,sender_peer:std::net::SocketAddr,client_list:&ClientList) {
        let msg = msg.clone();
        let re = Regex::new(r"/get_clients").unwrap();
        let captures = re.captures(&msg);
        if !captures.is_none(){
            let mut s = String::new();
            for client in client_list.values() {
                s.push_str(&format!("{}:{}\n",client.addr.ip(),client.addr.port()));
            }
            let (_reader,writer) = &client_list[&sender_peer].channel;
            writer.send(ServerStateKind::MessageResponse(s.as_bytes().to_vec()));
        }else if self.next.is_some(){
            self.next.as_ref().unwrap().do_parse(msg,sender_peer,client_list);
        }
    }
}