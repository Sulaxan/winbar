use async_trait::async_trait;
use windows::Win32::Foundation::{HWND, RECT};

pub mod datetime;
pub mod static_text;

#[async_trait]
pub trait Component {
    /// The width of the component.
    fn width(&self, hwnd: HWND) -> i32;

    /// Start any logic related to the component (e.g., a task to redraw).
    async fn start(&mut self, hwnd: HWND, rect: RECT);
}
