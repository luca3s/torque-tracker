use std::{cell::Cell, collections::VecDeque, rc::Rc};

use crate::{app::GlobalEvent, coordinates::CharRect, draw_buffer::DrawBuffer};

use super::{button::Button, NextWidget, Widget, WidgetResponse};

// dont need to store a callback as it gets pushed into the inner button callback
pub struct ToggleButton<T: Copy + PartialEq, R> {
    button: Button<R>,

    variant: T,
    state: Rc<Cell<T>>,
}

impl<T: Copy + PartialEq, R> Widget for ToggleButton<T, R> {
    type Response = R;
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        self.button
            .draw_overwrite_pressed(draw_buffer, selected, self.variant == self.state.get())
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        event: &mut VecDeque<GlobalEvent>,
    ) -> WidgetResponse<R> {
        self.button.process_input(modifiers, key_event, event)
    }
}

impl<T: Copy + PartialEq + 'static, R> ToggleButton<T, R> {
    pub fn new(
        text: &'static str,
        rect: CharRect,
        next_widget: NextWidget,
        variant: T,
        state: Rc<Cell<T>>,
        cb: impl Fn(T) -> R + 'static,
    ) -> Self {
        let button_clone = state.clone();
        let button = Button::new(text, rect, next_widget, move || {
            button_clone.set(variant);
            (cb)(variant)
        });
        Self {
            button,
            variant,
            state,
        }
    }
}
