use crate::visual::{
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{
        button::Button,
        text_in::TextIn,
        widget::{NextWidget, WidgetAny, WidgetResponse},
    },
};

use super::{Page, PageResponse};

pub struct HelpPage {
    selected_widget: usize,
    widgets: [Box<dyn WidgetAny>; Self::WIDGET_COUNT],
}

impl Page for HelpPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        self.widgets
            .iter()
            .enumerate()
            .for_each(|(num, widget)| widget.draw(draw_buffer, num == self.selected_widget));
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(0, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> PageResponse {
        let response = self.widgets[self.selected_widget].process_input(modifiers, key_event);
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
    const TEXT_IN: usize = 1;

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
            widgets: [Box::new(quit_button), Box::new(text_in)],
            selected_widget: 0,
        }
    }

    pub fn get_text_in(&self) -> &str {
        // need to deref here so i dont call Box.as_any(), but instead WidgetAny.as_any(). see: https://lucumr.pocoo.org/2022/1/7/as-any-hack/
        // could also use downcast_ref_unchecked as it is compile time constant
        (*self.widgets[Self::TEXT_IN])
            .as_any()
            .downcast_ref::<TextIn>()
            .unwrap()
            .get_str()
    }
}
