use std::sync::{mpsc::Sender, Arc};

use async_trait::async_trait;
use getset::Getters;
use styles::StyleOptions;
use util::rect::Rect;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::HDC,
};

pub mod client;
pub mod color;
pub mod protocol;
pub mod styles;
pub mod util;
pub mod windows_api;

pub const DEFAULT_PORT: i32 = 10989;
pub const DEFAULT_HOSTNAME: &str = "localhost";

pub enum WinbarAction {
    Shutdown,
    UpdateWindow,
    ShowWindow,
    HideWindow,
}

#[derive(Getters, Clone)]
pub struct WinbarContext {
    #[getset(get = "pub")]
    sender: Sender<WinbarAction>,
}

impl WinbarContext {
    pub fn new(sender: Sender<WinbarAction>) -> Self {
        Self { sender }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventAction {
    Ignored,
    Handled,
    Intercept,
}

#[derive(Clone, Copy, Debug)]
pub struct EventResult {
    pub action: EventAction,
    pub result: isize,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowEvent {
    pub msg_code: u32,
    pub hwnd: HWND,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
    pub component_location: Rect,
}

#[async_trait]
pub trait Component {
    fn styles(&self) -> Arc<StyleOptions>;

    /// The width of the component.
    fn width(&self, hwnd: HWND, hdc: HDC) -> i32;

    /// Draw the component. Note this this method is responsible for cleanup of any objects it
    /// creates.
    fn draw(&self, hwnd: HWND, rect: Rect, hdc: HDC);

    /// Start any logic related to the component (e.g., a task to UpdateDraw).
    fn start(&self, ctx: WinbarContext, hwnd: HWND);

    /// Handle a window event.
    ///
    /// The following resource is useful to look at when implementing this function.
    /// https://learn.microsoft.com/en-us/windows/win32/learnwin32/writing-the-window-procedure
    ///
    /// Note that the internal window process function proxies most of the events it receives to
    /// this function.
    fn handle_event(&self, event: WindowEvent);
}
