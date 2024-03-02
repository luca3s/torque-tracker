use crate::rendering::CharRect;

use super::widget::{NextWidget, Widget};

pub struct Button {
    text: &'static str,
    rect: CharRect,
    pressed: bool,
    next_widget: NextWidget,
    callback: Box<dyn Fn()>,
}

impl Widget for Button {
    fn draw(&self, buffer: &mut crate::rendering::DrawBuffer, selected: bool) {
        
        buffer.draw_rect(2, CharRect { top: self.rect.top+1, bot: self.rect.bot, right: self.rect.right, left: self.rect.left+1 });
        buffer.draw_box(self.rect, self.pressed);

        // buffer.draw_string(self.text, position, 0, 2)
    }

    fn process_input(&mut self, key_event: &winit::event::KeyEvent) -> Option<usize> {
        todo!()
    }
}

impl Button {
    pub fn new(
        text: &'static str,
        rect: CharRect,
        next_widget: NextWidget,
        cb: impl Fn() + 'static,
    ) -> Self {
        Button {
            text,
            rect,
            callback: Box::new(cb),
            pressed: false,
            next_widget,
        }
    }
}
