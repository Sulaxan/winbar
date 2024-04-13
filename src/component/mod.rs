use windows::Win32::Foundation::{HWND, RECT};

pub mod static_text;

pub trait Component {
    /// The width of the component.
    fn width(&self, hwnd: HWND) -> i32;

    /// Draw the component within the specified rect. Note that some default styles will be set for
    /// the window.
    fn draw(&self, hwnd: HWND, rect: &mut RECT);

    /// Start any logic related to the component (e.g., a task to redraw).
    fn start(&mut self, hwnd: HWND, rect: &mut RECT);

    /// Stop any logic related to the component. Anything that must be disposed should be disposed
    /// within this method.
    fn stop(&mut self);
}
