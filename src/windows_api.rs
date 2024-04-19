use windows::{
    core::HSTRING,
    Win32::{
        Foundation::COLORREF,
        Graphics::Gdi::{
            CreateFontW, CreatePen, CreateSolidBrush, SelectObject, SetBkColor, SetTextColor,
            ANSI_CHARSET, CLIP_DEFAULT_PRECIS, DEFAULT_PITCH, FF_DONTCARE, FW_DONTCARE, HDC,
            OUT_TT_PRECIS, PROOF_QUALITY, PS_SOLID,
        },
    },
};

use crate::{BACKGROUND, FONT_NAME, FOREGROUND};

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

            // let font = CreateFontIndirectW(&LOGFONTW {
            //     lfWeight: FW_NORMAL.0 as i32,
            //     lfQuality: FONT_QUALITY(PROOF_QUALITY.0),
            //     ..Default::default()
            // });

            let font = CreateFontW(
                18,
                0,
                0,
                0,
                FW_DONTCARE.0 as i32,
                0,
                0,
                0,
                ANSI_CHARSET.0.into(),
                OUT_TT_PRECIS.0.into(),
                CLIP_DEFAULT_PRECIS.0.into(),
                PROOF_QUALITY.0.into(),
                DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
                &HSTRING::from(FONT_NAME),
            );

            SelectObject(hdc, font);

            SetTextColor(hdc, COLORREF(FOREGROUND.to_single_rgb()));
        }
    }
}
