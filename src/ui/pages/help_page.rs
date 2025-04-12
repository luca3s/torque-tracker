use std::collections::VecDeque;

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{button::Button, text_in::TextIn, NextWidget, StandardResponse, WidgetResponse},
};

use super::{Page, PageResponse};

super::create_widget_list!(
    response: ();
    HelpWidgets
   {
        text_in: TextIn<()>,
        quit_button: Button<()>
    }
);

pub struct HelpPage {
    widgets: HelpWidgets,
}

impl Page for HelpPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        self.widgets.draw_widgets(draw_buffer);
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(0, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        let response = self.widgets.process_input(key_event, modifiers, events);
        match response.standard {
            StandardResponse::SwitchFocus(next) => {
                self.widgets.selected = next;
                PageResponse::RequestRedraw
            }
            StandardResponse::RequestRedraw => PageResponse::RequestRedraw,
            StandardResponse::None => PageResponse::None,
        }
    }
}

impl HelpPage {
    pub fn new() -> Self {
        let quit_button = Button::new(
            "quit",
            CharRect::new(30, 32, 2, 10),
            NextWidget::default(),
            || println!("quit"),
        );
        let mut text_in = TextIn::new(
            CharPosition::new(3, 26),
            12,
            NextWidget::default(),
            |new_text| println!("text changed to {new_text}"),
        );
        text_in.set_string("test".to_owned()).unwrap();

        Self {
            widgets: HelpWidgets {
                text_in,
                quit_button,
                selected: HelpWidgets::TEXT_IN,
            },
        }
    }
}
