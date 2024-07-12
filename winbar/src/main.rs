use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicI32, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use cli::WinbarCli;
use component_impl::manager::{ComponentLocation, ComponentManager};
use config::Config;
use lazy_static::lazy_static;
use tokio::runtime;
use tracing::instrument;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winbar_core::{
    color::Color, windows_api::WindowsApi, Component, WinbarAction, WinbarContext,
    DEFAULT_HOSTNAME, DEFAULT_PORT,
};
use winbar_plugin::plugin::PluginManager;
use windows::Win32::Foundation::BOOL;
use windows::Win32::{
    Foundation::HWND,
    System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT},
};

use crate::server::WinbarServer;

pub mod cli;
pub mod component_impl;
pub mod config;
pub mod container;
pub mod server;

// runtime variables
static SERVER_PORT: AtomicI32 = AtomicI32::new(DEFAULT_PORT);

lazy_static! {
    static ref WINBAR_HWND: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
    static ref COMPONENT_MANAGER: Arc<Mutex<ComponentManager>> =
        Arc::new(Mutex::new(ComponentManager::new(HWND(0))));
    static ref PLUGIN_MANAGER: Arc<Mutex<PluginManager>> =
        Arc::new(Mutex::new(PluginManager::new()));
}

// config variables
static WIDTH: AtomicI32 = AtomicI32::new(2560);
static HEIGHT: AtomicI32 = AtomicI32::new(25);
static POSITION_X: AtomicI32 = AtomicI32::new(0);
static POSITION_Y: AtomicI32 = AtomicI32::new(0);
static COMPONENT_GAP: AtomicI32 = AtomicI32::new(10);
static DEFAULT_FONT_SIZE: AtomicI32 = AtomicI32::new(18);

lazy_static! {
    static ref STATUS_BAR_BG_COLOR: Arc<Mutex<Color>> = Arc::new(Mutex::new(Color::Transparent));
    static ref DEFAULT_BG_COLOR: Arc<Mutex<Color>> = Arc::new(Mutex::new(Color::Rgb {
        r: 23,
        g: 23,
        b: 23,
    }));
    static ref DEFAULT_FG_COLOR: Arc<Mutex<Color>> = Arc::new(Mutex::new(Color::Rgb {
        r: 33,
        g: 181,
        b: 80,
    }));
    // Segoe UI Variable is the default windows font
    static ref DEFAULT_FONT: Arc<Mutex<String>> =
        Arc::new(Mutex::new("Segoe UI Variable".to_string()));
    static ref PLUGIN_DIR: Arc<Mutex<PathBuf>> = Arc::new(Mutex::new(PathBuf::new()));
}

pub fn gen_config(path: &PathBuf) {
    if path.try_exists().unwrap() {
        println!(include_str!("./res/config_exists_warning.txt"));
        std::process::exit(1);
    }

    let config = Config::default();
    config.write(path).unwrap();
    println!(include_str!("./res/config_gen_success.txt"));
    std::process::exit(0);
}

#[instrument]
pub fn read_config() -> anyhow::Result<Vec<(ComponentLocation, Arc<dyn Component + Send + Sync>)>> {
    let cli = WinbarCli::parse();
    if cli.generate_config {
        gen_config(&cli.config_path);
        return Ok(Vec::new());
    }

    let config = Config::read(&cli.config_path)?;
    config.set_global_constants()?;

    SERVER_PORT.store(cli.port, Ordering::SeqCst);

    tracing::info!("Processing components from config");

    Ok(config
        .components
        .iter()
        .map(|c| (c.location, c.component.to_component()))
        .collect())
}

fn main() -> anyhow::Result<()> {
    let (stdout_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(stdout_writer))
        // .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Reading config");
    let components = read_config()?;

    tracing::info!("Starting GDI+");
    let token = WindowsApi::startup_gdiplus()?;
    tracing::debug!("GDI+ token: {}", token);

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true)
            .with_context(|| "Could not set console ctrl handler")?;
    }

    tracing::info!("Initializing window");
    let status_bar_bg_color = {
        let color = STATUS_BAR_BG_COLOR
            .lock()
            .map_err(|e| anyhow!("Could not obtain status bar bg color lock: {}", e))?;

        color.clone()
    };
    let winbar_hwnd = container::create_window(status_bar_bg_color);

    {
        let mut hwnd = WINBAR_HWND
            .lock()
            .map_err(|e| anyhow!("Could not obtain winbar hwnd lock: {}", e))?;
        *hwnd = winbar_hwnd;
    }

    {
        let mut manager = COMPONENT_MANAGER.lock().unwrap();
        *manager = ComponentManager::new(winbar_hwnd);
    }

    let (send, recv) = mpsc::channel::<WinbarAction>();
    let winbar_ctx = WinbarContext::new(send);

    tracing::info!("Starting components");
    {
        let mut manager = COMPONENT_MANAGER.lock().unwrap();
        components.iter().for_each(|(loc, c)| {
            manager.add(*loc, c.clone(), winbar_ctx.clone());
        });
    }

    tracing::info!("Starting server");
    thread::spawn(move || {
        let rt = runtime::Runtime::new().unwrap();

        // need to block_on here, otherwise the thread shuts down prematurely
        rt.block_on(async move {
            let url = format!(
                "{}:{}",
                DEFAULT_HOSTNAME,
                SERVER_PORT.load(Ordering::SeqCst)
            );
            match WinbarServer::new(&url, winbar_ctx).await {
                Ok(mut server) => {
                    if let Err(e) = server.start_listening().await {
                        tracing::error!("Error while starting to listen for connections: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Error while starting server: {}", e);
                }
            }
        });
    });

    tracing::info!("Starting window listener");
    // this is blocking; we handle process termination below and through messages received on the
    // mspc channel
    container::listen(winbar_hwnd, recv);

    // SHUTDOWN LOGIC
    tracing::info!("Shutting down GDI+");
    WindowsApi::shutdown_gdiplus(token);

    tracing::info!("Shutting down winbar");
    Ok(())
}

#[instrument(level = "trace", name = "windows_ctrl_handler_function")]
pub extern "system" fn ctrl_handler(ctrltype: u32) -> BOOL {
    match ctrltype {
        CTRL_C_EVENT => {
            let hwnd = WINBAR_HWND.lock().unwrap();
            WindowsApi::send_window_shutdown_msg(*hwnd);

            true.into()
        }
        _ => false.into(),
    }
}
