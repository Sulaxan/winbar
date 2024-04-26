use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateWindow,
    Shutdown,
}

#[derive(Serialize, Deserialize)]
pub struct WinbarServerPayload {
    pub id: u32,
    pub message: ServerMessage,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    /// Denotes that the sent message was successful
    Success,
    /// Denotes that the sent message was not successful
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct WinbarClientPayload {
    pub id: u32,
    pub message: ClientMessage,
}
