use clap::{Parser, Subcommand};

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
    Start,
    /// Sends a message to shutdown winbar
    Stop,
    /// Sends a message to update the status bar window
    UpdateWindow,
}
