use super::*;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use tokio_rustls::{TlsAcceptor, rustls, TlsConnector};
use rustls::{ServerConfig, RootCertStore, pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName}};
use rcgen::{generate_simple_self_signed};
use std::sync::Arc;

fn generate_tls_config() -> (ServerConfig, rustls::ClientConfig) {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();

    let cert_der = CertificateDer::from(cert.cert);
    let private_der = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der()).into();

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der.clone()], private_der)
        .unwrap();

    let mut root_store = RootCertStore::empty();
    root_store.add(cert_der).unwrap();

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    (server_config, client_config)
}

#[tokio::test]
async fn handle_client_processes_basic_get_request() {
    let (server_config, client_config) = generate_tls_config();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server task
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(server_config));
        let tls_stream = acceptor.accept(tcp).await.unwrap();

        let dispatcher = Arc::new(Dispatcher::new());
        handle_client(tls_stream, dispatcher).await;
    });

    // Simulate a client
    let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();

    let connector = TlsConnector::from(Arc::new(client_config));
    let domain = ServerName::try_from("localhost").unwrap();
    let mut stream = connector.connect(domain, tcp).await.unwrap();

    // Write a simple GET request
    let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request).await.unwrap();
    stream.shutdown().await.unwrap();

    // Give server a moment to respond
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();

    let response = String::from_utf8_lossy(&buf[..n]);
    assert!(response.starts_with("HTTP/1.1 200"));
}
