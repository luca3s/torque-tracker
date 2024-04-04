use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::visual::{coordinates::CharRect, draw_buffer::DrawBuffer};

use super::widget::{NextWidget, Widget};

pub struct Button {
    text: &'static str,
    rect: CharRect,
    pressed: bool,
    next_widget: NextWidget,
    callback: Box<dyn Fn()>,
}

impl Widget for Button {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool) {
        let string_offset = {
            let half_string = self.text.len() / 2;
            if self.text.len() % 2 == 0 {
                half_string - 1
            } else {
                half_string
            }
        };

        // buffer.draw_rect(Self::BACKGROUND_COLOR, self.rect);

        // buffer.draw_box(self.rect, self.pressed);

        // new box drawing, is probably better but still needs work
        let colors = match self.pressed {
            true => (Self::BOTRIGHT_COLOR, Self::TOPLEFT_COLOR),
            false => (Self::TOPLEFT_COLOR, Self::BOTRIGHT_COLOR),
        };

        buffer.draw_box(self.rect, Self::BACKGROUND_COLOR, colors.0, colors.1);

        // let text_color = match self.pressed || selected {
        //     true => 11,
        //     false => 0,
        // };
        // buffer.draw_string(
        //     self.text,
        //     (
        //         ((self.rect.right - self.rect.left) / 2 + self.rect.left) - string_offset,
        //         (self.rect.bot - self.rect.top) / 2 + self.rect.top,
        //     ),
        //     text_color,
        //     Self::BACKGROUND_COLOR,
        // )
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> Option<usize> {
        if !key_event.repeat {
            if key_event.logical_key == Key::Named(NamedKey::Space)
                || key_event.logical_key == Key::Named(NamedKey::Enter)
            {
                if key_event.state.is_pressed() {
                    self.pressed = true;
                } else {
                    // only call the callback on a release event if the button was pressed in before
                    // meaning if the user pressed the key, then changed focus to another button and then releases
                    // no button should be triggered
                    if self.pressed {
                        (self.callback)();
                    }
                    self.pressed = false;
                }
                // change focus
            } else if key_event.logical_key == Key::Named(NamedKey::Tab)
                && key_event.state.is_pressed()
            {
                if modifiers.state() == ModifiersState::SHIFT {
                    self.pressed = false;
                    return self.next_widget.shift_tab;
                } else if modifiers.state().is_empty() {
                    self.pressed = false;
                    return self.next_widget.tab;
                }
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
                && modifiers.state().is_empty()
            {
                self.pressed = false;
                return self.next_widget.right;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
                && modifiers.state().is_empty()
            {
                self.pressed = false;
                return self.next_widget.left;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowUp)
                && modifiers.state().is_empty()
            {
                self.pressed = false;
                return self.next_widget.up;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
                && modifiers.state().is_empty()
            {
                self.pressed = false;
                return self.next_widget.down;
            }
        }
        None
    }
}

impl Button {
    const BACKGROUND_COLOR: usize = 2;
    const TOPLEFT_COLOR: usize = 3;
    const BOTRIGHT_COLOR: usize = 1;

    pub fn new(
        text: &'static str,
        rect: CharRect,
        next_widget: NextWidget,
        cb: impl Fn() + 'static,
    ) -> Self {
        // assert!(
        //     rect.bot() - rect.top() >= 3,
        //     "buttons needs to be at least 3 rows high"
        // );
        Button {
            text,
            rect,
            callback: Box::new(cb),
            pressed: false,
            next_widget,
        }
    }
}
