use std::ops::RangeInclusive;

/// font size in pixel. font is a square
pub const FONT_SIZE: usize = 8;
/// window size in characters
pub const WINDOW_SIZE_CHARS: (usize, usize) = (80, 50);
/// window size in pixel
pub const WINDOW_SIZE: (usize, usize) = (
    FONT_SIZE * WINDOW_SIZE_CHARS.0,
    FONT_SIZE * WINDOW_SIZE_CHARS.1,
);
/// bytes per pixel, mainly used to setup GPU as otherwise i use a u32 for one pixel
pub const PIXEL_SIZE: usize = 4;

/// CharRect as well as PixelRect uses all values inclusive, meaning the borders are included
#[derive(Clone, Copy)]
pub struct CharRect {
    top: usize,
    bot: usize,
    left: usize,
    right: usize,
}

impl CharRect {
    pub const PAGE_AREA: Self = Self::new(11, WINDOW_SIZE_CHARS.1 - 1, 0, WINDOW_SIZE_CHARS.0 - 1);
    pub const HEADER_AREA: Self = Self::new(0, 10, 0, WINDOW_SIZE_CHARS.0 - 1);

    pub const fn new(top: usize, bot: usize, left: usize, right: usize) -> Self {
        assert!(top <= bot, "top needs to be smaller than bot");
        assert!(left <= right, "left needs to be smaller than right");
        assert!(bot < WINDOW_SIZE_CHARS.1, "lower than window bounds");
        assert!(right < WINDOW_SIZE_CHARS.0, "right out of window bounds");

        Self {
            top,
            bot,
            right,
            left,
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }
    pub fn bot(&self) -> usize {
        self.bot
    }
    pub fn right(&self) -> usize {
        self.right
    }
    pub fn left(&self) -> usize {
        self.left
    }
}

/// PixelRect as well as CharRect uses all values inclusive, meaning the borders are included
#[derive(Debug, Clone, Copy)]
pub struct PixelRect {
    top: usize,
    bot: usize,
    right: usize,
    left: usize,
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
    pub fn new(top: usize, bot: usize, right: usize, left: usize) -> Self {
        assert!(top <= bot, "top needs to be smaller than bot");
        assert!(left <= right, "left needs to be smaller than right");
        assert!(bot < WINDOW_SIZE.1, "lower than window bounds");
        assert!(right < WINDOW_SIZE.0, "right out of window bounds");

        Self {
            top,
            bot,
            right,
            left,
        }
    }

    pub fn vertical_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.top, self.bot)
    }

    pub fn horizontal_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.left, self.right)
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bot(&self) -> usize {
        self.bot
    }

    pub fn right(&self) -> usize {
        self.right
    }

    pub fn left(&self) -> usize {
        self.left
    }
}
