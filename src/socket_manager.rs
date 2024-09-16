use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::{Mutex, RwLock};
use anyhow::Result;
use crate::session::SerializableSession;
use crate::socket_client::SocketClient;

pub struct SocketManager {
    active_clients: RwLock<HashMap<u64, Arc<Mutex<SocketClient>>>>,
    last_client_id: u64,
}

impl SocketManager {
    pub fn new() -> SocketManager {
        Self {
            active_clients: RwLock::new(HashMap::new()),
            last_client_id: 0,
        }
    }

    pub async fn add_client(&mut self, writer: OwnedWriteHalf) -> Arc<Mutex<SocketClient>> {
        let client_id = self.last_client_id + 1;
        let socket_client = Arc::new(Mutex::new(SocketClient {
            writer,
            id: client_id,
        }));

        let mut active_clients = self.active_clients.write().await;
        active_clients.insert(client_id, socket_client.clone());
        self.last_client_id = client_id;

        return socket_client;
    }

    pub async fn remove_client(&mut self, client_id: u64) {
        let mut active_clients = self.active_clients.write().await;
        active_clients.remove(&client_id);
    }

    pub async fn broadcast_status_change(&mut self, session: &SerializableSession) -> Result<()> {
        let active_clients = self.active_clients.read().await;
        for client in active_clients.values() {
            let mut client = client.lock().await;
            client.send_status_update(session).await
                .expect("Failed to send status update");
        }

        Ok(())
    }
}