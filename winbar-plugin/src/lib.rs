use std::ffi::c_char;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::HDC,
};

#[cfg(feature = "impl")]
pub mod plugin;

/// A type representing a component's id.
///
/// Since plugins can be used to render multiple components, a component id is used to differentiate
/// each instance.
pub type ComponentId = u32;

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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum EventAction {
    Ignored,
    Handled,
    Intercept,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EventResult {
    pub action: EventAction,
    pub result: isize,
}

/// A function returning the id of the plugin.
pub type FnId = unsafe extern "C" fn() -> *const c_char;
/// A function returning the width of the component.
pub type FnWidth = unsafe extern "C" fn(ComponentId, HWND, HDC) -> i32;
/// A function that draws the component within the given rect.
pub type FnDraw = unsafe extern "C" fn(ComponentId, HWND, PRect, HDC);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WindowEvent {
    pub id: ComponentId,
    pub msg_code: u32,
    pub hwnd: HWND,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
    pub component_location: PRect,
}

/// A function that handles events received by the window.
///
/// The following resource is useful to look at when implementing this function.
/// https://learn.microsoft.com/en-us/windows/win32/learnwin32/writing-the-window-procedure
///
/// Note that the internal window process function proxies most of the events it receives to this
/// function. To provide the most customizability possible for plugins, there are very few
/// limitations imposed via code, however, there are a few rules that *should* be followed:
/// - When handling something in a specific location of the window, ensure it is within the boundary
/// for the associated component;
/// - Do not intercept events unless absolutely necessary. Events which are intercepted are not sent
/// to other components.
///
/// Of course, the above rules should be applied in a case-by-case basis. If it makes sense for your
/// plugin, then it could be okay to not follow one or more of the rules.
pub type FnHandleEvent = unsafe extern "C" fn(WindowEvent) -> EventResult;

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
