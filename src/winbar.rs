use std::sync::{atomic::Ordering, mpsc::Receiver, Mutex};

use tokio::task::JoinSet;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::CreateSolidBrush,
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW, PostQuitMessage,
            RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage, LWA_COLORKEY,
            MSG, PM_REMOVE, SW_SHOWDEFAULT, WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_EX_LAYERED,
            WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{component::Component, COMPONENT_GAP, HEIGHT, TRANSPARENT_COLOR, WIDTH};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentLocation {
    LEFT,
    MIDDLE,
    RIGHT,
}

struct WinbarComponent {
    location_intention: ComponentLocation,
    location: RECT,
    component: Box<dyn Component>,
}

pub enum WinbarAction {
    UpdateWindow,
}

pub struct Winbar {
    components: Mutex<Vec<WinbarComponent>>,
}

impl Winbar {
    pub fn new() -> Self {
        Self {
            components: Mutex::new(Vec::new()),
        }
    }

    pub async fn start(&self, hwnd: HWND) {
        self.compute_component_locations(hwnd);

        let mut set = JoinSet::<()>::new();
        let mut components = self.components.lock().unwrap();
        for winbar_comp in components.drain(0..) {
            let component = winbar_comp.component;
            let location = winbar_comp.location;
            set.spawn(component.start(hwnd, location));
        }
    }

    pub fn add_component(&mut self, location: ComponentLocation, component: Box<dyn Component>) {
        let mut components = self.components.lock().unwrap();
        components.push(WinbarComponent {
            location_intention: location,
            location: RECT::default(),
            component,
        })
    }

    fn compute_component_locations(&self, hwnd: HWND) {
        let mut components = self.components.lock().unwrap();
        let width = WIDTH.load(Ordering::SeqCst);
        let height = HEIGHT.load(Ordering::SeqCst);
        let gap = COMPONENT_GAP.load(Ordering::SeqCst);

        let mut curr_loc_x = 0;

        // left
        components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::LEFT)
            .for_each(|c| {
                let component_width = c.component.width(hwnd);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x,
                    right: curr_loc_x + component_width,
                };
                curr_loc_x += component_width + gap;
            });

        // right
        curr_loc_x = width;
        components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::RIGHT)
            .for_each(|c| {
                let component_width = c.component.width(hwnd);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x - component_width,
                    right: curr_loc_x,
                };
                curr_loc_x -= component_width + gap;
            });

        // middle
        // FIXME: a bit inefficient, change in the future...
        let mut total_components = 0;
        let total_width = components
            .iter_mut()
            .filter_map(|c| {
                if c.location_intention == ComponentLocation::MIDDLE {
                    total_components += 1;
                    Some(c.component.width(hwnd))
                } else {
                    None
                }
            })
            .reduce(|acc, width| acc + width)
            .unwrap();

        // we multiply by 1 less gap since it's in between the components
        curr_loc_x = width / 2 - (total_width + (gap - 1) * total_components) / 2;
        components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::MIDDLE)
            .for_each(|c| {
                let component_width = c.component.width(hwnd);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x,
                    right: curr_loc_x + component_width,
                };
                curr_loc_x += component_width + gap;
            });
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

pub fn listen(hwnd: HWND) {
    let mut msg = MSG::default();

    loop {
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
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}
