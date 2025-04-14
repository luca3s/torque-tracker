use std::{collections::VecDeque, io::Write, str::from_utf8};

use tracker_engine::project::{
    note_event::{Note, NoteEvent},
    pattern::{InPatternPosition, Pattern, PatternOperation},
    song::SongOperation,
};
use winit::{
    event_loop::EventLoopProxy,
    keyboard::{Key, NamedKey, SmolStr},
};

use crate::{
    app::{get_song_edit, GlobalEvent, AUDIO, EXECUTOR},
    coordinates::{CharPosition, CharRect},
    ui::header::HeaderEvent,
};

use super::{Page, PageResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InEventPosition {
    Note,
    Octave,
    Sample1,
    Sample2,
    VolPan1,
    VolPan2,
    Effect1,
    Effect2,
    Effect3,
}

impl InEventPosition {
    fn to_right(self) -> Option<Self> {
        match self {
            InEventPosition::Note => Some(Self::Octave),
            InEventPosition::Octave => Some(Self::Sample1),
            InEventPosition::Sample1 => Some(Self::Sample2),
            InEventPosition::Sample2 => Some(Self::VolPan1),
            InEventPosition::VolPan1 => Some(Self::VolPan2),
            InEventPosition::VolPan2 => Some(Self::Effect1),
            InEventPosition::Effect1 => Some(Self::Effect2),
            InEventPosition::Effect2 => Some(Self::Effect3),
            InEventPosition::Effect3 => None,
        }
    }

    fn to_left(self) -> Option<Self> {
        match self {
            InEventPosition::Note => None,
            InEventPosition::Octave => Some(Self::Note),
            InEventPosition::Sample1 => Some(Self::Octave),
            InEventPosition::Sample2 => Some(Self::Sample1),
            InEventPosition::VolPan1 => Some(Self::Sample2),
            InEventPosition::VolPan2 => Some(Self::VolPan1),
            InEventPosition::Effect1 => Some(Self::VolPan2),
            InEventPosition::Effect2 => Some(Self::Effect1),
            InEventPosition::Effect3 => Some(Self::Effect2),
        }
    }
}

#[derive(Debug)]
pub enum PatternPageEvent {
    Loaded(Pattern, usize),
}

#[derive(Debug)]
pub struct PatternPage {
    pattern_index: usize,
    pattern: Pattern,
    cursor_position: (InPatternPosition, InEventPosition),
    draw_position: InPatternPosition,
    event_proxy: EventLoopProxy<GlobalEvent>,
}

impl PatternPage {
    const MAX_PATTERN: usize = 199;
    const DRAWN_ROWS: u16 = 32;
    const DRAWN_CHANNELS: u8 = 5;
    const MAX_CHANNELS: u8 = 64;
    /// how many rows the cursor is moved when pressing pageup/down
    const PAGE_AS_ROWS: u16 = 16;
    const CHANNEL_WIDTH: usize = 14;

    const ROW_HIGHTLIGHT_MINOR: u16 = 4;
    const ROW_HIGHTLIGHT_MAJOR: u16 = 16;

    pub fn process_event(
        &mut self,
        event: PatternPageEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        match event {
            PatternPageEvent::Loaded(pattern, idx) => {
                self.pattern = pattern;
                events.push_back(GlobalEvent::Header(HeaderEvent::SetCursorPattern(idx)));
                events.push_back(GlobalEvent::Header(HeaderEvent::SetMaxCursorRow(
                    self.pattern.row_count(),
                )));
                PageResponse::RequestRedraw
            }
        }
    }

    pub fn new(proxy: EventLoopProxy<GlobalEvent>) -> Self {
        Self {
            pattern_index: 0,
            pattern: Pattern::default(),
            cursor_position: (
                InPatternPosition { row: 0, channel: 0 },
                InEventPosition::Note,
            ),
            draw_position: InPatternPosition { row: 0, channel: 0 },
            event_proxy: proxy,
        }
    }

    /// returns true if the position was changed
    fn set_cursor(
        &mut self,
        mut pos: InPatternPosition,
        event: &mut VecDeque<GlobalEvent>,
    ) -> bool {
        if pos.row >= self.pattern.row_count() {
            pos.row = self.pattern.row_count() - 1;
        }
        if pos.channel >= Self::MAX_CHANNELS {
            pos.channel = Self::MAX_CHANNELS - 1;
        }

        if pos == self.cursor_position.0 {
            return false;
        }

        if pos.row != self.cursor_position.0.row {
            event.push_back(GlobalEvent::Header(HeaderEvent::SetCursorRow(pos.row)));
        }

        self.cursor_position.0 = pos;

        // update draw position
        if pos.channel >= self.draw_position.channel + Self::DRAWN_CHANNELS {
            self.draw_position.channel = pos.channel - Self::DRAWN_CHANNELS + 1;
        } else if pos.channel < self.draw_position.channel {
            self.draw_position.channel = pos.channel
        }

        if pos.row <= (Self::DRAWN_ROWS / 2) {
            self.draw_position.row = 0;
        } else if pos.row >= self.pattern.row_count() - (Self::DRAWN_ROWS / 2) {
            self.draw_position.row = self.pattern.row_count() - Self::DRAWN_ROWS
        } else {
            self.draw_position.row = pos.row - (Self::DRAWN_ROWS / 2);
        }

        true
    }

    fn load_pattern(&mut self, idx: usize) {
        let proxy = self.event_proxy.clone();
        EXECUTOR
            .spawn(async move {
                let lock = AUDIO.lock().await;
                let pattern = lock.get_song().patterns[idx].clone();
                drop(lock);
                proxy
                    .send_event(GlobalEvent::PageEvent(super::PageEvent::Pattern(
                        PatternPageEvent::Loaded(pattern, idx),
                    )))
                    .unwrap();
            })
            .detach();
    }

    fn set_event(&mut self, position: InPatternPosition, event: NoteEvent) {
        self.pattern.set_event(position, event);
        let index = self.pattern_index;
        // could in theory lead to race conditions as operations could be applied in different order.
        // This is extremely unlikely as editing happens on human time scales while the applying is much faster.
        // If this turns out to be a problem switch to a channel and a continuesly running coroutine.
        EXECUTOR
            .spawn(async move {
                let mut lock = AUDIO.lock().await;
                let mut edit = get_song_edit(&mut lock).await;
                edit.apply_operation(SongOperation::PatternOperation(
                    index,
                    PatternOperation::SetEvent { position, event },
                ))
                .unwrap();
            })
            .detach();
    }
}

impl Page for PatternPage {
    fn draw(&mut self, draw_buffer: &mut super::DrawBuffer) {
        // helper fns
        fn visible_channels(page: &PatternPage) -> impl Iterator<Item = (usize, u8)> {
            (page.draw_position.channel..page.draw_position.channel + PatternPage::DRAWN_CHANNELS)
                .enumerate()
        }

        fn visible_rows(page: &PatternPage) -> impl Iterator<Item = (usize, u16)> {
            (page.draw_position.row..page.draw_position.row + PatternPage::DRAWN_ROWS).enumerate()
        }

        // println!("cursor: {:?}", self.cursor_position);
        // println!("draw: {:?}", self.draw_position);
        // draw row numbers
        assert!(self.draw_position.row + Self::DRAWN_ROWS <= 999);
        let mut buf = [0; 3];
        for (index, value) in visible_rows(self) {
            const BASE_POS: CharPosition = CharPosition::new(1, 15);
            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:03}", value).unwrap();
            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                BASE_POS + CharPosition::new(0, index),
                0,
                2,
            );
        }

        // draw channel headings
        assert!(self.draw_position.channel + Self::DRAWN_CHANNELS <= 99);
        let mut buf: [u8; 2] = [0; 2];
        for (index, value) in visible_channels(self) {
            const BASE_POS: CharPosition = CharPosition::new(14, 14);

            let mut curse: std::io::Cursor<&mut [u8]> = std::io::Cursor::new(&mut buf);
            write!(&mut curse, "{:02}", value).unwrap();

            draw_buffer.draw_string(
                from_utf8(&buf).unwrap(),
                BASE_POS + (index * Self::CHANNEL_WIDTH, 0),
                3,
                1,
            );
        }

        // draw events
        const BLOCK_CODE: [u8; 8] = [0b0, 0b0, 0b0, 0b11000, 0b11000, 0b0, 0b0, 0b0];
        struct EventView {
            note1: [u8; 8],
            note2: [u8; 8],
            octave: [u8; 8],
            sample1: [u8; 8],
            sample2: [u8; 8],
            vol_pan1: [u8; 8],
            vol_pan2: [u8; 8],
            effect1: [u8; 8],
            effect2: [u8; 8],
            effect3: [u8; 8],
        }

        impl Default for EventView {
            fn default() -> Self {
                Self {
                    note1: BLOCK_CODE,
                    note2: BLOCK_CODE,
                    octave: BLOCK_CODE,
                    sample1: BLOCK_CODE,
                    sample2: BLOCK_CODE,
                    vol_pan1: BLOCK_CODE,
                    vol_pan2: BLOCK_CODE,
                    effect1: font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, '.').unwrap(),
                    effect2: font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, '0').unwrap(),
                    effect3: font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, '0').unwrap(),
                }
            }
        }

        impl From<NoteEvent> for EventView {
            fn from(value: NoteEvent) -> Self {
                let mut view = Self::default();
                let mut note_chars = value.note.get_note_name().chars();
                let first = note_chars.next().unwrap();
                let second = note_chars.next().unwrap_or('-');
                view.note1 = font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, first).unwrap();
                view.note2 = font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, second).unwrap();

                let octave = match value.note.get_octave() {
                    0 => '0',
                    1 => '1',
                    2 => '2',
                    3 => '3',
                    4 => '4',
                    5 => '5',
                    6 => '6',
                    7 => '7',
                    8 => '8',
                    9 => '9',
                    _ => panic!("invalid ocatave"),
                };
                view.octave = font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, octave).unwrap();
                view
                // TODO: rest noch
            }
        }

        const EVENT_BASE_POS: CharPosition = CharPosition::new(5, 15);
        const BACKGROUND: u8 = 0;
        const FOREGROUND: u8 = 6;
        for (c_idx, c_val) in visible_channels(self) {
            for (r_idx, r_val) in visible_rows(self) {
                let background_color = match r_val {
                    val if val == self.cursor_position.0.row => 1,
                    val if val % Self::ROW_HIGHTLIGHT_MAJOR == 0 => 14,
                    val if val % Self::ROW_HIGHTLIGHT_MINOR == 0 => 15,
                    _ => BACKGROUND,
                };
                let view: EventView = self
                    .pattern
                    .get_event(InPatternPosition {
                        row: r_val,
                        channel: c_val,
                    })
                    .map(|e| (*e).into())
                    .unwrap_or_default();
                let pos = EVENT_BASE_POS + (c_idx * Self::CHANNEL_WIDTH, r_idx);
                draw_buffer.draw_char(view.note1, pos, 6, background_color);
                draw_buffer.draw_char(view.note2, pos + (1, 0), FOREGROUND, background_color);
                draw_buffer.draw_char(view.octave, pos + (2, 0), FOREGROUND, background_color);
                draw_buffer.draw_rect(background_color, (pos + (3, 0)).into());
                draw_buffer.draw_char(view.sample1, pos + (4, 0), FOREGROUND, background_color);
                draw_buffer.draw_char(view.sample2, pos + (5, 0), FOREGROUND, background_color);
                draw_buffer.draw_rect(background_color, (pos + (6, 0)).into());
                draw_buffer.draw_char(view.vol_pan1, pos + (7, 0), FOREGROUND, background_color);
                draw_buffer.draw_char(view.vol_pan2, pos + (8, 0), FOREGROUND, background_color);
                draw_buffer.draw_rect(background_color, (pos + (9, 0)).into());
                draw_buffer.draw_char(view.effect1, pos + (10, 0), FOREGROUND, background_color);
                draw_buffer.draw_char(view.effect2, pos + (11, 0), FOREGROUND, background_color);
                draw_buffer.draw_char(view.effect3, pos + (12, 0), FOREGROUND, background_color);
            }
        }

        // draw cursor
        let view: EventView = self
            .pattern
            .get_event(self.cursor_position.0)
            .map(|e| (*e).into())
            .unwrap_or_default();
        assert!(self.cursor_position.0.channel >= self.draw_position.channel);
        assert!(self.cursor_position.0.row >= self.draw_position.row);
        let c_idx = self.cursor_position.0.channel - self.draw_position.channel;
        let r_idx = self.cursor_position.0.row - self.draw_position.row;
        let pos = EVENT_BASE_POS + (c_idx as usize * Self::CHANNEL_WIDTH, r_idx as usize);
        match self.cursor_position.1 {
            InEventPosition::Note => draw_buffer.draw_char(view.note1, pos, 0, 3),
            InEventPosition::Octave => draw_buffer.draw_char(view.octave, pos + (2, 0), 0, 3),
            InEventPosition::Sample1 => draw_buffer.draw_char(view.sample1, pos + (4, 0), 0, 3),
            InEventPosition::Sample2 => draw_buffer.draw_char(view.sample2, pos + (5, 0), 0, 3),
            InEventPosition::VolPan1 => draw_buffer.draw_char(view.vol_pan1, pos + (7, 0), 0, 3),
            InEventPosition::VolPan2 => draw_buffer.draw_char(view.vol_pan2, pos + (8, 0), 0, 3),
            InEventPosition::Effect1 => draw_buffer.draw_char(view.effect1, pos + (10, 0), 0, 3),
            InEventPosition::Effect2 => draw_buffer.draw_char(view.effect2, pos + (11, 0), 0, 3),
            InEventPosition::Effect3 => draw_buffer.draw_char(view.effect3, pos + (12, 0), 0, 3),
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut super::DrawBuffer) {
        draw_buffer.draw_rect(2, CharRect::PAGE_AREA);

        // draw channel headers const parts
        for index in 0..Self::DRAWN_CHANNELS as usize {
            const BASE_POS: CharPosition = CharPosition::new(5, 14);
            let pos = BASE_POS + (index * Self::CHANNEL_WIDTH, 0);
            draw_buffer.draw_rect(1, (pos + (11, 0)).into());

            draw_buffer.draw_string(" Channel ", pos, 3, 1);
        }
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        event: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        if !key_event.state.is_pressed() {
            return PageResponse::None;
        }

        if key_event.logical_key == Key::Character(SmolStr::new_static("+")) {
            if self.pattern_index != Self::MAX_PATTERN {
                self.load_pattern(self.pattern_index + 1);
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Character(SmolStr::new_static("-")) {
            if self.pattern_index != 0 {
                self.load_pattern(self.pattern_index - 1);
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown) {
            let mut pos = self.cursor_position.0;
            pos.row = pos.row.saturating_add(1);
            if self.set_cursor(pos, event) {
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowUp) {
            let mut pos = self.cursor_position.0;
            pos.row = pos.row.saturating_sub(1);
            if self.set_cursor(pos, event) {
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight) {
            match self.cursor_position.1.to_right() {
                Some(p) => {
                    self.cursor_position.1 = p;
                    return PageResponse::RequestRedraw;
                }
                None => {
                    let mut pos = self.cursor_position.0;
                    pos.channel = pos.channel.saturating_add(1);
                    if self.set_cursor(pos, event) {
                        self.cursor_position.1 = InEventPosition::Note;
                        return PageResponse::RequestRedraw;
                    }
                }
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft) {
            match self.cursor_position.1.to_left() {
                Some(p) => {
                    self.cursor_position.1 = p;
                    return PageResponse::RequestRedraw;
                }
                None => {
                    let mut pos = self.cursor_position.0;
                    pos.channel = pos.channel.saturating_sub(1);
                    if self.set_cursor(pos, event) {
                        self.cursor_position.1 = InEventPosition::Effect3;
                        return PageResponse::RequestRedraw;
                    }
                }
            }
        } else if key_event.logical_key == Key::Named(NamedKey::Tab) {
            // shift => move left
            // not shift => move right
            if modifiers.state().shift_key() {
                if self.cursor_position.1 == InEventPosition::Note {
                    let mut pos = self.cursor_position.0;
                    pos.channel = pos.channel.saturating_sub(1);
                    if self.set_cursor(pos, event) {
                        return PageResponse::RequestRedraw;
                    }
                } else {
                    self.cursor_position.1 = InEventPosition::Note;
                    return PageResponse::RequestRedraw;
                }
            } else {
                let mut pos = self.cursor_position.0;
                pos.channel = pos.channel.saturating_add(1);
                self.set_cursor(pos, event);
                self.cursor_position.1 = InEventPosition::Note;
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::PageDown) {
            let mut pos = self.cursor_position.0;
            pos.row = pos.row.saturating_add(Self::PAGE_AS_ROWS);
            if self.set_cursor(pos, event) {
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::PageUp) {
            // original has special behaviour on page up:
            // when on last row it only goes up 15 rows
            let mut pos = self.cursor_position.0;
            pos.row = pos.row.saturating_sub(Self::PAGE_AS_ROWS);
            if self.set_cursor(pos, event) {
                return PageResponse::RequestRedraw;
            }
        }

        // should be configurable
        const DEFAULT_OCTAVE: u8 = 5;
        if let Key::Character(char) = &key_event.logical_key {
            match self.cursor_position.1 {
                InEventPosition::Note => {
                    let note = match char.as_str() {
                        "q" => Some(Note::new(12 * DEFAULT_OCTAVE)),      // C
                        "2" => Some(Note::new(1 + 12 * DEFAULT_OCTAVE)),  // Db / C#
                        "w" => Some(Note::new(2 + 12 * DEFAULT_OCTAVE)),  // D
                        "3" => Some(Note::new(3 + 12 * DEFAULT_OCTAVE)),  // Eb / D#
                        "e" => Some(Note::new(4 + 12 * DEFAULT_OCTAVE)),  // E
                        "r" => Some(Note::new(5 + 12 * DEFAULT_OCTAVE)),  // F
                        "5" => Some(Note::new(6 + 12 * DEFAULT_OCTAVE)),  // Gb / F#
                        "t" => Some(Note::new(7 + 12 * DEFAULT_OCTAVE)),  // G
                        "6" => Some(Note::new(8 + 12 * DEFAULT_OCTAVE)),  // Ab / G#
                        "z" => Some(Note::new(9 + 12 * DEFAULT_OCTAVE)),  // A
                        "7" => Some(Note::new(10 + 12 * DEFAULT_OCTAVE)), // Bb / A#
                        _ => None,
                    };
                }
                InEventPosition::Octave => {
                    if let Some(event) = self.pattern.get_event(self.cursor_position.0) {
                        let mut new_event = *event;
                        // set octave fn needed
                        let octave: Result<u8, _> = char.as_str().parse();
                        if let Ok(octave) = octave {
                            new_event.note =
                                Note::new(event.note.get() % 12 + 12 * octave).unwrap();
                            self.set_event(self.cursor_position.0, new_event);
                        }
                    }
                }
                InEventPosition::Sample1 => todo!(),
                InEventPosition::Sample2 => todo!(),
                InEventPosition::VolPan1 => todo!(),
                InEventPosition::VolPan2 => todo!(),
                InEventPosition::Effect1 => todo!(),
                InEventPosition::Effect2 => todo!(),
                InEventPosition::Effect3 => todo!(),
            }
        }

        PageResponse::None
    }
}
