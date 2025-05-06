use std::{io::Read, net::TcpStream};


pub fn handle_client(stream: &mut TcpStream) {
    println!("Hello world");
    println!("Connected stream: {:?}", stream);

    let mut master_buffer: Vec<u8> = vec![];
    let mut scratch: [u8; 512] = [0u8; 512];

    collect_stream(stream, &mut scratch, &mut master_buffer);

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
        if master_buffer.ends_with(b"/r/n/r/n") {
            break;
        }
    }

}

