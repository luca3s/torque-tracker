use ascii::{AsciiChar, AsciiString};
use font8x8::UnicodeFonts;
use winit::keyboard::{Key, NamedKey};

use crate::{
    app::EventQueue,
    coordinates::{CharPosition, WINDOW_SIZE},
    draw_buffer::DrawBuffer,
    ui::widgets::StandardResponse,
};

use super::{NextWidget, Widget, WidgetResponse};

pub struct TextInScroll<R> {
    pos: CharPosition,
    width: usize,
    text: AsciiString,
    next_widget: NextWidget,
    callback: Box<dyn Fn(&str) -> R>,
    cursor_pos: usize,
    scroll_offset: usize,
}

impl<R> Widget for TextInScroll<R> {
    type Response = R;
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        draw_buffer.draw_string_length(
            &self.text.as_str()[self.scroll_offset..],
            self.pos,
            self.width,
            2,
            0,
        );
        // draw the cursor by overdrawing a letter
        if selected {
            let cursor_char_pos =
                self.pos + CharPosition::new(self.cursor_pos - self.scroll_offset, 0);
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
        _: &mut EventQueue<'_>,
    ) -> WidgetResponse<R> {
        if !key_event.state.is_pressed() {
            return WidgetResponse::default();
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
            return WidgetResponse {
                standard: StandardResponse::RequestRedraw,
                extra: Some((self.callback)(self.text.as_str())),
            };
        } else if key_event.logical_key == Key::Named(NamedKey::Space) {
            self.insert_char(AsciiChar::Space);
            return WidgetResponse::request_redraw();
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
            && modifiers.state().is_empty()
        {
            if self.move_cursor_left() {
                return WidgetResponse::request_redraw();
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            && modifiers.state().is_empty()
        {
            if self.move_cursor_right() {
                return WidgetResponse::request_redraw();
            }
        // backspace and delete keys
        // entf on German Keyboards
        } else if key_event.logical_key == Key::Named(NamedKey::Delete) {
            // cant delete if i'm outside the text. also includes text empty
            if self.cursor_pos < self.text.len() {
                let _ = self.text.remove(self.cursor_pos);
                return WidgetResponse {
                    standard: StandardResponse::RequestRedraw,
                    extra: Some((self.callback)(self.text.as_str())),
                };
            }
        } else if key_event.logical_key == Key::Named(NamedKey::Backspace) {
            // super + backspace clears the string
            if modifiers.state().super_key() {
                self.text.clear();
                self.cursor_pos = 0;
                self.scroll_offset = 0;
                return WidgetResponse {
                    standard: StandardResponse::RequestRedraw,
                    extra: Some((self.callback)(self.text.as_str())),
                };
            // if string is empty we cant remove from it
            } else if modifiers.state().is_empty() && !self.text.is_empty() {
                // if cursor at position 0 backspace starts to behave like delete, no idea why original is like that
                if self.cursor_pos == 0 {
                    let _ = self.text.remove(0);
                } else {
                    let _ = self.text.remove(self.cursor_pos - 1);
                    self.move_cursor_left();
                }

                return WidgetResponse {
                    standard: StandardResponse::RequestRedraw,
                    extra: Some((self.callback)(self.text.as_str())),
                };
            }
        // next widget
        } else {
            return self.next_widget.process_key_event(key_event, modifiers);
        }
        WidgetResponse::default()
    }

    #[cfg(feature = "accesskit")]
    fn build_tree(&self, tree: &mut Vec<(accesskit::NodeId, accesskit::Node)>) {
        todo!() // probably very similar / the same as the regular text in
    }
}

impl<R> TextInScroll<R> {
    pub fn new(
        pos: CharPosition,
        width: usize,
        next_widget: NextWidget,
        cb: impl Fn(&str) -> R + 'static,
    ) -> Self {
        assert!(pos.x() + width < WINDOW_SIZE.0);
        // right and left keys are used in the widget itself. doeesnt make sense to put NextWidget there
        assert!(next_widget.right.is_none());
        assert!(next_widget.left.is_none());

        Self {
            pos,
            width,
            text: AsciiString::new(), // size completely unknown, so i don't allocate
            next_widget,
            callback: Box::new(cb),
            cursor_pos: 0,
            scroll_offset: 0,
        }
    }

    pub fn set_string<'a>(
        &mut self,
        new_str: &'a str,
    ) -> Result<R, ascii::FromAsciiError<&'a str>> {
        self.text = AsciiString::from_ascii(new_str)?;
        self.text.truncate(self.width);
        if self.cursor_pos > self.text.len() {
            self.cursor_pos = self.text.len();
        }
        // never tested could be buggy
        self.scroll_offset = self.text.len().saturating_sub(self.width);

        Ok((self.callback)(self.text.as_str()))
    }

    fn insert_char(&mut self, char: AsciiChar) -> R {
        self.text.insert(self.cursor_pos, char);

        self.move_cursor_right();

        (self.callback)(self.text.as_str())
    }

    // returns if it moved
    fn move_cursor_left(&mut self) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            if self.cursor_pos < self.scroll_offset {
                self.scroll_offset -= 1;
            }
            true
        } else {
            false
        }
    }

    // returns if it moved
    fn move_cursor_right(&mut self) -> bool {
        if self.cursor_pos < self.text.len() {
            self.cursor_pos += 1;
            if self.cursor_pos > self.scroll_offset + self.width {
                self.scroll_offset += 1;
            }
            true
        } else {
            false
        }
    }

    pub fn get_str(&self) -> &str {
        self.text.as_str()
    }
}
