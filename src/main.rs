use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        mpsc::{self, channel},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use color::Color;
use component::{
    datetime::DateTimeComponent,
    manager::{ComponentLocation, ComponentManager},
    static_text::StaticTextComponent,
};
use lazy_static::lazy_static;
use tokio::{runtime, task::LocalSet};
use winbar::{WinbarAction, WinbarContext};
use windows::Win32::{
    Foundation::HWND,
    System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT},
};
use windows::Win32::{
    Foundation::{BOOL, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE},
};

pub mod color;
pub mod component;
pub mod winbar;
pub mod windows_api;

const TRANSPARENT_COLOR: u32 = 0;
const WIDTH: AtomicI32 = AtomicI32::new(2560);
const HEIGHT: AtomicI32 = AtomicI32::new(25);
const COMPONENT_GAP: AtomicI32 = AtomicI32::new(10);
const BACKGROUND: Color = Color::Rgb {
    r: 23,
    g: 23,
    b: 23,
};
const FOREGROUND: Color = Color::Rgb {
    r: 33,
    g: 181,
    b: 80,
};
const FONT_NAME: &str = "Segoe UI Variable";

const RUNNING: AtomicBool = AtomicBool::new(true);

lazy_static! {
    static ref WINBAR_HWND: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
    static ref COMPONENT_MANAGER: Arc<Mutex<ComponentManager>> =
        Arc::new(Mutex::new(ComponentManager::new()));
}

fn main() {
    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true).unwrap();
    }

    let mut manager = COMPONENT_MANAGER.lock().unwrap();
    manager.add(
        ComponentLocation::LEFT,
        Arc::new(StaticTextComponent::new("left".to_owned())),
    );
    manager.add(
        ComponentLocation::MIDDLE,
        Arc::new(StaticTextComponent::new("middle".to_owned())),
    );
    manager.add(
        ComponentLocation::RIGHT,
        Arc::new(StaticTextComponent::new("right".to_owned())),
    );
    manager.add(
        ComponentLocation::RIGHT,
        Arc::new(DateTimeComponent::new("%F %r".to_owned())),
    );
    drop(manager);

    let winbar_hwnd = winbar::create_window();
    {
        let mut hwnd = WINBAR_HWND.lock().unwrap();
        *hwnd = winbar_hwnd;
    }

    let (send, recv) = mpsc::channel::<WinbarAction>();
    let winbar_ctx = WinbarContext::new(send);

    thread::spawn(move || {
        let rt = runtime::Runtime::new().unwrap();
        let mut manager = COMPONENT_MANAGER.lock().unwrap();
        let set = manager.start(winbar_ctx, winbar_hwnd.clone());
        drop(manager);

        rt.block_on(set);
    });

    winbar::listen(winbar_hwnd, recv);
}

pub extern "system" fn ctrl_handler(ctrltype: u32) -> BOOL {
    match ctrltype {
        CTRL_C_EVENT => {
            WINBAR_HWND
                .lock()
                .and_then(|hwnd| unsafe {
                    PostMessageW(*hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).unwrap();
                    Ok(())
                })
                .unwrap();
            RUNNING.store(false, Ordering::SeqCst);

            true.into()
        }
        _ => false.into(),
    }
}
