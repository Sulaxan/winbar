use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use async_trait::async_trait;
use chrono::Local;
use tokio::time;
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetDC, GetTextExtentPoint32W, ReleaseDC, RoundRect, DT_CENTER, DT_SINGLELINE,
        DT_VCENTER,
    },
};

use crate::windows_api::WindowsApi;

use super::Component;

pub struct DateTimeComponent {
    pub format: String,
    update_thread: Option<JoinHandle<()>>,
}

impl DateTimeComponent {
    pub fn new(format: String) -> Self {
        Self {
            format: format,
            update_thread: None,
        }
    }

    fn draw(hwnd: HWND, rect: &mut RECT, datetime: &str) {
        unsafe {
            let hdc = GetDC(hwnd);
            WindowsApi::set_default_styles(hdc);

            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(datetime),
                rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );

            ReleaseDC(hwnd, hdc);
        }
    }
}

#[async_trait]
impl Component for DateTimeComponent {
    fn width(&self, hwnd: HWND) -> i32 {
        let time = Local::now();
        let formatted_time = time.format(&self.format).to_string();
        drop(time);

        unsafe {
            let hdc = GetDC(hwnd);
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(
                hdc,
                &WindowsApi::str_to_u16_slice(&formatted_time),
                &mut length,
            );

            length.cx + 20 // padding
        }
    }

    async fn start(&mut self, hwnd: HWND, mut rect: RECT) {
        if self.update_thread.is_some() {
            return;
        }

        loop {
            Self::draw(
                hwnd,
                &mut rect,
                &Local::now().format(&self.format).to_string(),
            );
            time::sleep(Duration::from_millis(500)).await;
        }
    }
}
