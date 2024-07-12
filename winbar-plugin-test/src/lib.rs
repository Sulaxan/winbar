use std::{
    ffi::{c_char, CStr}, mem::{size_of}, sync::atomic::{AtomicBool, AtomicI32, Ordering}, thread, time::Duration
};

use winbar_core::{
    color::Color,
    styles::{BorderStyle, Styles},
    util::rect::Rect,
    windows_api::WindowsApi,
};
use winbar_plugin::{
    ComponentId, EventAction, EventResult, LoadConfigResult, PRect, WindowEvent,
    IGNORED_EVENT_RESULT,
};
use windows::Win32::{
    Foundation::{COLORREF, HWND, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetTextExtentPoint32W, SetTextColor, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC,
    },
    UI::{
        Controls::{WM_MOUSEHOVER, WM_MOUSELEAVE},
        Input::KeyboardAndMouse::{TrackMouseEvent, TRACKMOUSEEVENT, TRACKMOUSEEVENT_FLAGS},
        WindowsAndMessaging::{WM_MOUSEFIRST, WM_MOUSEMOVE},
    },
};

static ID: &str = "test\0";
static NUM: AtomicI32 = AtomicI32::new(32);
// temp global hover
static HOVER: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn id() -> *const c_char {
    ID.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn width(_id: ComponentId, _hwnd: HWND, hdc: HDC) -> i32 {
    unsafe {
        let mut length: SIZE = SIZE::default();

        GetTextExtentPoint32W(
            hdc,
            &WindowsApi::str_to_u16_slice("PLUGIN TEST XXX"),
            &mut length,
        );

        length.cx + 10 * 2
    }
}

#[no_mangle]
pub extern "C" fn draw(_id: ComponentId, _hwnd: HWND, rect: PRect, hdc: HDC) {
    Styles::draw_rect(hdc, &rect.into(), &BorderStyle::Square);
    unsafe {
        if HOVER.load(Ordering::SeqCst) {
            SetTextColor(
                hdc,
                COLORREF(
                    Color::Rgb {
                        r: 25,
                        g: 200,
                        b: 50,
                    }
                    .bgr(),
                ),
            );
        }
        DrawTextW(
            hdc,
            &mut WindowsApi::str_to_u16_slice(&format!(
                "PLUGIN TEST {}",
                NUM.load(Ordering::SeqCst)
            )),
            &mut Rect::from(rect).into(),
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
    }
}

#[no_mangle]
pub extern "C" fn load_config(_id: ComponentId, config: *const c_char) -> LoadConfigResult {
    unsafe {
        let config = CStr::from_ptr(config).to_str().unwrap().to_string();
        println!("{}", config);
    }

    LoadConfigResult {
        ok: true,
        // FIXME: replace with null
        error_msg: "\0".as_ptr() as *const c_char,
    }
}

#[no_mangle]
pub extern "C" fn start(_id: ComponentId, hwnd: HWND) {
    let mut event = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TRACKMOUSEEVENT_FLAGS(0),
        hwndTrack: hwnd,
        dwHoverTime: 15,
    };
    unsafe {
        TrackMouseEvent(&mut event).unwrap();
    }

    println!("//////////////////////// STARTED TEST PLUGIN /////////////////////////////");
    loop {
        thread::sleep(Duration::from_millis(500));
        NUM.fetch_add(1, Ordering::SeqCst);
    }
}

#[no_mangle]
pub extern "C" fn stop(_id: ComponentId) {
    println!("//////////////////////// STOPPED TEST PLUGIN /////////////////////////////");
}

#[no_mangle]
pub extern "C" fn handle_event(event: WindowEvent) -> EventResult {
    return match event.msg_code {
        WM_MOUSEFIRST => {

            EventResult {
                action: EventAction::Handled,
                result: 0,
            }
        },
        WM_MOUSELEAVE => {
            HOVER.store(false, Ordering::SeqCst);
            println!("ylalala");
            EventResult {
                action: EventAction::Handled,
                result: 0,
            }
        }
        WM_MOUSEMOVE => {
            let x = (event.lparam.0 & 0xFFFF) as i32;
            let y = ((event.lparam.0 >> 16) & 0xFFFF) as i32;

            let loc = event.component_location;

            if x > loc.x && x < loc.x + loc.width && y > loc.y && y < loc.y + loc.height {
                HOVER.store(true, Ordering::SeqCst);
            } else {
                HOVER.store(false, Ordering::SeqCst);
            }

            EventResult {
                action: EventAction::Handled,
                result: 0,
            }
        }
        _ => IGNORED_EVENT_RESULT,
    };
}
