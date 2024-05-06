use std::sync::{atomic::Ordering, mpsc::Receiver};

use tracing::instrument;
use winbar::{color::Color, WinbarAction};
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, InvalidateRect, SelectObject,
            SetBkColor, SetTextColor, PAINTSTRUCT, PS_SOLID,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, PM_REMOVE, SW_SHOWNORMAL, WM_CLOSE, WM_DESTROY, WM_PAINT, WNDCLASSW,
            WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{
    styles::Styles, windows_api::WindowsApi, COMPONENT_MANAGER, DEFAULT_BG_COLOR, DEFAULT_FG_COLOR,
    DEFAULT_FONT, DEFAULT_FONT_SIZE, HEIGHT, WIDTH,
};

pub fn create_window(bg_color: Color) -> HWND {
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
            hbrBackground: CreateSolidBrush(COLORREF(bg_color.bgr())),
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

        SetLayeredWindowAttributes(hwnd, COLORREF(Color::Transparent.bgr()), 25, LWA_COLORKEY).ok();

        let _success = ShowWindow(hwnd, SW_SHOWNORMAL);

        hwnd
    }
}

#[instrument(name = "window_listener")]
pub fn listen(hwnd: HWND, recv: Receiver<WinbarAction>) {
    let mut msg = MSG::default();

    loop {
        if let Ok(action) = recv.try_recv() {
            match action {
                WinbarAction::Shutdown => {
                    WindowsApi::send_window_shutdown_msg(hwnd);
                }
                WinbarAction::UpdateWindow => unsafe {
                    // UpdateWindow(hwnd);
                    InvalidateRect(hwnd, None, true);
                },
                WinbarAction::ShowWindow => {
                    WindowsApi::show_window(hwnd);
                }
                WinbarAction::HideWindow => {
                    WindowsApi::hide_window(hwnd);
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

                let default_bg_color = {
                    let color = DEFAULT_BG_COLOR.lock().unwrap();
                    color.bgr()
                };
                let default_fg_color = {
                    let color = DEFAULT_FG_COLOR.lock().unwrap();
                    color.bgr()
                };
                let default_font = {
                    let font = DEFAULT_FONT.lock().unwrap();
                    font.to_string()
                };
                let default_font_size = DEFAULT_FONT_SIZE.load(Ordering::SeqCst);

                let pen = Styles::pen(default_bg_color, PS_SOLID);
                let brush = Styles::solid_brush(default_bg_color);
                let font = Styles::font(default_font_size, &default_font);

                SelectObject(hdc, pen);
                SelectObject(hdc, brush);
                SelectObject(hdc, font);
                SetBkColor(hdc, COLORREF(default_bg_color));
                SetTextColor(hdc, COLORREF(default_fg_color));

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

                DeleteObject(pen);
                DeleteObject(brush);
                DeleteObject(font);

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
