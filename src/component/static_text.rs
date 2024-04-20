use async_trait::async_trait;
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetTextExtentPoint32W, RoundRect, DT_CENTER, DT_SINGLELINE,
        DT_VCENTER, HDC,
    },
};

use crate::{winbar::WinbarContext, windows_api::WindowsApi};

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
        unsafe {
            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
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
