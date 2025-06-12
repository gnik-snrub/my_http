use pool::thread_pool::ThreadPool;
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer };
use tracing_appender::rolling;

mod core;
mod http;
mod handlers;
mod pool;

use core::connection::handle_client;

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

    let threadpool = ThreadPool::new(10);

    while let Ok((socket, _addr)) = listener.accept().await {
        //tokio::spawn(handle_client(socket));

        // The custom threadpool, at this stage, is about 30x slower than tokio
        // But, nonetheless, it is a functional custom threadpool!
        threadpool.enqueue(Box::new(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_client(socket));
        }));
        
    }

    Ok(())
}
