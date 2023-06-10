use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
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
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    loop {
        std::thread::sleep(Duration::from_secs(1));

        let received_message = rx.lock().unwrap().recv();

        match received_message {
            Ok(data) => {
                let sse_event = format!("data: {}\n\n", data);

                let write_status = stream.write_all(sse_event.as_bytes());

                match write_status {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "failed to write to connection, connection may be broken: {}",
                            e
                        );
                        break;
                    }
                }

                stream.flush().unwrap();
            }
            Err(_) => {
                stream.write(format!("{}\r\n\r\n", 0).as_bytes()).unwrap();
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

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\n\r\n"
    );

    let write_status = stream.write(response.as_bytes());

    match write_status {
        Ok(_) => (),
        Err(e) => {
            println!(
                "failed to write to connection, connection may be broken: {}",
                e
            );
            return;
        }
    }

    loop {
        thread::sleep(Duration::from_secs(1));

        let received_message = rx.lock().unwrap().recv();

        match received_message {
            Ok(data) => {
                println!("Received message: {}", data);

                let write_status =
                    stream.write(format!("{}\r\n{}\r\n", data.len(), data).as_bytes());

                match write_status {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "failed to write to connection, connection may be broken: {}",
                            e
                        );
                        break;
                    }
                }
                stream.flush().unwrap();
            }
            Err(_) => {
                stream.write(format!("{}\r\n\r\n", 0).as_bytes()).unwrap();
                println!("Producer might have closed the channel");
                break;
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(1);

    let (tx, rx) = mpsc::channel::<String>();

    start_message_passing_thread(tx);

    let rx = Arc::new(Mutex::new(rx));

    for stream in listener.incoming() {
        let rx = rx.clone();
        match stream {
            Ok(stream) => {
                // spawn a thread for each incoming request
                pool.execute(move || {
                    handle_client_sse(stream, rx);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
