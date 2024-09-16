use std::sync::Arc;
use anyhow::Result;
use simple_logger::SimpleLogger;
use crate::state::AppState;

mod state;
mod config;
mod session;
mod session_manager;
mod paths;
mod socket_client;
mod socket_manager;
mod socket_server;
mod command;
mod response;
mod command_handler;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    SimpleLogger::new().init()?;

    // Create app state
    let app_state = AppState::new().await;

    // Start the server
    app_state.socket_server.start(Arc::clone(&app_state)).await?;

    return Ok(());
}
