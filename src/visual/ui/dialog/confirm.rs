use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    ui::{
        pages::create_widget_list,
        widgets::{
            button::Button,
            widget::{NextWidget, Widget, WidgetResponse},
        },
    },
};

use super::{Dialog, DialogResponse};

create_widget_list!(
    {
        ok: Button,
        cancel: Button
    }
);

pub struct ConfirmDialog {
    text: &'static str,
    text_pos: CharPosition,
    // computed from the string length
    rect: CharRect,
    selected: usize,
    widgets: WidgetList,
}

impl ConfirmDialog {
    const OK_RECT: CharRect = CharRect::new(30, 32, 42, 50);
    const CANCEL_RECT: CharRect = CharRect::new(30, 32, 31, 38);
    pub fn new(
        text: &'static str,
        ok_event: impl Fn() -> Option<GlobalEvent> + 'static,
        cancel_event: impl Fn() -> Option<GlobalEvent> + 'static,
    ) -> Self {
        let width = (text.len() + 8).max(22);
        let per_side = width / 2;
        Self {
            text,
            text_pos: CharPosition::new(40 - per_side + 4, 27),
            selected: WidgetList::OK,
            widgets: WidgetList {
                ok: Button::new(
                    "Ok",
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
            rect: CharRect::new(27, 34, 40 - per_side, 40 + per_side),
        }
    }
}

impl Dialog for ConfirmDialog {
    fn draw(&self, draw_buffer: &mut super::DrawBuffer) {
        draw_buffer.show_colors();
        draw_buffer.draw_string(self.text, self.text_pos, 0, 2);
        for widget in 0..WidgetList::WIDGET_COUNT {
            let is_selected = widget == self.selected;
            self.widgets
                .get_widget(widget)
                .draw(draw_buffer, is_selected);
        }
    }

    fn process_input(
        &mut self,
        key_event: &winit::event::KeyEvent,
        modifiers: &winit::event::Modifiers,
        events: &mut std::collections::VecDeque<crate::visual::app::GlobalEvent>,
    ) -> DialogResponse {
        if key_event.logical_key == Key::Named(NamedKey::Escape) && modifiers.state().is_empty() {
            return DialogResponse::Close;
        }

        match self
            .widgets
            .get_widget_mut(self.selected)
            .process_input(modifiers, key_event, events)
        {
            WidgetResponse::SwitchFocus(s) => {
                self.selected = s;
                DialogResponse::RequestRedraw
            }
            WidgetResponse::RequestRedraw => DialogResponse::RequestRedraw,
            WidgetResponse::None => DialogResponse::None,
        }
    }
}
