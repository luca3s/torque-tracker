use ascii::{AsciiChar, AsciiString};
use font8x8::UnicodeFonts;
use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    coordinates::{CharPosition, CharRect, WINDOW_SIZE},
    draw_buffer::DrawBuffer,
};

use super::widget::{NextWidget, Widget};

/// text has max_len of the rect that was given, because the text_in cannot scroll
/// use text_in_scroll for that
// i only allow Ascii characters as i can only render ascii
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
        if selected {
            // if the cursor is inside the text
            if self.text.len() < self.cursor_pos {
                let mut chars = self.text.chars();
                for x in 0..self.width {
                    match chars.next() {
                        Some(ascii_char) => {
                            let (fg_color, bg_color) =
                                if self.cursor_pos == x { (0, 3) } else { (1, 0) };
                            draw_buffer.draw_char(
                                font8x8::BASIC_FONTS.get(ascii_char.into()).unwrap(),
                                self.pos + (x, 0),
                                fg_color,
                                bg_color,
                            );
                        }

                        None => draw_buffer.draw_rect(0, CharRect::from(self.pos + (x, 0))),
                    }
                    // let (fg_color, bg_color) = if self.cursor_pos == i { (0, 3) } else { (1, 0) };
                }
                self.text.chars().enumerate().for_each(|(i, ascii_char)| {
                    let (fg_color, bg_color) = if self.cursor_pos == i { (0, 3) } else { (1, 0) };
                    draw_buffer.draw_char(
                        font8x8::BASIC_FONTS.get(ascii_char.into()).unwrap(),
                        self.pos + (i, 0),
                        fg_color,
                        bg_color,
                    );
                });
            } else {
                draw_buffer.draw_string_length(self.text.as_str(), self.pos, self.width, 1, 0);
                draw_buffer.draw_rect(
                    3,
                    CharRect::new(
                        self.pos.y(),
                        self.pos.y(),
                        self.pos.x() + self.cursor_pos,
                        self.pos.x() + self.cursor_pos,
                    ),
                );
            }
        } else {
            draw_buffer.draw_string_length(self.text.as_str(), self.pos, self.width, 1, 0);
        }
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> Option<usize> {
        if !key_event.state.is_pressed() {
            return None;
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
        } else if key_event.logical_key == Key::Named(NamedKey::Space) {
            self.insert_char(AsciiChar::Space);
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
            && modifiers.state().is_empty()
        {
            // for left movement i dont need to check the cursor pos, as saturating add already guarantees it doesnt go below 0
            self.cursor_pos = self.cursor_pos.saturating_sub(1);
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            && modifiers.state().is_empty()
        {
            // check that the cursor doesnt go away from the string
            if self.text.len() > self.cursor_pos {
                // dont need saturating here as such big text_entry fields shouldnt be possible
                self.cursor_pos += 1;
            }
        // backspace and delete keys
        // entf on German Keyboards
        } else if key_event.logical_key == Key::Named(NamedKey::Delete) {
            // cant delete if im outside the text, also includes text empty
            if self.cursor_pos < self.text.len() {
                let _ = self.text.remove(self.cursor_pos);
                (self.callback)(self.text.as_str());
            }
        } else if key_event.logical_key == Key::Named(NamedKey::Backspace) {
            // shift + backspace clears the string
            if modifiers.state().super_key() {
                self.text.clear();
                self.cursor_pos = 0;
                (self.callback)(self.text.as_str());
            // if string is empty we cant remove from it
            } else if !self.text.is_empty() {
                // if cursor at position 0 backspace starts to behave like delete, no idea why original is like that
                if self.cursor_pos == 0 {
                    let _ = self.text.remove(0);
                    (self.callback)(self.text.as_str());
                } else {
                    let _ = self.text.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    (self.callback)(self.text.as_str());
                }
            }
        // next widget
        } else {
            return self.next_widget.process_key_event(key_event, modifiers);
        }
        None
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

        TextIn {
            pos,
            width,
            text: AsciiString::with_capacity(width),
            next_widget,
            callback: Box::new(cb),
            cursor_pos: 0,
        }
    }

    pub fn set_string<'a>(
        &mut self,
        new_str: &'a str,
    ) -> Result<(), ascii::FromAsciiError<&'a str>> {
        self.text = AsciiString::from_ascii(new_str)?;
        Ok(())
    }

    pub fn get_string(&self) -> &str {
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
