use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

pub struct ProxyClient {
    stream: TcpStream
}

impl ProxyClient {
    pub async fn new(stream: TcpStream) -> Self {
        Self {
            stream
        }
    }
    pub async fn run(self, data: String, tx: Sender<Vec<u8>>) {
        let mut stream = self.stream;
        let mut rx_bytes = [0u8; 4086];
        stream.write(data.as_bytes()).await.expect("");
        let data = stream.read(&mut rx_bytes).await.expect("");
        if data > 0 {
            tx.send(Vec::from(rx_bytes)).await.expect("cannot send tx from client");
        }
    }
}
