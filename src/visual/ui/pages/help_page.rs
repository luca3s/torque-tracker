use crate::visual::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{
        button::Button,
        text_in::TextIn,
        widget::{NextWidget, WidgetResponse},
    },
};

use super::{Page, PageResponse, Widget};

pub struct HelpPage {
    selected_widget: usize,
    quit_button: Button,
    text_in: TextIn,
}

impl Page for HelpPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        let selected = self.selected_widget;
        self.widgets().iter()
            .enumerate()
            .for_each(|(num, widget)| widget.draw(draw_buffer, num == selected));
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(0, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> PageResponse {
        let selected = self.selected_widget;
        let response = self.widgets()[selected].process_input(modifiers, key_event);
        match response {
            WidgetResponse::SwitchFocus(next) => {
                // can panic here, because all involved values should be compile time
                assert!(next < Self::WIDGET_COUNT);
                self.selected_widget = next;
                PageResponse::RequestRedraw
            }
            WidgetResponse::RequestRedraw => PageResponse::RequestRedraw,
            WidgetResponse::None => PageResponse::None,
            WidgetResponse::GlobalEvent(e) => PageResponse::GlobalEvent(e),
        }
    }
}

impl HelpPage {
    const WIDGET_COUNT: usize = 2;

    fn widgets(&mut self) -> [&mut dyn Widget; Self::WIDGET_COUNT] {
        [&mut self.quit_button, &mut self.text_in]
    }

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
            quit_button,
            text_in,
            selected_widget: 0,
        }
    }
}
