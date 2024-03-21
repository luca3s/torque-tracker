/// font size in pixel. font is a square
pub const FONT_SIZE: usize = 8;
/// window size in characters
pub const WINDOW_SIZE: (usize, usize) = (FONT_SIZE * 80, FONT_SIZE * 50);
/// bytes per pixel
pub const PIXEL_SIZE: usize = 4;
/// bytes per screen line
pub const LINE_SIZE: usize = PIXEL_SIZE * WINDOW_SIZE.0;

#[inline]
pub const fn char_into_screen_pos(position: u8) -> usize {
    position as usize * FONT_SIZE
}

#[inline]
pub const fn screen_into_byte_pos(position: (usize, usize)) -> usize {
    (position.1 * LINE_SIZE) + (position.0 * PIXEL_SIZE)
}

#[derive(Clone, Copy)]
pub struct CharRect {
    top: u8,
    bot: u8,
    left: u8,
    right: u8,
}

impl CharRect {
    pub fn new(top: u8, bot: u8, left: u8, right: u8) -> Self {
        assert!(top <= bot, "top needs to be smaller than bot");
        assert!(left <= right, "left needs to be smaller than right");
        Self {
            top,
            bot,
            right,
            left,
        }
    }

    pub fn top(&self) -> u8 {
        self.top
    }

    pub fn bot(&self) -> u8 {
        self.bot
    }

    pub fn left(&self) -> u8 {
        self.left
    }

    pub fn right(&self) -> u8 {
        self.right
    }
    
    // pub fn width(&self) -> u8 {
    //     self.bot - self.top
    // }

    // pub fn height(&self) -> u8 {
    //     self.right - self.left
    // }
}

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
            top: char_into_screen_pos(value.top),
            bot: char_into_screen_pos(value.bot+1),
            right: char_into_screen_pos(value.right+1),
            left: char_into_screen_pos(value.left),
        }
    }
}
