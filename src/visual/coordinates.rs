use std::ops::{Add, Mul, RangeInclusive};

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

    // pub const fn from_pos(x: usize, y: usize) -> Self {
    //     assert!(y < WINDOW_SIZE_CHARS.1, "lower than window bounds");
    //     assert!(x < WINDOW_SIZE_CHARS.0, "right out of window bounds");

    //     Self {
    //         top: y,
    //         bot: y,
    //         left: x,
    //         right: x,
    //     }
    // }

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

    pub fn width(&self) -> usize {
        self.right - self.left
    }
}

// impl From<(usize, usize)> for CharRect {
//     fn from(value: (usize, usize)) -> Self {
//         Self::from_pos(value.0, value.1)
//     }
// }

/// uncheck conversion, because CharPosition is a safe type
impl From<CharPosition> for CharRect {
    fn from(value: CharPosition) -> Self {
        Self {
            top: value.y,
            bot: value.y,
            left: value.x,
            right: value.x,
        }
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

impl PixelRect {
    pub const fn new(top: usize, bot: usize, right: usize, left: usize) -> Self {
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

/// unchecked conversion because CharRect is a safe type
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

/// unchecked conversion, because CharPosition is a safe type
impl From<CharPosition> for PixelRect {
    fn from(value: CharPosition) -> Self {
        Self::from(CharRect::from(value))
    }
}

#[derive(Clone, Copy)]
pub struct CharPosition {
    x: usize,
    y: usize,
}

impl CharPosition {
    pub const fn new(x: usize, y: usize) -> Self {
        assert!(y < WINDOW_SIZE.0);
        assert!(x < WINDOW_SIZE.1);

        Self { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }
}

impl Add for CharPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<(usize, usize)> for CharPosition {
    type Output = Self;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Self::new(self.x + rhs.0, self.y + rhs.1)
    }
}

impl Mul<usize> for CharPosition {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
