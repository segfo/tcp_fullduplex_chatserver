use serde_derive::{Deserialize, Serialize};
use crate::define::*;

#[derive(Deserialize)]
pub struct ServerConf {
    pub interface: Option<String>,
    pub port: Option<u16>,
}
impl ServerConf {
    pub fn default() -> Self {
        ServerConf {
            interface: Some("127.0.0.1".to_owned()),
            port: Some(8080),
        }
    }
}

// 送受信スレッド間の同期チャネル
pub struct SyncChannel<T>(std::marker::PhantomData<T>);
pub struct MixedSyncCannel<T> {
    rx: std::sync::mpsc::Receiver<T>,
    tx: std::sync::mpsc::Sender<T>,
}
impl<T> SyncChannel<T> {
    pub fn new(
        (tx0, rx0): (std::sync::mpsc::Sender<T>, std::sync::mpsc::Receiver<T>),
        (tx1, rx1): (std::sync::mpsc::Sender<T>, std::sync::mpsc::Receiver<T>),
    ) -> (MixedSyncCannel<T>, MixedSyncCannel<T>) {
        (
            MixedSyncCannel { tx: tx0, rx: rx1 },
            MixedSyncCannel { tx: tx1, rx: rx0 },
        )
    }
}

impl<T> MixedSyncCannel<T> {
    pub fn send(&mut self, data: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        self.tx.send(data)
    }
    pub fn recv(&self) -> Result<T, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }
    pub fn get_sender(&self) -> std::sync::mpsc::Sender<T> {
        self.tx.clone()
    }
}

// コネクション情報と各スレッドに対するメッセージパイプ
#[derive(Debug)]
pub struct SocketInfo {
    pub addr: std::net::SocketAddr,
    pub channel: (
        std::sync::mpsc::Sender<ServerStateKind>,
        std::sync::mpsc::Sender<ServerStateKind>,
    ),
}

impl SocketInfo {
    pub fn new(
        addr: std::net::SocketAddr,
        channel: (
            std::sync::mpsc::Sender<ServerStateKind>,
            std::sync::mpsc::Sender<ServerStateKind>,
        ),
    ) -> Self {
        SocketInfo {
            addr: addr,
            channel: channel,
        }
    }
}

// SocketInfoの比較はコネクション情報で行う。
impl PartialEq for SocketInfo {
    fn eq(&self, lhs: &Self) -> bool {
        self.addr == lhs.addr
    }
}
#[macro_export]
macro_rules! sync_channel {
    () => {
        SyncChannel::new(std::sync::mpsc::channel(), std::sync::mpsc::channel())
    };
    ($ch:expr) => {
        SyncChannel::new(std::sync::mpsc::channel(), $ch)
    };
}