use crate::visual::{coordinates::CharRect, draw_buffer::DrawBuffer};

pub struct Header {}

impl Header {
    pub fn draw_constant(&self, buffer: &mut DrawBuffer) {
        buffer.draw_rect(2, CharRect::new(0, 11, 0, 79));
        buffer.draw_string("Rust Tracker", (34, 1), 0, 2);
        buffer.draw_string("Song Name", (2, 3), 0, 2);
        buffer.draw_string("File Name", (2, 4), 0, 2);
        buffer.draw_string("Order", (6, 5), 0, 2);
        buffer.draw_string("Pattern", (4, 6), 0, 2);
        buffer.draw_string("Row", (8, 7), 0, 2);
        buffer.draw_string("Speed/Tempo", (38, 4), 0, 2);
        buffer.draw_string("Octave", (43, 5), 0, 2);
        buffer.draw_string("F1...Help       F9.....Load", (21, 6), 0, 2);
        buffer.draw_string("ESC..Main Menu  F5/F8..Play / Stop", (21, 7), 0, 2);
        buffer.draw_string("Time", (63, 9), 0, 2);
        buffer.draw_string("/", (15, 5), 1, 0);
        buffer.draw_string("/", (15, 6), 1, 0);
        buffer.draw_string("/", (15, 7), 1, 0);
        buffer.draw_string("/", (53, 4), 1, 0);
        buffer.draw_string("/", (52, 3), 1, 0);

        // except for borders, visual candy can be added later
    }
}
