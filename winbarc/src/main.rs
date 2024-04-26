use std::sync::Arc;

use tokio::sync::mpsc;
use winbar::{
    client::WinbarClient,
    protocol::{ServerMessage, WinbarClientPayload, WinbarServerPayload}, DEFAULT_URL,
};

fn handler(payload: WinbarClientPayload) {
    println!("Received payload: {:?}", payload);
}

#[tokio::main]
async fn main() {
    let mut client = WinbarClient::new(Arc::new(handler));
    let (send, recv) = mpsc::channel(100);

    let handle = client.start(DEFAULT_URL, recv).await.unwrap();

    println!("Sending update window payload");
    send.send(WinbarServerPayload {
        id: 100000,
        message: ServerMessage::UpdateWindow,
    })
    .await
    .unwrap();

    handle.await.unwrap();
}
