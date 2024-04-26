use anyhow::Result;
use getset::Getters;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use winbar::{
    protocol::{WinbarClientPayload, WinbarServerPayload},
    WinbarAction, WinbarContext,
};

#[derive(Getters)]
pub struct Connection {
    #[getset(get = "pub")]
    handle: JoinHandle<()>,
}

pub struct WinbarServer {
    listener: TcpListener,
    connections: Vec<Connection>,
    ctx: WinbarContext,
}

impl WinbarServer {
    pub async fn new(addr: &str, ctx: WinbarContext) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr.to_string()).await?,
            connections: Vec::new(),
            ctx,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            let (mut stream, _) = self.listener.accept().await?;
            let ctx = self.ctx.clone();

            let handle = tokio::spawn(async move {
                let mut buf = vec![0; 4096];

                loop {
                    let n = stream
                        .read(&mut buf)
                        .await
                        .expect("could not read from stream");

                    if n == 0 {
                        return;
                    }

                    let payload = match serde_json::from_slice::<WinbarServerPayload>(&buf[0..n]) {
                        Ok(payload) => payload,
                        Err(e) => {
                            tracing::error!("Error while parsing server payload: {}", e);
                            return;
                        }
                    };
                    match Self::process(&ctx, &payload, &stream) {
                        Ok(_) => {
                            let client_payload = WinbarClientPayload {
                                id: payload.id,
                                message: winbar::protocol::ClientMessage::Success,
                            };
                            Self::serialize_and_send(&mut stream, &client_payload).await;
                        }
                        Err(e) => {
                            tracing::error!("Error while processing server payload: {}", e);
                            let client_payload = WinbarClientPayload {
                                id: payload.id,
                                message: winbar::protocol::ClientMessage::Error(e.to_string()),
                            };
                            Self::serialize_and_send(&mut stream, &client_payload).await;
                        }
                    }
                }
            });

            self.connections.push(Connection { handle })
        }
    }

    pub fn stop(&mut self) {
        // dropping the handle should shut down the tcp stream
        self.connections
            .drain(0..)
            .for_each(|conn| conn.handle.abort());
    }

    fn process(
        ctx: &WinbarContext,
        payload: &WinbarServerPayload,
        _stream: &TcpStream,
    ) -> Result<()> {
        match payload.message {
            winbar::protocol::ServerMessage::UpdateWindow => {
                ctx.sender().send(WinbarAction::UpdateWindow)?;
            }
            winbar::protocol::ServerMessage::Shutdown => {
                ctx.sender().send(WinbarAction::Shutdown)?;
            }
        }

        Ok(())
    }

    async fn serialize_and_send(stream: &mut TcpStream, payload: &WinbarClientPayload) {
        match serde_json::to_vec(payload) {
            Ok(serialized) => {
                if let Err(e) = stream.write_all(&serialized).await {
                    tracing::error!("Error while sending payload: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Error while serializing payload: {}", e);
            }
        }
    }
}
