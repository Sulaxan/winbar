use async_trait::async_trait;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HDC,
};

use crate::winbar::WinbarContext;

pub mod datetime;
pub mod manager;
pub mod static_text;

#[async_trait]
pub trait Component {
    /// The width of the component.
    fn width(&self, hwnd: HWND, hdc: HDC) -> i32;

    fn draw(&self, hwnd: HWND, rect: RECT, hdc: HDC);

    /// Start any logic related to the component (e.g., a task to UpdateDraw).
    //TODO: Make this non-mut so that we no longer need to take in a mutex
    async fn start(&self, ctx: WinbarContext, hwnd: HWND, rect: RECT);
}
