use font8x8::UnicodeFonts;

use crate::visual::utility::FONT_SIZE;

use super::{
    palettes::{Palette, CAMOUFLAGE},
    utility::{char_into_screen_pos, CharRect, PixelRect, LINE_SIZE, PIXEL_SIZE, WINDOW_SIZE},
};

pub(crate) struct DrawBuffer {
    pub framebuffer: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1 * PIXEL_SIZE],
    color_palette: Palette,
}

impl DrawBuffer {
    pub fn new() -> Self {
        Self {
            framebuffer: [0; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4],
            color_palette: CAMOUFLAGE,
        }
    }

    pub fn draw_single_char(
        &mut self,
        char_data: [u8; 8],
        position: (u8, u8),
        fg_color: usize,
        bg_color: usize,
    ) {
        let position = (
            char_into_screen_pos(position.0),
            char_into_screen_pos(position.1),
        );
        let fg_color = self.extend_color(fg_color);
        let bg_color = self.extend_color(bg_color);
        for (y, line) in char_data.iter().enumerate() {
            for x in 0..8 {
                let color = match (line >> x) & 1 == 1 {
                    true => fg_color,
                    false => bg_color,
                };
                // compute the starting byte position of the pixel
                let pixel = ((position.0 + x) * PIXEL_SIZE) + ((position.1 + y) * LINE_SIZE);
                // could maybe be better with unsafe feature 'slice_as_chunks'
                self.framebuffer[pixel..(pixel+4)].copy_from_slice(&color);
            }
        }
    }

    pub fn draw_box_new(&mut self, mut rect: CharRect, background_color: usize, top_left_color: usize, bot_right_color: usize) {
        // needs to be between 0 and 8
        const BOX_THICKNESS: usize = 3;
        // rect.right += 1;
        // rect.bot += 1;
        // let pixel_rect = PixelRect::from(rect);

        // self.framebuffer.chunks_exact(LINE_SIZE).enumerate().filter(|i, line| );

        // at this point it would probably be easier to just write a iter function again
        // maybe even do the edges jagged, just because i can

        // {
        //     let mut top_background_rect = pixel_rect;
        //     top_background_rect.bot = top_background_rect.top + (FONT_SIZE - BOX_THICKNESS);
        //     self.draw_pixel_rect(background_color, top_background_rect);
        // }

        // // double draws the top right edge
        // {
        //     let mut left_background_rect = pixel_rect;
        //     left_background_rect.right = left_background_rect.left + (FONT_SIZE - BOX_THICKNESS);
        //     self.draw_pixel_rect(background_color, left_background_rect);
        // }
        
        // // double draws the top left edge
        // {
        //     let mut right_background_rect = pixel_rect;
        //     right_background_rect.left = right_background_rect.right - (FONT_SIZE - BOX_THICKNESS);
        //     self.draw_pixel_rect(background_color, right_background_rect);
        // }

        // // double draws both bot edges
        // {
        //     let mut bot_background_rect = pixel_rect;
        //     bot_background_rect.top = bot_background_rect.bot - (FONT_SIZE - BOX_THICKNESS);
        //     self.draw_pixel_rect(background_color, bot_background_rect);
        // }

        // {
        //     let mut top_foreground_rect = pixel_rect;
        //     top_foreground_rect.bot = top_foreground_rect.top + FONT_SIZE;
        //     top_foreground_rect.top += FONT_SIZE - BOX_THICKNESS;
        //     top_foreground_rect.left += FONT_SIZE;
        //     top_foreground_rect.right -= FONT_SIZE;
        //     self.draw_pixel_rect(top_left_color, top_foreground_rect);
        // }
        
    }

    pub fn draw_box(&mut self, rect: CharRect, color_inverse: bool) {
        let outer_color = 2;
        let (inner_top_left_color, inner_bot_right_color) = match color_inverse {
            true => (1, 3),
            false => (3, 1),
        };

        // top left corner
        self.draw_single_char(
            font8x8::BLOCK_UNICODE[23].into(),
            (rect.left(), rect.top()),
            inner_top_left_color,
            outer_color,
        );
        // top right corner
        self.draw_single_char(
            font8x8::BLOCK_UNICODE[22].into(),
            (rect.right(), rect.top()),
            inner_bot_right_color,
            outer_color,
        );
        // bot left corner
        self.draw_single_char(
            font8x8::BLOCK_UNICODE[29].into(),
            (rect.left(), rect.bot()),
            inner_top_left_color,
            outer_color,
        );
        // bot right corner
        self.draw_single_char(
            font8x8::BLOCK_UNICODE[24].into(),
            (rect.right(), rect.bot()),
            inner_bot_right_color,
            outer_color,
        );

        // bot & top border
        for i in rect.left() + 1..rect.right() {
            self.draw_single_char(
                font8x8::BLOCK_UNICODE[4].into(),
                (i, rect.top()),
                inner_top_left_color,
                outer_color,
            );
            self.draw_single_char(
                font8x8::BLOCK_UNICODE[0].into(),
                (i, rect.bot()),
                inner_bot_right_color,
                outer_color,
            );
        }

        // left & right border
        for i in rect.top() + 1..rect.bot() {
            self.draw_single_char(
                font8x8::BLOCK_UNICODE[12].into(),
                (rect.left(), i),
                outer_color,
                inner_top_left_color,
            );
            self.draw_single_char(
                font8x8::BLOCK_UNICODE[16].into(),
                (rect.right(), i),
                outer_color,
                inner_bot_right_color,
            );
        }
    }

    // draw rect of full characters
    pub fn draw_rect(&mut self, color: usize, rect: CharRect) {
        let pixel_rect = PixelRect::from(rect);
        self.draw_pixel_rect(color, pixel_rect)
    }

    // draw rects between character lines
    fn draw_pixel_rect(&mut self, color: usize, rect: PixelRect) {
        let color = self.extend_color(color);

        self.framebuffer
            .chunks_exact_mut(LINE_SIZE)
            .enumerate()
            .filter(|(y, _)| rect.top <= *y && *y < rect.bot)
            .for_each(|(_, data)| {
                data.chunks_exact_mut(PIXEL_SIZE)
                    .enumerate()
                    .filter(|(x, _)| rect.left <= *x && *x < rect.right)
                    .for_each(|(_, pixel)| pixel.copy_from_slice(&color))
            });
    }

    fn mark_char(&mut self, position: (usize, usize)) {
        let pixel = ((position.0 + 4) * PIXEL_SIZE) + ((position.1 + 4) * LINE_SIZE);
        self.framebuffer[pixel..(pixel+4)].copy_from_slice(&self.color_palette[2]);
    }

    pub fn draw_string(
        &mut self,
        string: &str,
        position: (u8, u8),
        fg_color: usize,
        bg_color: usize,
    ) {
        for (num, char) in string.char_indices() {
            self.draw_single_char(
                font8x8::BASIC_FONTS.get(char).unwrap(),
                (position.0 + num as u8, position.1),
                fg_color,
                bg_color,
            );
        }
    }

    pub fn clear(&mut self, color: usize) {
        let color = self.extend_color(color);
        // could maybe be better with nightly feature 'slice_as_chunks'
        self.framebuffer
            .chunks_exact_mut(4)
            .for_each(|pixel| pixel.copy_from_slice(&color));
    }

    #[inline]
    fn extend_color(&self, color: usize) -> [u8; 4] {
        [
            self.color_palette[color][0],
            self.color_palette[color][1],
            self.color_palette[color][2],
            0
        ]
    }
}
