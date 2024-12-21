mod help_page;
mod song_directory_config_page;

use help_page::HelpPage;
use song_directory_config_page::{SDCChange, SongDirectoryConfigPage};
use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::visual::{app::GlobalEvent, draw_buffer::DrawBuffer};

use super::widgets::widget::Widget;

pub trait Page {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer);
    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer);

    fn process_key_event(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> PageResponse;
}

/// creates a struct called WidgetList with all the specified fields.
/// 
/// inserts a const index for every field in WidgetList into the specified struct
/// as well as a function to query for the Widgets from a those indices.
/// 
/// Needs at least one fields to work. If it is less, just write from hand.
macro_rules! create_widget_list {
    (@function $($name:ident),*) => {
        fn get_widget(&mut self, idx: usize) -> &mut dyn Widget {
            paste::paste! (
                $(if idx == Self::[<$name:upper>] { &mut self.widgets.$name } else)*
                { panic!("invalid index {:?}", idx) }
            )
        }
    };
    // inital with more than one name
    ($page:ident; { $name:ident: $type:ty, $($n:ident: $t:ty),* }) => (
        struct WidgetList {
            $name: $type,
            $($n: $t),*
        }
        impl $page {
            paste::paste!(
                const [<$name:upper>]: usize = 0;
            );
            crate::visual::ui::pages::create_widget_list!($($n),* ; $name);
            crate::visual::ui::pages::create_widget_list!(@function $name, $($n),*);
        }
    );
    // last name
    ($name:ident ; $prev:ident) => (
        // const $name: usize = $num;
        paste::paste!(
            const [<$name:upper>]: usize = Self::[<$prev:upper>] + 1;
            const WIDGET_COUNT: usize = Self::[<$name:upper>] + 1usize;
        );
    );
    // loop over names
    ($name:ident, $($n:ident),+ ; $prev:ident) => (
        // const $name: usize = $num;
        paste::paste!(
            const [<$name:upper>]: usize = Self::[<$prev:upper>] + 1;
        );
        crate::visual::ui::pages::create_widget_list!($($n),+ ; $name);
    );
}

pub(crate) use create_widget_list;

pub enum PageResponse {
    RequestRedraw,
    GlobalEvent(GlobalEvent),
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PagesEnum {
    Help,
    SongDirectoryConfig,
}

pub enum PageEvent {
    Sdc(SDCChange),
}

impl PageEvent {
    fn get_page(&self) -> PagesEnum {
        match self {
            PageEvent::Sdc(_) => PagesEnum::SongDirectoryConfig,
        }
    }
}

pub struct AllPages {
    help: HelpPage,
    song_directory_config: SongDirectoryConfigPage,

    const_draw_needed: bool,
    current: PagesEnum,
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

    pub fn switch_page(&mut self, next_page: PagesEnum) {
        self.current = next_page;
        self.request_draw_const();
    }

    pub fn request_draw_const(&mut self) {
        self.const_draw_needed = true;
    }

    pub fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        if self.const_draw_needed {
            self.draw_constant(draw_buffer);
        }
        match self.current {
            PagesEnum::Help => self.help.draw(draw_buffer),
            PagesEnum::SongDirectoryConfig => self.song_directory_config.draw(draw_buffer),
        }
    }

    pub fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw_constant(draw_buffer),
            PagesEnum::SongDirectoryConfig => self.song_directory_config.draw_constant(draw_buffer),
        }
        self.const_draw_needed = false;
    }

    // pub fn update(&mut self) -> RequestRedraw {
    //     let help = self.help.update();
    //     let song_directory_config = self.song_directory_config.update();
    //     match self.current {
    //         PagesEnum::Help => help,
    //         PagesEnum::SongDirectoryConfig => song_directory_config,
    //     }
    // }

    // add key_events for changing pages here
    pub fn process_key_event(
        &mut self,
        modifiers: &Modifiers,
        key_event: &KeyEvent,
    ) -> PageResponse {
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

    pub fn process_page_event(&mut self, change: PageEvent) -> PageResponse {
        let page = change.get_page();
        let response = match change {
            PageEvent::Sdc(change) => self.song_directory_config.ui_change(change),
        };

        if page == self.current {
            response
        } else {
            PageResponse::None
        }
    }
}
