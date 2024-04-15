use windows::Win32::{
    Foundation::COLORREF,
    Graphics::Gdi::{
        CreateFontIndirectW, CreatePen, CreateSolidBrush, SelectObject, SetBkColor, SetTextColor,
        FONT_QUALITY, FW_NORMAL, HDC, LOGFONTW, PROOF_QUALITY, PS_SOLID,
    },
};

use crate::{BACKGROUND, FOREGROUND};

pub struct WindowsApi {}

impl WindowsApi {
    pub fn str_to_u16_slice(s: &str) -> Vec<u16> {
        s.encode_utf16().collect::<Vec<u16>>()
    }

    pub fn set_default_styles(hdc: HDC) {
        unsafe {
            let pen = CreatePen(PS_SOLID, 0, COLORREF(BACKGROUND.to_single_rgb()));
            let brush = CreateSolidBrush(COLORREF(BACKGROUND.to_single_rgb()));

            SelectObject(hdc, pen);
            SelectObject(hdc, brush);
            SetBkColor(hdc, COLORREF(BACKGROUND.to_single_rgb()));

            let font = CreateFontIndirectW(&LOGFONTW {
                lfWeight: FW_NORMAL.0 as i32,
                lfQuality: FONT_QUALITY(PROOF_QUALITY.0),
                ..Default::default()
            });

            SelectObject(hdc, font);

            SetTextColor(hdc, COLORREF(FOREGROUND.to_single_rgb()));
        }
    }
}
