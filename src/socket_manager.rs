use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::Mutex;
use crate::socket_client::SocketClient;

pub struct SocketManager {
    active_clients: HashMap<u64, Arc<Mutex<SocketClient>>>,
    last_client_id: u64,
}

impl SocketManager {
    pub fn new() -> SocketManager {
        Self {
            active_clients: HashMap::new(),
            last_client_id: 0,
        }
    }

    pub fn add_client(&mut self, writer: OwnedWriteHalf) -> Arc<Mutex<SocketClient>> {
        let client_id = self.last_client_id + 1;
        let socket_client = Arc::new(Mutex::new(SocketClient{
            id: client_id,
            writer
        }));

        self.active_clients.insert(client_id, socket_client.clone());
        self.last_client_id = client_id;

        return socket_client;
    }
}