use std::{
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
    time::Duration,
};

use async_trait::async_trait;
use chrono::Local;
use tokio::time::{self, Interval};
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetDC, GetTextExtentPoint32W, ReleaseDC, RoundRect, UpdateWindow, DT_CENTER,
        DT_SINGLELINE, DT_VCENTER, HDC,
    },
};

use crate::{
    winbar::{WinbarAction, WinbarContext},
    windows_api::WindowsApi,
};

use super::Component;

pub struct DateTimeComponent {
    pub format: String,
}

impl DateTimeComponent {
    pub fn new(format: String) -> Self {
        Self { format: format }
    }
}

#[async_trait]
impl Component for DateTimeComponent {
    fn width(&self, hwnd: HWND, hdc: HDC) -> i32 {
        let time = Local::now();
        let formatted_time = time.format(&self.format).to_string();

        unsafe {
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(
                hdc,
                &WindowsApi::str_to_u16_slice(&formatted_time),
                &mut length,
            );

            length.cx + 20 // padding
        }
    }

    fn draw(&self, hwnd: HWND, mut rect: RECT, hdc: HDC) {
        println!("a");
        let time = Local::now();
        let formatted = time.format(&self.format).to_string();

        unsafe {
            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&formatted),
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
    }

    async fn start(&mut self, ctx: WinbarContext, hwnd: HWND, rect: RECT) {
        let mut interval = time::interval(Duration::from_millis(500));
        loop {
            // first tick completes immediately
            interval.tick().await;
            ctx.sender().send(WinbarAction::UpdateWindow).unwrap();
        }
    }
}
