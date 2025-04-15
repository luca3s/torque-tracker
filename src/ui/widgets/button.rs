use std::collections::VecDeque;

use winit::keyboard::{Key, NamedKey};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
};

use super::{NextWidget, StandardResponse, Widget, WidgetResponse};

pub struct Button<R> {
    text: &'static str,
    rect: CharRect,
    pressed: bool,
    next_widget: NextWidget,
    callback: fn() -> R,
}

impl<R> Widget for Button<R> {
    type Response = R;
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool) {
        self.draw_overwrite_pressed(buffer, selected, false);
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        _: &mut VecDeque<GlobalEvent>,
    ) -> WidgetResponse<R> {
        if key_event.logical_key == Key::Named(NamedKey::Space)
            || key_event.logical_key == Key::Named(NamedKey::Enter)
        {
            if !key_event.repeat {
                if key_event.state.is_pressed() {
                    self.pressed = true;
                    return WidgetResponse::default();
                } else {
                    // only call the callback on a release event if the button was pressed in before
                    // meaning if the user pressed the key, then changed focus to another button and then releases
                    // no button should be triggered
                    let response = if self.pressed {
                        Some((self.callback)())
                    } else {
                        None
                    };
                    self.pressed = false;
                    return WidgetResponse {
                        standard: StandardResponse::RequestRedraw,
                        extra: response,
                    };
                }
            }
        // if focus is changed stop being pressed
        } else {
            return self.next_widget.process_key_event(key_event, modifiers);
        }
        WidgetResponse::default()
    }
}

impl<R> Button<R> {
    const TOPLEFT_COLOR: u8 = 3;
    const BOTRIGHT_COLOR: u8 = 1;

    pub fn new(text: &'static str, rect: CharRect, next_widget: NextWidget, cb: fn() -> R) -> Self {
        // is 3 rows high, because bot and top are inclusive
        assert!(
            rect.bot() - rect.top() >= 2,
            "buttons needs to be at least 3 rows high"
        );
        Button {
            text,
            rect,
            callback: cb,
            pressed: false,
            next_widget,
        }
    }

    /// pressed = argument || self.pressed
    pub fn draw_overwrite_pressed(&self, buffer: &mut DrawBuffer, selected: bool, pressed: bool) {
        // let string_offset = {
        //     let half_string = self.text.len() / 2;
        //     if self.text.len() % 2 == 0 {
        //         half_string - 1
        //     } else {
        //         half_string
        //     }
        // };

        // fill behind the text
        buffer.draw_rect(
            DrawBuffer::BACKGROUND_COLOR,
            CharRect::new(
                self.rect.top() + 1,
                self.rect.bot() - 1,
                self.rect.left() + 1,
                self.rect.right() - 1,
            ),
        );

        let box_colors = match pressed || self.pressed {
            true => (Self::BOTRIGHT_COLOR, Self::TOPLEFT_COLOR),
            false => (Self::TOPLEFT_COLOR, Self::BOTRIGHT_COLOR),
        };

        buffer.draw_box(
            self.rect,
            DrawBuffer::BACKGROUND_COLOR,
            box_colors.0,
            box_colors.1,
        );

        let text_color = match selected {
            true => 11,
            false => 0,
        };
        buffer.draw_string(
            self.text,
            CharPosition::new(
                // ((self.rect.right() - self.rect.left()) / 2 + self.rect.left()) - string_offset,
                self.rect.left() + 2,
                (self.rect.bot() - self.rect.top()) / 2 + self.rect.top(),
            ),
            text_color,
            DrawBuffer::BACKGROUND_COLOR,
        )
    }
}
