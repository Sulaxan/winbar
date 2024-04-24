use std::mem::MaybeUninit;

use async_trait::async_trait;
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::{
        Gdi::{DrawTextW, GetTextExtentPoint32W, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC},
        GdiPlus::{
            GdipCreateFromHDC, GdipCreatePen1, GdipFillRectangle, GdipGetPenBrushFill, UnitPixel,
        },
    },
};

use crate::{winbar::WinbarContext, windows_api::WindowsApi, DEFAULT_BG_COLOR};

use super::Component;

const PADDING_X: i32 = 10;

pub struct StaticTextComponent {
    text: String,
}

impl StaticTextComponent {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[async_trait]
impl Component for StaticTextComponent {
    fn width(&self, _hwnd: HWND, hdc: HDC) -> i32 {
        unsafe {
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(hdc, &WindowsApi::str_to_u16_slice(&self.text), &mut length);

            length.cx + PADDING_X * 2
        }
    }

    fn draw(&self, _hwnd: HWND, mut rect: RECT, hdc: HDC) {
        let default_bg_color = {
            let color = DEFAULT_BG_COLOR.lock().unwrap();
            color.argb()
        };

        unsafe {
            let mut graphics = MaybeUninit::uninit();
            GdipCreateFromHDC(hdc, graphics.as_mut_ptr());

            // based on: https://github.com/davidrios/gdiplus-rs
            let g = graphics.assume_init();

            let mut bg_pen = MaybeUninit::uninit();
            GdipCreatePen1(default_bg_color, 1.0, UnitPixel, bg_pen.as_mut_ptr());

            let pen = bg_pen.assume_init();

            let mut bg_brush = MaybeUninit::uninit();
            GdipGetPenBrushFill(pen, bg_brush.as_mut_ptr());

            let brush = bg_brush.assume_init();

            GdipFillRectangle(
                g,
                brush,
                rect.left as f32,
                rect.top as f32,
                (rect.right - rect.left) as f32,
                (rect.bottom - rect.top) as f32,
            );

            // RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);

            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&self.text),
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
    }

    async fn start(&self, _ctx: WinbarContext, _hwnd: HWND, _rect: RECT) {}
}
