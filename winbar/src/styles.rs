use windows::{
    core::HSTRING,
    Win32::{
        Foundation::COLORREF,
        Graphics::Gdi::{
            CreateFontW, CreatePen, CreateSolidBrush, ANSI_CHARSET, CLIP_DEFAULT_PRECIS,
            DEFAULT_PITCH, FF_DONTCARE, FW_DONTCARE, HBRUSH, HFONT, HPEN, OUT_TT_PRECIS, PEN_STYLE,
            PROOF_QUALITY,
        },
    },
};

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
}
