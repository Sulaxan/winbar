use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use clap::Parser;
use cli::WinbarSubcommand;
use tokio::sync::mpsc;
use winbar::{
    client::WinbarClient,
    protocol::{ClientMessage, ServerMessage, WinbarClientPayload, WinbarServerPayload},
};

use crate::cli::WinbarClientCli;

mod cli;

static QUIET: AtomicBool = AtomicBool::new(false);

macro_rules! log {
    ($($arg:tt)*) => {{
        if !QUIET.load(Ordering::SeqCst) {
            println!($($arg)*);
        }
    }};
}

fn response_handler(payload: WinbarClientPayload) {
    match payload.message {
        ClientMessage::Success => {
            log!("Message was sent and processed successfully!");
        }
        ClientMessage::Error(msg) => {
            log!("Message was sent, but an error occurred: {}", msg);
        }
    }
    std::process::exit(0);
}

#[tokio::main]
async fn main() {
    let cli = WinbarClientCli::parse();
    QUIET.store(cli.quiet, Ordering::SeqCst);

    let mut client = WinbarClient::new(Arc::new(response_handler));
    let (send, recv) = mpsc::channel(10);

    // it's ok to send to the mpsc channel before the winbar tcp server connection is established
    // since what we send is buffered
    match cli.command {
        WinbarSubcommand::Start { config_path, port } => {
            let path = if let Some(path) = config_path.to_str() {
                path
            } else {
                log!("Invalid config path");
                return;
            };

            let arguments = format!("--config-path {} --port {}", path, &port.to_string());
            let script = format!(
                "Start-Process winbar -ArgumentList '{}' -WindowStyle hidden",
                arguments
            );

            match powershell_script::run(&script) {
                Ok(_) => {
                    log!("Started winbar!");
                }
                Err(e) => {
                    log!("Could not start winbar: {}", e);
                    log!("Common solutions:");
                    log!("- Ensure winbar.exe is in your path");
                }
            }

            return;
        }
        WinbarSubcommand::Stop => {
            log!("Sending shutdown payload...");
            send.send(WinbarServerPayload {
                id: 0,
                message: ServerMessage::Shutdown,
            })
            .await
            .unwrap();
        }
        WinbarSubcommand::UpdateWindow => {
            log!("Sending update window payload...");
            send.send(WinbarServerPayload {
                id: 0,
                message: ServerMessage::UpdateWindow,
            })
            .await
            .unwrap();
        }
        WinbarSubcommand::Show => {
            log!("Sending show window payload...");
            send.send(WinbarServerPayload {
                id: 0,
                message: ServerMessage::ShowWindow,
            })
            .await
            .unwrap();
        }
        WinbarSubcommand::Hide => {
            log!("Sending hide window payload...");
            send.send(WinbarServerPayload {
                id: 0,
                message: ServerMessage::HideWindow,
            })
            .await
            .unwrap();
        }
    }

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            log!("Received no response within 5 seconds, shutting down...");
            std::process::exit(0);
        }
        res = client.start(&cli.url, recv) => {
            match res {
                Ok(_) => {},
                Err(e) => {
                    log!("Error occurred in the winbar server connection: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
