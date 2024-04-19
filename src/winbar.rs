use std::sync::{
    atomic::Ordering,
    mpsc::{Receiver, Sender},
    Mutex,
};

use getset::Getters;
use tokio::task::JoinSet;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, CreateSolidBrush, EndPaint, UpdateWindow, PAINTSTRUCT},
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, PM_REMOVE, SW_SHOWDEFAULT, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_PAINT,
            WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{
    component::Component, windows_api::WindowsApi, COMPONENT_GAP, COMPONENT_MANAGER, HEIGHT,
    RUNNING, TRANSPARENT_COLOR, WIDTH,
};

pub enum WinbarAction {
    UpdateWindow,
}

#[derive(Getters, Clone)]
pub struct WinbarContext {
    #[getset(get = "pub")]
    sender: Sender<WinbarAction>,
}

impl WinbarContext {
    pub fn new(sender: Sender<WinbarAction>) -> Self {
        Self { sender }
    }
}

pub fn create_window() -> HWND {
    unsafe {
        let class_name = w!("winbar");
        let h_inst = GetModuleHandleW(None).unwrap();
        let mut startup_info: STARTUPINFOW = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        GetStartupInfoW(&mut startup_info);

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: h_inst.into(),
            lpszClassName: class_name.clone(),
            hbrBackground: CreateSolidBrush(COLORREF(TRANSPARENT_COLOR)),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name.clone(),
            w!("winbar"),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            WIDTH.load(Ordering::SeqCst),
            HEIGHT.load(Ordering::SeqCst),
            None,
            None,
            h_inst,
            None,
        );

        SetLayeredWindowAttributes(hwnd, COLORREF(TRANSPARENT_COLOR), 25, LWA_COLORKEY).ok();

        let _success = ShowWindow(hwnd, SW_SHOWDEFAULT);

        hwnd
    }
}

pub fn listen(hwnd: HWND, recv: Receiver<WinbarAction>) {
    let mut msg = MSG::default();

    loop {
        if let Ok(action) = recv.try_recv() {
            match action {
                WinbarAction::UpdateWindow => unsafe {
                    UpdateWindow(hwnd);
                },
            }
        }

        unsafe {
            if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
        }

        if msg.message == WM_CLOSE {
            break;
        }
    }

    println!("Winbar shutting down...");
    //TODO: shut down the components
}

pub extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                WindowsApi::set_default_styles(hdc);
                println!("Drawing");

                let mut manager = COMPONENT_MANAGER.lock().unwrap();

                // FIXME: not ideal to compute locations every time... need to change in the
                // future
                manager.compute_locations(hwnd, hdc);
                println!("Computed locs");

                manager.draw_all(hwnd, hdc);
                println!("Drawn all");

                EndPaint(hwnd, &mut ps);
            }
            WM_ERASEBKGND => {
                return LRESULT(1);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}
