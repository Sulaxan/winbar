use std::sync::Arc;

use async_trait::async_trait;
use winbar_core::{
    styles::{StyleOptions, Styles}, util::rect::Rect, windows_api::WindowsApi, Component, EventResult, WinbarContext, WindowEvent, IGNORED_EVENT_RESULT
};
use windows::Win32::{
    Foundation::{HWND, SIZE},
    Graphics::Gdi::{DrawTextW, GetTextExtentPoint32W, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC},
};

pub struct StaticTextComponent {
    text: String,
    styles: Arc<StyleOptions>,
}

impl StaticTextComponent {
    pub fn new(text: String, styles: StyleOptions) -> Self {
        Self {
            text,
            styles: Arc::new(styles),
        }
    }
}

#[async_trait]
impl Component for StaticTextComponent {
    fn styles(&self) -> Arc<StyleOptions> {
        self.styles.clone()
    }

    fn width(&self, _hwnd: HWND, hdc: HDC) -> i32 {
        unsafe {
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(hdc, &WindowsApi::str_to_u16_slice(&self.text), &mut length);

            length.cx + self.styles.padding_x * 2
        }
    }

    fn draw(&self, _hwnd: HWND, rect: Rect, hdc: HDC) {
        Styles::draw_rect(hdc, &rect, &self.styles.border_style);

        unsafe {
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

            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&self.text),
                &mut rect.into(),
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );

            // GdipDeleteBrush(brush);
            // GdipDeletePen(pen);
            // GdipDeleteGraphics(g);
        }
    }

    fn start(&self, _ctx: WinbarContext, _hwnd: HWND) {}

    fn handle_event(&self, _event: WindowEvent) -> EventResult {
        IGNORED_EVENT_RESULT
    }
}
