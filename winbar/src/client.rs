use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Receiver,
    task::JoinHandle,
};

use crate::protocol::{WinbarClientPayload, WinbarServerPayload};

pub struct WinbarClient {
    response_handler: Arc<dyn Fn(WinbarClientPayload) + Sync + Send>,
    stream_handle: Option<JoinHandle<()>>,
}

impl WinbarClient {
    pub fn new(response_handler: Arc<dyn Fn(WinbarClientPayload) + Sync + Send>) -> Self {
        Self {
            response_handler,
            stream_handle: None,
        }
    }

    pub async fn start(
        &mut self,
        addr: &str,
        mut recv: Receiver<WinbarServerPayload>,
    ) -> Result<()> {
        let mut stream = TcpStream::connect(addr).await?;
        let response_handler = self.response_handler.clone();

        self.stream_handle = Some(tokio::spawn(async move {
            let mut buf = vec![0; 4096];
            let (mut rx, mut wx) = stream.split();

            loop {
                tokio::select! {
                    read_bytes = rx.read(&mut buf) => {
                        let read_bytes = read_bytes.unwrap();
                        let payload = serde_json::from_slice::<WinbarClientPayload>(&buf[..read_bytes]).unwrap();
                        response_handler(payload);
                    }
                    payload = recv.recv() => {
                        let payload = payload.unwrap();
                        wx.write_all(&serde_json::to_vec(&payload).unwrap()).await.unwrap();
                    }
                }
            }
        }));

        Ok(())
    }
}
