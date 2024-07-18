use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use lazy_static::lazy_static;
use winbar_core::{
    color::Color,
    styles::{StyleOptions, Styles},
    util::rect::Rect,
    windows_api::WindowsApi,
    Component, EventResult, WinbarContext, WindowEvent, IGNORED_EVENT_RESULT,
};
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DrawTextW, EndPaint, GetTextExtentPoint32W, DT_CENTER,
            DT_SINGLELINE, DT_VCENTER, HDC, PAINTSTRUCT,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, LoadCursorW, PeekMessageW,
            PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes, ShowWindow,
            TranslateMessage, IDC_ARROW, LWA_COLORKEY, MSG, PM_REMOVE, SW_SHOWNORMAL, WM_CLOSE,
            WM_DESTROY, WM_ERASEBKGND, WM_NCMOUSEHOVER, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
            WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

lazy_static! {
    static ref COMPONENTS: Arc<Mutex<HashMap<isize, ComponentData>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

struct ComponentData {
    text: String,
    styles: Arc<StyleOptions>,
}

pub struct StaticTextComponent {
    text: String,
    styles: Arc<StyleOptions>,
}

impl StaticTextComponent {
    pub fn new(text: String, styles: StyleOptions) -> Self {
        Self {
            text,
            styles: Arc::new(styles),
        }
    }
}

impl Component for StaticTextComponent {
    fn create_window(&self, rect: Rect) -> HWND {
        unsafe {
            let class_name = w!("static-text");
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
                hbrBackground: CreateSolidBrush(COLORREF(
                    self.styles.bg_color.as_ref().unwrap().bgr(),
                )),
                hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW).unwrap(),
                ..Default::default()
            };

            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                class_name,
                w!("winbar static text component"),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                100,
                rect.height,
                None,
                None,
                h_inst,
                None,
            );

            SetLayeredWindowAttributes(hwnd, COLORREF(Color::Transparent.bgr()), 25, LWA_COLORKEY)
                .ok();

            let _success = ShowWindow(hwnd, SW_SHOWNORMAL);

            hwnd
        }
    }

    fn start(&self, hwnd: HWND) {
        {
            let mut components = COMPONENTS.lock().unwrap();
            components.insert(
                hwnd.0,
                ComponentData {
                    text: self.text.clone(),
                    styles: self.styles.clone(),
                },
            );
        }

        let mut msg = MSG::default();
        loop {
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
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 100,
                    bottom: 20,
                };
                let components = COMPONENTS.lock().unwrap();
                let data = components.get(&hwnd.0).unwrap();

                Styles::draw_rect(
                    hdc,
                    &Rect {
                        x: rect.left,
                        y: rect.top,
                        width: 100,
                        height: 20,
                    },
                    &data.styles.border_style,
                );

                DrawTextW(
                    hdc,
                    &mut WindowsApi::str_to_u16_slice(data.text.as_str()),
                    &mut rect,
                    DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                );

                EndPaint(hwnd, &ps);
            }
            WM_ERASEBKGND => {
                return LRESULT(1);
            }
            WM_DESTROY => {
                let mut components = COMPONENTS.lock().unwrap();
                components.remove(&hwnd.0);

                PostQuitMessage(0);
            }
            WM_NCMOUSEHOVER => {
                return LRESULT(1);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}
