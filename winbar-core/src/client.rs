use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Receiver,
};

use crate::protocol::{WinbarClientPayload, WinbarServerPayload};

pub struct WinbarClient {
    response_handler: Arc<dyn Fn(WinbarClientPayload) + Sync + Send>,
}

impl WinbarClient {
    pub fn new(response_handler: Arc<dyn Fn(WinbarClientPayload) + Sync + Send>) -> Self {
        Self { response_handler }
    }

    pub async fn start(
        &mut self,
        addr: &str,
        mut recv: Receiver<WinbarServerPayload>,
    ) -> Result<()> {
        let mut stream = TcpStream::connect(addr).await?;
        let response_handler = self.response_handler.clone();

        let mut buf = vec![0; 4096];
        let (mut rx, mut wx) = stream.split();

        loop {
            tokio::select! {
                read_bytes = rx.read(&mut buf) => {
                    let read_bytes = read_bytes?;
                    if read_bytes == 0 {
                        break;
                    }

                    let payload = serde_json::from_slice::<WinbarClientPayload>(&buf[..read_bytes])?;
                    response_handler(payload);
                }
                payload = recv.recv() => {
                    let payload = payload.unwrap();
                    wx.write_all(&serde_json::to_vec(&payload).unwrap()).await?;
                }
            }
        }

        Ok(())
    }
}
