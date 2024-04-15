use winit::event::{KeyEvent, Modifiers};

use crate::visual::draw_buffer::DrawBuffer;

pub enum DialogResponse {
    RequestRedraw,
    Close,
    None,
}

pub trait Dialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer);
    fn process_input(&mut self, key_event: &KeyEvent, modifiers: &Modifiers) -> DialogResponse;
}

pub struct DialogState {
    stack: Vec<Box<dyn Dialog>>,
}

impl DialogState {
    pub fn new() -> Self {
        // try to match the capacity to the actually used maximum depth
        Self {
            stack: Vec::with_capacity(3),
        }
    }

    pub fn active_dialog_mut(&mut self) -> Option<&mut dyn Dialog> {
        match self.stack.last_mut() {
            Some(dialog) => Some(dialog.as_mut()),
            None => None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stack.len() > 0
    }

    pub fn open_dialog(&mut self, dialog: Box<dyn Dialog>) {
        self.stack.push(dialog);
    }

    pub fn close_dialog(&mut self) {
        self.stack.pop();
    }

    /// draws all currently open dialogs
    pub fn draw(&self, draw_buffer: &mut DrawBuffer) {
        println!("draw dialogs");
        self.stack
            .iter()
            .for_each(|dialog| dialog.draw(draw_buffer));
    }
}
