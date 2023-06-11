use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::ControlFlow;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;
use web_server::ThreadPool;

fn handle_client_sse(mut stream: TcpStream, rx: Arc<Mutex<mpsc::Receiver<String>>>) {
    let buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n");

    if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, response) {
        return;
    }

    loop {
        std::thread::sleep(Duration::from_secs(1));

        let received_message = rx.lock().unwrap().recv();

        match received_message {
            Ok(data) => {
                let sse_event = format!("data: {}\n\n", data);

                if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, sse_event) {
                    break;
                }
            }
            Err(_) => {
                let last_sse_event = format!("{}\r\n\r\n", 0);

                if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, last_sse_event) {
                    break;
                }
                println!("Producer might have closed the channel");
                break;
            }
        }
    }
}

// generate random data and each new connection will read any random data generated from this producer
// since we are using mpsc (i.e., multiple producer single consumer) channel
// each active connection will get any random data produced by producer and not the copy of data
// will be available for consumption by all the active connections
fn start_message_passing_thread(tx: Sender<String>) {
    thread::spawn(move || {
        // Simulating the server sending messages
        let mut message = 1;

        loop {
            // Send the message through the channel
            tx.send(message.to_string())
                .expect("Failed to send message");

            // Wait for a short interval between messages
            thread::sleep(Duration::from_secs(1));
            message = message + 1;
        }
    });
}

fn handle_client_chunk(mut stream: TcpStream, rx: Arc<Mutex<mpsc::Receiver<String>>>) {
    let buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    /*
    X-Content-Type-Options: nosniff is provided so that borwser will respect content type provided by the server and disable MIME type sniffing on the browser
    this is because browser can discard content type provided by server and determine content type based on the actual content in the response

    this is especially useful if we want the browser to streaming response immediately because for chunked response browser will buffer some response so that it
    perform MIME sniffing and hence client won't be able to consume the chunk immediately to prevent this we instruct browser/client to disable MIME sniffing

    Content-Type: text/event-stream will also disable buffering on client side this is because all the browsers implements SSE

    See: https://stackoverflow.com/questions/13557900/chunked-transfer-encoding-browser-behavior/56089065#56089065
    */

    let data = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nX-Content-Type-Options: nosniff\r\n\r\n"
    );

    if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, data) {
        return;
    }

    loop {
        thread::sleep(Duration::from_secs(1));

        let received_message = rx.lock().unwrap().recv();

        match received_message {
            Ok(data) => {
                println!("Received message: {}", data);

                let chunk = format!("{}\r\n{}\r\n", data.len(), data);

                if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, chunk) {
                    break;
                }
            }
            Err(_) => {
                let last_chunk = format!("{}\r\n\r\n", 0);

                if let ControlFlow::Break(_) = write_data_to_stream(&mut stream, last_chunk) {
                    break;
                }

                println!("Producer might have closed the channel");
                break;
            }
        }
    }
}

fn write_data_to_stream(stream: &mut TcpStream, data: String) -> ControlFlow<()> {
    let write_status = stream.write_all(data.as_bytes());
    match write_status {
        Ok(_) => {
            match stream.flush() {
                Ok(_) => (),
                Err(e) => {
                    println!("error flushing the stream, connection may be broken: {}", e);
                    return ControlFlow::Break(());
                }
            };
        }
        Err(e) => {
            println!(
                "failed to write to connection, connection may be broken: {}",
                e
            );
            return ControlFlow::Break(());
        }
    }

    ControlFlow::Continue(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(2);

    let (tx, rx) = mpsc::channel::<String>();

    start_message_passing_thread(tx);

    let rx = Arc::new(Mutex::new(rx));

    for stream in listener.incoming() {
        let rx = rx.clone();
        match stream {
            Ok(stream) => {
                // spawn a thread for each incoming request
                pool.execute(move || {
                    // handle_client_sse(stream, rx);
                    handle_client_chunk(stream, rx);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
