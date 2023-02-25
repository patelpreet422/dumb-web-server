use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let response = format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n");
    
    stream.write(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    loop {
        let (stream, _) = listener.accept().unwrap();
        handle_client(stream);
    }
    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(stream) => {
    //             std::thread::spawn(|| {
    //                 handle_client(stream);
    //             });
    //         }
    //         Err(e) => {
    //             println!("Error: {}", e);
    //         }
    //     }
    // }
}
