use std::sync::atomic::Ordering;

use async_trait::async_trait;
use winbar::{util::rect::Rect, Component, WinbarContext};
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

pub struct StaticTextComponent {
    text: String,
    styles: StyleOptions,
}

impl StaticTextComponent {
    pub fn new(text: String, styles: StyleOptions) -> Self {
        Self { text, styles }
    }
}

#[async_trait]
impl Component for StaticTextComponent {
    fn width(&self, _hwnd: HWND, hdc: HDC) -> i32 {
        unsafe {
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(hdc, &WindowsApi::str_to_u16_slice(&self.text), &mut length);

            length.cx + self.styles.padding_x * 2
        }
    }

    fn draw(&self, _hwnd: HWND, rect: Rect, hdc: HDC) {
        // let default_bg_color = {
        //     let color = DEFAULT_BG_COLOR.lock().unwrap();
        //     color.argb()
        // };
        let font = match &self.styles.font {
            Some(font) => font.to_string(),
            None => {
                let font = DEFAULT_FONT.lock().unwrap();
                font.to_string()
            }
        };
        let font_size = match &self.styles.font_size {
            Some(size) => *size,
            None => DEFAULT_FONT_SIZE.load(Ordering::SeqCst),
        };
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

            // let mut graphics = MaybeUninit::uninit();
            // GdipCreateFromHDC(hdc, graphics.as_mut_ptr());

            // // based on: https://github.com/davidrios/gdiplus-rs
            // let g = graphics.assume_init();

            // let mut bg_pen = MaybeUninit::uninit();
            // GdipCreatePen1(default_bg_color, 1.0, UnitPixel, bg_pen.as_mut_ptr());

            // let pen = bg_pen.assume_init();

            // let mut bg_brush = MaybeUninit::uninit();
            // GdipGetPenBrushFill(pen, bg_brush.as_mut_ptr());

            // let brush = bg_brush.assume_init();

            // GdipFillRectangleI(g, brush, rect.x, rect.y, rect.x2(), rect.y2());

            Styles::draw_rect(hdc, &rect, &self.styles.border_style);

            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&self.text),
                &mut rect.into(),
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );

            DeleteObject(pen);
            DeleteObject(brush);
            DeleteObject(font);

            // GdipDeleteBrush(brush);
            // GdipDeletePen(pen);
            // GdipDeleteGraphics(g);
        }
    }

    async fn start(&self, _ctx: WinbarContext, _hwnd: HWND, _rect: Rect) {}
}
