use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type SharedClients = Arc<Mutex<HashMap<String, TcpStream>>>;

fn handle_client(mut stream: TcpStream, clients: SharedClients, addr: String) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut nickname = String::new();

    writeln!(stream, ":server NOTICE * :Welcome to the IRC server!").unwrap();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // Client disconnected
                break;
            }
            Ok(_) => {
                let line = line.trim_end().to_string();
                if line.starts_with("NICK ") {
                    nickname = line[5..].to_string();
                    writeln!(stream, ":server 001 {} :Welcome to the IRC server!", nickname).unwrap();
                } else if line.starts_with("JOIN ") {
                    let channel = line[5..].to_string();
                    writeln!(stream, ":{} JOIN {}", nickname, channel).unwrap();
                    let mut clients_lock = clients.lock().unwrap();
                    for (nick, client) in clients_lock.iter_mut() {
                        if nick != &nickname {
                            writeln!(client, ":{} JOIN {}", nickname, channel).unwrap();
                        }
                    }
                } else if line.starts_with("QUIT") {
                    writeln!(stream, ":{} QUIT :Goodbye!", nickname).unwrap();
                    break;
                } else {
                    let mut clients_lock = clients.lock().unwrap();
                    for (nick, client) in clients_lock.iter_mut() {
                        if nick != &nickname {
                            writeln!(client, ":{} PRIVMSG #Main :{}", nickname, line).unwrap();
                        }
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    // Remove client from the shared map
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.remove(&nickname);
    }

    println!("{} disconnected.", addr);
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6667")?;
    println!("Server running on port 6667");

    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap().to_string();
                println!("Client connected: {}", addr);

                let clients_clone = Arc::clone(&clients);
                let stream_clone = stream.try_clone().unwrap();
                let nickname = format!("User{}", addr.replace('.', "").replace(':', ""));

                // Add client to the shared map
                {
                    let mut clients_lock = clients_clone.lock().unwrap();
                    clients_lock.insert(nickname.clone(), stream_clone);
                }

                thread::spawn(move || {
                    handle_client(stream, clients_clone, addr);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
