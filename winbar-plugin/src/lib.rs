use std::ffi::c_char;

use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

pub mod plugin;

#[repr(C)]
pub struct PRect {
    /// The x value of the top left corner of the rect.
    pub x: i32,
    /// The y value of the top left corner of the rect.
    pub y: i32,
    /// The width of the rect.
    pub width: i32,
    /// The height of the rect.
    pub height: i32,
}

/// A function returning the id of the plugin.
pub type FnId = unsafe extern "C" fn() -> *const c_char;
/// A function returning the width of the component.
pub type FnWidth = unsafe extern "C" fn(HWND, HDC) -> i32;
/// A function that draws the component within the given rect.
pub type FnDraw = unsafe extern "C" fn(HWND, PRect, HDC);
/// A function that starts anything processes required by the plugin. It can be assumed that this
/// function will be called in its own thread.
pub type FnStart = unsafe extern "C" fn(HWND, PRect);
