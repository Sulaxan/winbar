use std::{
    sync::{atomic::AtomicI32, mpsc, Arc, Mutex},
    thread,
};

use anyhow::{anyhow, Context};
use component_impl::{
    datetime::DateTimeComponent,
    manager::{ComponentLocation, ComponentManager},
    static_text::StaticTextComponent,
};
use lazy_static::lazy_static;
use tokio::runtime;
use tracing::instrument;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winbar::{color::Color, WinbarAction, WinbarContext};
use windows::Win32::{
    Foundation::HWND,
    System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT},
};
use windows::Win32::{
    Foundation::{BOOL, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE},
};
use windows_api::WindowsApi;

pub mod component_impl;
pub mod config;
pub mod container;
pub mod server;
pub mod windows_api;

const TRANSPARENT_COLOR: u32 = 0;
const WIDTH: AtomicI32 = AtomicI32::new(2560);
const HEIGHT: AtomicI32 = AtomicI32::new(25);
const POSITION_X: AtomicI32 = AtomicI32::new(0);
const POSITION_Y: AtomicI32 = AtomicI32::new(0);
const COMPONENT_GAP: AtomicI32 = AtomicI32::new(10);

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
    static ref DEFAULT_FONT: Arc<Mutex<String>> =
        Arc::new(Mutex::new("Segoe UI Variable".to_string()));
    static ref WINBAR_HWND: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
    static ref COMPONENT_MANAGER: Arc<Mutex<ComponentManager>> =
        Arc::new(Mutex::new(ComponentManager::new()));
}

fn main() -> anyhow::Result<()> {
    let (stdout_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(stdout_writer))
        // .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting GDI+");
    let token = WindowsApi::startup_gdiplus()?;
    tracing::debug!("GDI+ token: {}", token);

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true)
            .with_context(|| "Could not set console ctrl handler")?;
    }

    tracing::info!("Adding components");
    match COMPONENT_MANAGER.lock() {
        Ok(mut manager) => {
            manager.add(
                ComponentLocation::LEFT,
                Arc::new(StaticTextComponent::new("left".to_owned(), 10)),
            );
            manager.add(
                ComponentLocation::MIDDLE,
                Arc::new(StaticTextComponent::new("middle".to_owned(), 10)),
            );
            manager.add(
                ComponentLocation::RIGHT,
                Arc::new(StaticTextComponent::new("right".to_owned(), 10)),
            );
            manager.add(
                ComponentLocation::RIGHT,
                Arc::new(DateTimeComponent::new("%F %r".to_owned())),
            );
        }
        Err(e) => {
            tracing::error!("Error obtaining component manager lock: {}", e)
        }
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
    thread::spawn(move || {
        let rt = runtime::Runtime::new().unwrap();
        match COMPONENT_MANAGER.lock() {
            Ok(mut manager) => {
                let set = manager.start(winbar_ctx, winbar_hwnd.clone());
                drop(manager);

                rt.block_on(set);
            }
            Err(e) => {
                tracing::error!("Error obtaining component manager lock {}", e);
            }
        }
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
            match WINBAR_HWND.lock().and_then(|hwnd| unsafe {
                match PostMessageW(*hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) {
                    Err(e) => {
                        tracing::error!("Error posting WM_CLOSE message: {}", e);
                    }
                    _ => {}
                }
                Ok(())
            }) {
                Err(e) => {
                    tracing::error!("Error obtaining winbar hwnd lock: {}", e);
                }
                _ => {}
            }

            true.into()
        }
        _ => false.into(),
    }
}
