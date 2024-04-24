use std::path::PathBuf;

use clap::Parser;

/// Windows 10/11 status bar
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct WinbarCli {
    /// The path to the config
    #[arg(short, long)]
    pub config_path: PathBuf,
    /// Whether to generate the config. This will only generate the config if config_path does not
    /// exist.
    #[arg(long, default_value_t = false)]
    pub generate_config: bool,
}
