use winit::event::{KeyEvent, Modifiers};

use crate::visual::draw_buffer::DrawBuffer;

use super::help_page::HelpPage;

pub trait Page {
    fn draw(&self, draw_buffer: &mut DrawBuffer);
    fn draw_constant(&self, draw_buffer: &mut DrawBuffer);

    fn update(&mut self);
    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent);
}

enum PagesEnum {
    Help,
}

pub struct AllPages {
    help: HelpPage,

    current: PagesEnum,
}

impl Page for AllPages {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw(draw_buffer),
        }
    }

    fn draw_constant(&self, draw_buffer: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw_constant(draw_buffer),
        }
    }

    // different from the other functions as this updates all pages, so they are on the current state if they need to be rendered now
    // unsure if this is the best way to do it, the audio API needs to be worked out more to decide
    fn update(&mut self) {
        self.help.update();
    }

    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) {
        match self.current {
            PagesEnum::Help => self.help.process_key_event(modifiers, key_event),
        }
    }
}

impl AllPages {
    pub fn new() -> Self {
        AllPages {
            help: HelpPage::new(),
            current: PagesEnum::Help,
        }
    }
}
