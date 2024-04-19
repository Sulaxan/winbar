use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use color::Color;
use component::static_text::StaticTextComponent;
use lazy_static::lazy_static;
use winbar::{Winbar, WinbarAction};
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

const READY: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref WINBAR_HWND: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
}

#[tokio::main]
async fn main() {
    thread::spawn(move || {
        let winbar_hwnd = winbar::create_window();
        {
            let mut hwnd = WINBAR_HWND.lock().unwrap();
            *hwnd = winbar_hwnd;
        }

        winbar::listen(winbar_hwnd);
    });

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true).unwrap();
    }

    let mut winbar = Winbar::new();

    winbar.add_component(
        winbar::ComponentLocation::LEFT,
        Box::new(StaticTextComponent::new("left".to_owned())),
    );
    winbar.add_component(
        winbar::ComponentLocation::MIDDLE,
        Box::new(StaticTextComponent::new("middle".to_owned())),
    );
    winbar.add_component(
        winbar::ComponentLocation::RIGHT,
        Box::new(StaticTextComponent::new("right".to_owned())),
    );

    while !READY.load(Ordering::SeqCst) {}

    loop {}
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
            true.into()
        }
        _ => false.into(),
    }
}
