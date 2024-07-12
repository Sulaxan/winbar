use std::{
    ffi::{c_char, CStr},
    sync::atomic::{AtomicI32, Ordering},
    thread,
    time::Duration,
};

use winbar_core::{
    styles::{BorderStyle, Styles},
    util::rect::Rect,
    windows_api::WindowsApi,
};
use winbar_plugin::{ComponentId, LoadConfigResult, PRect};
use windows::Win32::{
    Foundation::{HWND, SIZE},
    Graphics::Gdi::{DrawTextW, GetTextExtentPoint32W, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC},
};

static ID: &str = "test\0";
static NUM: AtomicI32 = AtomicI32::new(32);

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
pub extern "C" fn start(_id: ComponentId, _hwnd: HWND) {
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
