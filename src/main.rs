use tokio::net::TcpListener;
mod connection;
use connection::handle_client;
mod parser;
mod router;
mod response;

#[tokio::main] async fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Listening on 127.0.0.1:7878");

    while let Ok((socket, _addr)) = listener.accept().await {
        tokio::spawn(handle_client(socket));
    }

    Ok(())
}
