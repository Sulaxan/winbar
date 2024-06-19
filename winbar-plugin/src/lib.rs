use std::ffi::c_char;

use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

#[cfg(feature = "impl")]
pub mod plugin;

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LoadConfigResult {
    pub ok: bool,
    pub error_msg: *const c_char,
}

/// A type representing a component's id.
///
/// Since plugins can be used to render multiple components, a component id is used to differentiate
/// each instance.
pub type ComponentId = u32;
/// A function returning the id of the plugin.
pub type FnId = unsafe extern "C" fn() -> *const c_char;
/// A function returning the width of the component.
pub type FnWidth = unsafe extern "C" fn(ComponentId, HWND, HDC) -> i32;
/// A function that draws the component within the given rect.
pub type FnDraw = unsafe extern "C" fn(ComponentId, HWND, PRect, HDC);

// lifecycle functions

/// A function called to load the configuration for a component. This function is called before the
/// start function.
pub type FnLoadConfig = unsafe extern "C" fn(ComponentId, *const c_char) -> LoadConfigResult;
/// A function called when a component should be started. Any processes or tasks required by the
/// component can be started in this function.
///
/// It can be assumed that this function will be called in its own thread.
pub type FnStart = unsafe extern "C" fn(ComponentId, HWND);
/// A function called when a component needs to be stopped. This will strictly only be called if the
/// start function is called beforehand. Note that start function's associated thread will be
/// terminated before this function is called.
pub type FnStop = unsafe extern "C" fn(ComponentId);
