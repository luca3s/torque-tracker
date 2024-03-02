use crate::rendering::{ui::widgets::{button::Button, widget::{Widget, NextWidget}}, CharRect};

use super::pages::Page;

enum HelpPageWidget {
    Button(Button),
}

impl Widget for HelpPageWidget {
    fn draw(&self, buffer: &mut crate::rendering::DrawBuffer, selected: bool) {
        match self {
            HelpPageWidget::Button(b) => b.draw(buffer, selected),
        }
    }

    fn process_input(&mut self, key_event: &winit::event::KeyEvent) -> Option<usize> {
        match self {
            HelpPageWidget::Button(b) => b.process_input(key_event),
        }
    }
}

pub struct HelpPage {

    // quit_button: Button,
    active_widget: usize,
    ui_widgets: Box<[HelpPageWidget]>,
}

impl Page for HelpPage {
    fn draw(&self, draw_buffer: &mut crate::rendering::DrawBuffer) {
        self.ui_widgets.iter().for_each(|widget| widget.draw(draw_buffer, false));

    }

    fn draw_constant(&self, render_state: &mut crate::rendering::DrawBuffer) {
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
        self.ui_widgets[self.active_widget].process_input(key_event);
    }
}

impl HelpPage {
    const QUIT_BUTTON: usize = 0;

    pub fn new() -> Self {
        let quit_button = Button::new("quit", CharRect::new(30, 34, 50, 20), NextWidget::default(), || println!("quit"));
        Self { ui_widgets: Box::new([HelpPageWidget::Button(quit_button)]), active_widget: 0 }
    }
}
