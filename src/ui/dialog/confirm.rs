use winit::keyboard::{Key, NamedKey};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{NextWidget, StandardResponse, Widget, WidgetResponse, button::Button},
};

use super::{Dialog, DialogResponse};

pub struct ConfirmDialog {
    text: &'static str,
    text_pos: CharPosition,
    // computed from the string length
    rect: CharRect,
    ok: Button<Option<GlobalEvent>>,
    cancel: Button<Option<GlobalEvent>>,
    selected: u8,
}

impl ConfirmDialog {
    const DIALOG_ID: u64 = 600_000_000;

    const OK_RECT: CharRect = CharRect::new(29, 31, 41, 50);
    const CANCEL_RECT: CharRect = CharRect::new(29, 31, 30, 39);
    const OK: u8 = 1;
    const CANCEL: u8 = 2;
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
            selected: Self::OK,
            ok: Button::new(
                "  Ok",
                Self::OK_RECT,
                NextWidget {
                    left: Some(Self::CANCEL),
                    right: Some(Self::CANCEL),
                    tab: Some(Self::CANCEL),
                    shift_tab: Some(Self::CANCEL),
                    ..Default::default()
                },
                ok_event,
                #[cfg(feature = "accesskit")]
                accesskit::NodeId(Self::DIALOG_ID + u64::from(Self::OK) * 20),
            ),
            cancel: Button::new(
                "Cancel",
                Self::CANCEL_RECT,
                NextWidget {
                    left: Some(Self::OK),
                    right: Some(Self::OK),
                    tab: Some(Self::OK),
                    shift_tab: Some(Self::OK),
                    ..Default::default()
                },
                cancel_event,
                #[cfg(feature = "accesskit")]
                accesskit::NodeId(Self::DIALOG_ID + u64::from(Self::CANCEL) * 20),
            ),
            rect: CharRect::new(25, 32, 40 - per_side, 40 + per_side),
        }
    }
}

impl Dialog for ConfirmDialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(2, self.rect);
        draw_buffer.draw_out_border(self.rect, 3, 3, 2);
        draw_buffer.draw_string(self.text, self.text_pos, 0, 2);
        self.ok.draw(draw_buffer, self.selected == Self::OK);
        self.cancel.draw(draw_buffer, self.selected == Self::CANCEL);
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

        let WidgetResponse { standard, extra } = match self.selected {
            Self::OK => self.ok.process_input(modifiers, key_event, events),
            Self::CANCEL => self.cancel.process_input(modifiers, key_event, events),
            _ => unreachable!(),
        };
        if let Some(global_option) = extra {
            if let Some(global) = global_option {
                events.push(global);
            }
            // if there is a response i also want to close myself
            return DialogResponse::Close;
        }

        match standard {
            StandardResponse::SwitchFocus(next) => {
                self.selected = next;
                DialogResponse::RequestRedraw
            }
            StandardResponse::RequestRedraw => DialogResponse::RequestRedraw,
            StandardResponse::None => DialogResponse::None,
        }
    }

    #[cfg(feature = "accesskit")]
    fn build_tree(
        &self,
        tree: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> crate::app::AccessResponse {
        use accesskit::{Node, Role};

        use crate::app::AccessResponse;

        let mut root_node = Node::new(Role::Dialog);

        AccessResponse {
            root: todo!(),
            selected: todo!(),
        }
    }
}
