use std::{io::Write, str::from_utf8};

use torque_tracker_engine::project::pattern::Pattern;

use crate::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
};

#[derive(Debug)]
pub enum HeaderEvent {
    SetCursorRow(u16),
    SetMaxCursorRow(u16),
    SetPattern(usize),
    SetMaxCursorPattern(usize),
    SetOrder(u8),
    SetOrderLen(u8),
    SetSample(usize),
    SetSpeed(usize),
    SetTempo(usize),
}

#[derive(Debug)]
pub struct Header {
    row: u16,
    max_row: u16,
    pattern: usize,
    max_pattern: usize,
    order: u8,
    order_len: u8,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            row: 0,
            max_row: Pattern::DEFAULT_ROWS,
            pattern: 0,
            max_pattern: 0,
            order: 0,
            order_len: 0,
        }
    }
}

impl Header {
    /// Header always needs a redraw after processing an event
    pub fn process_event(&mut self, event: HeaderEvent) {
        match event {
            HeaderEvent::SetCursorRow(r) => self.row = r,
            HeaderEvent::SetPattern(p) => self.pattern = p,
            HeaderEvent::SetOrder(o) => self.order = o,
            HeaderEvent::SetSample(_) => todo!(),
            HeaderEvent::SetSpeed(_) => todo!(),
            HeaderEvent::SetTempo(_) => todo!(),
            HeaderEvent::SetMaxCursorRow(r) => self.max_row = r,
            HeaderEvent::SetMaxCursorPattern(p) => self.max_pattern = p,
            HeaderEvent::SetOrderLen(l) => self.order_len = l,
        }
    }

    pub fn draw(&self, draw_buffer: &mut DrawBuffer) {
        let mut buf = [0; 3];
        // row
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.row).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(12, 7), 5, 0);
        // row max
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.max_row).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(16, 7), 5, 0);
        // pattern
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.pattern).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(12, 6), 5, 0);
        // max pattern
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.max_pattern).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(16, 6), 5, 0);

        // order
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.order).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(12, 5), 5, 0);
        // order len
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:03}", self.order_len).unwrap();
        draw_buffer.draw_string(from_utf8(&buf).unwrap(), CharPosition::new(16, 5), 5, 0);
    }

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
