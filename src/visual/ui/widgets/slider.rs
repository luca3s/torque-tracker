use std::ops::{AddAssign, Deref, SubAssign};

use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::visual::{coordinates::CharRect, draw_buffer::DrawBuffer};

use super::widget::{NextWidget, Widget};

pub struct BoundNumber<const MIN: i16, const MAX: i16> {
    inner: i16,
}

impl<const MIN: i16, const MAX: i16> AddAssign<i16> for BoundNumber<MIN, MAX> {
    fn add_assign(&mut self, rhs: i16) {
        let new = self.inner.saturating_add(rhs);
        if new > MAX {
            self.inner = MAX;
        } else {
            self.inner = new;
        }
    }
}

impl<const MIN: i16, const MAX: i16> SubAssign<i16> for BoundNumber<MIN, MAX> {
    fn sub_assign(&mut self, rhs: i16) {
        let new = self.inner.saturating_sub(rhs);
        if new < MIN {
            self.inner = MIN;
        } else {
            self.inner = new;
        }
    }
}

impl<const MIN: i16, const MAX: i16> Deref for BoundNumber<MIN, MAX> {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const MIN: i16, const MAX: i16> BoundNumber<MIN, MAX> {
    pub const fn new(mut value: i16) -> Self {
        assert!(MIN <= MAX, "MIN must be less than or equal to MAX");
        if value > MAX {
            value = MAX;
        } else if value < MIN {
            value = MIN;
        }

        Self { inner: value }
    }
}

pub struct Slider<const MIN: i16, const MAX: i16> {
    pub number: BoundNumber<MIN, MAX>,

    rect: CharRect,
    next_widget: NextWidget,
    callback: Box<dyn Fn(i16)>,
}

impl<const MIN: i16, const MAX: i16> Widget for Slider<MIN, MAX> {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool) {
        // let pixel_width: i16 = (self.rect.right - self.rect.left).into() * FONT_SIZE;
        // let amount_values = MAX - MIN;
        // let single_value
        todo!()
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> Option<usize> {
        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::ArrowRight) {
                if modifiers.state() == ModifiersState::SHIFT {
                    self.number += 4;
                    (self.callback)(*self.number);
                } else if modifiers.state().is_empty() {
                    self.number += 1;
                    (self.callback)(*self.number);
                }
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft) {
                if modifiers.state() == ModifiersState::SHIFT {
                    self.number -= 4;
                    (self.callback)(*self.number);
                } else {
                    self.number -= 1;
                    (self.callback)(*self.number);
                }
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
                && modifiers.state().is_empty()
            {
                return self.next_widget.down;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowUp)
                && modifiers.state().is_empty()
            {
                return self.next_widget.up;
            } else if key_event.logical_key == Key::Named(NamedKey::Tab) {
                if modifiers.state().is_empty() {
                    return self.next_widget.tab;
                } else if modifiers.state() == ModifiersState::SHIFT {
                    return self.next_widget.shift_tab;
                }
            } else if let Some(text) = &key_event.text {
                if text.chars().all(|c| c.is_ascii_digit()) {
                    todo!("open dialog window to input a value")
                }
            }
        }
        None
    }
}

impl<const MIN: i16, const MAX: i16> Slider<MIN, MAX> {
    pub fn new(
        inital_value: i16,
        rect: CharRect,
        next_widget: NextWidget,
        callback: impl Fn(i16) + 'static,
    ) -> Self {
        assert!(MIN <= MAX, "MIN must be less than or equal to MAX");

        Self {
            number: BoundNumber::new(inital_value),
            rect,
            next_widget,
            callback: Box::new(callback),
        }
    }
}
