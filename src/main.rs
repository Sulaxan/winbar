use std::{
    borrow::BorrowMut,
    ffi::{CStr, OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    sync::atomic::{AtomicI32, AtomicUsize, Ordering},
};

use color::Color;
use windows::{
    core::{h, w},
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateFontIndirectW, CreateFontW, CreatePen, CreateSolidBrush, DrawTextExW,
            DrawTextW, EndPaint, FillRect, GetDC, GetSysColor, GetSysColorBrush, ReleaseDC,
            RoundRect, SelectObject, SetBkColor, SetTextColor, COLOR_GRADIENTACTIVECAPTION,
            COLOR_HOTLIGHT, COLOR_MENUTEXT, COLOR_WINDOW, DT_CENTER, DT_END_ELLIPSIS, DT_NOCLIP,
            DT_RIGHT, DT_SINGLELINE, DT_VCENTER, FONT_QUALITY, FW_BOLD, FW_NORMAL, HBRUSH,
            LOGFONTW, PAINTSTRUCT, PROOF_QUALITY, PS_SOLID,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes, SetWindowLongW,
            ShowWindow, TranslateMessage, GWL_STYLE, LWA_ALPHA, LWA_COLORKEY, MSG, SHOW_WINDOW_CMD,
            SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WM_PAINT,
            WNDCLASSW, WNDPROC, WS_BORDER, WS_EX_DLGMODALFRAME, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
            WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_POPUPWINDOW, WS_VISIBLE,
        },
    },
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

fn main() {}
