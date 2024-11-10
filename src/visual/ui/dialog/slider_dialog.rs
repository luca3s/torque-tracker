use std::{cell::OnceCell, rc::Rc};

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, NamedKey},
};

use crate::visual::{
    coordinates::{CharPosition, CharRect}, event_loop::GlobalEvent, ui::widgets::{
        text_in::TextIn,
        widget::{NextWidget, Widget, WidgetResponse},
    }
};

use super::{Dialog, DialogResponse};

pub struct SliderDialog {
    text: TextIn,
    return_event: fn(i16) -> GlobalEvent,
}

impl Dialog for SliderDialog {
    fn draw(&self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        println!("draw slider dialog");
        draw_buffer.draw_box(CharRect::new(24, 28, 29, 50), 3, 2, 2);
        draw_buffer.draw_rect(2, CharRect::new(25, 27, 30, 49));
        draw_buffer.draw_string("Enter Value", CharPosition::new(32, 26), 3, 2);
        draw_buffer.draw_box(CharRect::new(25, 27, 44, 49), 2, 1, 3);
        self.text.draw(draw_buffer, true);
    }

    fn process_input(&mut self, key_event: &KeyEvent, modifiers: &Modifiers) -> DialogResponse {
        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::Escape) {
                return DialogResponse::Close;
            } else if key_event.logical_key == Key::Named(NamedKey::Enter) {
                if let Ok(num) = self.text.get_str().parse::<i16>() {
                    return DialogResponse::GlobalEvent((self.return_event)(num), true);
                }
                return DialogResponse::Close;
            }
        }

        match self.text.process_input(modifiers, key_event) {
            // cant switch focus as this is the only widget
            WidgetResponse::SwitchFocus(_) => DialogResponse::None,
            WidgetResponse::RequestRedraw => DialogResponse::RequestRedraw,
            WidgetResponse::None => DialogResponse::None,
            WidgetResponse::GlobalEvent(e) => DialogResponse::GlobalEvent(e, false),
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
