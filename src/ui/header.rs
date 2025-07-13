use std::{io::Write, str::from_utf8};

use font8x8::UnicodeFonts;
use torque_tracker_engine::{
    audio_processing::playback::PlaybackPosition, manager::PlaybackSettings,
    project::pattern::Pattern,
};

use crate::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
};

#[derive(Debug)]
pub enum HeaderEvent {
    SetCursorRow(u16),
    SetMaxCursorRow(u16),
    SetPattern(u8),
    SetMaxCursorPattern(u8),
    SetOrder(u16),
    SetOrderLen(u16),
    SetSample(u8, Box<str>),
    SetSpeed(usize),
    SetTempo(usize),
    SetPlayback(Option<PlaybackPosition>),
}

#[derive(Debug)]
pub struct Header {
    row: u16,
    max_row: u16,
    pattern: u8,
    max_pattern: u8,
    order: u16,
    order_len: u16,
    selected_sample: (u8, Box<str>),
    playback: Option<PlaybackPosition>,
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
            selected_sample: (0, Box::from("")),
            playback: None,
        }
    }
}

impl Header {
    pub fn play_current_pattern(&self) -> PlaybackSettings {
        PlaybackSettings::Pattern {
            idx: self.pattern,
            should_loop: true,
        }
    }

    pub fn play_current_order(&self) -> PlaybackSettings {
        PlaybackSettings::Order {
            idx: self.order,
            should_loop: true,
        }
    }

    /// Header always needs a redraw after processing an event
    pub fn process_event(&mut self, event: HeaderEvent) {
        match event {
            HeaderEvent::SetCursorRow(r) => self.row = r,
            HeaderEvent::SetPattern(p) => self.pattern = p,
            HeaderEvent::SetOrder(o) => self.order = o,
            HeaderEvent::SetOrderLen(l) => self.order_len = l,
            HeaderEvent::SetSample(i, n) => {
                self.selected_sample.0 = i;
                self.selected_sample.1 = n
            }
            HeaderEvent::SetSpeed(_) => todo!(),
            HeaderEvent::SetTempo(_) => todo!(),
            HeaderEvent::SetMaxCursorRow(r) => self.max_row = r,
            HeaderEvent::SetMaxCursorPattern(p) => self.max_pattern = p,
            HeaderEvent::SetPlayback(p) => self.playback = p,
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
        // sample
        draw_buffer.draw_string_length(&self.selected_sample.1, CharPosition::new(53, 3), 24, 5, 0);
        let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
        write!(&mut curse, "{:02}", self.selected_sample.0).unwrap();
        draw_buffer.draw_string(
            from_utf8(&buf[..2]).unwrap(),
            CharPosition::new(50, 3),
            5,
            0,
        );
        // playback position
        if let Some(position) = self.playback {
            // Window width - 20
            let mut buf = [b' '; 60];
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "Playing, ").unwrap();
            if let Some(order) = position.order {
                write!(&mut curse, "Order: {}/{}, ", order, self.order_len).unwrap();
            }
            // TODO: figure out how to get the row count of the currently playing pattern in here
            write!(
                &mut curse,
                "Pattern: {}, Row: {}",
                position.pattern, position.row
            )
            .unwrap();
            // TODO: add voice count
            let string = from_utf8(&buf).unwrap();
            for (index, char) in string.char_indices() {
                let char_color = if char.is_ascii_digit() { 3 } else { 0 };
                draw_buffer.draw_char(
                    font8x8::BASIC_FONTS.get(char).unwrap(),
                    CharPosition::new(2 + index, 9),
                    char_color,
                    2,
                );
            }
        } else {
            draw_buffer.draw_rect(2, CharRect::new(9, 9, 2, 62));
        }
    }

    pub fn draw_constant(&self, buffer: &mut DrawBuffer) {
        buffer.draw_rect(2, CharRect::new(0, 11, 0, 79));
        buffer.draw_string("Torque Tracker", CharPosition::new(34, 1), 0, 2);
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
        buffer.draw_string(":", CharPosition::new(52, 3), 7, 0);
        // TODO: Not actually constant as it changes between Sample and Instrument mode
        buffer.draw_string("Sample", CharPosition::new(43, 3), 0, 2);
    }
}
