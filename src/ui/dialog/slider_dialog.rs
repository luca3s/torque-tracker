use std::collections::VecDeque;

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, NamedKey},
};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{text_in::TextIn, NextWidget, StandardResponse, Widget},
};

use super::{Dialog, DialogResponse};

pub struct SliderDialog {
    text: TextIn<()>,
    return_event: fn(i16) -> GlobalEvent,
}

impl Dialog for SliderDialog {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_in_box(CharRect::new(24, 28, 29, 50), 3, 2, 2);
        draw_buffer.draw_rect(2, CharRect::new(25, 27, 30, 49));
        draw_buffer.draw_string("Enter Value", CharPosition::new(32, 26), 3, 2);
        draw_buffer.draw_in_box(CharRect::new(25, 27, 44, 49), 2, 1, 3);
        self.text.draw(draw_buffer, true);
    }

    fn process_input(
        &mut self,
        key_event: &KeyEvent,
        modifiers: &Modifiers,
        events: &mut VecDeque<GlobalEvent>,
    ) -> DialogResponse {
        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::Escape) {
                return DialogResponse::Close;
            } else if key_event.logical_key == Key::Named(NamedKey::Enter) {
                if let Ok(num) = self.text.get_str().parse::<i16>() {
                    events.push_back((self.return_event)(num));
                    return DialogResponse::Close;
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
}

impl SliderDialog {
    pub fn new(inital_char: char, return_event: fn(i16) -> GlobalEvent) -> Self {
        let mut text_in = TextIn::new(CharPosition::new(45, 26), 3, NextWidget::default(), |_| {});
        text_in.set_string(inital_char.to_string()).unwrap();
        Self {
            text: text_in,
            return_event,
        }
    }
}
