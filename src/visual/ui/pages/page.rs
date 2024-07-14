use winit::{
    event::{KeyEvent, Modifiers},
    event_loop::EventLoopProxy,
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::visual::{
    draw_buffer::DrawBuffer, event_loop::CustomWinitEvent, ui::widgets::widget::RequestRedraw,
};

use super::{help_page::HelpPage, song_directory_config_page::SongDirectoryConfigPage};

pub trait Page {
    const BACKGROUND_COLOR: usize = 2;

    fn draw(&mut self, draw_buffer: &mut DrawBuffer);
    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer);

    fn update(&mut self) -> RequestRedraw;
    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> PageResponse;
}

pub enum PageResponse {
    RequestRedraw,
    None,
}

#[derive(Clone, Copy)]
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
    fn update(&mut self) -> RequestRedraw {
        let help = self.help.update();
        let song_directory_config = self.song_directory_config.update();
        match self.current {
            PagesEnum::Help => help,
            PagesEnum::SongDirectoryConfig => song_directory_config,
        }
    }

    // add key_events for changing pages here
    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> PageResponse {
        if key_event.logical_key == Key::Named(NamedKey::F1) {
            self.switch_page(PagesEnum::Help);
            println!("open help page");
            PageResponse::RequestRedraw
        } else if key_event.logical_key == Key::Named(NamedKey::F5) {
            if modifiers.state() == ModifiersState::SHIFT {
                println!("open preferences page")
            } else if modifiers.state().is_empty() {
                println!("open info page");
            }
            return PageResponse::None;
        } else if key_event.logical_key == Key::Named(NamedKey::F12) && modifiers.state().is_empty()
        {
            self.switch_page(PagesEnum::SongDirectoryConfig);
            return PageResponse::RequestRedraw;
        } else {
            // make the current page handle the event
            return match self.current {
                PagesEnum::Help => self.help.process_key_event(modifiers, key_event),
                PagesEnum::SongDirectoryConfig => self
                    .song_directory_config
                    .process_key_event(modifiers, key_event),
            };
        }
    }
}

impl AllPages {
    pub fn new(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        AllPages {
            help: HelpPage::new(),
            song_directory_config: SongDirectoryConfigPage::new(event_loop_proxy),
            current: PagesEnum::SongDirectoryConfig,
            const_draw_needed: true,
        }
    }

    pub fn switch_page(&mut self, next_page: PagesEnum) {
        self.current = next_page;
        self.request_draw_const();
    }

    pub fn request_draw_const(&mut self) {
        self.const_draw_needed = true;
    }
}
