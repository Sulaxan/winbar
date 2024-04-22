use std::io;

use tokio::{io::AsyncReadExt, net::TcpListener};

pub struct WinbarServer {
    listener: TcpListener,
}

impl WinbarServer {
    pub async fn new(addr: &str) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr.to_string()).await?,
        })
    }

    pub async fn start(&self) -> io::Result<()> {
        loop {
            let (mut stream, _) = self.listener.accept().await?;

            tokio::spawn(async move {
                let mut buf = vec![0; 4096];
                loop {
                    let n = stream
                        .read(&mut buf)
                        .await
                        .expect("could not read from stream");

                    if n == 0 {
                        return;
                    }

                    // TODO: write to stream...
                }
            });
        }
    }
}
