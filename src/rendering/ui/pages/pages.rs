use winit::event::{KeyEvent, Modifiers};

use crate::rendering::DrawBuffer;

use super::help_page::HelpPage;

pub trait Page {
    fn draw(&self, render_state: &mut DrawBuffer);
    fn draw_constant(&self, render_state: &mut DrawBuffer);

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
    fn draw(&self, render_state: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw(render_state),
        }
    }

    fn draw_constant(&self, render_state: &mut DrawBuffer) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) {
        todo!()
    }
}

impl AllPages {
    pub fn new() -> Self {
        AllPages { help: HelpPage::new(), current: PagesEnum::Help }
    }

    fn update_all(&mut self) {
        self.help.update();
    }
}
