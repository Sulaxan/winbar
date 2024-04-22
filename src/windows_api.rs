use std::mem::MaybeUninit;

use anyhow::{bail, Result};
use tracing::instrument;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::COLORREF,
        Graphics::{
            Gdi::{
                CreateFontW, CreatePen, CreateSolidBrush, SelectObject, SetBkColor, SetTextColor,
                ANSI_CHARSET, CLIP_DEFAULT_PRECIS, DEFAULT_PITCH, FF_DONTCARE, FW_DONTCARE, HDC,
                OUT_TT_PRECIS, PROOF_QUALITY, PS_SOLID,
            },
            GdiPlus::{GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, Status},
        },
    },
};

use crate::{DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT};

pub struct WindowsApi {}

impl WindowsApi {
    pub fn str_to_u16_slice(s: &str) -> Vec<u16> {
        s.encode_utf16().collect::<Vec<u16>>()
    }

    pub fn set_default_styles(hdc: HDC) {
        unsafe {
            let pen = CreatePen(PS_SOLID, 0, COLORREF(DEFAULT_BG_COLOR.bgr()));
            let brush = CreateSolidBrush(COLORREF(DEFAULT_BG_COLOR.bgr()));

            SelectObject(hdc, pen);
            SelectObject(hdc, brush);
            SetBkColor(hdc, COLORREF(DEFAULT_BG_COLOR.bgr()));
            // SetBkColor(hdc, COLORREF(TRANSPARENT_COLOR));

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
                &HSTRING::from(DEFAULT_FONT),
            );

            SelectObject(hdc, font);

            SetTextColor(hdc, COLORREF(DEFAULT_FG_COLOR.bgr()));
        }
    }

    // inspired from: https://github.com/davidrios/gdiplus-rs
    #[instrument(name = "windows_api_gdi+_init")]
    pub fn startup_gdiplus() -> Result<usize> {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            SuppressBackgroundThread: false.into(),
            SuppressExternalCodecs: false.into(),
            ..Default::default()
        };

        let mut token: usize = 0;
        let mut output = MaybeUninit::uninit();
        unsafe {
            let status = GdiplusStartup(&mut token, &input, output.as_mut_ptr());
            if status != Status(0) {
                bail!("GDI+ startup returned non-zero: {:?}", status);
            }
        }

        Ok(token)
    }

    pub fn shutdown_gdiplus(token: usize) {
        unsafe {
            GdiplusShutdown(token);
        }
    }
}
