use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetDC, GetTextExtentPoint32W, ReleaseDC, RoundRect, DT_CENTER, DT_SINGLELINE,
        DT_VCENTER,
    },
};

use crate::windows_api::WindowsApi;

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

impl Component for StaticTextComponent {
    fn width(&self, hwnd: HWND) -> i32 {
        unsafe {
            let hdc = GetDC(hwnd);
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(hdc, &WindowsApi::str_to_u16_slice(&self.text), &mut length);

            length.cx + PADDING_X * 2
        }
    }

    fn draw(&self, hwnd: HWND, rect: &mut RECT) {
        unsafe {
            let hdc = GetDC(hwnd);
            WindowsApi::set_default_styles(hdc);

            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&self.text),
                rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );

            ReleaseDC(hwnd, hdc);
        }
    }

    fn start(&mut self, _hwnd: HWND, _rect: &mut RECT) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}
