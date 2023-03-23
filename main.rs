use std::{sync::{Arc, atomic::{AtomicU64, Ordering}}, time::Duration, process, pin::Pin, future::Future};

use http_body_util::{Full, BodyExt, combinators::BoxBody, Empty};
use hyper::{Request, server::conn::http1, service::{service_fn, Service}, Response, body::{Bytes, Incoming}, Method};
use tokio::{net::{TcpListener, TcpStream}, time::{Instant, sleep}};

#[tokio::main]
async fn main() {
    let conn_count: Arc<AtomicU64> = Default::default();

    tokio::spawn({
        let conn_count = conn_count.clone();
        let mut activity = Instant::now();

        async move {
            loop {
                if conn_count.load(Ordering::SeqCst) > 0 {
                    activity = Instant::now();
                } else {
                    let idle_time = activity.elapsed();
                    println!("Idle for {idle_time:?}");
                    if idle_time > Duration::from_secs(30) {
                        process::exit(0)
                    }
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    });

    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    while let Ok((ingress, _)) = listener.accept().await {
        let conn_count = conn_count.clone();
       
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(ingress, service_fn(get_headers))
                .await 
            {
                    println!("Error serving connection: {:?}", err);
            }

            conn_count.fetch_add(1, Ordering::SeqCst);


        });
    }
}

async fn get_headers(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, hyper::Error> {
    if req.method() != Method::CONNECT {
        let addr = format!("{}:{}", 
                           req.uri().host().unwrap(),
                           req.uri().port_u16().unwrap_or(80));
        println!("{addr:?}");

        let stream: TcpStream = TcpStream::connect(addr).await.expect("connection error");

        let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(stream)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;
        println!("{resp:?}");

        Ok(resp.map(|b| b.boxed()))
    } else {
        Ok(Response::new(empty()))
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
