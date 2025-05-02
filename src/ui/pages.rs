mod help_page;
mod order_list;
mod pattern;
mod song_directory_config_page;

use std::collections::VecDeque;

use help_page::HelpPage;
use order_list::{OrderListPage, OrderListPageEvent};
use pattern::{PatternPage, PatternPageEvent};
use song_directory_config_page::{SDCChange, SongDirectoryConfigPage};
use winit::{
    event::{KeyEvent, Modifiers},
    event_loop::EventLoopProxy,
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect, WINDOW_SIZE_CHARS},
    draw_buffer::DrawBuffer,
};

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
    (@function rsp: $response:ty; $($name:ident),*) => {
        fn get_widget_mut(&mut self, idx: usize) -> &mut dyn crate::ui::widgets::Widget<Response = $response> {
            paste::paste! (
                $(if idx == Self::[<$name:upper>] { &mut self.$name } else)*
                { panic!("invalid index {:?}", idx) }
            )
        }
        fn get_widget(&self, idx: usize) -> &dyn crate::ui::widgets::Widget<Response = $response> {
            paste::paste! (
                $(if idx == Self::[<$name:upper>] { &self.$name } else)*
                { panic!("invalid index {:?}", idx) }
            )
        }
    };
    // inital with more than one name
    (response: $response:ty; $list_name: ident { $name:ident: $type:ty, $($n:ident: $t:ty),* }) => (
        struct $list_name {
            selected: usize,
            $name: $type,
            $($n: $t),*
        }

        impl $list_name {
            paste::paste!(
                const [<$name:upper>]: usize = 0;
            );
            const INDEX_RANGE: std::ops::Range<usize> = 0..Self::WIDGET_COUNT;

            pub fn draw_widgets(&self, draw_buffer: &mut crate::draw_buffer::DrawBuffer) {
                for widget in Self::INDEX_RANGE {
                    let is_selected = widget == self.selected;
                    self.get_widget(widget).draw(draw_buffer, is_selected);
                }
            }

            pub fn process_input(
                &mut self,
                key_event: &winit::event::KeyEvent,
                modifiers: &winit::event::Modifiers,
                events: &mut std::collections::VecDeque<crate::app::GlobalEvent>,
            ) -> WidgetResponse<$response> {
                self.get_widget_mut(self.selected).process_input(modifiers, key_event, events)
            }

            crate::ui::pages::create_widget_list!($($n),* ; $name);
            crate::ui::pages::create_widget_list!(@function rsp: $response; $name, $($n),*);
        }
    );
    // last name
    ($name:ident ; $prev:ident) => (
        // const $name: usize = $num;
        paste::paste!(
            const [<$name:upper>]: usize = Self::[<$prev:upper>] + 1usize;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PagesEnum {
    Help,
    SongDirectoryConfig,
    Pattern,
    OrderList,
}

#[derive(Debug)]
pub enum PageEvent {
    Sdc(SDCChange),
    Pattern(PatternPageEvent),
    OrderList(OrderListPageEvent),
}

impl PageEvent {
    fn get_page(&self) -> PagesEnum {
        match self {
            PageEvent::Sdc(_) => PagesEnum::SongDirectoryConfig,
            PageEvent::Pattern(_) => PagesEnum::Pattern,
            PageEvent::OrderList(_) => PagesEnum::OrderList,
        }
    }
}

pub struct AllPages {
    help: HelpPage,
    song_directory_config: SongDirectoryConfigPage,
    pattern: PatternPage,
    order_list: OrderListPage,

    const_draw_needed: bool,
    current: PagesEnum,
}

impl AllPages {
    pub fn new(proxy: EventLoopProxy<GlobalEvent>) -> Self {
        AllPages {
            help: HelpPage::new(),
            song_directory_config: SongDirectoryConfigPage::new(),
            pattern: PatternPage::new(proxy),
            current: PagesEnum::SongDirectoryConfig,
            order_list: OrderListPage::new(),
            const_draw_needed: true,
        }
    }

    fn get_title(&self) -> &'static str {
        match self.current {
            PagesEnum::Help => "Help",
            PagesEnum::SongDirectoryConfig => "Song Variables & Directory Configuration (F12)",
            PagesEnum::Pattern => "Pattern Editor (F2)",
            PagesEnum::OrderList => match self.order_list.mode() {
                order_list::Mode::Volume => "Order List and Channel Volume (F11)",
                order_list::Mode::Panning => "Order List and Panning (F11)",
            },
        }
    }

    fn get_page(&self) -> &dyn Page {
        match self.current {
            PagesEnum::Help => &self.help,
            PagesEnum::SongDirectoryConfig => &self.song_directory_config,
            PagesEnum::Pattern => &self.pattern,
            PagesEnum::OrderList => &self.order_list,
        }
    }

    fn get_page_mut(&mut self) -> &mut dyn Page {
        match self.current {
            PagesEnum::Help => &mut self.help,
            PagesEnum::SongDirectoryConfig => &mut self.song_directory_config,
            PagesEnum::Pattern => &mut self.pattern,
            PagesEnum::OrderList => &mut self.order_list,
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
        self.get_page_mut().draw(draw_buffer);
    }

    pub fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        // draw page title
        let title = self.get_title();
        let middle = WINDOW_SIZE_CHARS.0 / 2;
        let str_start = middle - (title.len() / 2);
        draw_buffer.draw_string(title, CharPosition::new(str_start, 11), 0, 2);
        const DOTTED: [u8; 8] = [0, 0, 0, 0b01010101, 0, 0, 0, 0];
        draw_buffer.draw_rect(2, CharRect::new(11, 11, str_start - 1, str_start - 1));
        draw_buffer.draw_rect(
            2,
            CharRect::new(11, 11, str_start + title.len(), str_start + title.len()),
        );
        for x in 1..=(str_start - 2) {
            draw_buffer.draw_char(DOTTED, CharPosition::new(x, 11), 1, 2);
        }
        for x in (str_start + title.len() + 1)..=(WINDOW_SIZE_CHARS.0 - 2) {
            draw_buffer.draw_char(DOTTED, CharPosition::new(x, 11), 1, 2);
        }
        // draw page const
        self.get_page_mut().draw_constant(draw_buffer);
        self.const_draw_needed = false;
    }

    // add key_events for changing pages here
    pub fn process_key_event(
        &mut self,
        modifiers: &Modifiers,
        key_event: &KeyEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        if key_event.state.is_pressed() && modifiers.state().is_empty() {
            if key_event.logical_key == Key::Named(NamedKey::F1) {
                self.switch_page(PagesEnum::Help);
                return PageResponse::RequestRedraw;
            } else if key_event.logical_key == Key::Named(NamedKey::F2) {
                self.switch_page(PagesEnum::Pattern);
                return PageResponse::RequestRedraw;
            } else if key_event.logical_key == Key::Named(NamedKey::F11) {
                if self.current == PagesEnum::OrderList {
                    self.order_list.switch_mode();
                    self.request_draw_const();
                    return PageResponse::RequestRedraw;
                } else {
                    self.switch_page(PagesEnum::OrderList);
                    return PageResponse::RequestRedraw;
                }
            } else if key_event.logical_key == Key::Named(NamedKey::F12) {
                self.switch_page(PagesEnum::SongDirectoryConfig);
                return PageResponse::RequestRedraw;
            }
        }

        self.get_page_mut()
            .process_key_event(modifiers, key_event, events)
    }

    pub fn process_page_event(
        &mut self,
        event: PageEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        let page = event.get_page();
        let response = match event {
            PageEvent::Sdc(change) => self.song_directory_config.ui_change(change),
            PageEvent::Pattern(event) => self.pattern.process_event(event, events),
            PageEvent::OrderList(event) => self.order_list.process_event(event),
        };

        if page == self.current {
            response
        } else {
            PageResponse::None
        }
    }
}
