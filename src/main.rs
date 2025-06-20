use http::middleware::{add_header::AddHeader, auth::Auth, Dispatcher, logger::Logger, timer::Timer};
use pool::thread_pool::ThreadPool;
use tokio::{net::TcpListener, runtime};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer };
use tracing_appender::rolling;

mod core;
mod http;
mod handlers;
mod pool;

use core::connection::handle_client;
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

    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Listening on 127.0.0.1:7878");

    let runtime = runtime::Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let threadpool = ThreadPool::new(10, handle);

    let mut dispatcher = Dispatcher::new();
    dispatcher.add(Timer);
    dispatcher.add(AddHeader);
    let logger = Logger::new();
    dispatcher.add(logger);
    dispatcher.add(Auth);

    let dispatcher_arc = Arc::new(dispatcher);
    while let Ok((socket, _addr)) = listener.accept().await {
        let dispatcher_clone = dispatcher_arc.clone();
        threadpool.enqueue(move || async move {
            handle_client(socket, dispatcher_clone).await;
        });
        
    }

    Ok(())
}
