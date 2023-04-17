use std::{
    fs::File,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Method, Request, Response};
use hyper_rustls::ConfigBuilderExt;
use tls_listener::{AsyncTls, TlsListener};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{
    rustls::{Certificate, ClientConfig, PrivateKey, ServerConfig, ServerName},
    TlsConnector,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    println!("Listening on port 9000...");

    while let Ok((ingress, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(ingress, service_fn(get_headers))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn get_headers(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, hyper::Error> {
    if req.method() != Method::CONNECT {
        let addr = format!(
            "{}:{}",
            req.uri().host().unwrap(),
            req.uri().port_u16().unwrap_or(80)
        );
        println!("{addr:?}");

        let resp = match req.uri().port_u16().unwrap_or(80) {
            443 => {
                let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
                root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
                    |ta| {
                        tokio_rustls::rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                            ta.subject,
                            ta.spki,
                            ta.name_constraints,
                        )
                    },
                ));

                let tls_config = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                let connector = TlsConnector::from(Arc::new(tls_config));

                let stream: TcpStream = TcpStream::connect(&addr).await.expect("connection error");

                let domain =
                    tokio_rustls::rustls::ServerName::try_from(req.uri().host().unwrap()).unwrap();

                let stream = connector.connect(domain, stream).await.unwrap();
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

                sender.send_request(req).await?
            }
            _ => {
                let stream = TcpStream::connect(&addr).await.expect("connection error");
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

                sender.send_request(req).await?
            }
        };

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

type Acceptor = tokio_rustls::TlsAcceptor;

fn tls_acceptor() -> Acceptor {
    let cert = File::open("/etc/letsencrypt/live/tadash.io/fullchain.pem").unwrap();
    let key = File::open("/etc/letsencrypt/live/tadash.io/privkey.pem").unwrap();
    let cert = &mut std::io::BufReader::new(&cert);
    let key = &mut std::io::BufReader::new(&key);

    let cert = rustls_pemfile::certs(cert).unwrap().concat();
    let key = rustls_pemfile::pkcs8_private_keys(key).unwrap().concat();

    Arc::new(
        ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![Certificate(cert)], PrivateKey(key))
            .unwrap(),
    )
    .into()
}
