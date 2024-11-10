use std::any::Any;

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::visual::{draw_buffer::DrawBuffer, event_loop::GlobalEvent};

pub type RequestRedraw = bool;

pub trait Widget {
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool);
    /// returns a Some(usize) if the next widget gets selected
    fn process_input(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> WidgetResponse;

    fn update(&mut self) -> RequestRedraw {
        false
    }
}

// SwitchFocus also has to request a redraw
pub enum WidgetResponse {
    SwitchFocus(usize),
    RequestRedraw,
    GlobalEvent(GlobalEvent),
    None,
}

impl From<Option<usize>> for WidgetResponse {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(num) => Self::SwitchFocus(num),
            None => Self::None,
        }
    }
}

// type needed due to limitation in type system. see: https://lucumr.pocoo.org/2022/1/7/as-any-hack/
pub trait WidgetAny: Any + Widget {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// implement WidgetAny for all Widget Types
impl<T: Any + Widget> WidgetAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct NextWidget {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
    pub tab: Option<usize>,
    pub shift_tab: Option<usize>,
}

impl NextWidget {
    /// supposed to be called from Widgets, that own a NextWidget after catching their custom KeyEvents to pick a return
    pub fn process_key_event(&self, key_event: &KeyEvent, modifiers: &Modifiers) -> Option<usize> {
        if !key_event.state.is_pressed() {
            return None;
        }

        if key_event.logical_key == Key::Named(NamedKey::ArrowUp) && modifiers.state().is_empty() {
            self.up
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
            && modifiers.state().is_empty()
        {
            self.down
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            && modifiers.state().is_empty()
        {
            self.right
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
            && modifiers.state().is_empty()
        {
            self.left
        } else if key_event.logical_key == Key::Named(NamedKey::Tab) {
            if modifiers.state() == ModifiersState::SHIFT {
                self.shift_tab
            } else {
                self.tab
            }
        } else {
            None
        }
    }
}
