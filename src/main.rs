#![allow(non_snake_case)]
use std::io::{self, BufRead, Write, Read}; // Add `Read` here
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::broadcast;


#[derive(Debug)]
struct Client {
    nickname: String,
    stream: TcpStream,
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6667")?;
    let clients = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = broadcast::channel::<String>(100);

    println!("IRC server running on 127.0.0.1:6667");
    let clients_clone = Arc::clone(&clients);
    // Main server shutdown listener
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap_or_else(|_| "".to_string());
            if line.trim() == "/exit" {
                println!("Shutting down server...");
                std::process::exit(0);
            }
        }
    });

    // Accept incoming client connections
    for stream in listener.incoming() {
        let stream = stream?;
        let clients_clone = Arc::clone(&clients_clone);
        let tx_clone = tx.clone();

        thread::spawn(move || {
            handle_client(stream, clients_clone, tx_clone);
        });
    }

    Ok(())
}

fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>, tx: broadcast::Sender<String>) {
    let mut stream = stream;
    let mut username = String::new();
    let mut nickname = String::new();
    let mut buffer = [0; 1024];

    // Greeting and username setup
    write!(stream, "Welcome to #Main channel!\r\nEnter your username: ").unwrap();
    stream.flush().unwrap();

    // Reading initial message (NICK, USER, etc.)
    match stream.read(&mut buffer) {
        Ok(size) => {
            let input = String::from_utf8_lossy(&buffer[..size]).trim().to_string();

            // Handle NICK and USER commands
            if input.starts_with("NICK") {
                nickname = input.split_whitespace().nth(1).unwrap_or("").to_string();
                write!(stream, "NICK set to: {}\r\n", nickname).unwrap();
            }

            if input.starts_with("USER") {
                username = input.split_whitespace().nth(1).unwrap_or("").to_string();
                write!(stream, "USER set to: {}\r\n", username).unwrap();
            }

            // Ensure both NICK and USER are set
            if !nickname.is_empty() && !username.is_empty() {
                let welcome_message = format!("{} has joined #Main.", nickname);
                tx.send(welcome_message.clone()).unwrap_or_else(|_| 0);

                // Add client to list (no need to store the stream here anymore)
                let client = Client {
                    nickname: nickname.clone(),
                    stream: stream.try_clone().unwrap(),
                };
                clients.lock().unwrap().push(client);

                write!(stream, "Welcome, {}! Type your messages below.\r\n", username).unwrap();
                stream.flush().unwrap();

                // Broadcast join
                println!("{}", welcome_message);
            } else {
                eprintln!("Failed to set NICK/USER correctly.");
                return; // Disconnect if NICK or USER not set properly
            }
        }
        Err(_) => return,
    }

    // Main client interaction loop
    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let input = String::from_utf8_lossy(&buffer[..size]).trim().to_string();

                if input.starts_with("/quit") {
                    let leave_message = format!("{} has left #Main.", nickname);
                    tx.send(leave_message.clone()).unwrap_or_else(|_| 0);
                    println!("{}", leave_message);
                    break; // Client is quitting, exit loop
                }

                let message = format!("{}: {}", nickname, input);
                tx.send(message.clone()).unwrap_or_else(|_| 0);
                println!("{}", message);

/*
FIGURE OUT if direct messaging is allowed...
// Later in the code, you could use client.stream to send messages directly to the client
// Example: Send a message to a specific client:
write!(client.stream, "This is a message just for you!\r\n").unwrap();
client.stream.flush().unwrap()
*/

            }
            Ok(_) | Err(_) => break, // Handle EOF or error gracefully
        }
    }

    // Handle disconnection and client removal
    println!("{} disconnected.", nickname);

    // Remove client on disconnect
    clients.lock().unwrap().retain(|c| c.nickname != nickname);
}
