use std::sync::atomic::Ordering;

use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            CreateFontIndirectW, CreatePen, CreateSolidBrush, GetDC, ReleaseDC, SelectObject,
            SetBkColor, SetTextColor, FONT_QUALITY, FW_NORMAL, LOGFONTW, PROOF_QUALITY, PS_SOLID,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, SW_SHOWDEFAULT, WM_DESTROY, WM_QUIT, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
            WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{BACKGROUND, FOREGROUND, HEIGHT, TRANSPARENT_COLOR, WIDTH};

pub struct Winbar {
    hwnd: HWND,
}

impl Winbar {
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
                lpfnWndProc: Some(Self::window_proc),
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

    pub fn new() -> Self {
        let hwnd = Self::create_window();

        Self { hwnd }
    }

    pub fn listen(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, self.hwnd, 0, 0).as_bool() {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
        }
    }

    pub fn set_default_styles(&self) {
        unsafe {
            let hdc = GetDC(self.hwnd);

            let pen = CreatePen(PS_SOLID, 0, COLORREF(BACKGROUND.to_single_rgb()));
            let brush = CreateSolidBrush(COLORREF(BACKGROUND.to_single_rgb()));

            SelectObject(hdc, pen);
            SelectObject(hdc, brush);
            SetBkColor(hdc, COLORREF(BACKGROUND.to_single_rgb()));

            let font = CreateFontIndirectW(&LOGFONTW {
                lfWeight: FW_NORMAL.0 as i32,
                lfQuality: FONT_QUALITY(PROOF_QUALITY.0),
                ..Default::default()
            });

            SelectObject(hdc, font);

            SetTextColor(hdc, COLORREF(FOREGROUND.to_single_rgb()));

            ReleaseDC(self.hwnd, hdc);
        }
    }

    pub extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                }
                _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
        LRESULT(0)
    }
}
