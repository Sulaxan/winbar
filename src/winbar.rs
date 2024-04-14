use std::sync::{atomic::Ordering, mpsc::Receiver, Mutex};

use windows::{
    core::w,
    Win32::{
        Foundation::{BOOL, COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateFontIndirectW, CreatePen, CreateSolidBrush, DrawTextW, EndPaint,
            GetDC, ReleaseDC, SelectObject, SetBkColor, SetTextColor, DT_CENTER, DT_SINGLELINE,
            DT_VCENTER, FONT_QUALITY, FW_NORMAL, LOGFONTW, PAINTSTRUCT, PROOF_QUALITY, PS_SOLID,
        },
        System::{
            Console::{SetConsoleCtrlHandler, CTRL_C_EVENT},
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, GetThreadId, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW, PostQuitMessage, PostThreadMessageW, RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY, MSG, PM_REMOVE, SW_SHOWDEFAULT, WM_CLOSE, WM_DESTROY, WM_PAINT, WM_QUIT, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE
        },
    },
};

use crate::{
    component::Component, windows_api::WindowsApi, BACKGROUND, FOREGROUND, HEIGHT,
    TRANSPARENT_COLOR, WIDTH,
};

#[derive(Clone, Copy, Debug)]
pub enum ComponentLocation {
    LEFT,
    MIDDLE,
    RIGHT,
}

struct WinbarComponent {
    location: ComponentLocation,
    component: Box<dyn Component>,
}

pub enum WinbarAction {
    UpdateWindow,
}

pub struct Winbar {
    hwnd: HWND,
    channel_receiver: Receiver<WinbarAction>,
    components: Mutex<Vec<WinbarComponent>>,
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

    pub fn new(recv: Receiver<WinbarAction>) -> Self {
        let hwnd = Self::create_window();

        Self {
            hwnd,
            channel_receiver: recv,
            components: Mutex::new(Vec::new()),
        }
    }

    pub fn hwnd(&self) -> HWND {
        return HWND(self.hwnd.0);
    }

    pub fn listen(&self) {
        let mut msg = MSG::default();

        loop {
            let action = self.channel_receiver.try_recv();
            if action.is_ok() {
                match action.unwrap() {
                    WinbarAction::UpdateWindow => {
                        self.update();
                    }
                }
            }

            unsafe {
                if PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
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

    pub fn add_component(&mut self, location: ComponentLocation, component: Box<dyn Component>) {
        let mut components = self.components.lock().unwrap();
        components.push(WinbarComponent {
            location,
            component,
        })
    }

    fn update(&self) {
        let components = self.components.lock().unwrap();
        components.iter().for_each(|component| {
            component.component.draw(
                self.hwnd,
                &mut RECT {
                    left: 0,
                    top: 0,
                    right: 1000,
                    bottom: 20,
                },
            )
        })
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
