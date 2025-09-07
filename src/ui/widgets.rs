pub mod button;
pub mod slider;
pub mod text_in;
pub mod text_in_scroll;
pub mod toggle;
pub mod toggle_button;

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::{app::EventQueue, draw_buffer::DrawBuffer, ui::pages::PageResponse};

pub(crate) trait Widget {
    type Response;
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool);
    /// returns a Some(usize) if the next widget gets selected
    fn process_input(
        &mut self,
        modifiers: &Modifiers,
        key_event: &KeyEvent,
        events: &mut EventQueue<'_>,
    ) -> WidgetResponse<Self::Response>;
}

#[derive(Debug)]
pub struct WidgetResponse<R> {
    pub standard: StandardResponse,
    pub extra: Option<R>,
}

impl<R> Default for WidgetResponse<R> {
    fn default() -> Self {
        Self {
            standard: StandardResponse::default(),
            extra: None,
        }
    }
}

impl<R> WidgetResponse<R> {
    pub fn request_redraw() -> Self {
        Self {
            standard: StandardResponse::RequestRedraw,
            extra: None,
        }
    }

    pub fn next_widget(value: u8) -> Self {
        Self {
            standard: StandardResponse::SwitchFocus(value),
            extra: None,
        }
    }
}

// SwitchFocus also has to request a redraw
#[derive(Debug, Default)]
pub enum StandardResponse {
    SwitchFocus(u8),
    RequestRedraw,
    // GlobalEvent(GlobalEvent),
    #[default]
    None,
}

impl<R> From<StandardResponse> for WidgetResponse<R> {
    fn from(value: StandardResponse) -> Self {
        Self {
            standard: value,
            extra: None,
        }
    }
}

impl StandardResponse {
    pub fn to_page_resp(self, selected: &mut u8) -> PageResponse {
        match self {
            StandardResponse::SwitchFocus(s) => {
                *selected = s;
                PageResponse::RequestRedraw
            }
            StandardResponse::RequestRedraw => PageResponse::RequestRedraw,
            StandardResponse::None => PageResponse::None,
        }
    }
}

#[derive(Debug, Default)]
pub struct NextWidget {
    pub left: Option<u8>,
    pub right: Option<u8>,
    pub up: Option<u8>,
    pub down: Option<u8>,
    pub tab: Option<u8>,
    pub shift_tab: Option<u8>,
}

impl NextWidget {
    /// supposed to be called from Widgets, that own a NextWidget after catching their custom KeyEvents to pick a return
    pub fn process_key_event<R>(
        &self,
        key_event: &KeyEvent,
        modifiers: &Modifiers,
    ) -> WidgetResponse<R> {
        if !key_event.state.is_pressed() {
            return WidgetResponse::default();
        }

        #[expect(
            non_local_definitions,
            reason = "this is only valid with these specific Option<u8> not in general"
        )]
        impl<R> From<Option<u8>> for WidgetResponse<R> {
            fn from(value: Option<u8>) -> Self {
                Self {
                    standard: match value {
                        Some(num) => StandardResponse::SwitchFocus(num),
                        None => StandardResponse::None,
                    },
                    extra: None,
                }
            }
        }

        if key_event.logical_key == Key::Named(NamedKey::ArrowUp) && modifiers.state().is_empty() {
            self.up.into()
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
            && modifiers.state().is_empty()
        {
            self.down.into()
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            && modifiers.state().is_empty()
        {
            self.right.into()
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
            && modifiers.state().is_empty()
        {
            self.left.into()
        } else if key_event.logical_key == Key::Named(NamedKey::Tab) {
            if modifiers.state() == ModifiersState::SHIFT {
                self.shift_tab.into()
            } else {
                self.tab.into()
            }
        } else {
            WidgetResponse::default()
        }
    }
}
