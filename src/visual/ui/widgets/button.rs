use winit::keyboard::{Key, NamedKey};

use crate::visual::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
};

use super::widget::{NextWidget, Widget, WidgetResponse};

pub struct Button {
    text: &'static str,
    rect: CharRect,
    pressed: bool,
    next_widget: NextWidget,
    callback: Box<dyn Fn()>,
}

impl Widget for Button {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool) {
        self.draw_overwrite_pressed(buffer, selected, false);
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> WidgetResponse {
        if key_event.logical_key == Key::Named(NamedKey::Space)
            || key_event.logical_key == Key::Named(NamedKey::Enter)
        {
            if !key_event.repeat {
                if key_event.state.is_pressed() {
                    self.pressed = true;
                    return WidgetResponse::RequestRedraw;
                } else {
                    // only call the callback on a release event if the button was pressed in before
                    // meaning if the user pressed the key, then changed focus to another button and then releases
                    // no button should be triggered
                    if self.pressed {
                        (self.callback)();
                    }
                    self.pressed = false;
                    return WidgetResponse::RequestRedraw;
                }
            }
        // if focus is changed stop being pressed
        } else {
            match self.next_widget.process_key_event(key_event, modifiers) {
                Some(next) => {
                    self.pressed = false;
                    return WidgetResponse::SwitchFocus(next);
                }
                None => return WidgetResponse::None,
            }
        }
        WidgetResponse::None
    }
}

impl Button {
    const BACKGROUND_COLOR: usize = 2;
    const TOPLEFT_COLOR: usize = 3;
    const BOTRIGHT_COLOR: usize = 1;

    pub fn new(
        text: &'static str,
        rect: CharRect,
        next_widget: NextWidget,
        cb: impl Fn() + 'static,
    ) -> Self {
        // is 3 rows high, because bot and top are inclusive
        assert!(
            rect.bot() - rect.top() >= 2,
            "buttons needs to be at least 3 rows high"
        );
        Button {
            text,
            rect,
            callback: Box::new(cb),
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
            Self::BACKGROUND_COLOR,
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
            Self::BACKGROUND_COLOR,
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
            Self::BACKGROUND_COLOR,
        )
    }
}
