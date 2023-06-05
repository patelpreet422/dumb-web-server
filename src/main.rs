use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, self};
use std::thread::{self};
use std::time::Duration;
use web_server::ThreadPool;

// generate random data and each new connection will read any random data generated from this producer
// since we are using mpsc (i.e., multiple producer single consumer) channel
// each active connection will get any random data produced by producer and not the copy of data
// will be available for consumption by all the active connections
fn start_message_passing_thread(tx: Sender<String>) {
    thread::spawn(move || {
        // Simulating the server sending messages
        let messages = vec![
            "Hello",
            "This",
            "Is",
            "A",
            "Message",
            "From",
            "The",
            "Server",
        ];

        loop {
            for message in &messages {
                // Send the message through the channel
                tx.send(message.to_string()).expect("Failed to send message");

                // Wait for a short interval between messages
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

fn handle_client(mut stream: TcpStream, rx: Arc<Mutex<mpsc::Receiver<String>>>) {
    
    let buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);
    
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\n\r\n");
    stream.write(response.as_bytes());

    loop {
        thread::sleep(Duration::from_secs(1));

        let received_message = rx.lock().unwrap().recv();

        match received_message {
            Ok(data) => {
                println!("Received message: {}", data);
                stream.write(format!("{}\r\n{}\r\n", data.len(), data).as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                stream.write(format!("{}\r\n\r\n", 0).as_bytes()).unwrap();
                println!("Producer might have closed the channel");
                break;
            }
        }
    }

    stream.flush().unwrap();
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
                    handle_client(stream, rx);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
