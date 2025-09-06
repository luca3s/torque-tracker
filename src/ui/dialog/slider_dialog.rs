use std::ops::RangeInclusive;

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, NamedKey},
};

use crate::{
    app::{EventQueue, GlobalEvent},
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{NextWidget, StandardResponse, Widget, text_in::TextIn},
};

use super::{Dialog, DialogResponse};

pub struct SliderDialog {
    text: TextIn<()>,
    range: RangeInclusive<i16>,
    return_event: fn(i16) -> GlobalEvent,
}

impl Dialog for SliderDialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_in_box(CharRect::new(24, 28, 29, 50), 3, 2, 2, 1);
        draw_buffer.draw_rect(2, CharRect::new(25, 27, 30, 49));
        draw_buffer.draw_string("Enter Value", CharPosition::new(32, 26), 3, 2);
        draw_buffer.draw_in_box(CharRect::new(25, 27, 44, 49), 2, 1, 3, 1);
        self.text.draw(draw_buffer, true);
    }

    fn process_input(
        &mut self,
        key_event: &KeyEvent,
        modifiers: &Modifiers,
        events: &mut EventQueue<'_>,
    ) -> DialogResponse {
        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::Escape) {
                return DialogResponse::Close;
            } else if key_event.logical_key == Key::Named(NamedKey::Enter) {
                if let Ok(num) = self.text.get_str().parse::<i16>()
                    && self.range.contains(&num)
                {
                    events.push((self.return_event)(num));
                }
                return DialogResponse::Close;
            }
        }

        match self
            .text
            .process_input(modifiers, key_event, events)
            .standard
        {
            // cant switch focus as this is the only widget
            StandardResponse::SwitchFocus(_) => DialogResponse::None,
            StandardResponse::RequestRedraw => DialogResponse::RequestRedraw,
            StandardResponse::None => DialogResponse::None,
        }
    }

    #[cfg(feature = "accesskit")]
    fn build_tree(
        &self,
        tree: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> crate::app::AccessResponse {
        use accesskit::{Node, NodeId, Role};

        use crate::app::AccessResponse;

        const ROOT_ID: NodeId = NodeId(400_000_000);
        const TEXT_ID: NodeId = NodeId(400_000_001);

        let mut root_node = Node::new(Role::Dialog);
        root_node.set_label("Slider Dialog");
        root_node.push_child(TEXT_ID);

        let mut text_node = Node::new(Role::NumberInput);
        text_node.set_min_numeric_value(*self.range.start() as f64);
        text_node.set_max_numeric_value(*self.range.end() as f64);
        text_node.set_value(self.text.get_str());
        text_node.set_label("Set Value");

        tree.push((ROOT_ID, root_node));
        tree.push((TEXT_ID, text_node));
        AccessResponse {
            root: ROOT_ID,
            selected: TEXT_ID,
        }
    }
}

impl SliderDialog {
    const NODE_ID: u64 = 400_000_000;
    pub fn new(
        inital_char: char,
        range: RangeInclusive<i16>,
        return_event: fn(i16) -> GlobalEvent,
    ) -> Self {
        let mut text_in = TextIn::new(
            CharPosition::new(45, 26),
            3,
            NextWidget::default(),
            |_| {},
            #[cfg(feature = "accesskit")]
            (accesskit::NodeId(Self::NODE_ID + 20), "value"),
        );
        text_in.set_string(inital_char.to_string()).unwrap();
        Self {
            text: text_in,
            return_event,
            range,
        }
    }
}
