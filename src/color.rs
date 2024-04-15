pub enum Color {
    Rgb { r: u32, g: u32, b: u32 },
    Hex(String),
}

impl Color {
    pub fn to_single_rgb(&self) -> u32 {
        match self {
            Self::Rgb { r, g, b } => 0x00 | b << 16 | g << 8 | r,
            Self::Hex(_hex) => unimplemented!("not yet implemented"),
        }
    }
}
