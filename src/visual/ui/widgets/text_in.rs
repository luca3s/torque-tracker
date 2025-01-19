use std::collections::VecDeque;

use ascii::{AsciiChar, AsciiString};
use font8x8::UnicodeFonts;
use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    app::GlobalEvent,
    coordinates::{CharPosition, WINDOW_SIZE},
    draw_buffer::DrawBuffer,
};

use super::widget::{NextWidget, Widget, WidgetResponse};

/// text has max_len of the rect that was given, because the text_in cannot scroll
/// use text_in_scroll for that
/// i only allow Ascii characters as i can only render ascii
pub struct TextIn {
    pos: CharPosition,
    width: usize,
    text: AsciiString,
    next_widget: NextWidget,
    callback: Box<dyn Fn(&str)>,
    cursor_pos: usize,
}

impl Widget for TextIn {
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        draw_buffer.draw_string_length(self.text.as_str(), self.pos, self.width, 2, 0);
        // draw the cursor by overdrawing a letter
        if selected {
            let cursor_char_pos = self.pos + CharPosition::new(self.cursor_pos, 0);
            if self.cursor_pos < self.text.len() {
                draw_buffer.draw_char(
                    font8x8::BASIC_FONTS
                        .get(self.text[self.cursor_pos].into())
                        .unwrap(),
                    cursor_char_pos,
                    0,
                    3,
                );
            } else {
                draw_buffer.draw_rect(3, cursor_char_pos.into());
            }
        }
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        _event: &mut VecDeque<GlobalEvent>,
    ) -> WidgetResponse {
        if !key_event.state.is_pressed() {
            return WidgetResponse::None;
        }

        if let Key::Character(str) = &key_event.logical_key {
            let mut char_iter = str.chars();
            // why would i get a character event if there wasnt a char?? so i unwrap
            let first_char = char_iter.next().unwrap();

            if let Ok(ascii_char) = AsciiChar::from_ascii(first_char) {
                self.insert_char(ascii_char);
            }
            // make sure i only got one char. dont know why i should get more than one
            // if this ever panics switch to a loop implementation above
            assert!(char_iter.next().is_none());
            return WidgetResponse::RequestRedraw;
        } else if key_event.logical_key == Key::Named(NamedKey::Space) {
            self.insert_char(AsciiChar::Space);
            return WidgetResponse::RequestRedraw;
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
            && modifiers.state().is_empty()
        {
            // when moving left from 0 i dont need to redraw
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
                return WidgetResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            && modifiers.state().is_empty()
        {
            // check that the cursor doesnt go away from the string
            if self.text.len() > self.cursor_pos {
                self.cursor_pos += 1;
                return WidgetResponse::RequestRedraw;
            }
        // backspace and delete keys
        // entf on German Keyboards
        } else if key_event.logical_key == Key::Named(NamedKey::Delete) {
            // cant delete if im outside the text, also includes text empty
            if self.cursor_pos < self.text.len() {
                let _ = self.text.remove(self.cursor_pos);
                (self.callback)(self.text.as_str());
                return WidgetResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::Backspace) {
            // super + backspace clears the string
            if modifiers.state().super_key() {
                self.text.clear();
                self.cursor_pos = 0;
                (self.callback)(self.text.as_str());
                return WidgetResponse::RequestRedraw;
            // if string is empty we cant remove from it
            } else if modifiers.state().is_empty() && !self.text.is_empty() {
                // if cursor at position 0 backspace starts to behave like delete, no idea why original is like that
                if self.cursor_pos == 0 {
                    let _ = self.text.remove(0);
                    (self.callback)(self.text.as_str());
                } else {
                    let _ = self.text.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    (self.callback)(self.text.as_str());
                }
                return WidgetResponse::RequestRedraw;
            }
        // next widget
        } else {
            return self
                .next_widget
                .process_key_event(key_event, modifiers)
                .into();
        }
        WidgetResponse::None
    }
}

impl TextIn {
    pub fn new(
        pos: CharPosition,
        width: usize,
        next_widget: NextWidget,
        cb: impl Fn(&str) + 'static,
    ) -> Self {
        assert!(pos.x() + width < WINDOW_SIZE.0);
        // right and left keys are used in the widget itself. doeesnt make sense to put NextWidget there
        assert!(next_widget.right.is_none());
        assert!(next_widget.left.is_none());

        TextIn {
            pos,
            width,
            text: AsciiString::with_capacity(width), // allows to never allocate or deallocate in TextIn
            next_widget,
            callback: Box::new(cb),
            cursor_pos: 0,
        }
    }

    // not tested
    pub fn set_string(&mut self, new_str: String) -> Result<(), ascii::FromAsciiError<String>> {
        self.text = AsciiString::from_ascii(new_str)?;
        self.text.truncate(self.width);
        self.cursor_pos = self.text.len();

        (self.callback)(self.text.as_str());
        Ok(())
    }

    pub fn get_str(&self) -> &str {
        self.text.as_str()
    }

    fn insert_char(&mut self, char: AsciiChar) {
        if self.cursor_pos < self.width {
            self.cursor_pos += 1;
        }
        self.text.insert(self.cursor_pos - 1, char);
        self.text.truncate(self.width);

        (self.callback)(self.text.as_str());
    }
}
