use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use cli::WinbarCli;
use config::Config;
use lazy_static::lazy_static;
use status_bar::{ComponentLayout, StatusBar};
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
// pub mod container;
pub mod server;
pub mod status_bar;

// runtime variables
static SERVER_PORT: AtomicI32 = AtomicI32::new(DEFAULT_PORT);
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref STATUS_BARS: Arc<Mutex<Vec<StatusBar>>> = Arc::new(Mutex::new(Vec::new()));
    static ref PLUGIN_MANAGER: Arc<Mutex<PluginManager>> =
        Arc::new(Mutex::new(PluginManager::new()));
}

// config variables
lazy_static! {
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
pub fn read_config() -> anyhow::Result<Config> {
    let cli = WinbarCli::parse();
    if cli.generate_config {
        gen_config(&cli.config_path);
        return Ok(Config::default());
    }

    let config = Config::read(&cli.config_path)?;
    config.set_global_constants()?;

    SERVER_PORT.store(cli.port, Ordering::SeqCst);

    tracing::info!("Processing components from config");

    Ok(config)
}

fn main() -> anyhow::Result<()> {
    let (stdout_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(stdout_writer))
        // .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Reading config");
    let config = read_config()?;

    tracing::info!("Starting GDI+");
    let token = WindowsApi::startup_gdiplus()?;
    tracing::debug!("GDI+ token: {}", token);

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true)
            .with_context(|| "Could not set console ctrl handler")?;
    }

    // tracing::info!("Initializing window");
    // let status_bar_bg_color = {
    //     let color = STATUS_BAR_BG_COLOR
    //         .lock()
    //         .map_err(|e| anyhow!("Could not obtain status bar bg color lock: {}", e))?;

    //     color.clone()
    // };
    // let winbar_hwnd = container::create_window(status_bar_bg_color);

    // {
    //     let mut hwnd = WINBAR_HWND
    //         .lock()
    //         .map_err(|e| anyhow!("Could not obtain winbar hwnd lock: {}", e))?;
    //     *hwnd = winbar_hwnd;
    // }

    let (send, recv) = mpsc::channel::<WinbarAction>();
    let winbar_ctx = WinbarContext::new(send);

    tracing::info!("Processing config status bars");
    {
        let mut status_bars = STATUS_BARS.lock().unwrap();
        for sb in config.status_bars.iter() {
            match config.layouts.iter().find(|l| l.id == sb.layout_id) {
                Some(target_layout) => {
                    let layout = target_layout
                        .components
                        .iter()
                        .map(|c| ComponentLayout {
                            location: c.location.into(),
                            component: c.component.to_component(),
                        })
                        .collect();

                    status_bars.push(StatusBar::new(
                        sb.x,
                        sb.y,
                        sb.width,
                        sb.height,
                        sb.component_gap,
                        layout,
                    ));
                }
                None => bail!("Could not find layout id {}", sb.layout_id),
            }
        }
    }

    tracing::info!("Starting status bars");
    {
        let mut status_bars = STATUS_BARS.lock().unwrap();
        status_bars.iter_mut().for_each(|sb| sb.start());
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
    loop {
        if SHOULD_EXIT.load(Ordering::SeqCst) {
            break;
        }

        {
            let mut sb = STATUS_BARS.lock().unwrap();
            sb.iter_mut().for_each(|s| s.update_locations());
        }

        thread::sleep(Duration::from_secs(2));
    }
    // // this is blocking; we handle process termination below and through messages received on the
    // // mspc channel
    // container::listen(winbar_hwnd, recv);

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
            // let hwnd = WINBAR_HWND.lock().unwrap();
            // WindowsApi::send_window_shutdown_msg(*hwnd);
            SHOULD_EXIT.store(true, Ordering::SeqCst);

            true.into()
        }
        _ => false.into(),
    }
}
