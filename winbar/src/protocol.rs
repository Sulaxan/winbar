//! Winbar client and server protocol.
//!
//! This module defines the concrete types used by clients to communicate with the winbar server.
//! The protocol is a simple and straightforward JSON based request-response protocol. The types
//! defined in this file are serialized directly into JSON, sent over the wire, then deserialized
//! into the Rust types on the other end.
//!
//! # A note on ids
//! The protocol has a notion of ids used by the server and client. These ids are not intended to be
//! globally unique (i.e., across all connections), but rather connection-unique (i.e., essentially
//! unique for the duration of some server-client connection; however, see next paragraph).
//!
//! At this time, since this is a request-response protocol, the generation and usage of ids is
//! solely the client's responsibility. This means that the client is able to use or reuse ids as it
//! sees fit.
use serde::{Deserialize, Serialize};

/// A message sent to the server by the client.
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateWindow,
    Shutdown,
}

/// A server-bound payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct WinbarServerPayload {
    /// The id of the payload.
    ///
    /// The id does not need to be globally unique (i.e., across all connections), only connection
    /// unique.
    ///
    /// See this module's docs for more information on ids.
    pub id: u32,
    /// The server-bound message.
    pub message: ServerMessage,
}

/// A message sent to the client by the server.
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Denotes that the sent message was successful
    Success,
    /// Denotes that the sent message was not successful
    Error(String),
}

/// A client-bound payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct WinbarClientPayload {
    /// The id of the payload.
    ///
    /// The id does not need to be globally unique (i.e., across all connections), only connection
    /// unique.
    ///
    /// See this module's docs for more information on ids.
    pub id: u32,
    /// The client-bound message.
    pub message: ClientMessage,
}
