use crate::visual::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
};

pub struct Header {}

impl Header {
    pub fn draw_constant(&self, buffer: &mut DrawBuffer) {
        buffer.draw_rect(2, CharRect::new(0, 11, 0, 79));
        buffer.draw_string("Rust Tracker", CharPosition::new(34, 1), 0, 2);
        buffer.draw_string("Song Name", CharPosition::new(2, 3), 0, 2);
        buffer.draw_string("File Name", CharPosition::new(2, 4), 0, 2);
        buffer.draw_string("Order", CharPosition::new(6, 5), 0, 2);
        buffer.draw_string("Pattern", CharPosition::new(4, 6), 0, 2);
        buffer.draw_string("Row", CharPosition::new(8, 7), 0, 2);
        buffer.draw_string("Speed/Tempo", CharPosition::new(38, 4), 0, 2);
        buffer.draw_string("Octave", CharPosition::new(43, 5), 0, 2);
        buffer.draw_string(
            "F1...Help       F9.....Load",
            CharPosition::new(21, 6),
            0,
            2,
        );
        buffer.draw_string(
            "ESC..Main Menu  F5/F8..Play / Stop",
            CharPosition::new(21, 7),
            0,
            2,
        );
        buffer.draw_string("Time", CharPosition::new(63, 9), 0, 2);
        buffer.draw_string("/", CharPosition::new(15, 5), 1, 0);
        buffer.draw_string("/", CharPosition::new(15, 6), 1, 0);
        buffer.draw_string("/", CharPosition::new(15, 7), 1, 0);
        buffer.draw_string("/", CharPosition::new(53, 4), 1, 0);
        buffer.draw_string("/", CharPosition::new(52, 3), 1, 0);

        // except for borders, visual candy can be added later
    }
}
