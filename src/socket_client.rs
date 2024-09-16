use tokio::io::AsyncWriteExt;
use tokio::net::unix::OwnedWriteHalf;
use crate::response::Response;
use crate::session::SerializableSession;

pub struct SocketClient {
    pub id: u64,
    pub writer: OwnedWriteHalf
}

impl SocketClient {
    pub async fn send_response(&mut self, response: Response) -> anyhow::Result<()> {
        self.writer.write_all((response.to_string() + "\n").as_bytes()).await.map_err(anyhow::Error::from)
    }

    pub async fn send_status_update(&mut self, session: &SerializableSession) -> anyhow::Result<()> {
        let message = format!("{}:{}:{:?}", session.config.guid, session.config.name, session.status);
        let message = format!("!{}:{}\n", message.len(), message);
        self.writer.write_all(message.as_bytes()).await.map_err(anyhow::Error::from)
    }
}