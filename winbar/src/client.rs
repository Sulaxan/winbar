use anyhow::Result;
use tokio::net::TcpStream;

use crate::protocol::WinbarClientPayload;

pub struct WinbarClient {}

impl WinbarClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(addr: &str) {
        let mut stream = TcpStream::connect(addr).await;
    }

    pub async fn send() -> Result<WinbarClientPayload> {
        // tokio::select! to wait on the payload or error out if we received nothing within a few
        // seconds
        todo!()
    }
}
