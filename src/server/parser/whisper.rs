use crate::server::parser::*;

// 1:1の通知:Wisperチャットのパーサ
pub struct WhisperParser {
    next: Option<Container<dyn ParserHandler>>,
}
impl WhisperParser {
    pub fn new()->Self {
        WhisperParser{
            next:None,
        }
    }
}
impl ParserHandler for WhisperParser {
    fn set_next(&mut self, next: Container<dyn ParserHandler>) {
        self.next = Some(next);
    }
    fn do_parse(&self,msg: String,sender_peer:std::net::SocketAddr,client_list:&ClientList) {
        let msg = msg.clone();
        let re = Regex::new(r"/wisper\s(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\s?+?:\s?+?(\d{1,5})\s(.+)").unwrap();
        let captures = re.captures(&msg);

        if let Some(captures) = captures{
            let clients = client_list;
            let recv_peer = std::net::SocketAddr::new(captures.get(1).unwrap().as_str().parse().unwrap(),captures.get(2).unwrap().as_str().parse().unwrap());
            if clients.contains_key(&recv_peer){
                println!("wisper chat from:{}:{} to:{}:{}",sender_peer.ip(),sender_peer.port(),recv_peer.ip(),recv_peer.port());
                let (_reader,writer) = &clients[&recv_peer].channel;
                let s = format!("[{}:{}#wisper] {}",sender_peer.ip(),sender_peer.port(),msg);
                writer.send(ServerStateKind::MessageResponse(s.as_bytes().to_vec()));
            }else{
                let (_reader,writer) = &clients[&sender_peer].channel;
                writer.send(ServerStateKind::MessageResponse("[system]wisper chat non-available host.".as_bytes().to_vec()));        
            }
        }else if self.next.is_some(){
            self.next.as_ref().unwrap().do_parse(msg,sender_peer,client_list);
        }
    }
}

