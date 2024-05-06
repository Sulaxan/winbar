use std::{sync::atomic::Ordering, time::Duration};

use async_trait::async_trait;
use chrono::Local;
use tokio::time::{self};
use winbar::{util::rect::Rect, Component, WinbarAction, WinbarContext};
use windows::Win32::{
    Foundation::{COLORREF, HWND, SIZE},
    Graphics::Gdi::{
        DeleteObject, DrawTextW, GetTextExtentPoint32W, SelectObject, SetBkColor, SetTextColor,
        DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC, PS_SOLID,
    },
};

use crate::{
    styles::{StyleOptions, Styles},
    windows_api::WindowsApi,
    DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT, DEFAULT_FONT_SIZE,
};

pub struct DateTimeComponent {
    pub format: String,
    pub styles: StyleOptions,
}

impl DateTimeComponent {
    pub fn new(format: String, styles: StyleOptions) -> Self {
        Self { format, styles }
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

            length.cx + self.styles.padding_x * 2
        }
    }

    fn draw(&self, _hwnd: HWND, rect: Rect, hdc: HDC) {
        let time = Local::now();
        let formatted = time.format(&self.format).to_string();

        let font = {
            let font = DEFAULT_FONT.lock().unwrap();
            font.to_string()
        };
        let font_size = DEFAULT_FONT_SIZE.load(Ordering::SeqCst);
        let bg_color = match &self.styles.bg_color {
            Some(color) => color.bgr(),
            None => {
                let color = DEFAULT_BG_COLOR.lock().unwrap();
                color.bgr()
            }
        };
        let fg_color = match &self.styles.fg_color {
            Some(color) => color.bgr(),
            None => {
                let color = DEFAULT_FG_COLOR.lock().unwrap();
                color.bgr()
            }
        };

        unsafe {
            let pen = Styles::pen(bg_color, PS_SOLID);
            let brush = Styles::solid_brush(bg_color);
            let font = Styles::font(font_size, &font);

            SelectObject(hdc, pen);
            SelectObject(hdc, brush);
            SelectObject(hdc, font);
            SetBkColor(hdc, COLORREF(bg_color));
            SetTextColor(hdc, COLORREF(fg_color));

            Styles::draw_rect(hdc, &rect, &self.styles.border_style);

            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&formatted),
                &mut rect.into(),
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
            println!("Drawn");

            DeleteObject(pen);
            DeleteObject(brush);
            DeleteObject(font);
        }
    }

    async fn start(&self, ctx: WinbarContext, _hwnd: HWND, _rect: Rect) {
        let mut interval = time::interval(Duration::from_millis(500));
        loop {
            // first tick completes immediately
            interval.tick().await;
            if let Err(e) = ctx.sender().send(WinbarAction::UpdateWindow) {
                tracing::error!("Could not send update window action over channel: {}", e);
            }
        }
    }
}
