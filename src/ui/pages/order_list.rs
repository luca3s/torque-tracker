use std::str::from_utf8;
use std::{array, io::Write};

use torque_tracker_engine::channel::Pan;
use torque_tracker_engine::project::song::SongOperation;
use torque_tracker_engine::{file::impulse_format::header::PatternOrder, project::song::Song};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::app::{EventQueue, GlobalEvent, send_song_op};
use crate::ui::header::HeaderEvent;
use crate::ui::widgets::{NextWidget, StandardResponse, Widget};
use crate::{
    coordinates::{CharPosition, CharRect},
    ui::widgets::slider::Slider,
};

use super::{Page, PageEvent, PageResponse};

#[derive(Debug, Clone)]
pub enum OrderListPageEvent {
    SetVolumeCurrent(i16),
    SetPanCurrent(i16),
    SetPlayback(Option<u16>),
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Panning,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cursor {
    Order,
    VolPan(u8),
}

#[derive(Debug)]
struct OrderCursor {
    order: u16,
    // 0 - 2 valid
    digit: u8,
}

#[derive(Debug)]
pub struct OrderListPage {
    mode: Mode,
    cursor: Cursor,
    order_cursor: OrderCursor,
    order_draw: u16,
    order_playback: Option<u16>,
    pattern_order: [PatternOrder; Song::MAX_ORDERS],
    volume: [Slider<0, 64, ()>; 64],
    pan: [Slider<0, 64, ()>; 64],
}

impl OrderListPage {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::Order,
            mode: Mode::Panning,
            order_cursor: OrderCursor { order: 0, digit: 0 },
            order_draw: 0,
            pattern_order: [PatternOrder::EndOfSong; Song::MAX_ORDERS],
            order_playback: None,
            volume: array::from_fn(|idx| {
                let pos = if idx >= 32 {
                    CharPosition::new(65, 15 + idx - 32)
                } else {
                    CharPosition::new(31, 15 + idx)
                };
                let idx = u8::try_from(idx).unwrap();
                Slider::new(
                    64,
                    pos,
                    8,
                    NextWidget::default(),
                    |value| {
                        GlobalEvent::Page(PageEvent::OrderList(
                            OrderListPageEvent::SetVolumeCurrent(value),
                        ))
                    },
                    move |vol| {
                        // impossible to trigger as long as the slider is correct
                        let vol = u8::try_from(vol).unwrap();
                        send_song_op(SongOperation::SetVolume(idx, vol));
                    },
                )
            }),
            pan: array::from_fn(|idx| {
                let pos = if idx >= 32 {
                    CharPosition::new(65, 15 + idx - 32)
                } else {
                    CharPosition::new(31, 15 + idx)
                };
                let idx = u8::try_from(idx).unwrap();
                Slider::new(
                    32,
                    pos,
                    8,
                    NextWidget::default(),
                    |value| {
                        GlobalEvent::Page(PageEvent::OrderList(OrderListPageEvent::SetPanCurrent(
                            value,
                        )))
                    },
                    move |pan| {
                        // surround and disabled not supported yet
                        // This panic can't be triggered as long as the slider is correct.
                        let pan = Pan::Value(u8::try_from(pan).unwrap());
                        send_song_op(SongOperation::SetPan(idx, pan));
                    },
                )
            }),
        }
    }

    pub fn process_event(&mut self, event: OrderListPageEvent) -> PageResponse {
        match event {
            OrderListPageEvent::SetVolumeCurrent(vol) => {
                let cursor = match self.cursor {
                    Cursor::Order => unreachable!(
                        "when a set volume event is created a volume slider has to be selected"
                    ),
                    Cursor::VolPan(c) => c,
                };
                self.volume[usize::from(cursor - 1)]
                    .try_set(vol)
                    .expect("the value was created from the slider, so it has to fit.");
            }
            OrderListPageEvent::SetPanCurrent(pan) => {
                let cursor = match self.cursor {
                    Cursor::Order => unreachable!(
                        "when a set pan event is created a pan slider has to be selected"
                    ),
                    Cursor::VolPan(c) => c,
                };
                self.pan[usize::from(cursor - 1)]
                    .try_set(pan)
                    .expect("the event was created from the slider, so has to fit.")
            }
            OrderListPageEvent::SetPlayback(o) => self.order_playback = o,
        };
        PageResponse::RequestRedraw
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

    fn send_order_position(&self, events: &mut EventQueue<'_>) {
        events.push(GlobalEvent::Header(HeaderEvent::SetOrder(
            self.order_cursor.order,
        )));
    }

    fn send_order_len(&self, events: &mut EventQueue<'_>) {
        let order_len = self
            .pattern_order
            .iter()
            .take_while(|o| **o != PatternOrder::EndOfSong)
            .count();
        events.push(GlobalEvent::Header(HeaderEvent::SetOrderLen(
            u16::try_from(order_len).unwrap(),
        )));
    }

    fn order_cursor_up(&mut self, count: u16, events: &mut EventQueue<'_>) -> PageResponse {
        debug_assert!(count != 0, "why would you do this");
        if self.order_cursor.order == 0 {
            return PageResponse::None;
        }

        self.order_cursor.order = self.order_cursor.order.saturating_sub(count);
        self.order_draw = self.order_draw.min(self.order_cursor.order);
        self.send_order_position(events);
        PageResponse::RequestRedraw
    }

    fn order_cursor_down(&mut self, count: u16, events: &mut EventQueue<'_>) -> PageResponse {
        debug_assert!(count != 0, "why would you do this");
        if self.order_cursor.order == 255 {
            return PageResponse::None;
        }

        self.order_cursor.order = self.order_cursor.order.saturating_add(count);
        self.order_draw = self
            .order_draw
            .max(self.order_cursor.order.saturating_sub(31));
        self.send_order_position(events);
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

        // draw heading for vol pan columns
        match self.mode {
            Mode::Panning => {
                draw_buffer.draw_string("L   M   R", CharPosition::new(31, 14), 0, 3);
                draw_buffer.draw_string("L   M   R", CharPosition::new(65, 14), 0, 3);
            }
            Mode::Volume => {
                draw_buffer.draw_string(" Volumes ", CharPosition::new(31, 14), 0, 3);
                draw_buffer.draw_string(" Volumes ", CharPosition::new(65, 14), 0, 3);
            }
        }

        const ORDER_BASE_POS: CharPosition = CharPosition::new(2, 15);
        let mut buf = [0; 3];
        for (pos, order) in (self.order_draw..=self.order_draw + 31).enumerate() {
            // row index
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:03}", order).unwrap();
            let text_color = if Some(order) == self.order_playback {
                3
            } else {
                0
            };
            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                ORDER_BASE_POS + CharPosition::new(0, pos),
                text_color,
                2,
            );
            // row value
            draw_buffer.draw_string(
                write_pattern_order(self.pattern_order[usize::from(order)], &mut buf),
                ORDER_BASE_POS + CharPosition::new(4, pos),
                2,
                0,
            );
        }

        // draw channel and numbers
        // TODO: make the channel number strings const
        const CHANNEL_BASE_LEFT: CharPosition = CharPosition::new(20, 15);
        const CHANNEL_BASE_RIGHT: CharPosition = CharPosition::new(54, 15);
        const CHANNEL: &str = "Channel";
        let mut buf = [0; 2];
        for row in 0..32 {
            draw_buffer.draw_string(CHANNEL, CHANNEL_BASE_LEFT + CharPosition::new(0, row), 0, 2);
            draw_buffer.draw_string(
                CHANNEL,
                CHANNEL_BASE_RIGHT + CharPosition::new(0, row),
                0,
                2,
            );
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:02}", row + 1).unwrap();
            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                CHANNEL_BASE_LEFT + CharPosition::new(8, row),
                0,
                2,
            );
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:02}", row + 33).unwrap();
            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                CHANNEL_BASE_RIGHT + CharPosition::new(8, row),
                0,
                2,
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
                            4 + usize::from(self.order_cursor.digit),
                            usize::from(self.order_cursor.order - self.order_draw),
                        ),
                    0,
                    3,
                );
            }
            Cursor::VolPan(c) => {
                let mut buf = [0; 2];
                let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
                write!(&mut curse, "{:02}", c).unwrap();
                let pos = if c <= 32 {
                    CHANNEL_BASE_LEFT + CharPosition::new(0, usize::from(c - 1))
                } else {
                    CHANNEL_BASE_RIGHT + CharPosition::new(0, usize::from(c - 33))
                };
                draw_buffer.draw_string(CHANNEL, pos, 3, 2);
                draw_buffer.draw_string(
                    from_utf8(&buf).unwrap(),
                    pos + CharPosition::new(8, 0),
                    3,
                    2,
                );
            }
        }

        // draw sliders
        match self.mode {
            Mode::Panning => {
                for (idx, pan) in self.pan.iter().enumerate() {
                    pan.draw(
                        draw_buffer,
                        self.cursor == Cursor::VolPan(u8::try_from(idx + 1).unwrap()),
                    );
                }
            }
            Mode::Volume => {
                for (idx, vol) in self.volume.iter().enumerate() {
                    vol.draw(
                        draw_buffer,
                        self.cursor == Cursor::VolPan(u8::try_from(idx + 1).unwrap()),
                    );
                }
            }
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut crate::draw_buffer::DrawBuffer) {
        // background fill
        draw_buffer.draw_rect(2, CharRect::PAGE_AREA);
        // box around order list
        draw_buffer.draw_in_box(CharRect::new(14, 14 + 33, 5, 9), 2, 1, 3, 2);
        // boxes aroung pan or vol
        draw_buffer.draw_in_box(CharRect::new(13, 14 + 33, 30, 30 + 10), 2, 3, 3, 2);
        draw_buffer.draw_in_box(CharRect::new(13, 14 + 33, 64, 64 + 10), 2, 3, 3, 2);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut EventQueue<'_>,
    ) -> PageResponse {
        match self.cursor {
            Cursor::Order => {
                if key_event.logical_key == Key::Named(NamedKey::Tab)
                    && key_event.state.is_pressed()
                {
                    if modifiers.state() == ModifiersState::SHIFT {
                        self.cursor = Cursor::VolPan(33);
                        return PageResponse::RequestRedraw;
                    } else if modifiers.state().is_empty() {
                        self.cursor = Cursor::VolPan(1);
                        return PageResponse::RequestRedraw;
                    }
                }
                if modifiers.state().is_empty() && key_event.state.is_pressed() {
                    if key_event.logical_key == Key::Named(NamedKey::ArrowUp) {
                        return self.order_cursor_up(1, events);
                    } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown) {
                        return self.order_cursor_down(1, events);
                    } else if key_event.logical_key == Key::Named(NamedKey::PageDown) {
                        return self.order_cursor_down(16, events);
                    } else if key_event.logical_key == Key::Named(NamedKey::PageUp) {
                        return self.order_cursor_up(16, events);
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
                    } else if key_event.logical_key == Key::Named(NamedKey::Tab) {
                        self.cursor = Cursor::VolPan(1); // go to channel 1
                        return PageResponse::RequestRedraw;
                    } else if let Key::Character(text) = &key_event.logical_key {
                        let mut chars = text.chars();
                        let first = chars.next();
                        assert!(chars.next().is_none());
                        match first {
                            Some('+') => {
                                self.pattern_order[usize::from(self.order_cursor.order)] =
                                    PatternOrder::SkipOrder;
                                send_song_op(SongOperation::SetOrder(
                                    self.order_cursor.order,
                                    PatternOrder::SkipOrder,
                                ));
                                self.order_cursor_down(1, events);
                                self.order_cursor.digit = 0;
                                return PageResponse::RequestRedraw;
                            }
                            Some('-') | Some('.') => {
                                self.pattern_order[usize::from(self.order_cursor.order)] =
                                    PatternOrder::EndOfSong;
                                send_song_op(SongOperation::SetOrder(
                                    self.order_cursor.order,
                                    PatternOrder::EndOfSong,
                                ));
                                self.order_cursor_down(1, events);
                                self.order_cursor.digit = 0;
                                return PageResponse::RequestRedraw;
                            }
                            Some(num) if num.is_ascii_digit() => {
                                let mut buf = [0; 1];
                                let string = num.encode_utf8(&mut buf);
                                let num: u8 = string.parse().unwrap();
                                assert!(num <= 9);
                                let cursor = usize::from(self.order_cursor.order);
                                let new_num =
                                    match (self.pattern_order[cursor], self.order_cursor.digit) {
                                        // hooooly shit, i love match
                                        (PatternOrder::Number(old_num), 0) => u8::try_from(
                                            (u16::from(num) * 100 + u16::from(old_num % 100))
                                                .min(199),
                                        )
                                        .unwrap(),
                                        (PatternOrder::Number(old_num), 1) => {
                                            num * 10 + 100 * (old_num / 100) + old_num % 10
                                        }
                                        (PatternOrder::Number(old_num), 2) => {
                                            num + 10 * (old_num / 10)
                                        }
                                        (PatternOrder::Number(_), _) => unreachable!(),
                                        (_, 0) => num.checked_mul(100).unwrap_or(199).min(199),
                                        (_, 1) => 10 * num,
                                        (_, 2) => num,
                                        _ => unreachable!(),
                                    };
                                self.pattern_order[cursor] = PatternOrder::Number(new_num);
                                send_song_op(SongOperation::SetOrder(
                                    self.order_cursor.order,
                                    PatternOrder::Number(new_num),
                                ));
                                match self.order_cursor.digit {
                                    0 => self.order_cursor.digit = 1,
                                    1 => self.order_cursor.digit = 2,
                                    2 => {
                                        self.order_cursor.digit = 0;
                                        self.order_cursor_down(1, events);
                                    }
                                    _ => unreachable!(),
                                }
                                self.send_order_len(events);
                                return PageResponse::RequestRedraw;
                            }
                            _ => return PageResponse::None,
                        }
                    }
                }
            }
            Cursor::VolPan(c) => {
                if key_event.logical_key == Key::Named(NamedKey::Tab)
                    && key_event.state.is_pressed()
                {
                    if modifiers.state() == ModifiersState::SHIFT {
                        if c <= 32 {
                            self.cursor = Cursor::Order;
                            return PageResponse::RequestRedraw;
                        } else {
                            self.cursor = Cursor::VolPan(c - 32);
                            return PageResponse::RequestRedraw;
                        }
                    } else if modifiers.state().is_empty() {
                        if c <= 32 {
                            self.cursor = Cursor::VolPan(c + 32);
                            return PageResponse::RequestRedraw;
                        } else {
                            self.cursor = Cursor::Order;
                            return PageResponse::RequestRedraw;
                        }
                    }
                    return PageResponse::None;
                }
                if modifiers.state().is_empty() && key_event.state.is_pressed() {
                    if key_event.logical_key == Key::Named(NamedKey::ArrowUp) {
                        if c == 1 {
                            return PageResponse::None;
                        } else {
                            self.cursor = Cursor::VolPan(c - 1);
                            return PageResponse::RequestRedraw;
                        }
                    } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown) {
                        if c == 64 {
                            return PageResponse::None;
                        } else {
                            self.cursor = Cursor::VolPan(c + 1);
                            return PageResponse::RequestRedraw;
                        }
                    }
                }

                match self.mode {
                    Mode::Panning => match self.pan[usize::from(c - 1)]
                        .process_input(modifiers, key_event, events)
                        .standard
                    {
                        StandardResponse::SwitchFocus(_) => return PageResponse::None,
                        StandardResponse::RequestRedraw => return PageResponse::RequestRedraw,
                        StandardResponse::None => return PageResponse::None,
                    },
                    Mode::Volume => match self.volume[usize::from(c - 1)]
                        .process_input(modifiers, key_event, events)
                        .standard
                    {
                        StandardResponse::SwitchFocus(_) => return PageResponse::None,
                        StandardResponse::RequestRedraw => return PageResponse::RequestRedraw,
                        StandardResponse::None => return PageResponse::None,
                    },
                }
            }
        }
        PageResponse::None
    }
}
