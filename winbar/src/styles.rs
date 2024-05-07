use serde::{Deserialize, Serialize};
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::COLORREF,
        Graphics::Gdi::{
            CreateFontW, CreatePen, CreateSolidBrush, RoundRect, ANSI_CHARSET, CLIP_DEFAULT_PRECIS,
            DEFAULT_PITCH, FF_DONTCARE, FW_DONTCARE, HBRUSH, HDC, HFONT, HPEN, OUT_TT_PRECIS,
            PEN_STYLE, PROOF_QUALITY,
        },
    },
};

use crate::{color::Color, util::rect::Rect};

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum BorderStyle {
    #[default]
    Square,
    Rounded {
        radius: i32,
    },
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct StyleOptions {
    pub bg_color: Option<Color>,
    pub fg_color: Option<Color>,
    #[serde(default)]
    pub border_style: BorderStyle,
    pub font: Option<String>,
    pub font_size: Option<i32>,
    #[serde(default)]
    pub padding_x: i32,
}

pub struct Styles {}

impl Styles {
    /// Creates a new pen.
    ///
    /// Note that it is the caller's responsibility to call SelectObject to use the object, and
    /// DeleteObject to cleanup the resource.
    pub fn pen(color: u32, style: PEN_STYLE) -> HPEN {
        unsafe { CreatePen(style, 0, COLORREF(color)) }
    }

    /// Creates a new solid brush.
    ///
    /// Note that it is the caller's responsibility to call SelectObject to use the object, and
    /// DeleteObject to cleanup the resource.
    pub fn solid_brush(color: u32) -> HBRUSH {
        unsafe { CreateSolidBrush(COLORREF(color)) }
    }

    /// Creates a new font.
    ///
    /// Note that it is the caller's responsibility to call SelectObject to use the object, and
    /// DeleteObject to cleanup the resource.
    pub fn font(size: i32, name: &str) -> HFONT {
        unsafe {
            CreateFontW(
                size,
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
                &HSTRING::from(name),
            )
        }
    }

    pub fn draw_rect(hdc: HDC, rect: &Rect, border: &BorderStyle) {
        unsafe {
            match border {
                BorderStyle::Square => {
                    RoundRect(hdc, rect.x, rect.y, rect.x2(), rect.y2(), 0, 0);
                }
                BorderStyle::Rounded { radius } => {
                    RoundRect(hdc, rect.x, rect.y, rect.x2(), rect.y2(), *radius, *radius);
                }
            }
        }
    }
}
