use tokio::io::AsyncWriteExt;
use tokio::net::unix::OwnedWriteHalf;
use crate::response::Response;

pub struct SocketClient {
    pub id: u64,
    pub writer: OwnedWriteHalf
}

impl SocketClient {
    pub async fn send_response(&mut self, response: Response) -> anyhow::Result<()> {
        self.writer.write_all((response.to_string() + "\n").as_bytes()).await.map_err(anyhow::Error::from)
    }
}