use windows::Win32::Foundation::RECT;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    /// The x value of the top left corner of the rect.
    pub x: i32,
    /// The y value of the top left corner of the rect.
    pub y: i32,
    /// The width of the rect.
    pub width: i32,
    /// The height of the rect.
    pub height: i32,
}

impl Rect {
    /// Returns x value with the width added.
    pub fn x2(&self) -> i32 {
        self.x + self.width
    }

    /// Returns y value with the height added.
    pub fn y2(&self) -> i32 {
        self.y + self.height
    }
}

impl From<Rect> for RECT {
    fn from(value: Rect) -> Self {
        Self {
            left: value.x,
            top: value.y,
            right: value.x2(),
            bottom: value.y2(),
        }
    }
}
