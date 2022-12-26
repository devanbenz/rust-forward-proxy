use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ProxyServer<'a> {
    pub addr: &'a str,
}

impl<'a> ProxyServer<'a> {
    pub async fn new(addr: &'a str) -> ProxyServer<'a> {
        ProxyServer {
            addr
        }
    }
    pub async fn run(self, tx: Sender<Vec<u8>>, mut rx: Receiver<Vec<u8>>) {
        let listener = TcpListener::bind(self.addr).await.expect("cannot bind to addr");
        loop {
            let (mut socket, _) = listener.accept().await.expect("could not accept listener");
            let mut rx_bytes = [0u8; 4086];
            socket.read(&mut rx_bytes).await.expect("cannot read buf");
            tx.send(Vec::from(rx_bytes)).await.expect("TODO: panic message");
            if let Some(message) = rx.recv().await {
                socket.write_all(&message[..]).await.expect("cannot write to listener");
            }
        }
    }
}
