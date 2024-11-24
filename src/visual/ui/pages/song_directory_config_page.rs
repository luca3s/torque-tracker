use std::{cell::Cell, rc::Rc};

use crate::visual::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    ui::widgets::{
        button::Button,
        slider::Slider,
        text_in::TextIn,
        text_in_scroll::TextInScroll,
        toggle::Toggle,
        toggle_button::ToggleButton,
        widget::{NextWidget, Widget, WidgetResponse},
    },
};

use super::{Page, PageResponse};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Control {
    Instruments,
    Samples,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Playback {
    Stereo,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PitchSlides {
    Linear,
    Amiga,
}

pub enum SDCChange {
    SetSongName(String),
    InitialTempo(i16),
    InitialSpeed(i16),
    GlobalVolume(i16),
    MixingVolume(i16),
    Seperation(i16),
}

pub struct SongDirectoryConfigPage {
    // widgets:
    song_name: TextIn,
    initial_tempo: Slider<31, 255>,
    initial_speed: Slider<1, 255>,
    global_volume: Slider<0, 128>,
    mixing_volume: Slider<0, 128>,
    seperation: Slider<0, 128>,

    old_effects: Toggle<bool>,
    compatible_gxx: Toggle<bool>,
    
    instruments_button: ToggleButton<Control>,
    samples_button: ToggleButton<Control>,
    
    stereo_button: ToggleButton<Playback>,
    mono_button: ToggleButton<Playback>,

    linear_slides_button: ToggleButton<PitchSlides>,
    amiga_slides_button: ToggleButton<PitchSlides>,
    
    module_path: TextInScroll,
    sample_path: TextInScroll,
    instrument_path: TextInScroll,
    save_button: Button,
    
    selected_widget: usize,
}

impl Page for SongDirectoryConfigPage {
    fn draw(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        let selected = self.selected_widget;
        self.widgets().iter()
            .enumerate()
            .for_each(|(num, widget)| widget.draw(draw_buffer, num == selected));
    }

    fn draw_constant(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        const BACKGROUND_COLOR: usize = 2;
        const TOPLEFT_COLOR: usize = 1;
        const BOTRIGHT_COLOR: usize = 3;

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
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
    ) -> PageResponse {
        let selected = self.selected_widget;
        match self.widgets()[selected].process_input(modifiers, key_event) {
            WidgetResponse::SwitchFocus(next) => {
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

impl SongDirectoryConfigPage {
    pub fn ui_change(&mut self, change: SDCChange) -> PageResponse {
        match change {
            SDCChange::SetSongName(s) => match self.song_name.set_string(s) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialTempo(n) => match self.initial_tempo.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialSpeed(n) => match self.initial_speed.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::GlobalVolume(n) => match self.global_volume.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::MixingVolume(n) => match self.mixing_volume.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::Seperation(n) => match self.seperation.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
        }
    }

    const WIDGET_COUNT: usize = 18;
    const SONG_NAME: usize = 0;
    const INITIAL_TEMPO: usize = 1;
    const INITIAL_SPEED: usize = 2;
    const GLOBAL_VOLUME: usize = 3;
    const MIXING_VOLUME: usize = 4;
    const SEPERATION: usize = 5;
    const OLD_EFFECTS: usize = 6;
    const COMPATIBLE_GXX: usize = 7;
    const MODULE_PATH: usize = 14;
    const SAMPLE_PATH: usize = 15;
    const INSTRUMENT_PATH: usize = 16;

    fn widgets(&mut self) -> [&mut dyn Widget; Self::WIDGET_COUNT] {
        [
            &mut self.song_name,
            &mut self.initial_tempo,
            &mut self.initial_speed,
            &mut self.global_volume,
            &mut self.mixing_volume,
            &mut self.seperation,
            &mut self.old_effects,
            &mut self.compatible_gxx,
            &mut self.instruments_button,
            &mut self.samples_button,
            &mut self.stereo_button,
            &mut self.mono_button,
            &mut self.linear_slides_button,
            &mut self.amiga_slides_button,
            &mut self.module_path,
            &mut self.sample_path,
            &mut self.instrument_path,
            &mut self.save_button
        ]
    }

    pub fn new() -> Self {
        // widget 0
        let song_name = TextIn::new(
            CharPosition::new(17, 16),
            25,
            NextWidget {
                down: Some(Self::INITIAL_TEMPO),
                tab: Some(Self::INITIAL_TEMPO),
                ..Default::default()
            },
            |s| println!("new song name: {}", s),
        );

        // widget 1
        let initial_tempo: Slider<31, 255> = Slider::new(
            125,
            CharPosition::new(17, 19),
            32,
            NextWidget {
                up: Some(Self::SONG_NAME),
                shift_tab: Some(Self::SONG_NAME),
                down: Some(Self::INITIAL_SPEED),
                tab: Some(Self::INITIAL_SPEED),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::InitialTempo(n))),
            |value| println!("initial tempo set to: {}", value),
        );
        // widget 2
        let initial_speed: Slider<1, 255> = Slider::new(
            6,
            CharPosition::new(17, 20),
            32,
            NextWidget {
                up: Some(Self::INITIAL_TEMPO),
                shift_tab: Some(Self::INITIAL_TEMPO),
                down: Some(Self::GLOBAL_VOLUME),
                tab: Some(Self::GLOBAL_VOLUME),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::InitialSpeed(n))),
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
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::GlobalVolume(n))),
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
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::MixingVolume(n))),
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
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::Seperation(n))),
            |value| println!("seperation set to: {}", value),
        );

        // widget 6
        let old_effects: Toggle<bool> = Toggle::new(
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

        let control_rc = Rc::new(Cell::new(Control::Samples));
        // widget 8
        let instruments_button: ToggleButton<Control> = ToggleButton::new(
            "Instruments",
            CharRect::new(29, 31, 16, 30),
            NextWidget {
                left: Some(9),
                right: Some(9),
                up: Some(7),
                down: Some(10),
                tab: Some(9),
                shift_tab: Some(7),
            },
            Control::Instruments,
            control_rc.clone(),
            |_| println!("Instruments activated"),
        );

        // widget 9
        let samples_button: ToggleButton<Control> = ToggleButton::new(
            "Samples",
            CharRect::new(29, 31, 31, 45),
            NextWidget {
                left: Some(8),
                right: Some(8),
                up: Some(7),
                down: Some(11),
                tab: Some(8),
                shift_tab: Some(8),
            },
            Control::Samples,
            control_rc,
            |_| println!("Samples activated"),
        );

        let stereo_mono_rs = Rc::new(Cell::new(Playback::Stereo));
        // widget 10
        let stereo_button: ToggleButton<Playback> = ToggleButton::new(
            "Stereo",
            CharRect::new(32, 34, 16, 30),
            NextWidget {
                left: Some(11),
                right: Some(11),
                up: Some(8),
                down: Some(12),
                tab: Some(11),
                shift_tab: Some(11),
            },
            Playback::Stereo,
            stereo_mono_rs.clone(),
            |_| println!("stereo activated"),
        );

        // widget 11
        let mono_button: ToggleButton<Playback> = ToggleButton::new(
            "Mono",
            CharRect::new(32, 34, 31, 45),
            NextWidget {
                left: Some(10),
                right: Some(10),
                up: Some(9),
                down: Some(13),
                tab: Some(10),
                shift_tab: Some(10),
            },
            Playback::Mono,
            stereo_mono_rs,
            |_| println!("stereo activated"),
        );

        let pitch_slides_rc = Rc::new(Cell::new(PitchSlides::Linear));
        // widget 12
        let linear_slides_button: ToggleButton<PitchSlides> = ToggleButton::new(
            "Linear",
            CharRect::new(35, 37, 16, 30),
            NextWidget {
                left: Some(13),
                right: Some(13),
                up: Some(10),
                down: Some(14),
                tab: Some(13),
                shift_tab: Some(13),
            },
            PitchSlides::Linear,
            pitch_slides_rc.clone(),
            |_| println!("pitch slides set to linear"),
        );
        // widget 13
        let amiga_slides_button: ToggleButton<PitchSlides> = ToggleButton::new(
            "Amiga",
            CharRect::new(35, 37, 31, 45),
            NextWidget {
                left: Some(12),
                right: Some(12),
                up: Some(11),
                down: Some(14),
                tab: Some(12),
                shift_tab: Some(12),
            },
            PitchSlides::Amiga,
            pitch_slides_rc,
            |_| println!("set to amiga pitch slide"),
        );
        // widget 14
        let module_path = TextInScroll::new(
            CharPosition::new(13, 42),
            64,
            NextWidget {
                left: None,
                right: None,
                up: Some(12),
                down: Some(15),
                tab: Some(15),
                shift_tab: Some(12),
            },
            |text| println!("Module path set to {text}"),
        );
        // widget 15
        let sample_path = TextInScroll::new(
            CharPosition::new(13, 43),
            64,
            NextWidget {
                left: None,
                right: None,
                up: Some(14),
                down: Some(16),
                tab: Some(16),
                shift_tab: Some(14),
            },
            |text| println!("Sample path set to {text}"),
        );
        // widget 16
        let instrument_path = TextInScroll::new(
            CharPosition::new(13, 44),
            64,
            NextWidget {
                left: None,
                right: None,
                up: Some(15),
                down: Some(17),
                tab: Some(17),
                shift_tab: Some(15),
            },
            |text| println!("Instrument path set to {text}"),
        );

        // widget 17
        let save_button = Button::new(
            "Save all Preferences",
            CharRect::new(46, 48, 28, 51),
            NextWidget {
                up: Some(16),
                shift_tab: Some(16),
                ..Default::default()
            },
            || println!("save preferences"),
        );
        Self {
            selected_widget: 0,
            song_name,
            initial_tempo,
            initial_speed,
            global_volume,
            mixing_volume,
            seperation,
            old_effects,
            compatible_gxx,
            instruments_button,
            samples_button,
            stereo_button,
            mono_button,
            linear_slides_button,
            amiga_slides_button,
            module_path,
            sample_path,
            instrument_path,
            save_button,
        }
    }
}
