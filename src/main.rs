use http::middleware::{logger::Logger, session_tracker::SessionTracker, set_cookie::SetCookie, timer::Timer, Dispatcher};
use pool::thread_pool::ThreadPool;
use tokio::{net::TcpListener, runtime};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer };
use tracing_appender::rolling;

mod core;
mod http;
mod handlers;
mod pool;

use core::{connection::handle_client, tls::load_certs_and_key};
use std::sync::Arc;

#[tokio::main] async fn main() -> std::io::Result<()>{
    let file_appender = rolling::daily("logs", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::Layer::default()
        .json()
        .with_writer(non_blocking)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(file_layer)
        .init();

    dotenvy::dotenv().ok();
    let address = std::env::var("BIND_ADDRESS").expect("BIND_ADDRESS not set");

    println!("Listening on {:?}", &address);
    let listener = TcpListener::bind(address).await?;

    let (tls_cert, tls_key) = load_certs_and_key().unwrap();
    let tls_conf = ServerConfig::builder().with_no_client_auth().with_single_cert(tls_cert, tls_key).unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(tls_conf));

    let runtime = runtime::Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let threadpool = ThreadPool::new(10, handle);

    let mut dispatcher = Dispatcher::new();
    dispatcher.add(Logger::new());
    dispatcher.add(Timer);
    dispatcher.add(SetCookie);
    let sessions = SessionTracker::new();
    dispatcher.add(sessions);

    let dispatcher_arc = Arc::new(dispatcher);

    while let Ok((socket, _addr)) = listener.accept().await {
        let tls_stream = match acceptor.accept(socket).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("TLS Handshake failed: {}", e);
                continue;
            }
        };
        let dispatcher_clone = dispatcher_arc.clone();
        threadpool.enqueue(move || async move {
            handle_client(tls_stream, dispatcher_clone).await;
        });
        
    }

    Ok(())
}
