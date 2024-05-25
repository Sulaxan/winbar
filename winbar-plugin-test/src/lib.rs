use std::ffi::c_char;

use winbar_plugin::PRect;
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

static ID: &str = "test";

#[no_mangle]
pub extern "C" fn id() -> *const c_char {
    ID.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn width(_hwnd: HWND, _hdc: HDC) -> i32 {
    989
}

#[no_mangle]
pub extern "C" fn draw(hwnd: HWND, _rect: PRect, _hdc: HDC) {
    println!("draw invoked {}", hwnd.0);
}

#[no_mangle]
pub extern "C" fn start(_hwnd: HWND, _rect: PRect) {}
