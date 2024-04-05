use super::{
    coordinates::{CharRect, PixelRect, FONT_SIZE, PIXEL_SIZE, WINDOW_SIZE},
    palettes::Palette,
};
use font8x8::UnicodeFonts;

pub(crate) struct DrawBuffer {
    pub framebuffer: [u32; WINDOW_SIZE.0 * WINDOW_SIZE.1],
    color_palette: Palette,
}

impl DrawBuffer {
    pub fn new() -> Self {
        Self {
            framebuffer: [0; WINDOW_SIZE.0 * WINDOW_SIZE.1],
            color_palette: Palette::CAMOUFLAGE,
        }
    }

    pub fn draw_char(
        &mut self,
        char_data: [u8; 8],
        position: (usize, usize),
        fg_color: usize,
        bg_color: usize,
    ) {
        // this is the top_left pixel
        let position = (position.0 * FONT_SIZE, position.1 * FONT_SIZE);
        let fg_color = self.color_palette.get_packed_color(fg_color);
        let bg_color = self.color_palette.get_packed_color(bg_color);
        for (y, line) in char_data.iter().enumerate() {
            for x in 0..8 {
                let color = match (line >> x) & 1 == 1 {
                    true => fg_color,
                    false => bg_color,
                };
                let pixel = position.0 + x + ((position.1 + y) * WINDOW_SIZE.0);
                self.framebuffer[pixel] = color;
            }
        }
    }

    pub fn draw_string(
        &mut self,
        string: &str,
        position: (usize, usize),
        fg_color: usize,
        bg_color: usize,
    ) {
        for (num, char) in string.char_indices() {
            self.draw_char(
                font8x8::BASIC_FONTS.get(char).unwrap(),
                (position.0 + num, position.1),
                fg_color,
                bg_color,
            );
        }
    }

    pub fn draw_box(
        &mut self,
        char_rect: CharRect,
        background_color: usize,
        top_left_color: usize,
        bot_right_color: usize,
    ) {
        // needs to be between 0 and FONT_SIZE
        const BOX_THICKNESS: usize = 2;
        const SPACE_FROM_BORDER: usize = FONT_SIZE - BOX_THICKNESS;

        let pixel_rect = PixelRect::from(char_rect);
        let background_color = self.color_palette.get_packed_color(background_color);
        let top_left_color = self.color_palette.get_packed_color(top_left_color);
        let bot_right_color = self.color_palette.get_packed_color(bot_right_color);

        // all pixel lines except those in top and bottom char line
        for y in (pixel_rect.top + FONT_SIZE)..=(pixel_rect.bot - FONT_SIZE) {
            // left side foreground
            for x in (pixel_rect.left + SPACE_FROM_BORDER)..(pixel_rect.left + FONT_SIZE) {
                let pixel = x + (y * WINDOW_SIZE.0);
                self.framebuffer[pixel] = top_left_color;
            }
            // left side background
            for x in pixel_rect.left..(pixel_rect.left + SPACE_FROM_BORDER) {
                let pixel = x + (y * WINDOW_SIZE.0);
                self.framebuffer[pixel] = background_color;
            }

            // need the plus ones, as the '..' would need to be exclusive on the low and inclusive on the high, which i dont know how to do
            for x in (pixel_rect.right - FONT_SIZE + 1)..(pixel_rect.right - SPACE_FROM_BORDER + 1)
            {
                let pixel = x + (y * WINDOW_SIZE.0);
                self.framebuffer[pixel] = bot_right_color;
            }
            // right side background
            for x in (pixel_rect.right - SPACE_FROM_BORDER + 1)..=pixel_rect.right {
                let pixel = x + (y * WINDOW_SIZE.0);
                self.framebuffer[pixel] = background_color;
            }
        }

        let (lines, remainer) = self.framebuffer.as_chunks_mut::<{ WINDOW_SIZE.0 }>();
        assert!(remainer.is_empty());

        // top char line
        for y in pixel_rect.top..(pixel_rect.top + FONT_SIZE) {
            if y < pixel_rect.top + SPACE_FROM_BORDER {
                for x in pixel_rect.horizontal_range() {
                    lines[y][x] = background_color;
                }
            } else {
                for x in pixel_rect.left..=pixel_rect.right {
                    let color = if x < pixel_rect.left + SPACE_FROM_BORDER
                        || x > pixel_rect.right - SPACE_FROM_BORDER
                    {
                        background_color
                    } else if x < pixel_rect.right
                        - SPACE_FROM_BORDER
                        - (y - (pixel_rect.top + SPACE_FROM_BORDER))
                    {
                        top_left_color
                    } else {
                        bot_right_color
                    };

                    lines[y][x] = color;
                }
            }
        }

        // bottom char line
        for y in (pixel_rect.bot - FONT_SIZE + 1)..=pixel_rect.bot {
            // does the top 'SPACE_FROM_BORDER' rows in background color
            if y > pixel_rect.bot - SPACE_FROM_BORDER {
                for x in pixel_rect.horizontal_range() {
                    // lines[y][x * PIXEL_SIZE..(x * PIXEL_SIZE) + PIXEL_SIZE]
                    //     .copy_from_slice(&background_color);
                    lines[y][x] = background_color;
                }
            } else {
                for x in pixel_rect.horizontal_range() {
                    let color = if x < pixel_rect.left + SPACE_FROM_BORDER
                        || x > pixel_rect.right - SPACE_FROM_BORDER
                    {
                        background_color
                    } else if x < pixel_rect.left + (pixel_rect.bot - y) {
                        top_left_color
                    } else {
                        bot_right_color
                    };

                    // lines[y][x * PIXEL_SIZE..(x * PIXEL_SIZE) + PIXEL_SIZE].copy_from_slice(&color);
                    lines[y][x] = color;
                }
            }
        }
    }

    // draw rect of full characters
    pub fn draw_rect(&mut self, color: usize, rect: CharRect) {
        let pixel_rect = PixelRect::from(rect);
        self.draw_pixel_rect(color, pixel_rect)
    }

    // draw rects between character lines
    fn draw_pixel_rect(&mut self, color: usize, rect: PixelRect) {
        let color = self.color_palette.get_packed_color(color);

        let (lines, remainder) = self.framebuffer.as_chunks_mut::<{WINDOW_SIZE.0}>();
        assert!(remainder.is_empty());

        for line in &mut lines[rect.top..=rect.bot] {
            line[rect.left..=rect.right].fill(color);
        }
    }

    fn mark_char(&mut self, position: (usize, usize)) {
        let pixel = position.0 + 4 + ((position.1 + 4) * WINDOW_SIZE.0);
        self.framebuffer[pixel] = self.color_palette.get_packed_color(1);
    }

    pub fn clear(&mut self, color: usize) {
        let color = self.color_palette.get_packed_color(color);
        // could maybe be better with nightly feature 'slice_as_chunks'
        self.framebuffer.fill(color);
    }
}
