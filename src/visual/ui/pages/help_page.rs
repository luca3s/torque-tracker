use std::collections::VecDeque;

use crate::visual::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{
        button::Button,
        text_in::TextIn,
        widget::{NextWidget, WidgetResponse},
    },
};

use super::{Page, PageResponse, Widget};

super::create_widget_list!(
   {
        text_in: TextIn,
        quit_button: Button
    }
);

pub struct HelpPage {
    widgets: WidgetList,
    selected_widget: usize,
}

impl Page for HelpPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        let selected = self.selected_widget;
        for idx in WidgetList::INDEX_RANGE {
            self.widgets
                .get_widget(idx)
                .draw(draw_buffer, selected == idx);
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(0, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        event: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        let selected = self.selected_widget;
        let response = self
            .widgets
            .get_widget_mut(selected)
            .process_input(modifiers, key_event, event);
        match response {
            WidgetResponse::SwitchFocus(next) => {
                self.selected_widget = next;
                PageResponse::RequestRedraw
            }
            WidgetResponse::RequestRedraw => PageResponse::RequestRedraw,
            WidgetResponse::None => PageResponse::None,
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
            widgets: WidgetList {
                text_in,
                quit_button,
            },
            selected_widget: 0,
        }
    }
}
