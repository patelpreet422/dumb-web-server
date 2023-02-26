use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;
use itertools::{Either, Itertools};

fn handle_client(mut stream: TcpStream) {
    
    let buf_reader = BufReader::new(&stream);

    // this will terminate the tcp-stream since we are reading all the lines in a tcp stream
    // but the tcp stream only closed only when the server returns the response only so the client will keep the
    // tcp-stream open 
    let (http_request, _failures): (Vec<_>, Vec<_>) = buf_reader
        .lines()
        .partition_map(|result| match result {
            Ok(value) => Either::Left(value),
            Err(err) => Either::Right(err)
        });

    println!("Request: {:#?}", http_request);
    let response = format!("HTTP/1.1 200 OK\r\n\r\n");
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    loop {
        let (stream, _) = listener.accept().unwrap();
        thread::spawn(|| {
            handle_client(stream);
        });
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
