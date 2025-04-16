use crate::palettes;

use super::{
    coordinates::{CharPosition, CharRect, PixelRect, FONT_SIZE, WINDOW_SIZE},
    palettes::{Palette, RGB10A2},
};
use font8x8::UnicodeFonts;

pub struct DrawBuffer {
    pub framebuffer: Box<[[u32; WINDOW_SIZE.0]; WINDOW_SIZE.1]>,
    color_palette: Palette<RGB10A2>,
}

impl Default for DrawBuffer {
    fn default() -> Self {
        Self {
            framebuffer: Box::new([[0; WINDOW_SIZE.0]; WINDOW_SIZE.1]),
            color_palette: Palette::CAMOUFLAGE.into(),
        }
    }
}

impl DrawBuffer {
    pub const BACKGROUND_COLOR: u8 = 2;

    pub fn new() -> Self {
        Self {
            framebuffer: Box::new([[0; WINDOW_SIZE.0]; WINDOW_SIZE.1]),
            color_palette: Palette::CAMOUFLAGE.into(),
        }
    }

    pub fn get_raw_color(&self, index: u8) -> u32 {
        self.color_palette.get_raw(index)
    }

    pub fn draw_char(
        &mut self,
        char_data: [u8; 8],
        position: CharPosition,
        fg_color: u8,
        bg_color: u8,
    ) {
        // this is the top_left pixel
        let position = (position.x() * FONT_SIZE, position.y() * FONT_SIZE);
        let fg_color = self.color_palette.get_raw(fg_color);
        let bg_color = self.color_palette.get_raw(bg_color);
        for (y, line) in char_data.iter().enumerate() {
            for x in 0..8 {
                let color = match (line >> x) & 1 == 1 {
                    true => fg_color,
                    false => bg_color,
                };
                self.framebuffer[position.1 + y][position.0 + x] = color;
            }
        }
    }

    pub fn draw_string(
        &mut self,
        string: &str,
        position: CharPosition,
        fg_color: u8,
        bg_color: u8,
    ) {
        for (num, char) in string.char_indices() {
            self.draw_char(
                font8x8::BASIC_FONTS.get(char).unwrap(),
                position + (num, 0),
                fg_color,
                bg_color,
            );
        }
    }

    /// cuts off the string if it's too long
    /// if it's too short it gets filled with whitespace
    pub fn draw_string_length(
        &mut self,
        string: &str,
        position: CharPosition,
        lenght: usize,
        fg_color: u8,
        bg_color: u8,
    ) {
        if string.len() > lenght {
            self.draw_string(&string[..=lenght], position, fg_color, bg_color)
        } else {
            self.draw_string(string, position, fg_color, bg_color);
            self.draw_rect(
                bg_color,
                CharRect::new(
                    position.y(),
                    position.y(),
                    position.x() + string.len(),
                    position.x() + lenght,
                ),
            );
        }
    }

    /// doesn't draw background. That should be done before calling this function
    pub fn draw_out_border(
        &mut self,
        char_rect: CharRect,
        top_left_color: u8,
        bot_right_color: u8,
        thickness: usize,
    ) {
        assert!(thickness <= FONT_SIZE);
        assert!(thickness > 0);
        let pixel_rect = PixelRect::from(char_rect);
        let top_left_color = self.get_raw_color(top_left_color);
        let bot_right_color = self.get_raw_color(bot_right_color);

        for x in pixel_rect.left()..=pixel_rect.right() {
            for y in 0..thickness {
                self.framebuffer[pixel_rect.top() + y][x] = top_left_color;
                self.framebuffer[pixel_rect.bot() - y][x] = bot_right_color;
            }
        }
        for y in pixel_rect.top()..=pixel_rect.bot() {
            for x in 0..thickness {
                self.framebuffer[y][pixel_rect.right() - x] = bot_right_color;
                self.framebuffer[y][pixel_rect.left() + x] = top_left_color;
            }
        }
    }

    pub fn draw_in_box(
        &mut self,
        char_rect: CharRect,
        background_color: u8,
        top_left_color: u8,
        bot_right_color: u8,
    ) {
        // needs to be between 0 and FONT_SIZE
        const BOX_THICKNESS: usize = 1;
        const SPACE_FROM_BORDER: usize = FONT_SIZE - BOX_THICKNESS;

        let pixel_rect = PixelRect::from(char_rect);
        let background_color = self.color_palette.get_raw(background_color);
        let top_left_color = self.color_palette.get_raw(top_left_color);
        let bot_right_color = self.color_palette.get_raw(bot_right_color);

        // all pixel lines except those in top and bottom char line
        for y in (pixel_rect.top() + FONT_SIZE)..=(pixel_rect.bot() - FONT_SIZE) {
            // left side foreground
            for x in (pixel_rect.left() + SPACE_FROM_BORDER)..(pixel_rect.left() + FONT_SIZE) {
                self.framebuffer[y][x] = top_left_color;
            }
            // left side background
            for x in pixel_rect.left()..(pixel_rect.left() + SPACE_FROM_BORDER) {
                self.framebuffer[y][x] = background_color;
            }

            // need the plus ones, as the '..' would need to be exclusive on the low and inclusive on the high, which i dont know how to do
            for x in
                (pixel_rect.right() - FONT_SIZE + 1)..(pixel_rect.right() - SPACE_FROM_BORDER + 1)
            {
                self.framebuffer[y][x] = bot_right_color;
            }
            // right side background
            for x in (pixel_rect.right() - SPACE_FROM_BORDER + 1)..=pixel_rect.right() {
                self.framebuffer[y][x] = background_color;
            }
        }

        // top char line
        for y in pixel_rect.top()..(pixel_rect.top() + FONT_SIZE) {
            if y < pixel_rect.top() + SPACE_FROM_BORDER {
                for x in pixel_rect.horizontal_range() {
                    self.framebuffer[y][x] = background_color;
                }
            } else {
                for x in pixel_rect.left()..=pixel_rect.right() {
                    let color = if x < pixel_rect.left() + SPACE_FROM_BORDER
                        || x > pixel_rect.right() - SPACE_FROM_BORDER
                    {
                        background_color
                    } else if x < pixel_rect.right()
                        - SPACE_FROM_BORDER
                        - (y - (pixel_rect.top() + SPACE_FROM_BORDER))
                    {
                        top_left_color
                    } else {
                        bot_right_color
                    };

                    self.framebuffer[y][x] = color;
                }
            }
        }

        // bottom char line
        for y in (pixel_rect.bot() - FONT_SIZE + 1)..=pixel_rect.bot() {
            // does the top 'SPACE_FROM_BORDER' rows in background color
            if y > pixel_rect.bot() - SPACE_FROM_BORDER {
                for x in pixel_rect.horizontal_range() {
                    self.framebuffer[y][x] = background_color;
                }
            } else {
                for x in pixel_rect.horizontal_range() {
                    let color = if x < pixel_rect.left() + SPACE_FROM_BORDER
                        || x > pixel_rect.right() - SPACE_FROM_BORDER
                    {
                        background_color
                    } else if x < pixel_rect.left() + (pixel_rect.bot() - y) {
                        top_left_color
                    } else {
                        bot_right_color
                    };

                    self.framebuffer[y][x] = color;
                }
            }
        }
    }

    pub fn draw_rect(&mut self, color: u8, rect: CharRect) {
        let pixel_rect = PixelRect::from(rect);
        self.draw_pixel_rect(color, pixel_rect)
    }

    pub fn draw_pixel_rect(&mut self, color: u8, rect: PixelRect) {
        let color = self.color_palette.get_raw(color);

        for line in &mut self.framebuffer[rect.top()..=rect.bot()] {
            line[rect.left()..=rect.right()].fill(color);
        }
    }

    /// for debugging. draws a pixel in the middle of the char
    fn mark_char(&mut self, position: CharPosition) {
        self.framebuffer[(position.y() + 4) * WINDOW_SIZE.0][position.x() + 4] =
            self.color_palette.get_raw(1);
    }

    pub fn show_colors(&mut self) {
        for i in 0..palettes::PALETTE_SIZE as u8 {
            self.draw_rect(i, CharPosition::new(i as usize, 5).into());
        }
    }
}
