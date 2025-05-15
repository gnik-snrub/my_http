use std::net::TcpListener;
mod connection;
use connection::handle_client;
mod parser;
mod response;

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7878");

    match listener {
        Ok(l) => {
            println!("Listening on 127.0.0.1:7878");
            for stream in l.incoming() {
                match stream {
                    Ok(mut s) => {
                        handle_client(&mut s);
                    },
                    Err(e) => {
                        println!("Error in stream: {:?}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("Error binding listener: {:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
