use winit::event::{KeyEvent, Modifiers};

use crate::visual::draw_buffer::DrawBuffer;

use super::{help_page::HelpPage, song_directory_config_page::SongDirectoryConfigPage};

pub trait Page {
    const BACKGROUND_COLOR: usize = 2;

    fn draw(&mut self, draw_buffer: &mut DrawBuffer);
    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer);

    fn update(&mut self);
    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent);
}

pub enum PagesEnum {
    Help,
    SongDirectoryConfig,
}

pub struct AllPages {
    help: HelpPage,
    song_directory_config: SongDirectoryConfigPage,

    const_draw_needed: bool,
    current: PagesEnum,
}

// pretty dumb implementation with a lot of code duplication
// should probably be done more elegantly with a macro or some stuff
impl Page for AllPages {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        if self.const_draw_needed {
            self.draw_constant(draw_buffer);
        }

        match self.current {
            PagesEnum::Help => self.help.draw(draw_buffer),
            PagesEnum::SongDirectoryConfig => self.song_directory_config.draw(draw_buffer),
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw_constant(draw_buffer),
            PagesEnum::SongDirectoryConfig => self.song_directory_config.draw_constant(draw_buffer),
        }
        self.const_draw_needed = false;
    }

    // different from the other functions as this updates all pages, so they are on the current state if they need to be rendered now
    // unsure if this is the best way to do it, the audio API needs to be worked out more to decide
    fn update(&mut self) {
        self.help.update();
        self.song_directory_config.update()
    }

    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) {
        match self.current {
            PagesEnum::Help => self.help.process_key_event(modifiers, key_event),
            PagesEnum::SongDirectoryConfig => self
                .song_directory_config
                .process_key_event(modifiers, key_event),
        }
    }
}

impl AllPages {
    pub fn new() -> Self {
        AllPages {
            help: HelpPage::new(),
            song_directory_config: SongDirectoryConfigPage::new(),
            current: PagesEnum::SongDirectoryConfig,
            const_draw_needed: true,
        }
    }

    pub fn switch_page(&mut self, new_page: PagesEnum) {
        self.current = new_page;
        self.const_draw_needed = true;
    }
}
