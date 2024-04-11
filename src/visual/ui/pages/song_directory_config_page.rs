use crate::visual::{
    coordinates::{CharPosition, CharRect},
    ui::widgets::{
        button::Button,
        slider::Slider,
        text_in::TextIn,
        toggle::Toggle,
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

        draw_buffer.draw_string("Song Variables", CharPosition::new(33, 13), 3, 2);

        draw_buffer.draw_string("Song Name", CharPosition::new(7, 16), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(15, 17, 16, 43),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Initial Tempo", CharPosition::new(3, 19), 0, 2);
        draw_buffer.draw_string("Initial Speed", CharPosition::new(3, 20), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(18, 21, 16, 50),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Global Volume", CharPosition::new(3, 23), 0, 2);
        draw_buffer.draw_string("Mixing Volume", CharPosition::new(3, 24), 0, 2);
        draw_buffer.draw_string("Seperation", CharPosition::new(6, 25), 0, 2);
        draw_buffer.draw_string("Old Effects", CharPosition::new(5, 26), 0, 2);
        draw_buffer.draw_string("Compatible Gxx", CharPosition::new(2, 27), 0, 2);
        draw_buffer.draw_box(
            CharRect::new(22, 28, 16, 34),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
        );

        draw_buffer.draw_string("Control", CharPosition::new(9, 30), 0, 2);

        draw_buffer.draw_string("Playback", CharPosition::new(8, 33), 0, 2);

        draw_buffer.draw_string("Pitch Slides", CharPosition::new(4, 36), 0, 2);

        draw_buffer.draw_string("Directories", CharPosition::new(34, 40), 3, 2);

        draw_buffer.draw_string("Module", CharPosition::new(6, 42), 0, 2);
        draw_buffer.draw_string("Sample", CharPosition::new(6, 43), 0, 2);
        draw_buffer.draw_string("Instrument", CharPosition::new(2, 44), 0, 2);
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
    const WIDGET_COUNT: usize = 9;
    const SONG_NAME: usize = 0;
    const OLD_EFFECTS: usize = 6;
    const COMPATIBLE_GXX: usize = 7;

    pub fn new() -> Self {
        // widget 0
        let song_name = TextIn::new(
            CharPosition::new(17, 16),
            25,
            NextWidget {
                down: Some(1),
                ..Default::default()
            },
            |s| println!("new song name: {}", s),
        );

        // widget 1
        let inital_tempo: Slider<31, 255> = Slider::new(
            125,
            CharPosition::new(17, 19),
            32,
            NextWidget {
                up: Some(0),
                shift_tab: Some(0),
                down: Some(2),
                tab: Some(2),
                ..Default::default()
            },
            |value| println!("initial tempo set to: {}", value),
        );
        // widget 2
        let initial_speed: Slider<1, 255> = Slider::new(
            6,
            CharPosition::new(17, 20),
            32,
            NextWidget {
                up: Some(1),
                shift_tab: Some(1),
                down: Some(3),
                tab: Some(3),
                ..Default::default()
            },
            |value| println!("initial speed set to: {}", value),
        );
        // widget 3
        let global_volume: Slider<0, 128> = Slider::new(
            128,
            CharPosition::new(17, 23),
            16,
            NextWidget {
                up: Some(2),
                shift_tab: Some(2),
                down: Some(4),
                tab: Some(4),
                ..Default::default()
            },
            |value| println!("gloabl volume set to: {}", value),
        );
        // widget 4
        let mixing_volume: Slider<0, 128> = Slider::new(
            48,
            CharPosition::new(17, 24),
            16,
            NextWidget {
                up: Some(3),
                shift_tab: Some(3),
                down: Some(5),
                tab: Some(5),
                ..Default::default()
            },
            |value| println!("mixing volume set to: {}", value),
        );
        // widget 5
        let seperation: Slider<0, 128> = Slider::new(
            48,
            CharPosition::new(17, 25),
            16,
            NextWidget {
                up: Some(4),
                shift_tab: Some(4),
                down: Some(6),
                tab: Some(6),
                ..Default::default()
            },
            |value| println!("seperation set to: {}", value),
        );

        // widget 6
        let old_effect: Toggle<bool> = Toggle::new(
            CharPosition::new(17, 26),
            16,
            NextWidget {
                left: Some(5),
                right: Some(7),
                up: Some(5),
                down: Some(7),
                tab: Some(7),
                shift_tab: Some(5),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Old Effects: {}", onoff),
        );

        // widget 7
        let compatible_gxx: Toggle<bool> = Toggle::new(
            CharPosition::new(17, 27),
            16,
            NextWidget {
                left: Some(6),
                right: Some(8),
                up: Some(6),
                down: Some(8),
                tab: Some(8),
                shift_tab: Some(6),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Compatible Gxx: {}", onoff),
        );

        // widget 8
        let save_button = Button::new(
            "Save all Preferences",
            CharRect::new(46, 48, 28, 51),
            NextWidget {
                up: Some(7),
                shift_tab: Some(7),
                ..Default::default()
            },
            || println!("save preferences"),
        );
        Self {
            widgets: [
                Box::new(song_name),
                Box::new(inital_tempo),
                Box::new(initial_speed),
                Box::new(global_volume),
                Box::new(mixing_volume),
                Box::new(seperation),
                Box::new(old_effect),
                Box::new(compatible_gxx),
                Box::new(save_button),
            ],
            selected_widget: 0,
        }
    }

    pub fn get_song_name(&self) -> &str {
        (*self.widgets[Self::SONG_NAME])
            .as_any()
            .downcast_ref::<TextIn>()
            .unwrap()
            .get_string()
    }

    pub fn get_old_effects_state(&self) -> bool {
        (*self.widgets[Self::OLD_EFFECTS])
            .as_any()
            .downcast_ref::<Toggle<bool>>()
            .unwrap()
            .get_variant()
    }

    pub fn get_compatible_gxx_state(&self) -> bool {
        (*self.widgets[Self::COMPATIBLE_GXX])
            .as_any()
            .downcast_ref::<Toggle<bool>>()
            .unwrap()
            .get_variant()
    }
}
