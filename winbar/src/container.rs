use std::sync::{atomic::Ordering, mpsc::Receiver};

use tracing::instrument;
use winbar::WinbarAction;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{BeginPaint, CreateSolidBrush, EndPaint, InvalidateRect, PAINTSTRUCT},
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, PM_REMOVE, SW_SHOWDEFAULT, WM_CLOSE, WM_DESTROY, WM_PAINT, WNDCLASSW,
            WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{windows_api::WindowsApi, COMPONENT_MANAGER, HEIGHT, TRANSPARENT_COLOR, WIDTH};

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
            lpszClassName: class_name,
            hbrBackground: CreateSolidBrush(COLORREF(TRANSPARENT_COLOR)),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name,
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

#[instrument(name = "window_listener")]
pub fn listen(hwnd: HWND, recv: Receiver<WinbarAction>) {
    let mut msg = MSG::default();

    loop {
        if let Ok(action) = recv.try_recv() {
            match action {
                WinbarAction::UpdateWindow => unsafe {
                    // UpdateWindow(hwnd);
                    InvalidateRect(hwnd, None, true);
                },
                WinbarAction::Shutdown => {
                    WindowsApi::send_window_shutdown_msg(hwnd);
                }
            }
        }

        unsafe {
            if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        if msg.message == WM_CLOSE {
            break;
        }
    }

    tracing::info!("Winbar shutting down...");
}

#[instrument(level = "trace", name = "window_process_function")]
pub extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                tracing::trace!("Starting painting...");
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                WindowsApi::set_default_styles(hdc);

                match COMPONENT_MANAGER.lock() {
                    Ok(mut manager) => {
                        // FIXME: not ideal to compute locations every time... need to change in the
                        // future
                        manager.compute_locations(hwnd, hdc);

                        manager.draw_all(hwnd, hdc);
                    }
                    Err(e) => {
                        tracing::error!("Error obtaining component manager lock: {}", e);
                    }
                }

                EndPaint(hwnd, &ps);
                tracing::trace!("Finished painting");
            }
            // WM_ERASEBKGND => {
            //     return LRESULT(1);
            // }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}
