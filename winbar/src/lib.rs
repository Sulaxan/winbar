use std::sync::mpsc::Sender;

use async_trait::async_trait;
use getset::Getters;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HDC,
};

pub mod client;
pub mod color;
pub mod protocol;

pub enum WinbarAction {
    UpdateWindow,
    Shutdown,
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

#[async_trait]
pub trait Component {
    /// The width of the component.
    fn width(&self, hwnd: HWND, hdc: HDC) -> i32;

    fn draw(&self, hwnd: HWND, rect: RECT, hdc: HDC);

    /// Start any logic related to the component (e.g., a task to UpdateDraw).
    //TODO: Make this non-mut so that we no longer need to take in a mutex
    async fn start(&self, ctx: WinbarContext, hwnd: HWND, rect: RECT);
}
