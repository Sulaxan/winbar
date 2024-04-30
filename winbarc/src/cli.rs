use std::path::PathBuf;

use clap::{Parser, Subcommand};
use winbar::DEFAULT_PORT;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct WinbarClientCli {
    /// Whether to suppress logging
    #[arg(short, long, action)]
    pub quiet: bool,

    /// Command to run
    #[command(subcommand)]
    pub command: WinbarSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum WinbarSubcommand {
    /// Start winbar
    Start {
        #[arg(short, long)]
        config_path: PathBuf,
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: i32,
    },
    /// Sends a message to shutdown winbar
    Stop,
    /// Sends a message to update the status bar window
    UpdateWindow,
}
