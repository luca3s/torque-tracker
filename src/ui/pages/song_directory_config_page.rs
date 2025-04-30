use std::{cell::Cell, collections::VecDeque, rc::Rc};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{
        button::Button, slider::Slider, text_in::TextIn, text_in_scroll::TextInScroll,
        toggle::Toggle, toggle_button::ToggleButton, NextWidget, StandardResponse, WidgetResponse,
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

#[derive(Debug)]
pub enum SDCChange {
    SetSongName(String),
    InitialTempo(i16),
    InitialSpeed(i16),
    GlobalVolume(i16),
    MixingVolume(i16),
    Seperation(i16),
}

super::create_widget_list!(
    response: ();
    WidgetList
    {
        song_name: TextIn<()>,
        initial_tempo: Slider<31, 255, ()>,
        initial_speed: Slider<1, 255, ()>,
        global_volume: Slider<0, 128, ()>,
        mixing_volume: Slider<0, 128, ()>,
        seperation: Slider<0, 128, ()>,

        old_effects: Toggle<bool, ()>,
        compatible_gxx: Toggle<bool, ()>,

        instruments: ToggleButton<Control, ()>,
        samples: ToggleButton<Control, ()>,

        stereo: ToggleButton<Playback, ()>,
        mono: ToggleButton<Playback, ()>,

        linear_slides: ToggleButton<PitchSlides, ()>,
        amiga_slides: ToggleButton<PitchSlides, ()>,

        module_path: TextInScroll<()>,
        sample_path: TextInScroll<()>,
        instrument_path: TextInScroll<()>,
        save: Button<()>
    }
);

pub struct SongDirectoryConfigPage {
    widgets: WidgetList,
}

impl Page for SongDirectoryConfigPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        self.widgets.draw_widgets(draw_buffer);
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        const BACKGROUND_COLOR: u8 = 2;
        const TOPLEFT_COLOR: u8 = 1;
        const BOTRIGHT_COLOR: u8 = 3;

        // fill complete page
        draw_buffer.draw_rect(BACKGROUND_COLOR, CharRect::PAGE_AREA);
        // draw_buffer.draw_box(CharRect::new(top, bot, left, right), background_color, top_left_color, bot_right_color)

        draw_buffer.draw_string("Song Variables", CharPosition::new(33, 13), 3, 2);

        draw_buffer.draw_string("Song Name", CharPosition::new(7, 16), 0, 2);
        draw_buffer.draw_in_box(
            CharRect::new(15, 17, 16, 43),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
            1,
        );

        draw_buffer.draw_string("Initial Tempo", CharPosition::new(3, 19), 0, 2);
        draw_buffer.draw_string("Initial Speed", CharPosition::new(3, 20), 0, 2);
        draw_buffer.draw_in_box(
            CharRect::new(18, 21, 16, 50),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
            1,
        );

        draw_buffer.draw_string("Global Volume", CharPosition::new(3, 23), 0, 2);
        draw_buffer.draw_string("Mixing Volume", CharPosition::new(3, 24), 0, 2);
        draw_buffer.draw_string("Seperation", CharPosition::new(6, 25), 0, 2);
        draw_buffer.draw_string("Old Effects", CharPosition::new(5, 26), 0, 2);
        draw_buffer.draw_string("Compatible Gxx", CharPosition::new(2, 27), 0, 2);
        draw_buffer.draw_in_box(
            CharRect::new(22, 28, 16, 34),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
            1,
        );

        draw_buffer.draw_string("Control", CharPosition::new(9, 30), 0, 2);

        draw_buffer.draw_string("Playback", CharPosition::new(8, 33), 0, 2);

        draw_buffer.draw_string("Pitch Slides", CharPosition::new(4, 36), 0, 2);

        draw_buffer.draw_string("Directories", CharPosition::new(34, 40), 3, 2);

        draw_buffer.draw_string("Module", CharPosition::new(6, 42), 0, 2);
        draw_buffer.draw_string("Sample", CharPosition::new(6, 43), 0, 2);
        draw_buffer.draw_string("Instrument", CharPosition::new(2, 44), 0, 2);
        draw_buffer.draw_in_box(
            CharRect::new(41, 45, 12, 78),
            BACKGROUND_COLOR,
            TOPLEFT_COLOR,
            BOTRIGHT_COLOR,
            1,
        );
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        match self
            .widgets
            .process_input(key_event, modifiers, events)
            .standard
        {
            StandardResponse::SwitchFocus(next) => {
                self.widgets.selected = next;
                PageResponse::RequestRedraw
            }
            StandardResponse::RequestRedraw => PageResponse::RequestRedraw,
            StandardResponse::None => PageResponse::None,
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
            SDCChange::InitialTempo(n) => match self.widgets.initial_tempo.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialSpeed(n) => match self.widgets.initial_speed.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::GlobalVolume(n) => match self.widgets.global_volume.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::MixingVolume(n) => match self.widgets.mixing_volume.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::Seperation(n) => match self.widgets.seperation.try_set(n) {
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
                down: Some(WidgetList::INITIAL_TEMPO),
                tab: Some(WidgetList::INITIAL_TEMPO),
                ..Default::default()
            },
            |s| println!("new song name: {}", s),
        );

        let initial_tempo = Slider::new(
            125,
            CharPosition::new(17, 19),
            32,
            NextWidget {
                up: Some(WidgetList::SONG_NAME),
                shift_tab: Some(WidgetList::SONG_NAME),
                down: Some(WidgetList::INITIAL_SPEED),
                tab: Some(WidgetList::INITIAL_SPEED),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::InitialTempo(n))),
            |value| println!("initial tempo set to: {}", value),
        );
        let initial_speed = Slider::new(
            6,
            CharPosition::new(17, 20),
            32,
            NextWidget {
                up: Some(WidgetList::INITIAL_TEMPO),
                shift_tab: Some(WidgetList::INITIAL_TEMPO),
                down: Some(WidgetList::GLOBAL_VOLUME),
                tab: Some(WidgetList::GLOBAL_VOLUME),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::InitialSpeed(n))),
            |value| println!("initial speed set to: {}", value),
        );
        let global_volume = Slider::new(
            128,
            CharPosition::new(17, 23),
            16,
            NextWidget {
                up: Some(WidgetList::INITIAL_SPEED),
                shift_tab: Some(WidgetList::INITIAL_SPEED),
                down: Some(WidgetList::MIXING_VOLUME),
                tab: Some(WidgetList::MIXING_VOLUME),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::GlobalVolume(n))),
            |value| println!("gloabl volume set to: {}", value),
        );
        let mixing_volume = Slider::new(
            48,
            CharPosition::new(17, 24),
            16,
            NextWidget {
                up: Some(WidgetList::GLOBAL_VOLUME),
                shift_tab: Some(WidgetList::GLOBAL_VOLUME),
                down: Some(WidgetList::SEPERATION),
                tab: Some(WidgetList::SEPERATION),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::MixingVolume(n))),
            |value| println!("mixing volume set to: {}", value),
        );
        let seperation = Slider::new(
            48,
            CharPosition::new(17, 25),
            16,
            NextWidget {
                up: Some(WidgetList::MIXING_VOLUME),
                shift_tab: Some(WidgetList::MIXING_VOLUME),
                down: Some(WidgetList::OLD_EFFECTS),
                tab: Some(WidgetList::OLD_EFFECTS),
                ..Default::default()
            },
            |n| GlobalEvent::PageEvent(super::PageEvent::Sdc(SDCChange::Seperation(n))),
            |value| println!("seperation set to: {}", value),
        );

        let old_effects = Toggle::new(
            CharPosition::new(17, 26),
            16,
            NextWidget {
                left: Some(WidgetList::SEPERATION),
                right: Some(WidgetList::COMPATIBLE_GXX),
                up: Some(WidgetList::SEPERATION),
                down: Some(WidgetList::COMPATIBLE_GXX),
                tab: Some(WidgetList::COMPATIBLE_GXX),
                shift_tab: Some(WidgetList::SEPERATION),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Old Effects: {}", onoff),
        );

        let compatible_gxx = Toggle::new(
            CharPosition::new(17, 27),
            16,
            NextWidget {
                left: Some(WidgetList::OLD_EFFECTS),
                right: Some(WidgetList::INSTRUMENTS),
                up: Some(WidgetList::OLD_EFFECTS),
                down: Some(WidgetList::INSTRUMENTS),
                tab: Some(WidgetList::INSTRUMENTS),
                shift_tab: Some(WidgetList::OLD_EFFECTS),
            },
            &[(false, "Off"), (true, "On")],
            |onoff| println!("Compatible Gxx: {}", onoff),
        );

        let control_rc = Rc::new(Cell::new(Control::Samples));
        let instruments = ToggleButton::new(
            "Instruments",
            CharRect::new(29, 31, 16, 30),
            NextWidget {
                left: Some(WidgetList::SAMPLES),
                right: Some(WidgetList::SAMPLES),
                up: Some(WidgetList::COMPATIBLE_GXX),
                down: Some(WidgetList::STEREO),
                tab: Some(WidgetList::SAMPLES),
                shift_tab: Some(WidgetList::SAMPLES),
            },
            Control::Instruments,
            control_rc.clone(),
            |_| println!("Instruments activated"),
        );
        let samples = ToggleButton::new(
            "Samples",
            CharRect::new(29, 31, 31, 45),
            NextWidget {
                left: Some(WidgetList::INSTRUMENTS),
                right: Some(WidgetList::INSTRUMENTS),
                up: Some(WidgetList::COMPATIBLE_GXX),
                down: Some(WidgetList::MONO),
                tab: Some(WidgetList::INSTRUMENTS),
                shift_tab: Some(WidgetList::INSTRUMENTS),
            },
            Control::Samples,
            control_rc,
            |_| println!("Samples activated"),
        );

        let stereo_mono_rs = Rc::new(Cell::new(Playback::Stereo));
        let stereo = ToggleButton::new(
            "Stereo",
            CharRect::new(32, 34, 16, 30),
            NextWidget {
                left: Some(WidgetList::MONO),
                right: Some(WidgetList::MONO),
                up: Some(WidgetList::INSTRUMENTS),
                down: Some(WidgetList::LINEAR_SLIDES),
                tab: Some(WidgetList::MONO),
                shift_tab: Some(WidgetList::MONO),
            },
            Playback::Stereo,
            stereo_mono_rs.clone(),
            |_| println!("stereo activated"),
        );

        let mono = ToggleButton::new(
            "Mono",
            CharRect::new(32, 34, 31, 45),
            NextWidget {
                left: Some(WidgetList::STEREO),
                right: Some(WidgetList::STEREO),
                up: Some(WidgetList::SAMPLES),
                down: Some(WidgetList::AMIGA_SLIDES),
                tab: Some(WidgetList::STEREO),
                shift_tab: Some(WidgetList::STEREO),
            },
            Playback::Mono,
            stereo_mono_rs,
            |_| println!("stereo activated"),
        );

        let pitch_slides_rc = Rc::new(Cell::new(PitchSlides::Linear));
        let linear_slides = ToggleButton::new(
            "Linear",
            CharRect::new(35, 37, 16, 30),
            NextWidget {
                left: Some(WidgetList::AMIGA_SLIDES),
                right: Some(WidgetList::AMIGA_SLIDES),
                up: Some(WidgetList::STEREO),
                down: Some(WidgetList::MODULE_PATH),
                tab: Some(WidgetList::AMIGA_SLIDES),
                shift_tab: Some(WidgetList::AMIGA_SLIDES),
            },
            PitchSlides::Linear,
            pitch_slides_rc.clone(),
            |_| println!("pitch slides set to linear"),
        );
        let amiga_slides = ToggleButton::new(
            "Amiga",
            CharRect::new(35, 37, 31, 45),
            NextWidget {
                left: Some(WidgetList::LINEAR_SLIDES),
                right: Some(WidgetList::LINEAR_SLIDES),
                up: Some(WidgetList::MONO),
                down: Some(WidgetList::MODULE_PATH),
                tab: Some(WidgetList::LINEAR_SLIDES),
                shift_tab: Some(WidgetList::LINEAR_SLIDES),
            },
            PitchSlides::Amiga,
            pitch_slides_rc,
            |_| println!("set to amiga pitch slide"),
        );

        let module_path = TextInScroll::new(
            CharPosition::new(13, 42),
            64,
            NextWidget {
                up: Some(WidgetList::LINEAR_SLIDES),
                down: Some(WidgetList::SAMPLE_PATH),
                tab: Some(WidgetList::SAMPLE_PATH),
                shift_tab: Some(WidgetList::AMIGA_SLIDES), // whyy???
                ..Default::default()
            },
            |text| println!("Module path set to {text}"),
        );
        let sample_path = TextInScroll::new(
            CharPosition::new(13, 43),
            64,
            NextWidget {
                up: Some(WidgetList::MODULE_PATH),
                shift_tab: Some(WidgetList::MODULE_PATH),
                down: Some(WidgetList::INSTRUMENT_PATH),
                tab: Some(WidgetList::INSTRUMENT_PATH),
                ..Default::default()
            },
            |text| println!("Sample path set to {text}"),
        );
        let instrument_path = TextInScroll::new(
            CharPosition::new(13, 44),
            64,
            NextWidget {
                up: Some(WidgetList::MODULE_PATH),
                shift_tab: Some(WidgetList::MODULE_PATH),
                down: Some(WidgetList::SAVE),
                tab: Some(WidgetList::SAVE),
                ..Default::default()
            },
            |text| println!("Instrument path set to {text}"),
        );

        let save = Button::new(
            "Save all Preferences",
            CharRect::new(46, 48, 28, 51),
            NextWidget {
                up: Some(WidgetList::INSTRUMENT_PATH),
                ..Default::default()
            },
            || {
                println!("save preferences");
            },
        );
        Self {
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
                selected: WidgetList::SONG_NAME,
            },
        }
    }
}
