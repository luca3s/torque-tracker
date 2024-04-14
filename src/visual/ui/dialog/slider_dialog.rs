use winit::{event::{KeyEvent, Modifiers}, keyboard::{NamedKey, Key}};

use crate::visual::{
    coordinates::{CharRect, CharPosition},
    ui::widgets::{text_in::TextIn, widget::{NextWidget, Widget}},
};

use super::dialog::{Dialog, DialogResponse};

pub struct SliderDialog {
    text: TextIn,
    return_value: Option<i16>
}

impl Dialog for SliderDialog {
    fn draw(&self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        draw_buffer.draw_box(CharRect::new(24, 28, 28, 51), 1, 2, 2);
        draw_buffer.draw_string("Enter Value", CharPosition::new(32, 26), 1, 2);
        draw_buffer.draw_box(CharRect::new(25, 27, 44, 49), 2, 3, 1);
        self.text.draw(draw_buffer, true);
    }

    fn process_input(&mut self, key_event: &KeyEvent, modifiers: &Modifiers) -> DialogResponse {
        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::Escape) {
                return DialogResponse::Close;
            } else if key_event.logical_key == Key::Named(NamedKey::Enter) {
                if let Ok(num) = self.text.get_string().parse::<i16>() {
                    if self.return_value.is_some() {
                        unreachable!();
                    }
                    self.return_value = Some(num)
                }
                return DialogResponse::Close;
            }
        }

        self.text.process_input(modifiers, key_event);
        DialogResponse::None
    }
}

impl SliderDialog {
    // pub fn new(inital_char: char) -> Self {
    //     let mut text_in = TextIn::new(CharPosition::new(45, 26), 3, NextWidget::default(), |_| {});
    //     // input should be valid here as it got checked to open the Dialog, but it isnt that bad when the input doesnt appear, so i ignore error
    //     let _ = text_in.set_string(&inital_char.to_string());

    //     Self { text: text_in, return_value: None }
    // }

    pub fn new() -> Self {
        let text_in = TextIn::new(CharPosition::new(45, 26), 3, NextWidget::default(), |_| {});
        Self { text: text_in, return_value: None }
    }

    pub fn start_dialog(&mut self, inital_char: char) {
        self.text.set_string(&inital_char.to_string());
        self.return_value = None;
    }

    pub fn get_return_value(&self) -> Option<i16> {
        self.return_value
    }

    fn parse_text(&self) -> Option<i16> {
        match self.text.get_string().parse() {
            Ok(num) => Some(num),
            Err(_) => None,
        }
    }
}
