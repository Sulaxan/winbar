use std::{
    collections::HashMap,
    sync::{atomic::Ordering, mpsc::Receiver, RwLock},
};

use lazy_static::lazy_static;
use tracing::instrument;
use winbar::{color::Color, styles::Styles, WinbarAction};
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush,
            DeleteDC, DeleteObject, EndPaint, GetDC, InvalidateRect, SelectObject, SetBkColor,
            SetTextColor, HBITMAP, HDC, PAINTSTRUCT, PS_SOLID, SRCCOPY,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, PM_REMOVE, SW_SHOWNORMAL, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_PAINT,
            WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{
    windows_api::WindowsApi, COMPONENT_MANAGER, DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT,
    DEFAULT_FONT_SIZE, HEIGHT, WIDTH,
};

lazy_static! {
    // HWND -> HBITMAP
    // we use isize since HWND is not hashable
    static ref WINDOW_BUFFERS: RwLock<HashMap<isize, HBITMAP>> = RwLock::new(HashMap::new());
}

pub fn create_window(bg_color: Color) -> HWND {
    let width = WIDTH.load(Ordering::SeqCst);
    let height = HEIGHT.load(Ordering::SeqCst);
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
            width,
            height,
            None,
            None,
            h_inst,
            None,
        );

        // create window buffer
        let hdc = GetDC(hwnd);
        let bitmap = CreateCompatibleBitmap(hdc, width, height);
        {
            let mut buffers = WINDOW_BUFFERS.write().unwrap();
            buffers.insert(hwnd.0, bitmap);
        }

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

    // cleanup

    // release window buffer
    {
        let mut buffers = WINDOW_BUFFERS.write().unwrap();
        if let Some(bitmap) = buffers.remove(&hwnd.0) {
            unsafe {
                DeleteObject(bitmap);
            }
        }
    }

    tracing::info!("Winbar shutting down...");
}

#[instrument(level = "trace")]
pub fn paint(hwnd: HWND, hdc: HDC) {
    let mut manager = match COMPONENT_MANAGER.lock() {
        Ok(manager) => manager,
        Err(e) => {
            tracing::error!("Error obtaining component manager lock: {}", e);
            return;
        }
    };

    // FIXME: not ideal computing locations every time... optimize in the future
    manager.compute_locations(hwnd, hdc);

    manager.for_each(|state| {
        let styles = state.component().styles();

        // set styles
        let font = match &styles.font {
            Some(font) => font.to_string(),
            None => {
                let font = DEFAULT_FONT.lock().unwrap();
                font.to_string()
            }
        };
        let font_size = match &styles.font_size {
            Some(size) => *size,
            None => DEFAULT_FONT_SIZE.load(Ordering::SeqCst),
        };
        let bg_color = match &styles.bg_color {
            Some(color) => color.bgr(),
            None => {
                let color = DEFAULT_BG_COLOR.lock().unwrap();
                color.bgr()
            }
        };
        let fg_color = match &styles.fg_color {
            Some(color) => color.bgr(),
            None => {
                let color = DEFAULT_FG_COLOR.lock().unwrap();
                color.bgr()
            }
        };

        let pen = Styles::pen(bg_color, PS_SOLID);
        let brush = Styles::solid_brush(bg_color);
        let font = Styles::font(font_size, &font);

        unsafe {
            SelectObject(hdc, pen);
            SelectObject(hdc, brush);
            SelectObject(hdc, font);
            SetBkColor(hdc, COLORREF(bg_color));
            SetTextColor(hdc, COLORREF(fg_color));
        }

        state.component().draw(hwnd, *state.location(), hdc);

        unsafe {
            DeleteObject(pen);
            DeleteObject(brush);
            DeleteObject(font);
        }
    });
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

                // double buffered window
                let buffers = WINDOW_BUFFERS.read().unwrap();
                if let Some(buffer) = buffers.get(&hwnd.0) {
                    let hdc_buffer = CreateCompatibleDC(hdc);
                    let old_hdc = SelectObject(hdc_buffer, *buffer);
                    let width = WIDTH.load(Ordering::SeqCst);
                    let height = HEIGHT.load(Ordering::SeqCst);

                    paint(hwnd, hdc_buffer);

                    BitBlt(hdc, 0, 0, width, height, hdc_buffer, 0, 0, SRCCOPY).unwrap();

                    SelectObject(hdc_buffer, old_hdc);

                    DeleteDC(hdc_buffer);
                } else {
                    paint(hwnd, hdc);
                }

                EndPaint(hwnd, &ps);
                tracing::trace!("Finished painting");
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
