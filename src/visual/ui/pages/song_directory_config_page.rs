use std::{cell::Cell, collections::VecDeque, rc::Rc};

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

super::create_widget_list!(
    SongDirectoryConfigPage;
    {
        song_name: TextIn,
        initial_tempo: Slider<31, 255>,
        initial_speed: Slider<1, 255>,
        global_volume: Slider<0, 128>,
        mixing_volume: Slider<0, 128>,
        seperation: Slider<0, 128>,

        old_effects: Toggle<bool>,
        compatible_gxx: Toggle<bool>,

        instruments: ToggleButton<Control>,
        samples: ToggleButton<Control>,

        stereo: ToggleButton<Playback>,
        mono: ToggleButton<Playback>,

        linear_slides: ToggleButton<PitchSlides>,
        amiga_slides: ToggleButton<PitchSlides>,

        module_path: TextInScroll,
        sample_path: TextInScroll,
        instrument_path: TextInScroll,
        save: Button
    }
);

pub struct SongDirectoryConfigPage {
    widgets: WidgetList,
    selected_widget: usize,
}

impl Page for SongDirectoryConfigPage {
    fn draw(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        let selected = self.selected_widget;
        for idx in Self::INDEX_RANGE {
            self.get_widget(idx).draw(draw_buffer, idx == selected);
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        const BACKGROUND_COLOR: u8 = 2;
        const TOPLEFT_COLOR: u8 = 1;
        const BOTRIGHT_COLOR: u8 = 3;

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
        event: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        match self
            .get_widget_mut(self.selected_widget)
            .process_input(modifiers, key_event, event)
        {
            WidgetResponse::SwitchFocus(next) => {
                assert!(next < Self::WIDGET_COUNT);
                self.selected_widget = next;
                PageResponse::RequestRedraw
            }
            WidgetResponse::RequestRedraw => PageResponse::RequestRedraw,
            WidgetResponse::None => PageResponse::None,
        }
    }
}

impl SongDirectoryConfigPage {
    pub fn ui_change(&mut self, change: SDCChange) -> PageResponse {
        match change {
            SDCChange::SetSongName(s) => match self.widgets.song_name.set_string(s) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialTempo(n) => match self.widgets.initial_tempo.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialSpeed(n) => match self.widgets.initial_speed.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::GlobalVolume(n) => match self.widgets.global_volume.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::MixingVolume(n) => match self.widgets.mixing_volume.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::Seperation(n) => match self.widgets.seperation.number.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
        }
    }

    pub fn new() -> Self {
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
        let global_volume: Slider<0, 128> = Slider::new(
            128,
            CharPosition::new(17, 23),
            16,
            NextWidget {
                up: Some(Self::INITIAL_SPEED),
                shift_tab: Some(Self::INITIAL_SPEED),
                down: Some(Self::MIXING_VOLUME),
                tab: Some(Self::MIXING_VOLUME),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::GlobalVolume(n))),
            |value| println!("gloabl volume set to: {}", value),
        );
        let mixing_volume: Slider<0, 128> = Slider::new(
            48,
            CharPosition::new(17, 24),
            16,
            NextWidget {
                up: Some(Self::GLOBAL_VOLUME),
                shift_tab: Some(Self::GLOBAL_VOLUME),
                down: Some(Self::SEPERATION),
                tab: Some(Self::SEPERATION),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::MixingVolume(n))),
            |value| println!("mixing volume set to: {}", value),
        );
        let seperation: Slider<0, 128> = Slider::new(
            48,
            CharPosition::new(17, 25),
            16,
            NextWidget {
                up: Some(Self::MIXING_VOLUME),
                shift_tab: Some(Self::MIXING_VOLUME),
                down: Some(Self::OLD_EFFECTS),
                tab: Some(Self::OLD_EFFECTS),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::Seperation(n))),
            |value| println!("seperation set to: {}", value),
        );

        let old_effects: Toggle<bool> = Toggle::new(
            CharPosition::new(17, 26),
            16,
            NextWidget {
                left: Some(Self::SEPERATION),
                right: Some(Self::COMPATIBLE_GXX),
                up: Some(Self::SEPERATION),
                down: Some(Self::COMPATIBLE_GXX),
                tab: Some(Self::COMPATIBLE_GXX),
                shift_tab: Some(Self::SEPERATION),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Old Effects: {}", onoff),
        );

        let compatible_gxx: Toggle<bool> = Toggle::new(
            CharPosition::new(17, 27),
            16,
            NextWidget {
                left: Some(Self::OLD_EFFECTS),
                right: Some(Self::INSTRUMENTS),
                up: Some(Self::OLD_EFFECTS),
                down: Some(Self::INSTRUMENTS),
                tab: Some(Self::INSTRUMENTS),
                shift_tab: Some(Self::OLD_EFFECTS),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Compatible Gxx: {}", onoff),
        );

        let control_rc = Rc::new(Cell::new(Control::Samples));
        let instruments: ToggleButton<Control> = ToggleButton::new(
            "Instruments",
            CharRect::new(29, 31, 16, 30),
            NextWidget {
                left: Some(Self::SAMPLES),
                right: Some(Self::SAMPLES),
                up: Some(Self::COMPATIBLE_GXX),
                down: Some(Self::STEREO),
                tab: Some(Self::SAMPLES),
                shift_tab: Some(Self::SAMPLES),
            },
            Control::Instruments,
            control_rc.clone(),
            |_| println!("Instruments activated"),
        );
        let samples: ToggleButton<Control> = ToggleButton::new(
            "Samples",
            CharRect::new(29, 31, 31, 45),
            NextWidget {
                left: Some(Self::INSTRUMENTS),
                right: Some(Self::INSTRUMENTS),
                up: Some(Self::COMPATIBLE_GXX),
                down: Some(Self::MONO),
                tab: Some(Self::INSTRUMENTS),
                shift_tab: Some(Self::INSTRUMENTS),
            },
            Control::Samples,
            control_rc,
            |_| println!("Samples activated"),
        );

        let stereo_mono_rs = Rc::new(Cell::new(Playback::Stereo));
        let stereo: ToggleButton<Playback> = ToggleButton::new(
            "Stereo",
            CharRect::new(32, 34, 16, 30),
            NextWidget {
                left: Some(Self::MONO),
                right: Some(Self::MONO),
                up: Some(Self::INSTRUMENTS),
                down: Some(Self::LINEAR_SLIDES),
                tab: Some(Self::MONO),
                shift_tab: Some(Self::MONO),
            },
            Playback::Stereo,
            stereo_mono_rs.clone(),
            |_| println!("stereo activated"),
        );

        let mono: ToggleButton<Playback> = ToggleButton::new(
            "Mono",
            CharRect::new(32, 34, 31, 45),
            NextWidget {
                left: Some(Self::STEREO),
                right: Some(Self::STEREO),
                up: Some(Self::SAMPLES),
                down: Some(Self::AMIGA_SLIDES),
                tab: Some(Self::STEREO),
                shift_tab: Some(Self::STEREO),
            },
            Playback::Mono,
            stereo_mono_rs,
            |_| println!("stereo activated"),
        );

        let pitch_slides_rc = Rc::new(Cell::new(PitchSlides::Linear));
        let linear_slides: ToggleButton<PitchSlides> = ToggleButton::new(
            "Linear",
            CharRect::new(35, 37, 16, 30),
            NextWidget {
                left: Some(Self::AMIGA_SLIDES),
                right: Some(Self::AMIGA_SLIDES),
                up: Some(Self::STEREO),
                down: Some(Self::MODULE_PATH),
                tab: Some(Self::AMIGA_SLIDES),
                shift_tab: Some(Self::AMIGA_SLIDES),
            },
            PitchSlides::Linear,
            pitch_slides_rc.clone(),
            |_| println!("pitch slides set to linear"),
        );
        let amiga_slides: ToggleButton<PitchSlides> = ToggleButton::new(
            "Amiga",
            CharRect::new(35, 37, 31, 45),
            NextWidget {
                left: Some(Self::LINEAR_SLIDES),
                right: Some(Self::LINEAR_SLIDES),
                up: Some(Self::MONO),
                down: Some(Self::MODULE_PATH),
                tab: Some(Self::LINEAR_SLIDES),
                shift_tab: Some(Self::LINEAR_SLIDES),
            },
            PitchSlides::Amiga,
            pitch_slides_rc,
            |_| println!("set to amiga pitch slide"),
        );

        let module_path = TextInScroll::new(
            CharPosition::new(13, 42),
            64,
            NextWidget {
                up: Some(Self::LINEAR_SLIDES),
                down: Some(Self::SAMPLE_PATH),
                tab: Some(Self::SAMPLE_PATH),
                shift_tab: Some(Self::AMIGA_SLIDES), // whyy???
                ..Default::default()
            },
            |text| println!("Module path set to {text}"),
        );
        let sample_path = TextInScroll::new(
            CharPosition::new(13, 43),
            64,
            NextWidget {
                up: Some(Self::MODULE_PATH),
                shift_tab: Some(Self::MODULE_PATH),
                down: Some(Self::INSTRUMENT_PATH),
                tab: Some(Self::INSTRUMENT_PATH),
                ..Default::default()
            },
            |text| println!("Sample path set to {text}"),
        );
        let instrument_path = TextInScroll::new(
            CharPosition::new(13, 44),
            64,
            NextWidget {
                up: Some(Self::MODULE_PATH),
                shift_tab: Some(Self::MODULE_PATH),
                down: Some(Self::SAVE),
                tab: Some(Self::SAVE),
                ..Default::default()
            },
            |text| println!("Instrument path set to {text}"),
        );

        let save = Button::new(
            "Save all Preferences",
            CharRect::new(46, 48, 28, 51),
            NextWidget {
                up: Some(Self::INSTRUMENT_PATH),
                ..Default::default()
            },
            || println!("save preferences"),
        );
        Self {
            selected_widget: Self::SONG_NAME,
            widgets: WidgetList {
                song_name,
                initial_tempo,
                initial_speed,
                global_volume,
                mixing_volume,
                seperation,
                old_effects,
                compatible_gxx,
                instruments,
                samples,
                stereo,
                mono,
                linear_slides,
                amiga_slides,
                module_path,
                sample_path,
                instrument_path,
                save,
            },
        }
    }
}
