use bytes::BytesMut;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::http::middleware::{AddHeader, Auth, Dispatcher, Logger, Timer};
use crate::core::{
    parser::{generate_body, generate_headers, parse_request},
    response::{Response, StatusCode},
    router::router,
};

pub async fn handle_client(mut socket: TcpStream) {

    let mut master_buffer = BytesMut::new();

    collect_socket(&mut socket, &mut master_buffer).await;

    let parse_result = parse_request(&master_buffer);
    let mut response = match parse_result {
        Ok((mut idx, mut req)) => {
            req.headers = generate_headers(&mut master_buffer, &mut idx);
            req.body = generate_body(req.headers.get("Content-Length"), &mut master_buffer, idx);

            let mut dispatcher = Dispatcher::new();
            dispatcher.add(Timer);
            dispatcher.add(AddHeader);
            dispatcher.add(Logger);
            dispatcher.add(Auth);

            let mut mw_response = dispatcher.dispatch(req.clone()).await;

            if mw_response.status == StatusCode::Unauthorized {
                mw_response.body = Vec::from(b"401 Unauthorized");
                mw_response
            } else {
                router(req, mw_response).await
            }
        }
        Err(_) => {
            Response::new().status(StatusCode::BadRequest)
        }
    };
    send_response(socket, response.finalize().to_vec()).await;
}

async fn collect_socket(socket: &mut TcpStream, master_buffer: &mut BytesMut) {
    loop {
        if master_buffer.len() >= 8000 {
            break;
        }
        let bytes_read = socket.read_buf(master_buffer).await;
        match bytes_read {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            },
            Err(e) => {
                println!("Error reading socket data: {:?}", e);
            }
        }
        for window in master_buffer.windows(4) {
            if window == [13, 10, 13, 10] {
                return;
            }
        }
    }
}

async fn send_response(mut socket: TcpStream, res_bytes: Vec<u8>) {
    let _ = socket.writable().await;
    let result = socket.try_write(&res_bytes);
    match result {
        Ok(_) => {
            // No errors
        }
        Err(e) => {
            println!("Error sending response: {:?}", e);
        }
    }
    let _ = socket.flush();
    let _ = socket.shutdown().await;
}
