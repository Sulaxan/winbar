use std::{sync::Arc, thread, time::Duration};

use async_trait::async_trait;
use chrono::Local;
use winbar_core::{
    styles::{StyleOptions, Styles},
    util::rect::Rect,
    windows_api::WindowsApi,
    Component, WinbarAction, WinbarContext, WindowEvent,
};
use windows::Win32::{
    Foundation::{HWND, SIZE},
    Graphics::Gdi::{DrawTextW, GetTextExtentPoint32W, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC},
};

pub struct DateTimeComponent {
    pub format: String,
    pub styles: Arc<StyleOptions>,
}

impl DateTimeComponent {
    pub fn new(format: String, styles: StyleOptions) -> Self {
        Self {
            format,
            styles: Arc::new(styles),
        }
    }
}

#[async_trait]
impl Component for DateTimeComponent {
    fn styles(&self) -> Arc<StyleOptions> {
        self.styles.clone()
    }

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

            length.cx + self.styles.padding_x * 2
        }
    }

    fn draw(&self, _hwnd: HWND, rect: Rect, hdc: HDC) {
        let time = Local::now();
        let formatted = time.format(&self.format).to_string();

        Styles::draw_rect(hdc, &rect, &self.styles.border_style);
        unsafe {
            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&formatted),
                &mut rect.into(),
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
    }

    fn start(&self, ctx: WinbarContext, _hwnd: HWND) {
        loop {
            if let Err(e) = ctx.sender().send(WinbarAction::UpdateWindow) {
                tracing::error!("Could not send update window action over channel: {}", e);
            }
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn handle_event(&self, _event: WindowEvent) {}
}
