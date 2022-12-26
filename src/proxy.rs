use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;
use crate::client::ProxyClient;
use crate::server::ProxyServer;

pub async fn run_proxy(target_addr: String, listener_addr: String) {
    let target_addr_str: &str = Box::leak(Box::new(target_addr));
    let listener_addr_str: &str = Box::leak(Box::new(listener_addr));

    let (tx, mut rx) = channel::<Vec<u8>>(4086);
    let (tx2, rx2) = channel::<Vec<u8>>(4086);

    let proxy_server = ProxyServer::new(listener_addr_str).await;

    tokio::spawn(async move {
        proxy_server.run(tx, rx2).await;
    });

    while let Some(message) = rx.recv().await {
        let proxy_client = ProxyClient::new(TcpStream::connect(target_addr_str).await.expect("unable to connect to server")).await;
        let data_str = String::from_utf8(message).expect("invalid data");
        let header_host = format!("Host: {}", &listener_addr_str);
        let modified_host = format!("Host: {}", &target_addr_str);
        let trimmed_str = data_str.trim_end_matches('\0').replace(&header_host, &modified_host);
        println!("{:?}", &trimmed_str);
        proxy_client.run(trimmed_str.to_string(), tx2.clone()).await;
    }

}
