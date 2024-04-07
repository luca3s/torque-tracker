use crate::visual::{
    coordinates::CharRect,
    ui::widgets::{
        button::Button,
        slider::Slider,
        widget::{NextWidget, WidgetAny},
    },
};

use super::page::Page;

pub struct SongDirectoryConfigPage {
    widgets: [Box<dyn WidgetAny>; Self::WIDGET_COUNT],
    selected_widget: usize,
}

impl Page for SongDirectoryConfigPage {
    fn draw(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        self.widgets
            .iter()
            .enumerate()
            .for_each(|(num, widget)| widget.draw(draw_buffer, num == self.selected_widget));
    }

    fn draw_constant(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        const BACKGROUND_COLOR: usize = 2;
        const TOPLEFT_COLOR: usize = 3;
        const BOTRIGHT_COLOR: usize = 1;

        // fill complete page
        draw_buffer.draw_rect(BACKGROUND_COLOR, CharRect::PAGE_AREA);
        // draw_buffer.draw_box(CharRect::new(top, bot, left, right), background_color, top_left_color, bot_right_color)

        draw_buffer.draw_string("Song Variables", (33, 13), 3, 2);

        draw_buffer.draw_string("Song Name", (7, 16), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(15, 17, 16, 43),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Initial Tempo", (3, 19), 0, 2);
        draw_buffer.draw_string("Initial Speed", (3, 20), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(18, 21, 16, 50),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Global Volume", (3, 23), 0, 2);
        draw_buffer.draw_string("Mixing Volume", (3, 24), 0, 2);
        draw_buffer.draw_string("Seperation", (6, 25), 0, 2);
        draw_buffer.draw_string("Old Effects", (5, 26), 0, 2);
        draw_buffer.draw_string("Compatible Gxx", (2, 27), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(22, 28, 16, 34),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Control", (9, 30), 0, 2);

        draw_buffer.draw_string("Playback", (8, 33), 0, 2);

        draw_buffer.draw_string("Pitch Slides", (4, 36), 0, 2);

        draw_buffer.draw_string("Directories", (34, 40), 3, 2);

        draw_buffer.draw_string("Module", (6, 42), 0, 2);
        draw_buffer.draw_string("Sample", (6, 43), 0, 2);
        draw_buffer.draw_string("Instrument", (2, 44), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(41, 45, 12, 78),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        // draw_buffer.draw_rect(0, CharRect::new(20, 26, 27, 33));
    }

    fn update(&mut self) {
        todo!()
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) {
        let next_widget = self.widgets[self.selected_widget].process_input(modifiers, key_event);
        if let Some(next) = next_widget {
            // can panic here, because all involved values should be compile time
            assert!(next < Self::WIDGET_COUNT);
            self.selected_widget = next;
        }
    }
}

impl SongDirectoryConfigPage {
    const WIDGET_COUNT: usize = 2;
    pub fn new() -> Self {
        let save_buttons = Button::new(
            "Save all Preferences",
            CharRect::new(46, 48, 28, 51),
            NextWidget {
                up: Some(1),
                ..Default::default()
            },
            || println!("save preferences"),
        );
        let global_volume: Slider<0, 128> =
            Slider::new(2, (17, 23), 16,
            NextWidget {
                down: Some(0),
                ..Default::default()
            }, |value| {
                println!("gloabl volume set to: {}", value)
            });

        Self {
            widgets: [Box::new(save_buttons), Box::new(global_volume)],
            selected_widget: 0,
        }
    }
}
