use std::ops::{Add, RangeInclusive};

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
#[derive(Debug, Clone, Copy)]
pub struct CharRect {
    top: usize,
    bot: usize,
    left: usize,
    right: usize,
}

impl CharRect {
    /// row 11 is reserved for the page title. The page shouldn't draw on it
    pub const PAGE_AREA: Self = Self::new(12, WINDOW_SIZE_CHARS.1 - 1, 0, WINDOW_SIZE_CHARS.0 - 1);
    pub const HEADER_AREA: Self = Self::new(0, 10, 0, WINDOW_SIZE_CHARS.0 - 1);

    pub const fn new(top: usize, bot: usize, left: usize, right: usize) -> Self {
        assert!(top <= bot, "top needs to be smaller than bot");
        assert!(left <= right, "left needs to be smaller than right");
        assert!(bot < WINDOW_SIZE_CHARS.1, "lower than window bounds");
        assert!(right < WINDOW_SIZE_CHARS.0, "right out of window bounds");

        Self {
            top,
            bot,
            left,
            right,
        }
    }

    pub const fn top(self) -> usize {
        self.top
    }
    pub const fn bot(self) -> usize {
        self.bot
    }
    pub const fn right(self) -> usize {
        self.right
    }
    pub const fn left(self) -> usize {
        self.left
    }

    pub const fn top_left(self) -> CharPosition {
        CharPosition {
            x: self.left,
            y: self.top,
        }
    }

    pub const fn width(self) -> usize {
        self.right - self.left
    }

    pub const fn height(self) -> usize {
        self.bot - self.top
    }
}

/// uncheck conversion, because CharPosition is a safe type
impl From<CharPosition> for CharRect {
    fn from(value: CharPosition) -> Self {
        Self::new(value.y, value.y, value.x, value.x)
        // Self {
        //     top: value.y,
        //     bot: value.y,
        //     left: value.x,
        //     right: value.x,
        // }
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

    pub const fn vertical_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.top, self.bot)
    }

    pub const fn horizontal_range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.left, self.right)
    }

    pub const fn top(&self) -> usize {
        self.top
    }

    pub const fn bot(&self) -> usize {
        self.bot
    }

    pub const fn right(&self) -> usize {
        self.right
    }

    pub const fn left(&self) -> usize {
        self.left
    }
}

/// unchecked conversion because CharRect is a safe type
impl From<CharRect> for PixelRect {
    fn from(value: CharRect) -> Self {
        Self::new(
            value.top * FONT_SIZE,
            (value.bot * FONT_SIZE) + FONT_SIZE - 1,
            (value.right * FONT_SIZE) + FONT_SIZE - 1,
            value.left * FONT_SIZE,
        )
        // Self {
        //     top: value.top * FONT_SIZE,
        //     bot: (value.bot * FONT_SIZE) + FONT_SIZE - 1,
        //     right: (value.right * FONT_SIZE) + FONT_SIZE - 1,
        //     left: value.left * FONT_SIZE,
        // }
    }
}

/// unchecked conversion, because CharPosition is a safe type
impl From<CharPosition> for PixelRect {
    fn from(value: CharPosition) -> Self {
        Self::from(CharRect::from(value))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharPosition {
    x: usize,
    y: usize,
}

impl CharPosition {
    #[track_caller]
    pub const fn new(x: usize, y: usize) -> Self {
        assert!(y < WINDOW_SIZE_CHARS.1);
        assert!(x < WINDOW_SIZE_CHARS.0);

        Self { x, y }
    }

    pub const fn x(&self) -> usize {
        self.x
    }

    pub const fn y(&self) -> usize {
        self.y
    }
}

impl Add for CharPosition {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<(usize, usize)> for CharPosition {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Self::new(self.x + rhs.0, self.y + rhs.1)
    }
}

// impl Mul<usize> for CharPosition {
//     type Output = Self;

//     fn mul(self, rhs: usize) -> Self::Output {
//         Self::new(self.x * rhs, self.y * rhs)
//     }
// }
