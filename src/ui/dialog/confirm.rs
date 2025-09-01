use winit::keyboard::{Key, NamedKey};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::{
        pages::create_widget_list,
        widgets::{NextWidget, StandardResponse, WidgetResponse, button::Button},
    },
};

use super::{Dialog, DialogResponse};

create_widget_list!(
    response: Option<GlobalEvent>;
    WidgetList
    {
        ok: Button<Option<GlobalEvent>>,
        cancel: Button<Option<GlobalEvent>>
    }
);

pub struct ConfirmDialog {
    text: &'static str,
    text_pos: CharPosition,
    // computed from the string length
    rect: CharRect,
    widgets: WidgetList,
}

impl ConfirmDialog {
    const OK_RECT: CharRect = CharRect::new(29, 31, 41, 50);
    const CANCEL_RECT: CharRect = CharRect::new(29, 31, 30, 39);
    pub fn new(
        text: &'static str,
        ok_event: fn() -> Option<GlobalEvent>,
        cancel_event: fn() -> Option<GlobalEvent>,
    ) -> Self {
        let width = (text.len() + 10).max(22);
        let per_side = width / 2;
        Self {
            text,
            text_pos: CharPosition::new(40 - per_side + 5, 27),
            widgets: WidgetList {
                selected: WidgetList::OK,
                ok: Button::new(
                    "  Ok",
                    Self::OK_RECT,
                    NextWidget {
                        left: Some(WidgetList::CANCEL),
                        right: Some(WidgetList::CANCEL),
                        tab: Some(WidgetList::CANCEL),
                        shift_tab: Some(WidgetList::CANCEL),
                        ..Default::default()
                    },
                    ok_event,
                ),
                cancel: Button::new(
                    "Cancel",
                    Self::CANCEL_RECT,
                    NextWidget {
                        left: Some(WidgetList::OK),
                        right: Some(WidgetList::OK),
                        tab: Some(WidgetList::OK),
                        shift_tab: Some(WidgetList::OK),
                        ..Default::default()
                    },
                    cancel_event,
                ),
            },
            rect: CharRect::new(25, 32, 40 - per_side, 40 + per_side),
        }
    }
}

impl Dialog for ConfirmDialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(2, self.rect);
        draw_buffer.draw_out_border(self.rect, 3, 3, 2);
        draw_buffer.draw_string(self.text, self.text_pos, 0, 2);
        self.widgets.draw_widgets(draw_buffer);
    }

    fn process_input(
        &mut self,
        key_event: &winit::event::KeyEvent,
        modifiers: &winit::event::Modifiers,
        events: &mut crate::app::EventQueue<'_>,
    ) -> DialogResponse {
        if key_event.logical_key == Key::Named(NamedKey::Escape) && modifiers.state().is_empty() {
            return DialogResponse::Close;
        }

        let WidgetResponse { standard, extra } =
            self.widgets.process_input(key_event, modifiers, events);
        if let Some(global_option) = extra {
            if let Some(global) = global_option {
                events.push(global);
            }
            // if there is a response i also want to close myself
            return DialogResponse::Close;
        }

        match standard {
            StandardResponse::SwitchFocus(next) => {
                self.widgets.selected = next;
                DialogResponse::RequestRedraw
            }
            StandardResponse::RequestRedraw => DialogResponse::RequestRedraw,
            StandardResponse::None => DialogResponse::None,
        }
    }
}
