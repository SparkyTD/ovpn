use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::config::ConfigManager;
use crate::session::Session;
use crate::session_manager::SessionManager;
use crate::socket_manager::SocketManager;
use crate::socket_server::SocketServer;

pub struct AppState {
    pub active_session: Arc<RwLock<Option<Session>>>,
    pub config_manager: Arc<RwLock<ConfigManager>>,
    pub session_manager: SessionManager,
    pub socket_manager: Arc<Mutex<SocketManager>>,
    pub socket_server: SocketServer,
}

impl AppState {
    pub async fn new() -> Arc<AppState> {
        Arc::new(Self {
            active_session: Arc::new(RwLock::new(None)),
            config_manager: Arc::new(RwLock::new(ConfigManager::new().await.expect("Failed to load the config manager"))),
            session_manager: SessionManager::new(),
            socket_manager: Arc::new(Mutex::new(SocketManager::new())),
            socket_server: SocketServer::new(),
        })
    }

    pub async fn has_active_session(&self) -> bool {
        match self.active_session.read().await.as_ref() {
            Some(_) => true,
            None => false,
        }
    }
}

