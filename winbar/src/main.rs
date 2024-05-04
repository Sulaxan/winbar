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
use component_impl::manager::ComponentManager;
use config::Config;
use lazy_static::lazy_static;
use tokio::runtime;
use tracing::instrument;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winbar::{color::Color, WinbarAction, WinbarContext, DEFAULT_HOSTNAME, DEFAULT_PORT};
use windows::Win32::Foundation::BOOL;
use windows::Win32::{
    Foundation::HWND,
    System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT},
};
use windows_api::WindowsApi;

use crate::server::WinbarServer;

pub mod cli;
pub mod component_impl;
pub mod config;
pub mod container;
pub mod server;
pub mod windows_api;

// runtime variables
static SERVER_PORT: AtomicI32 = AtomicI32::new(DEFAULT_PORT);

// config variables
const TRANSPARENT_COLOR: u32 = 0;
static WIDTH: AtomicI32 = AtomicI32::new(2560);
static HEIGHT: AtomicI32 = AtomicI32::new(25);
static POSITION_X: AtomicI32 = AtomicI32::new(0);
static POSITION_Y: AtomicI32 = AtomicI32::new(0);
static COMPONENT_GAP: AtomicI32 = AtomicI32::new(10);
static DEFAULT_FONT_SIZE: AtomicI32 = AtomicI32::new(18);

lazy_static! {
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
    static ref WINBAR_HWND: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
    static ref COMPONENT_MANAGER: Arc<Mutex<ComponentManager>> =
        Arc::new(Mutex::new(ComponentManager::new()));
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
pub fn read_config() -> anyhow::Result<()> {
    let cli = WinbarCli::parse();
    if cli.generate_config {
        gen_config(&cli.config_path);
        return Ok(());
    }

    let config = Config::read(&cli.config_path)?;
    config.set_global_constants()?;

    SERVER_PORT.store(cli.port, Ordering::SeqCst);

    tracing::info!("Adding components from config");
    match COMPONENT_MANAGER.lock() {
        Ok(mut manager) => {
            config.components.iter().for_each(|data| {
                manager.add(data.location, data.component.to_component());
            });
        }
        Err(e) => {
            tracing::error!("Error obtaining component manager lock: {}", e)
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let (stdout_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(stdout_writer))
        // .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Reading config");
    read_config()?;

    tracing::info!("Starting GDI+");
    let token = WindowsApi::startup_gdiplus()?;
    tracing::debug!("GDI+ token: {}", token);

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true)
            .with_context(|| "Could not set console ctrl handler")?;
    }

    tracing::info!("Initializing window");
    let winbar_hwnd = container::create_window();
    {
        let mut hwnd = WINBAR_HWND
            .lock()
            .map_err(|e| anyhow!("Could not obtain winbar hwnd lock: {}", e))?;
        *hwnd = winbar_hwnd;
    }

    let (send, recv) = mpsc::channel::<WinbarAction>();
    let winbar_ctx = WinbarContext::new(send);

    tracing::info!("Starting component runner thread");
    let cloned_ctx = winbar_ctx.clone();
    thread::spawn(move || {
        let rt = runtime::Runtime::new().unwrap();
        match COMPONENT_MANAGER.lock() {
            Ok(mut manager) => {
                let set = manager.start(cloned_ctx, winbar_hwnd);
                drop(manager);

                rt.block_on(set);
            }
            Err(e) => {
                tracing::error!("Error obtaining component manager lock {}", e);
            }
        }
    });

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
    container::listen(winbar_hwnd, recv);

    tracing::info!("Shutting down GDI+");
    WindowsApi::shutdown_gdiplus(token);

    Ok(())
}

#[instrument(level = "trace", name = "windows_ctrl_handler_function")]
pub extern "system" fn ctrl_handler(ctrltype: u32) -> BOOL {
    match ctrltype {
        CTRL_C_EVENT => {
            match WINBAR_HWND.lock() {
                Ok(hwnd) => {
                    WindowsApi::send_window_shutdown_msg(*hwnd);
                }
                Err(e) => {
                    tracing::error!("Error obtaining winbar hwnd lock: {}", e);
                }
            }

            true.into()
        }
        _ => false.into(),
    }
}
