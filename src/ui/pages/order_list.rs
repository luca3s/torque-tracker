use std::io::Write;
use std::str::from_utf8;

use tracker_engine::{
    channel::Pan, file::impulse_format::header::PatternOrder, project::song::Song,
};
use winit::keyboard::{Key, NamedKey};

use crate::coordinates::{CharPosition, CharRect};

use super::{Page, PageResponse};

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Panning,
    Volume,
}

#[derive(Debug)]
enum Cursor {
    Order,
    VolPan,
}

#[derive(Debug)]
struct OrderCursor {
    order: u8,
    // 0 - 2 valid
    digit: u8,
}

#[derive(Debug)]
pub struct OrderListPage {
    mode: Mode,
    cursor: Cursor,
    order_cursor: OrderCursor,
    order_draw: u8,
    pattern_order: [PatternOrder; Song::<true>::MAX_ORDERS],
    volume: [u8; Song::<true>::MAX_CHANNELS],
    pan: [Pan; Song::<true>::MAX_CHANNELS],
}

impl OrderListPage {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::Order,
            mode: Mode::Panning,
            order_cursor: OrderCursor { order: 0, digit: 0 },
            order_draw: 0,
            pattern_order: [PatternOrder::EndOfSong; Song::<true>::MAX_ORDERS],
            volume: [64; Song::<true>::MAX_CHANNELS],
            pan: [Pan::default(); Song::<true>::MAX_CHANNELS],
        }
    }

    pub fn switch_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Panning => Mode::Volume,
            Mode::Volume => Mode::Panning,
        }
    }

    pub fn reset_mode(&mut self) {
        self.mode = Mode::Panning
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn order_cursor_up(&mut self) -> PageResponse {
        if self.order_cursor.order == 0 {
            return PageResponse::None;
        }

        self.order_cursor.order -= 1;
        self.order_draw = self.order_draw.min(self.order_cursor.order);
        PageResponse::RequestRedraw
    }
    fn order_cursor_down(&mut self) -> PageResponse {
        if self.order_cursor.order == 255 {
            return PageResponse::None;
        }

        self.order_cursor.order += 1;
        self.order_draw = self
            .order_draw
            .max(self.order_cursor.order.saturating_sub(31));
        PageResponse::RequestRedraw
    }
}

impl Page for OrderListPage {
    fn draw(&mut self, draw_buffer: &mut crate::draw_buffer::DrawBuffer) {
        fn write_pattern_order(order: PatternOrder, buf: &mut [u8; 3]) -> &str {
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(buf);
            match order {
                PatternOrder::Number(n) => write!(&mut curse, "{:03}", n).unwrap(),
                PatternOrder::EndOfSong => write!(&mut curse, "---").unwrap(),
                PatternOrder::SkipOrder => write!(&mut curse, "+++").unwrap(),
            }
            from_utf8(buf).unwrap()
        }

        const ORDER_BASE_POS: CharPosition = CharPosition::new(2, 15);
        let mut buf = [0; 3];
        for (pos, order) in (self.order_draw..self.order_draw + 32).enumerate() {
            // row index
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:03}", order).unwrap();
            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                ORDER_BASE_POS + CharPosition::new(0, pos),
                0,
                2,
            );
            // row value
            draw_buffer.draw_string(
                write_pattern_order(self.pattern_order[usize::from(order)], &mut buf),
                ORDER_BASE_POS + CharPosition::new(5, pos),
                2,
                0,
            );
        }

        // draw cursor
        match self.cursor {
            Cursor::Order => {
                let mut buf = [0; 3];
                let row = write_pattern_order(
                    self.pattern_order[usize::from(self.order_cursor.order)],
                    &mut buf,
                );
                draw_buffer.draw_char(
                    font8x8::UnicodeFonts::get(
                        &font8x8::BASIC_FONTS,
                        row.chars()
                            .nth(usize::from(self.order_cursor.digit))
                            .unwrap(),
                    )
                    .unwrap(),
                    ORDER_BASE_POS
                        + CharPosition::new(
                            5 + usize::from(self.order_cursor.digit),
                            usize::from(self.order_cursor.order - self.order_draw),
                        ),
                    0,
                    3,
                );
            }
            Cursor::VolPan => todo!(),
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut crate::draw_buffer::DrawBuffer) {
        draw_buffer.draw_rect(2, CharRect::PAGE_AREA);
        draw_buffer.draw_in_box(CharRect::new(14, 14 + 33, 6, 10), 2, 1, 3, 2);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut std::collections::VecDeque<crate::app::GlobalEvent>,
    ) -> PageResponse {
        dbg!(&self.order_cursor);
        match self.cursor {
            Cursor::Order => {
                if modifiers.state().is_empty() && key_event.state.is_pressed() {
                    if key_event.logical_key == Key::Named(NamedKey::ArrowUp) {
                        return self.order_cursor_up();
                    } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown) {
                        return self.order_cursor_down();
                    } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft) {
                        self.order_cursor.digit = if self.order_cursor.digit == 0 {
                            2
                        } else {
                            self.order_cursor.digit - 1
                        };
                        return PageResponse::RequestRedraw;
                    } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight) {
                        self.order_cursor.digit = if self.order_cursor.digit == 2 {
                            0
                        } else {
                            self.order_cursor.digit + 1
                        };
                        return PageResponse::RequestRedraw;
                    } else if let Key::Character(text) = &key_event.logical_key {
                        let mut chars = text.chars();
                        let first = chars.next();
                        assert!(chars.next().is_none());
                        match first {
                            Some('+') => {
                                self.pattern_order[usize::from(self.order_cursor.order)] =
                                    PatternOrder::SkipOrder;
                                self.order_cursor_down();
                                self.order_cursor.digit = 0;
                                return PageResponse::RequestRedraw;
                            }
                            Some('-') | Some('.') => {
                                self.pattern_order[usize::from(self.order_cursor.order)] =
                                    PatternOrder::EndOfSong;
                                self.order_cursor_down();
                                self.order_cursor.digit = 0;
                                return PageResponse::RequestRedraw;
                            }
                            _ => return PageResponse::None,
                        }
                    }
                }
            }
            Cursor::VolPan => {}
        }
        PageResponse::None
    }
}
