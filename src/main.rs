use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;

type SharedState = Arc<Mutex<broadcast::Sender<String>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6667").await?;
    println!("IRC server running on 127.0.0.1:6667...");

    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(Mutex::new(tx));

    loop {
        let (socket, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, state).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(
    mut socket: TcpStream,
    state: SharedState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];
    let username;
    let tx = state.lock().await.clone();
    let mut rx = tx.subscribe();

    socket.write_all(b"Welcome to #Main channel!\nEnter your username: ").await?;
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    username = String::from_utf8_lossy(&buf[..n]).trim().to_string();
    socket.write_all(format!("Welcome, {}! Type your messages below.\n", username).as_bytes())
        .await?;

    let user_tx = tx.clone();
    let join_message = format!("{} has joined #Main.\n", username);
    user_tx.send(join_message)?;

    loop {
        tokio::select! {
            Ok(n) = socket.read(&mut buf) => {
                if n == 0 {
                    break;
                }
                let message = format!("{}: {}", username, String::from_utf8_lossy(&buf[..n]).trim());
                tx.send(message)?;
            }
            Ok(message) = rx.recv() => {
                socket.write_all(message.as_bytes()).await?;
            }
        }
    }

    let leave_message = format!("{} has left #Main.\n", username);
    user_tx.send(leave_message)?;
    Ok(())
}
