use std::ops::RangeInclusive;

/// font size in pixel. font is a square
pub const FONT_SIZE: usize = 8;
/// window size in characters
pub const WINDOW_SIZE: (usize, usize) = (FONT_SIZE * 80, FONT_SIZE * 50);
/// bytes per pixel
pub const PIXEL_SIZE: usize = 4;
/// bytes per pixel line
// pub const LINE_SIZE: usize = WINDOW_SIZE.0;
// pub const LINE_SIZE: usize = PIXEL_SIZE * WINDOW_SIZE.0;
/// bytes per Character line
// pub const CHAR_LINE_SIZE: usize = LINE_SIZE * FONT_SIZE;

/// CharRect as well as PixelRect uses all values inclusive, meaning the borders are included
#[derive(Clone, Copy)]
pub struct CharRect {
    pub top: usize,
    pub bot: usize,
    pub left: usize,
    pub right: usize,
}

impl CharRect {
    pub fn new(top: usize, bot: usize, left: usize, right: usize) -> Self {
        assert!(top <= bot, "top needs to be smaller than bot");
        assert!(left <= right, "left needs to be smaller than right");
        Self {
            top,
            bot,
            right,
            left,
        }
    }
}

/// PixelRect as well as CharRect uses all values inclusive, meaning the borders are included
#[derive(Debug, Clone, Copy)]
pub struct PixelRect {
    pub top: usize,
    pub bot: usize,
    pub right: usize,
    pub left: usize,
}

impl From<CharRect> for PixelRect {
    fn from(value: CharRect) -> Self {
        Self {
            top: value.top * FONT_SIZE,
            bot: (value.bot * FONT_SIZE) + FONT_SIZE - 1,
            right: (value.right * FONT_SIZE) + FONT_SIZE - 1,
            left: value.left * FONT_SIZE,
        }
    }
}

impl PixelRect {
    pub fn vertical_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.top, self.bot)
    }

    pub fn horizontal_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.left, self.right)
    }
}
