mod help_page;
mod pattern;
mod song_directory_config_page;

use std::collections::VecDeque;

use help_page::HelpPage;
use pattern::PatternPage;
use song_directory_config_page::{SDCChange, SongDirectoryConfigPage};
use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::{app::GlobalEvent, draw_buffer::DrawBuffer};

use super::widgets::widget::Widget;

pub trait Page {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer);
    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer);

    fn process_key_event(
        &mut self,
        modifiers: &Modifiers,
        key_event: &KeyEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse;
}

/// creates a struct called WidgetList with all the specified fields.
///
/// inserts a const index for every field in WidgetList into the specified struct
/// as well as a function to query for the Widgets from a those indices.
///
/// Needs at least one fields to work. If it is less, just write from hand.
macro_rules! create_widget_list {
    (@function $($name:ident),*) => {
        fn get_widget_mut(&mut self, idx: usize) -> &mut dyn Widget {
            paste::paste! (
                $(if idx == Self::[<$name:upper>] { &mut self.$name } else)*
                { panic!("invalid index {:?}", idx) }
            )
        }
        fn get_widget(&self, idx: usize) -> &dyn Widget {
            paste::paste! (
                $(if idx == Self::[<$name:upper>] { &self.$name } else)*
                { panic!("invalid index {:?}", idx) }
            )
        }
    };
    // inital with more than one name
    ({ $name:ident: $type:ty, $($n:ident: $t:ty),* }) => (
        struct WidgetList {
            $name: $type,
            $($n: $t),*
        }

        impl WidgetList {
            paste::paste!(
                const [<$name:upper>]: usize = 0;
            );
            const INDEX_RANGE: std::ops::Range<usize> = 0..Self::WIDGET_COUNT;
            crate::ui::pages::create_widget_list!($($n),* ; $name);
            crate::ui::pages::create_widget_list!(@function $name, $($n),*);
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
        crate::ui::pages::create_widget_list!($($n),+ ; $name);
    );
}

pub(crate) use create_widget_list;

pub enum PageResponse {
    RequestRedraw,
    // GlobalEvent(GlobalEvent),
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PagesEnum {
    Help,
    SongDirectoryConfig,
    Pattern,
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
    pattern: PatternPage,

    const_draw_needed: bool,
    current: PagesEnum,
}

impl AllPages {
    pub fn new() -> Self {
        AllPages {
            help: HelpPage::new(),
            song_directory_config: SongDirectoryConfigPage::new(),
            pattern: PatternPage::new(),
            current: PagesEnum::SongDirectoryConfig,
            const_draw_needed: true,
        }
    }

    /// requests a redraw if it is needed
    pub fn switch_page(&mut self, next_page: PagesEnum) -> PageResponse {
        if next_page != self.current {
            self.current = next_page;
            self.request_draw_const();
            PageResponse::RequestRedraw
        } else {
            PageResponse::None
        }
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
            PagesEnum::Pattern => self.pattern.draw(draw_buffer),
        }
    }

    pub fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        match self.current {
            PagesEnum::Help => self.help.draw_constant(draw_buffer),
            PagesEnum::SongDirectoryConfig => self.song_directory_config.draw_constant(draw_buffer),
            PagesEnum::Pattern => self.pattern.draw_constant(draw_buffer),
        }
        self.const_draw_needed = false;
    }

    // add key_events for changing pages here
    pub fn process_key_event(
        &mut self,
        modifiers: &Modifiers,
        key_event: &KeyEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        if key_event.logical_key == Key::Named(NamedKey::F1) && key_event.state.is_pressed() {
            self.switch_page(PagesEnum::Help)
        } else if key_event.logical_key == Key::Named(NamedKey::F2)
            && modifiers.state().is_empty()
            && key_event.state.is_pressed()
        {
            self.switch_page(PagesEnum::Pattern)
        } else if key_event.logical_key == Key::Named(NamedKey::F5) && key_event.state.is_pressed()
        {
            if modifiers.state() == ModifiersState::SHIFT {
                println!("open preferences page")
            } else if modifiers.state().is_empty() {
                println!("open info page");
            }
            PageResponse::None
        } else if key_event.logical_key == Key::Named(NamedKey::F12)
            && modifiers.state().is_empty()
            && key_event.state.is_pressed()
        {
            self.switch_page(PagesEnum::SongDirectoryConfig)
        } else {
            // make the current page handle the event
            match self.current {
                PagesEnum::Help => self.help.process_key_event(modifiers, key_event, events),
                PagesEnum::SongDirectoryConfig => self
                    .song_directory_config
                    .process_key_event(modifiers, key_event, events),
                PagesEnum::Pattern => self.pattern.process_key_event(modifiers, key_event, events),
            }
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
