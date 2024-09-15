use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::config::ConfigManager;
use crate::session::Session;
use crate::session_manager::SessionManager;
use crate::socket_manager::SocketManager;
use crate::socket_server::SocketServer;

pub struct AppState {
    pub active_session: Arc<RwLock<Option<Session>>>,
    pub config_manager: ConfigManager,
    pub session_manager: SessionManager,
    pub socket_manager: Arc<Mutex<SocketManager>>,
    pub socket_server: SocketServer,
}

impl AppState {
    pub fn new() -> Arc<AppState> {
        Arc::new(Self {
            active_session: Arc::new(RwLock::new(None)),
            config_manager: ConfigManager::new(),
            session_manager: SessionManager::new(),
            socket_manager: Arc::new(Mutex::new(SocketManager::new())),
            socket_server: SocketServer::new(),
        })
    }
}

