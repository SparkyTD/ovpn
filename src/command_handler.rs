use std::ops::Deref;
use std::sync::Arc;
use crate::command::{Cli, Commands, ConfigCommand, SessionCommand};
use crate::response::Response;
use crate::session::Session;
use crate::state::AppState;

pub struct CommandHandler {}

impl CommandHandler {
    pub async fn handle_command(command: Cli, app_state: Arc<AppState>) -> anyhow::Result<Option<Response>> {
        return match command.commands {
            Commands::Config { commands } => match commands {
                ConfigCommand::List => {
                    let config_index = app_state.config_manager.get_index();
                    let entries = config_index.get_entries();
                    Ok(Some(Response::success(serde_json::to_string_pretty(&entries)?)))
                }
                ConfigCommand::Import { .. } => Ok(Some(Response::success("Not implemented".to_string()))),
                ConfigCommand::Export { .. } => Ok(Some(Response::success("Not implemented".to_string()))),
                ConfigCommand::Delete { .. } => Ok(Some(Response::success("Not implemented".to_string()))),
            },
            Commands::Session { commands } => match commands {
                SessionCommand::Start { name } => {
                    if let Some(_) = app_state.active_session.read().await.as_ref() {
                        return Ok(Some(Response::fail("Cannot start new session because another one is already active".to_string())));
                    }

                    match app_state.config_manager.get_by_name(name).await {
                        Ok(config_entry) => {
                            match app_state.session_manager.start(config_entry).await {
                                Ok(session) => {
                                    *app_state.active_session.write().await = Some(session);
                                    Ok(Some(Response::success("Session started successfully".to_string())))
                                }
                                Err(_) => Ok(Some(Response::fail("Failed to start the session".to_string()))),
                            }
                        }
                        Err(_) => Ok(Some(Response::fail("The specified configuration cannot be found".to_string())))
                    }
                }
                SessionCommand::Stop => {
                    if app_state.has_active_session().await {
                        match app_state.session_manager.stop(app_state.clone()).await {
                            Ok(_) => Ok(Some(Response::success("The session was stopped successfully".to_string()))),
                            Err(_) => Ok(Some(Response::fail("Failed to stop the session".to_string())))
                        }
                    } else {
                        Ok(Some(Response::fail("No session is currently active".to_string())))
                    }
                }
                SessionCommand::Status => {
                    let active_session_guard = app_state.active_session.read().await;
                    match active_session_guard.as_ref() {
                        Some(session) => Ok(Some(Response::success(serde_json::to_string_pretty(&session)?))),
                        None => Ok(Some(Response::success("No active sessions".to_string())))
                    }
                }
            }
        };
    }
}