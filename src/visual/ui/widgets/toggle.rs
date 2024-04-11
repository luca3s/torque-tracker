use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    coordinates::{CharPosition, CharRect, WINDOW_SIZE},
    draw_buffer::DrawBuffer,
};

use super::widget::{NextWidget, Widget};

pub struct Toggle<T: Copy> {
    pos: CharPosition,
    width: usize,
    state: usize,
    next_widget: NextWidget,
    // know its bad, dont know how to do it better, except put everything as static
    variants: Box<[(T, &'static str)]>,
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
    ) -> Option<usize> {
        if key_event.logical_key == Key::Named(NamedKey::Space)
            && modifiers.state().is_empty()
            && key_event.state.is_pressed()
        {
            self.next();
            (*self.cb)(self.variants[self.state].0);
        } else {
            return self.next_widget.process_key_event(key_event, modifiers);
        }
        None
    }
}

impl<T: Copy> Toggle<T> {
    pub fn new(
        pos: CharPosition,
        width: usize,
        next_widget: NextWidget,
        variants: &[(T, &'static str)],
        cb: impl Fn(T) + 'static,
    ) -> Self {
        assert!(pos.x() + width < WINDOW_SIZE.0);

        Self {
            pos,
            width,
            state: 0,
            next_widget,
            variants: variants.into(),
            cb: Box::new(cb),
        }
    }

    pub fn set_state(&mut self, variant: usize) {
        assert!(variant < self.variants.len());
        self.state = variant;
    }

    pub fn next(&mut self) {
        if self.state + 1 == self.variants.len() {
            self.set_state(0);
        } else {
            self.set_state(self.state + 1);
        }
    }

    pub fn get_variant(&self) -> T {
        self.variants[self.state].0
    }
}
