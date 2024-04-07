use std::any::Any;

use winit::event::{KeyEvent, Modifiers};

use crate::visual::draw_buffer::DrawBuffer;

pub trait Widget {
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool);
    /// returns a Some(usize) if the next widget gets selected
    fn process_input(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> Option<usize>;
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
