use std::time::Duration;

use async_trait::async_trait;
use chrono::Local;
use tokio::time::{self};
use winbar::{Component, WinbarAction, WinbarContext};
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetTextExtentPoint32W, RoundRect, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC,
    },
};

use crate::windows_api::WindowsApi;

pub struct DateTimeComponent {
    pub format: String,
}

impl DateTimeComponent {
    pub fn new(format: String) -> Self {
        Self { format }
    }
}

#[async_trait]
impl Component for DateTimeComponent {
    fn width(&self, _hwnd: HWND, hdc: HDC) -> i32 {
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

    fn draw(&self, _hwnd: HWND, mut rect: RECT, hdc: HDC) {
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

    async fn start(&self, ctx: WinbarContext, _hwnd: HWND, _rect: RECT) {
        let mut interval = time::interval(Duration::from_millis(500));
        loop {
            // first tick completes immediately
            interval.tick().await;
            match ctx.sender().send(WinbarAction::UpdateWindow) {
                Err(e) => {
                    tracing::error!("Could not send update window action over channel: {}", e);
                }
                _ => {}
            }
        }
    }
}
