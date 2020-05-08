use crate::define::*;
use crate::structs::*;
use crate::server::parser::*;

use std::{
    io::{BufRead, BufReader, BufWriter, Write},
};

pub struct Server {
    listener: std::net::TcpListener,
}

type ClientList = std::collections::HashMap<std::net::SocketAddr, SocketInfo>;

impl Server {
    // パーサの初期化
    fn init_parser()->Box<dyn ParserHandler>{
        let mut connection_list = Box::new(connection_list::ConnectionListParser::new());
        let mut whisper = Box::new(whisper::WhisperParser::new());
        let global = Box::new(global::GlobalParser::new());
        // connection_list -> whisper -> global(default action)
        whisper.set_next(global);
        connection_list.set_next(whisper);
        connection_list
    }
    pub fn new(listener: std::net::TcpListener) -> Self {
        Server { 
            listener: listener,
        }
    }
    // 各クライアントから送信された情報がここに集約される
    // イベント発生時は、server_events.rsの中にある処理に委譲する。
    fn run_peer_eventloop(rx: std::sync::mpsc::Receiver<ServerStateKind>) {
        std::thread::spawn(move || {
            let mut clients: ClientList =
                std::collections::HashMap::new();
            // パーサの初期化
            let parser = Server::init_parser();
            loop {
                let msg = rx.recv().unwrap();
                match msg {
                    // 特定のTCPセッションが終了される
                    ServerStateKind::BeforeClosing(sockaddr) => {
                        let s =
                            format!("disconnection from {}:{} ", sockaddr.ip(), sockaddr.port());
                        println!("{}", s);
                        clients.remove_entry(&sockaddr);
                        println!("entry removed.");
                        for client in clients.values() {
                            let (_reader, writer) = &client.channel;
                            writer.send(ServerStateKind::MessageResponse(s.as_bytes().to_vec()));
                        }
                    }
                    // クライアントとTCPコネクションを確立した。
                    ServerStateKind::ConnectEstablished(sockinfo) => {
                        let s = format!(
                            "incomming host {}:{} ",
                            sockinfo.addr.ip(),
                            sockinfo.addr.port()
                        );
                        for client in clients.values() {
                            let (_reader, writer) = &client.channel;
                            writer.send(ServerStateKind::MessageResponse(s.as_bytes().to_vec()));
                        }
                        clients.insert(sockinfo.addr, sockinfo);
                    }
                    // メッセージをいい感じに取り回す。
                    ServerStateKind::MessageRequest(msg, sender_peer) => {
                        println!("(server)in message {:?}", msg);
                        // バイナリをutf-8に変換する
                        let s = match String::from_utf8(msg) {
                            Ok(s) => s,
                            Err(_e) => {
                                // エラーだったら自分自身にエラーを返す。
                                let (_reader, writer) = &clients[&sender_peer].channel;
                                writer.send(ServerStateKind::MessageResponse(
                                    "<system : invalid utf8>".as_bytes().to_vec(),
                                ));
                                return;
                            }
                        };
                        // メッセージをパースして、いい感じにやる
                        parser.do_parse(s,sender_peer,&clients);
                    }
                    _ => {}
                }
            }
        });
    }

    // 接続待受と送受信の待機処理（実際の処理はsender/reciever workerが行う
    pub fn serve(&self) {
        let (tx, rx) = std::sync::mpsc::channel();
        Server::run_peer_eventloop(rx);
        loop {
            // サーバの接続待受
            let sockinfo = self.listener.accept().unwrap();
            let (stream, sockaddr) = sockinfo;
            println!("connect host : {}:{}", sockaddr.ip(), sockaddr.port());
            let reader_stream = stream.try_clone().expect("Cannot clone stream");
            let writer_stream = stream.try_clone().expect("Cannot clone stream");
            let mut reader =
                BufReader::new(reader_stream.try_clone().expect("Cannot clone stream"));
            let mut writer =
                BufWriter::new(writer_stream.try_clone().expect("Cannot clone stream"));
            //
            let (mut sender_ch, mut reciever_ch) = sync_channel!();
            // 各コネクション情報とサービスと送受信スレッドのメッセージパイプ
            // メインループにコネクションが成立したことを報告する。
            let info =
                SocketInfo::new(sockaddr, (sender_ch.get_sender(), reciever_ch.get_sender()));
            let recviever_to_server_tx = tx.clone();
            let sender_to_server = tx.clone();
            tx.clone().send(ServerStateKind::ConnectEstablished(info));
            let stream_recv = stream.try_clone().expect("spawn");
            // 受信スレッド
            std::thread::spawn(move || {
                Server::reciever_worker(
                    reader,
                    reciever_ch,
                    recviever_to_server_tx,
                    (stream_recv, sockaddr),
                );
            });
            // 送信スレッド
            std::thread::spawn(move || {
                let sockinfo = (stream, sockaddr);
                Server::sender_worker(writer, sender_ch, sender_to_server, sockinfo);
            });
        }
    }
    // 各クライアントに対応するreciever worker
    fn reciever_worker(
        mut reader: std::io::BufReader<std::net::TcpStream>,
        mut sender_sync_channel: MixedSyncCannel<ServerStateKind>,
        server_tx: std::sync::mpsc::Sender<ServerStateKind>,
        sockinfo: (std::net::TcpStream, std::net::SocketAddr),
    ) {
        let (tcp_stream, sockaddr) = sockinfo;
        // sender workerへのtx
        let server = server_tx.clone();
        loop {
            let mut s = String::new();
            let result = reader.read_line(&mut s);
            if result.is_err() || result.unwrap() == 0 {
                sender_sync_channel
                    .send(ServerStateKind::BeforeClosing(sockaddr))
                    .expect("send error");
                server
                    .send(ServerStateKind::BeforeClosing(sockaddr))
                    .expect("send error");
                tcp_stream.shutdown(std::net::Shutdown::Both);
                break;
            }
            // サーバスレッドに受信したメッセージを送る
            let r = server.send(ServerStateKind::MessageRequest(
                s.as_bytes().to_vec(),
                sockaddr,
            ));
        }
    }
    // 各クライアントに対するsender worker
    fn sender_worker(
        mut writer: std::io::BufWriter<std::net::TcpStream>,
        reciever_sync_channel: MixedSyncCannel<ServerStateKind>,
        _server_tx: std::sync::mpsc::Sender<ServerStateKind>,
        sockinfo: (std::net::TcpStream, std::net::SocketAddr),
    ) {
        loop {
            let r = reciever_sync_channel.recv();
            let r = r.expect("sync channel recv error");
            match r {
                ServerStateKind::BeforeClosing(_sockaddr) => {
                    eprintln!("Broken pipe.");
                    break;
                }
                ServerStateKind::MessageResponse(msg) => {
                    writer.write(&msg);
                    writer.flush();
                }
                _ => {}
            }
        }
    }
}
