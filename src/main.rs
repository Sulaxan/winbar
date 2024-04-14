use std::{
    sync::{atomic::AtomicI32, mpsc::channel, Arc, Mutex},
    thread,
    time::Duration,
};

use color::Color;
use component::static_text::StaticTextComponent;
use lazy_static::lazy_static;
use winbar::{Winbar, WinbarAction};
use windows::Win32::{Foundation::HWND, System::Console::SetConsoleCtrlHandler};
use windows::Win32::{
    Foundation::{BOOL, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{DestroyWindow, PostMessageW, WM_CLOSE},
};

pub mod color;
pub mod component;
pub mod winbar;
pub mod windows_api;

const TRANSPARENT_COLOR: u32 = 0;
const WIDTH: AtomicI32 = AtomicI32::new(2560);
const HEIGHT: AtomicI32 = AtomicI32::new(20);
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

lazy_static! {
    static ref WINBAR: Arc<Mutex<HWND>> = Arc::new(Mutex::new(HWND(0)));
}

fn main() {
    let (send, recv) = channel::<WinbarAction>();

    thread::spawn(move || {
        let mut winbar = Winbar::new(recv);
        {
            let mut hwnd = WINBAR.lock().unwrap();
            *hwnd = winbar.hwnd();
        }

        winbar.set_default_styles();
        winbar.add_component(
            winbar::ComponentLocation::LEFT,
            Box::new(StaticTextComponent::new("TEST".to_owned())),
        );
        winbar.listen();
    });

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), true).unwrap();
    }

    loop {
        println!("Sending update window....");
        if send.send(WinbarAction::UpdateWindow).is_ok() {
            thread::sleep(Duration::from_secs(1));
        } else {
            break;
        }
    }
}

pub extern "system" fn ctrl_handler(ctrltype: u32) -> BOOL {
    match ctrltype {
        CTRL_C_EVENT => {
            WINBAR
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
