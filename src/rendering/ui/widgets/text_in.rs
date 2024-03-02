use crate::rendering::CharRect;

use super::widget::{NextWidget, Widget};

/// text has max_len of the rect that was given, because the text_in cannot scroll
/// use text_in_scroll for that
struct TextIn {
    rect: CharRect,
    text: String,
    next_widget: NextWidget,
    callback: Box<dyn Fn(&str)>,
    cursor_pos: usize,
}

impl Widget for TextIn {
    fn draw(&self, buffer: &mut crate::rendering::DrawBuffer, selected: bool) {
        todo!()
    }

    fn process_input(&mut self, key_event: &winit::event::KeyEvent) -> Option<usize> {
        todo!()
    }
}

impl TextIn {
    pub fn new(rect: CharRect, next_widget: NextWidget, cb: impl Fn(&str) + 'static) -> Self {
        TextIn {
            rect,
            text: String::with_capacity((rect.right - rect.left).into()),
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
