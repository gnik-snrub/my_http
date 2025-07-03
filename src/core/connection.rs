use std::sync::Arc;

use bytes::BytesMut;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

use crate::core::{
    parser::{generate_body, generate_headers, parse_request},
    response::{Response, StatusCode},
    router::router,
};
use crate::http::middleware::Dispatcher;

use super::parser::generate_cookies;

pub async fn handle_client(mut socket: TlsStream<TcpStream>, dispatcher: Arc<Dispatcher>) {

    loop {
        let mut master_buffer = BytesMut::new();

        let early_return = collect_socket(&mut socket, &mut master_buffer).await;

        if early_return {
            break;
        }

        let parse_result = parse_request(&master_buffer);
        let (mut idx, mut req) = if parse_result.is_ok() {
            parse_result.unwrap()
        } else {
            send_response(&mut socket, Response::new()
                .status(StatusCode::BadRequest)
                .finalize().to_vec()).await;
            break;
        };

        if req.headers.get("Connection")
            .map(|v| v.eq_ignore_ascii_case("close"))
                .unwrap_or(false) {
            break;
        }

        req.headers = generate_headers(&mut master_buffer, &mut idx);
        req.cookies = Some(generate_cookies(&req));
        req.body = generate_body(req.headers.get("Content-Length"), &mut master_buffer, idx);

        let mut mw_response = dispatcher.dispatch(req.clone()).await;

        let mut response = if mw_response.status == StatusCode::Unauthorized {
            mw_response.body = Vec::from(b"401 Unauthorized");
            mw_response
        } else {
            router(req, mw_response).await
        };

        send_response(&mut socket, response.finalize().to_vec()).await;
    }
}

async fn collect_socket(socket: &mut TlsStream<TcpStream>, master_buffer: &mut BytesMut) -> bool {
    loop {
        if master_buffer.len() >= 8000 {
            return false;
        }
        let bytes_read = socket.read_buf(master_buffer).await;
        match bytes_read {
            Ok(n) => {
                if n == 0 {
                    return true;
                }
            },
            Err(e) => {
                println!("Error reading socket data: {:?}", e);
            }
        }
        for window in master_buffer.windows(4) {
            if window == [13, 10, 13, 10] {
                return false;
            }
        }
    }
}

async fn send_response(socket: &mut TlsStream<TcpStream>, res_bytes: Vec<u8>) {
    let result = socket.write(&res_bytes).await;
    match result {
        Ok(_) => {
            // No errors
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                return;
            } else {
                eprintln!("Error sending response: {:?}", e);
                return;
            }
        }
    }
    let _ = socket.flush();
    let _ = socket.shutdown().await;
}

#[cfg(test)]
#[path ="tests/connection.rs"]
mod connection_tests;
