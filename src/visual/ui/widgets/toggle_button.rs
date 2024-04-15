use std::{cell::Cell, rc::Rc};

use crate::visual::coordinates::CharRect;

use super::{
    button::Button,
    widget::{NextWidget, Widget, WidgetResponse},
};

// dont need to store a callback as it gets pushed into the inner button callback
pub struct ToggleButton<T: Copy + PartialEq> {
    button: Button,

    variant: T,
    state: Rc<Cell<T>>,
}

impl<T: Copy + PartialEq> Widget for ToggleButton<T> {
    fn draw(&self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer, selected: bool) {
        self.button
            .draw_overwrite_pressed(draw_buffer, selected, self.variant == self.state.get())
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> WidgetResponse {
        self.button.process_input(modifiers, key_event)
    }
}

impl<T: Copy + PartialEq + 'static> ToggleButton<T> {
    pub fn new(
        text: &'static str,
        rect: CharRect,
        next_widget: NextWidget,
        variant: T,
        state: Rc<Cell<T>>,
        cb: impl Fn(T) + 'static,
    ) -> Self {
        let button_clone = state.clone();
        let button = Button::new(text, rect, next_widget, move || {
            button_clone.set(variant);
            (cb)(variant);
        });
        Self {
            button,
            variant,
            state,
        }
    }
}
