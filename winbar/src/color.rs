use serde::{Deserialize, Serialize};

pub const TRANSPARENT_COLOR: u32 = 0;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Rgb { r: u32, g: u32, b: u32 },
    Argb { r: u32, g: u32, b: u32, alpha: u32 },
    Hex(String),
    Transparent,
}

impl Color {
    /// Returns the BGR value, where blue is encoded in the most significant bits, followed by
    /// green, and red.
    pub fn bgr(&self) -> u32 {
        match self {
            Self::Rgb { r, g, b } => b << 16 | g << 8 | r,
            Self::Argb { r, g, b, alpha: _ } => b << 16 | g << 8 | r,
            Self::Hex(_hex) => unimplemented!("not yet implemented"),
            Self::Transparent => TRANSPARENT_COLOR,
        }
    }

    /// Returns the ARGB value, where alpha is encoded in the most significant bits, followed by
    /// red, green, and blue.
    pub fn argb(&self) -> u32 {
        match self {
            // 0xFF alpha = opaque
            Self::Rgb { r, g, b } => 0xFF << 24 | r << 16 | g << 8 | b,
            Self::Argb { r, g, b, alpha } => alpha << 24 | r << 16 | g << 8 | b,
            Self::Hex(_hex) => unimplemented!("not yet implemented"),
            Self::Transparent => TRANSPARENT_COLOR,
        }
    }
}
