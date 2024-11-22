use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type SharedClients = Arc<Mutex<HashMap<String, TcpStream>>>;

fn handle_client(mut stream: TcpStream, clients: SharedClients, addr: String) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut nickname = format!("User{}", addr.replace('.', "").replace(':', ""));
    let mut registered = false;

    writeln!(stream, ":server NOTICE * :Welcome to the IRC server!").ok();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // Client disconnected
                stream.shutdown(std::net::Shutdown::Both).ok();
                let mut clients_lock = clients.lock().unwrap();
                for (nick, client) in clients_lock.iter_mut() {
                    if nick != &nickname {
                        writeln!(client, ":{} QUIT :Goodbye!", nickname).ok();
                    }
                }
                clients_lock.remove(&nickname);
                stream.shutdown(std::net::Shutdown::Both).ok();
                let mut clients_lock = clients.lock().unwrap();
                for (nick, client) in clients_lock.iter_mut() {
                    if nick != &nickname {
                        writeln!(client, ":{} QUIT :Goodbye!", nickname).ok();
                    }
                }
                clients_lock.remove(&nickname);
                break;
            }
            Ok(_) => {
                let line = line.trim_end().to_string();

                if line.starts_with("NICK ") {
                    let new_nick = line[5..].to_string();
                    if !new_nick.is_empty() {
                        let mut clients_lock = clients.lock().unwrap();
                        let old_nickname = nickname.clone();
                        clients_lock.remove(&nickname);
                        nickname = new_nick.clone();
                        clients_lock.insert(nickname.clone(), stream.try_clone().unwrap());
                        writeln!(
                            stream,
                            ":server 001 {} :Your nickname is now {}",
                            nickname, nickname
                        )
                        .ok();
                        for (nick, client) in clients_lock.iter_mut() {
                            if nick != &nickname {
                                writeln!(client, ":{} NICK :{}", old_nickname, nickname).ok();
                            }
                        }
                    } else {
                        writeln!(stream, ":server 431 * :No nickname given").ok();
                    }
                } else if line.starts_with("USER ") {
                    if registered {
                        writeln!(stream, ":server 462 * :You are already registered").ok();
                    } else {
                        registered = true;
                        writeln!(
                            stream,
                            ":server 001 {} :Welcome, {}!",
                            nickname, nickname
                        )
                        .ok();
                    }
                } else if line.starts_with("USERHOST ") {
                    writeln!(
                        stream,
                        ":server 302 {} :{}=+{}",
                        nickname, nickname, addr
                    )
                    .ok();
                } else if line.starts_with("JOIN ") {
                    let channel = line[5..].to_string();
                    if !channel.is_empty() && channel == "#Main" {
                        writeln!(stream, ":{} JOIN {}", nickname, channel).ok();
                        let mut clients_lock = clients.lock().unwrap();
                        for (nick, client) in clients_lock.iter_mut() {
                            if nick != &nickname {
                                writeln!(client, ":{} JOIN {}", nickname, channel).ok();
                            }
                        }
                    } else {
                        writeln!(stream, ":server 403 {} :No such channel", nickname).ok();
                    }
                } else if line.starts_with("PRIVMSG #Main :") {
                    let message = line[14..].to_string();
                    if !message.is_empty() {
                        let mut clients_lock = clients.lock().unwrap();
                        for (nick, client) in clients_lock.iter_mut() {
                            // Prevent sending the message back to the sender
                            if nick != &nickname {
                                writeln!(client, ":{} PRIVMSG #Main :{}", nickname, message).ok();
                            }
                        }
                    } else {
                        writeln!(stream, ":server 412 {} :No text to send", nickname).ok();
                    }
                } else if line.starts_with("QUIT") {
                    writeln!(stream, ":{} QUIT :Goodbye!", nickname).ok();
                    break;
                } else if line.starts_with("CAP ") {
                    // Ignore unsupported CAP commands
                    continue;
                } else {
                    // Unknown command, include the invalid command in the response
                    let unknown_command = line.split_whitespace().next().unwrap_or("Unknown");
                    writeln!(
                        stream,
                        ":server 421 {} :Unknown command ({})",
                        nickname, unknown_command
                    )
                    .ok();
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    // Remove client from the shared map when they disconnect
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
