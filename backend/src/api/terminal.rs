use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    #[cfg(unix)]
    {
        use portable_pty::{CommandBuilder, PtySize, native_pty_system};
        use std::io::Read;
        
        let pty_system = native_pty_system();
        let pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                let _ = sender.send(Message::Text(format!("Failed to open PTY: {}", e).into())).await;
                return;
            }
        };

        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");

        let mut child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                let _ = sender.send(Message::Text(format!("Failed to spawn shell: {}", e).into())).await;
                return;
            }
        };

        let master = pair.master;
        let mut reader = master.try_clone_reader().unwrap();
        let mut writer = master.take_writer().unwrap();

        // Read from PTY and send to WebSocket
        let read_task = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if sender.send(Message::Binary(buf[..n].to_vec().into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Read from WebSocket and write to PTY
        let write_task = tokio::spawn(async move {
            use std::io::Write;
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if writer.write_all(text.as_bytes()).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        if writer.write_all(&data).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    _ => {}
                }
            }
        });

        tokio::select! {
            _ = read_task => {},
            _ = write_task => {},
        }

        let _ = child.kill();
    }

    #[cfg(not(unix))]
    {
        // Mock terminal for Windows development
        let _ = sender.send(Message::Text("Welcome to Mana Panel Terminal (Mock Mode)\r\n$ ".into())).await;
        
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let response = format!("{}\r\n$ ", text.trim());
                    if sender.send(Message::Text(response.into())).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    let text = String::from_utf8_lossy(&data);
                    let response = format!("{}\r\n$ ", text.trim());
                    if sender.send(Message::Text(response.into())).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    }
}
