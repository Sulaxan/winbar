use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateWindow,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinbarServerPayload {
    pub id: u32,
    pub message: ServerMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Denotes that the sent message was successful
    Success,
    /// Denotes that the sent message was not successful
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinbarClientPayload {
    pub id: u32,
    pub message: ClientMessage,
}
