use std::{io::{Read, Write}, net::TcpStream};

use crate::parser::{generate_body, generate_headers, parse_request};
use crate::response::{Response, StatusCode};
use crate::router::router;

pub fn handle_client(stream: &mut TcpStream) {
    println!("Hello world");
    println!("Connected stream: {:?}", stream);

    let mut master_buffer: Vec<u8> = vec![];
    let mut scratch: [u8; 512] = [0u8; 512];

    collect_stream(stream, &mut scratch, &mut master_buffer);

    let parse_result = parse_request(&master_buffer);
    let mut response = match parse_result {
        Ok((mut idx, mut req)) => {
            req.headers = generate_headers(&mut master_buffer, &mut idx);
            req.body = generate_body(req.headers.get("Content-Length"), &mut master_buffer, idx);

            router(req)
        }
        Err(_) => {
            Response::new().status(StatusCode::BadRequest)
        }
    };
    send_response(stream, response.finalize().to_vec());
}

fn collect_stream(stream: &mut TcpStream, scratch: &mut [u8; 512], master_buffer: &mut Vec<u8>) {
    loop {
        if master_buffer.len() >= 8000 {
            break;
        }
        let bytes_read = stream.read(scratch);
        match bytes_read {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                master_buffer.extend_from_slice(&scratch[..n]);
            },
            Err(e) => {
                println!("Error reading stream data: {:?}", e);
            }
        }
        for window in master_buffer.windows(4) {
            if window == [13, 10, 13, 10] {
                return;
            }
        }
    }
}

fn send_response(stream: &mut TcpStream, res_bytes: Vec<u8>) {
    let result = stream.write_all(&res_bytes);
    match result {
        Ok(_) => {
            println!("Response sent...");
        }
        Err(e) => {
            println!("Error sending response: {:?}", e);
        }
    }
    let _ = stream.flush();
}
