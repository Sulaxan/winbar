use std::ffi::c_char;

use winbar::{
    styles::{BorderStyle, Styles},
    util::rect::Rect,
    windows_api::WindowsApi,
};
use winbar_plugin::{ComponentId, PRect};
use windows::Win32::{
    Foundation::{HWND, SIZE},
    Graphics::Gdi::{DrawTextW, GetTextExtentPoint32W, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC},
};

static ID: &str = "test";

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
            &WindowsApi::str_to_u16_slice("PLUGIN TEST"),
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
            &mut WindowsApi::str_to_u16_slice("PLUGIN TEST"),
            &mut Rect::from(rect).into(),
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
    }
}

#[no_mangle]
pub extern "C" fn start(_id: ComponentId, _hwnd: HWND, _rect: PRect) {
    println!("//////////////////////// STARTED TEST PLUGIN /////////////////////////////");
}
