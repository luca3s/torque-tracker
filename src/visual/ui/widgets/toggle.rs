use std::collections::VecDeque;

use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect, WINDOW_SIZE},
    draw_buffer::DrawBuffer,
};

use super::widget::{NextWidget, Widget, WidgetResponse};

pub struct Toggle<T: Copy + 'static> {
    pos: CharPosition,
    width: usize,
    state: usize,
    next_widget: NextWidget,
    variants: &'static [(T, &'static str)],
    cb: Box<dyn Fn(T)>,
}

impl<T: Copy> Widget for Toggle<T> {
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        let str = self.variants[self.state].1;
        draw_buffer.draw_rect(
            0,
            CharRect::new(
                self.pos.y(),
                self.pos.y(),
                self.pos.x() + str.len(),
                self.pos.x() + self.width,
            ),
        );
        let (fg_color, bg_color) = match selected {
            true => (0, 3),
            false => (2, 0),
        };
        draw_buffer.draw_string(str, self.pos, fg_color, bg_color);
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        _events: &mut VecDeque<GlobalEvent>,
    ) -> WidgetResponse {
        if key_event.logical_key == Key::Named(NamedKey::Space)
            && modifiers.state().is_empty()
            && key_event.state.is_pressed()
        {
            self.next();
            (*self.cb)(self.variants[self.state].0);
            WidgetResponse::RequestRedraw
        } else {
            self.next_widget
                .process_key_event(key_event, modifiers)
                .into()
        }
    }
}

impl<T: Copy + 'static> Toggle<T> {
    pub fn new(
        pos: CharPosition,
        width: usize,
        next_widget: NextWidget,
        variants: &'static [(T, &'static str)],
        cb: impl Fn(T) + 'static,
    ) -> Self {
        assert!(pos.x() + width < WINDOW_SIZE.0);

        Self {
            pos,
            width,
            state: 0,
            next_widget,
            variants,
            cb: Box::new(cb),
        }
    }

    pub fn next(&mut self) {
        self.state += 1;
        if self.state >= self.variants.len() {
            self.state = 0;
        }
    }

    pub fn get_variant(&self) -> T {
        self.variants[self.state].0
    }
}
