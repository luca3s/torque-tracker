use std::{rc::Rc, cell::Cell};

use winit::event::{KeyEvent, Modifiers};

use crate::visual::{draw_buffer::DrawBuffer, ui::pages::page::PagesEnum};

pub enum DialogResponse {
    RequestRedraw,
    // should also close all Dialogs
    SwitchToPage(PagesEnum),
    Close,
    None,
}

pub trait Dialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer);
    fn process_input(&mut self, key_event: &KeyEvent, modifiers: &Modifiers) -> DialogResponse;
}

pub struct DialogManager {
    stack: Vec<Box<dyn Dialog>>,
    new_dialog: Rc<Cell<Option<Box<dyn Dialog>>>>,
}

impl DialogManager {
    pub fn new() -> Self {
        // try to match the capacity to the actually used maximum depth
        Self {
            stack: Vec::with_capacity(3),
            new_dialog: Rc::new(Cell::new(None)),
        }
    }

    pub fn active_dialog_mut(&mut self) -> Option<&mut dyn Dialog> {
        match self.stack.last_mut() {
            Some(dialog) => Some(dialog.as_mut()),
            None => None,
        }
    }

    // pub fn open_dialog(&mut self, dialog: Box<dyn Dialog>) {
    //     self.stack.push(dialog);
    // }

    pub fn try_open_dialog(&mut self) {
        // let new = self.new_dialog.replace(None);
        if let Some(new) = self.new_dialog.replace(None) {
            self.stack.push(new);
        }
    }

    pub fn close_dialog(&mut self) {
        self.stack.pop();
    }

    pub fn close_all(&mut self) {
        self.stack.clear();
    }

    /// draws all currently open dialogs
    pub fn draw(&self, draw_buffer: &mut DrawBuffer) {
        self.stack
            .iter()
            .for_each(|dialog| dialog.draw(draw_buffer));
    }

    pub fn get_dialog_send(&self) -> Rc<Cell<Option<Box<dyn Dialog>>>> {
        self.new_dialog.clone()
    }
}
