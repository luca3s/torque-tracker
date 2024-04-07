use crate::visual::{coordinates::CharRect, draw_buffer::DrawBuffer};

use super::widget::{NextWidget, Widget};

/// text has max_len of the rect that was given, because the text_in cannot scroll
/// use text_in_scroll for that
pub struct TextIn {
    rect: CharRect,
    text: String,
    next_widget: NextWidget,
    callback: Box<dyn Fn(&mut str)>,
    cursor_pos: usize,
}

impl Widget for TextIn {
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        draw_buffer.draw_string(&self.text, (self.rect.left(), self.rect.top()), 1, 0);
        if selected {
            // still needs to draw the cursor when selected
        }
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> Option<usize> {
        if let Some(text) = &key_event.text {
            if self.cursor_pos >= self.text.len() {
                self.text.push_str(text);
            } else {
                self.text.insert_str(self.cursor_pos, text);
            }
        } else {
        }
        None
    }
}

impl TextIn {
    pub fn new(rect: CharRect, next_widget: NextWidget, cb: impl Fn(&mut str) + 'static) -> Self {
        TextIn {
            rect,
            text: String::with_capacity(rect.right() - rect.left()),
            next_widget,
            callback: Box::new(cb),
            cursor_pos: 0,
        }
    }

    pub fn set_string(&mut self, new_str: &str) {
        self.text = String::from(new_str);
    }

    pub fn get_string(&self) -> &str {
        &self.text
    }
}
