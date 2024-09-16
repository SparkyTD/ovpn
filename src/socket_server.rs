use anyhow::Result;
use std::sync::Arc;
use clap::{Parser};
use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use crate::command::Cli;
use crate::command_handler::CommandHandler;
use crate::paths::SOCKET_PATH;
use crate::response::Response;
use crate::state::AppState;

pub struct SocketServer {
}

impl SocketServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(&self, app_state: Arc<AppState>) -> Result<()> {
        let _ = std::fs::remove_file(SOCKET_PATH);

        let listener = UnixListener::bind(SOCKET_PATH)?;

        info!("Socket listener started, listening on {}", SOCKET_PATH);

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let app_state = Arc::clone(&app_state);
                    tokio::spawn(SocketServer::handle_client(stream, app_state));
                },
                _ = tokio::signal::ctrl_c() => {
                    info!("Ctrl+C received, shutting down");
                    break;
                }
            }
        }

        return Ok(());
    }

    async fn handle_client(stream: UnixStream, app_state: Arc<AppState>) {
        info!("New socket connection");

        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        let mut socket_manager = app_state.socket_manager.lock().await;
        let client = socket_manager.add_client(writer).await;
        drop(socket_manager);

        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    let command = line.trim_end_matches('\n').trim_end_matches('\r');
                    info!("Received command: {}", command);

                    let mut args = command.split_whitespace().collect::<Vec<&str>>();
                    args.insert(0, "ovpn");

                    let response: Option<Response> = match Cli::try_parse_from(args) {
                        Ok(command) => {
                            let app_state = app_state.clone();
                            CommandHandler::handle_command(command, app_state).await.unwrap_or_else(|_| None)
                        },
                        Err(_) => Some(Response::fail("Invalid command".to_string()))
                    };

                    if let Some(response) = response {
                        let mut client = client.lock().await;
                        if let Err(err) = client.send_response(response).await {
                            error!("Failed to send response: {}", err);
                            break;
                        }
                    }
                }
                Err(_) => {
                    error!("Error while reading from socket");
                    break;
                }
            }
        }

        let mut socket_manager = app_state.socket_manager.lock().await;
        let client = client.lock().await;
        socket_manager.remove_client(client.id).await;
        drop(socket_manager);
    }
}