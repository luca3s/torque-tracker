use crate::visual::{
    coordinates::CharRect,
    draw_buffer::DrawBuffer,
    ui::widgets::{
        button::Button,
        widget::{NextWidget, Widget},
    },
};

use super::page::Page;

enum HelpPageWidget {
    Button(Button),
}

impl Widget for HelpPageWidget {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool) {
        match self {
            HelpPageWidget::Button(b) => b.draw(buffer, selected),
        }
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> Option<usize> {
        match self {
            HelpPageWidget::Button(b) => b.process_input(modifiers, key_event),
        }
    }
}

pub struct HelpPage {
    active_widget: usize,
    ui_widgets: Box<[HelpPageWidget]>,
}

impl Page for HelpPage {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        self.ui_widgets
            .iter()
            .enumerate()
            .for_each(|(num, widget)| widget.draw(draw_buffer, num == self.active_widget));
    }

    fn draw_constant(&self, draw_buffer: &mut DrawBuffer) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) {
        let next_widget = self.ui_widgets[self.active_widget].process_input(modifiers, key_event);
        if let Some(next) = next_widget {
            self.active_widget = next;
        }
    }
}

impl HelpPage {
    pub fn new() -> Self {
        let quit_button = Button::new(
            "quit1",
            CharRect::new(30, 32, 2, 10),
            NextWidget::default(),
            || println!("quit"),
        );
        Self {
            ui_widgets: Box::new([HelpPageWidget::Button(quit_button)]),
            active_widget: 0,
        }
    }
}
