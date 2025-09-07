use std::num::NonZero;

use torque_tracker_engine::project::song::SongOperation;

use crate::{
    app::{EventQueue, GlobalEvent, send_song_op},
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::widgets::{NextWidget, Widget, slider::Slider, text_in::TextIn},
};

use super::{Page, PageResponse};

// #[derive(Debug, Clone, Copy, PartialEq)]
// enum Control {
//     Instruments,
//     Samples,
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// enum Playback {
//     Stereo,
//     Mono,
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// enum PitchSlides {
//     Linear,
//     Amiga,
// }

#[derive(Debug, Clone)]
pub enum SDCChange {
    SetSongName(String),
    InitialTempo(i16),
    InitialSpeed(i16),
    GlobalVolume(i16),
    // MixingVolume(i16),
    // Seperation(i16),
}

// super::create_widget_list!(
//     response: ();
//     WidgetList
//     {
//         song_name: TextIn<()>,
//         initial_tempo: Slider<31, 255, ()>,
//         initial_speed: Slider<1, 255, ()>,
//         global_volume: Slider<0, 128, ()>
//         // mixing_volume: Slider<0, 128, ()>,
//         // seperation: Slider<0, 128, ()>,

//         // old_effects: Toggle<bool, ()>,
//         // compatible_gxx: Toggle<bool, ()>,

//         // instruments: ToggleButton<Control, ()>,
//         // samples: ToggleButton<Control, ()>,

//         // stereo: ToggleButton<Playback, ()>,
//         // mono: ToggleButton<Playback, ()>,

//         // linear_slides: ToggleButton<PitchSlides, ()>,
//         // amiga_slides: ToggleButton<PitchSlides, ()>,

//         // module_path: TextInScroll<()>,
//         // sample_path: TextInScroll<()>,
//         // instrument_path: TextInScroll<()>,
//         // save: Button<()>
//     }
// );

pub struct SongDirectoryConfigPage {
    // widgets: WidgetList,
    song_name: TextIn<()>,
    initial_tempo: Slider<31, 255, ()>,
    initial_speed: Slider<1, 255, ()>,
    global_volume: Slider<0, 128, ()>,
    selected: u8,
}

impl Page for SongDirectoryConfigPage {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        // self.widgets.draw_widgets(draw_buffer);
        self.song_name
            .draw(draw_buffer, Self::SONG_NAME == self.selected);
        self.initial_tempo
            .draw(draw_buffer, Self::INITIAL_TEMPO == self.selected);
        self.initial_speed
            .draw(draw_buffer, Self::INITIAL_SPEED == self.selected);
        self.global_volume
            .draw(draw_buffer, Self::GLOBAL_VOLUME == self.selected);
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
        // draw_buffer.draw_string("Mixing Volume", CharPosition::new(3, 24), 0, 2);
        // draw_buffer.draw_string("Seperation", CharPosition::new(6, 25), 0, 2);
        // draw_buffer.draw_string("Old Effects", CharPosition::new(5, 26), 0, 2);
        // draw_buffer.draw_string("Compatible Gxx", CharPosition::new(2, 27), 0, 2);
        // draw_buffer.draw_in_box(
        //     CharRect::new(22, 28, 16, 34),
        //     BACKGROUND_COLOR,
        //     TOPLEFT_COLOR,
        //     BOTRIGHT_COLOR,
        //     1,
        // );

        // draw_buffer.draw_string("Control", CharPosition::new(9, 30), 0, 2);

        // draw_buffer.draw_string("Playback", CharPosition::new(8, 33), 0, 2);

        // draw_buffer.draw_string("Pitch Slides", CharPosition::new(4, 36), 0, 2);

        // draw_buffer.draw_string("Directories", CharPosition::new(34, 40), 3, 2);

        // draw_buffer.draw_string("Module", CharPosition::new(6, 42), 0, 2);
        // draw_buffer.draw_string("Sample", CharPosition::new(6, 43), 0, 2);
        // draw_buffer.draw_string("Instrument", CharPosition::new(2, 44), 0, 2);
        // draw_buffer.draw_in_box(
        //     CharRect::new(41, 45, 12, 78),
        //     BACKGROUND_COLOR,
        //     TOPLEFT_COLOR,
        //     BOTRIGHT_COLOR,
        //     1,
        // );
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut EventQueue<'_>,
    ) -> PageResponse {
        let resp = match self.selected {
            Self::SONG_NAME => self.song_name.process_input(modifiers, key_event, events),
            Self::INITIAL_TEMPO => self
                .initial_tempo
                .process_input(modifiers, key_event, events),
            Self::INITIAL_SPEED => self
                .initial_speed
                .process_input(modifiers, key_event, events),
            Self::GLOBAL_VOLUME => self
                .global_volume
                .process_input(modifiers, key_event, events),
            _ => unreachable!(),
        };

        resp.standard.to_page_resp(&mut self.selected)
    }

    #[cfg(feature = "accesskit")]
    fn build_tree(
        &self,
        tree: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> crate::app::AccessResponse {
        use accesskit::{Node, NodeId, Role};

        use crate::app::AccessResponse;

        let mut root_node = Node::new(Role::Menu);
        let nodes = [
            NodeId(Self::SONG_NAME_ID),
            NodeId(Self::INITIAL_TEMPO_ID),
            NodeId(Self::INITIAL_SPEED_ID),
            NodeId(Self::GLOBAL_VOLUME_ID),
        ];
        root_node.set_children(nodes);
        root_node.set_label("Song Directory Config Page");

        self.song_name.build_tree(tree);
        self.initial_tempo.build_tree(tree);
        self.initial_speed.build_tree(tree);
        self.global_volume.build_tree(tree);

        tree.push((NodeId(Self::PAGE_ID), root_node));
        AccessResponse {
            root: NodeId(Self::PAGE_ID),
            selected: nodes[usize::from(self.selected - 1)],
        }
    }
}

impl SongDirectoryConfigPage {
    const PAGE_ID: u64 = 12_000_000_000;
    const SONG_NAME: u8 = 1;
    const SONG_NAME_ID: u64 = Self::PAGE_ID + Self::SONG_NAME as u64 * 20;
    const INITIAL_TEMPO: u8 = 2;
    const INITIAL_TEMPO_ID: u64 = Self::PAGE_ID + Self::INITIAL_TEMPO as u64 * 20;
    const INITIAL_SPEED: u8 = 3;
    const INITIAL_SPEED_ID: u64 = Self::PAGE_ID + Self::INITIAL_SPEED as u64 * 20;
    const GLOBAL_VOLUME: u8 = 4;
    const GLOBAL_VOLUME_ID: u64 = Self::PAGE_ID + Self::GLOBAL_VOLUME as u64 * 20;

    pub fn ui_change(&mut self, change: SDCChange) -> PageResponse {
        match change {
            SDCChange::SetSongName(s) => match self.song_name.set_string(s) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialTempo(n) => match self.initial_tempo.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::InitialSpeed(n) => match self.initial_speed.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            SDCChange::GlobalVolume(n) => match self.global_volume.try_set(n) {
                Ok(_) => PageResponse::RequestRedraw,
                Err(_) => PageResponse::None,
            },
            // SDCChange::MixingVolume(n) => match self.widgets.mixing_volume.try_set(n) {
            //     Ok(_) => PageResponse::RequestRedraw,
            //     Err(_) => PageResponse::None,
            // },
            // SDCChange::Seperation(n) => match self.widgets.seperation.try_set(n) {
            //     Ok(_) => PageResponse::RequestRedraw,
            //     Err(_) => PageResponse::None,
            // },
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
            #[cfg(feature = "accesskit")]
            (
                accesskit::NodeId(Self::PAGE_ID + u64::from(Self::SONG_NAME) * 20),
                "Song Name",
            ),
        );
        let initial_tempo = Slider::new(
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
            |n| GlobalEvent::Page(super::PageEvent::Sdc(SDCChange::InitialTempo(n))),
            |value| {
                send_song_op(SongOperation::SetInitialTempo(
                    NonZero::new(u8::try_from(value).unwrap()).unwrap(),
                ));
            },
            #[cfg(feature = "accesskit")]
            (
                accesskit::NodeId(Self::PAGE_ID + u64::from(Self::INITIAL_TEMPO) * 20),
                "Initial Tempo".into(),
            ),
        );
        let initial_speed = Slider::new(
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
            |n| GlobalEvent::Page(super::PageEvent::Sdc(SDCChange::InitialSpeed(n))),
            |value| {
                send_song_op(SongOperation::SetInitialSpeed(
                    NonZero::new(u8::try_from(value).unwrap()).unwrap(),
                ));
            },
            #[cfg(feature = "accesskit")]
            (
                accesskit::NodeId(Self::PAGE_ID + u64::from(Self::INITIAL_SPEED) * 20),
                "Initial Speed".into(),
            ),
        );
        let global_volume = Slider::new(
            128,
            CharPosition::new(17, 23),
            16,
            NextWidget {
                up: Some(Self::INITIAL_SPEED),
                shift_tab: Some(Self::INITIAL_SPEED),
                // down: Some(Self::MIXING_VOLUME),
                // tab: Some(Self::MIXING_VOLUME),
                ..Default::default()
            },
            |n| GlobalEvent::Page(super::PageEvent::Sdc(SDCChange::GlobalVolume(n))),
            |value| send_song_op(SongOperation::SetGlobalVol(u8::try_from(value).unwrap())),
            #[cfg(feature = "accesskit")]
            (
                accesskit::NodeId(Self::PAGE_ID + u64::from(Self::GLOBAL_VOLUME) * 20),
                "Global Volume".into(),
            ),
        );
        // let mixing_volume = Slider::new(
        //     48,
        //     CharPosition::new(17, 24),
        //     16,
        //     NextWidget {
        //         up: Some(WidgetList::GLOBAL_VOLUME),
        //         shift_tab: Some(WidgetList::GLOBAL_VOLUME),
        //         down: Some(WidgetList::SEPERATION),
        //         tab: Some(WidgetList::SEPERATION),
        //         ..Default::default()
        //     },
        //     |n| GlobalEvent::Page(super::PageEvent::Sdc(SDCChange::MixingVolume(n))),
        //     |value| println!("mixing volume set to: {}", value),
        // );
        // let seperation = Slider::new(
        //     48,
        //     CharPosition::new(17, 25),
        //     16,
        //     NextWidget {
        //         up: Some(WidgetList::MIXING_VOLUME),
        //         shift_tab: Some(WidgetList::MIXING_VOLUME),
        //         down: Some(WidgetList::OLD_EFFECTS),
        //         tab: Some(WidgetList::OLD_EFFECTS),
        //         ..Default::default()
        //     },
        //     |n| GlobalEvent::Page(super::PageEvent::Sdc(SDCChange::Seperation(n))),
        //     |value| println!("seperation set to: {}", value),
        // );

        // let old_effects = Toggle::new(
        //     CharPosition::new(17, 26),
        //     16,
        //     NextWidget {
        //         left: Some(WidgetList::SEPERATION),
        //         right: Some(WidgetList::COMPATIBLE_GXX),
        //         up: Some(WidgetList::SEPERATION),
        //         down: Some(WidgetList::COMPATIBLE_GXX),
        //         tab: Some(WidgetList::COMPATIBLE_GXX),
        //         shift_tab: Some(WidgetList::SEPERATION),
        //     },
        //     &[(false, "Off"), (true, "On")],
        //     |onoff| println!("Old Effects: {}", onoff),
        // );

        // let compatible_gxx = Toggle::new(
        //     CharPosition::new(17, 27),
        //     16,
        //     NextWidget {
        //         left: Some(WidgetList::OLD_EFFECTS),
        //         right: Some(WidgetList::INSTRUMENTS),
        //         up: Some(WidgetList::OLD_EFFECTS),
        //         down: Some(WidgetList::INSTRUMENTS),
        //         tab: Some(WidgetList::INSTRUMENTS),
        //         shift_tab: Some(WidgetList::OLD_EFFECTS),
        //     },
        //     &[(false, "Off"), (true, "On")],
        //     |onoff| println!("Compatible Gxx: {}", onoff),
        // );

        // let control_rc = Rc::new(Cell::new(Control::Samples));
        // let instruments = ToggleButton::new(
        //     "Instruments",
        //     CharRect::new(29, 31, 16, 30),
        //     NextWidget {
        //         left: Some(WidgetList::SAMPLES),
        //         right: Some(WidgetList::SAMPLES),
        //         up: Some(WidgetList::COMPATIBLE_GXX),
        //         down: Some(WidgetList::STEREO),
        //         tab: Some(WidgetList::SAMPLES),
        //         shift_tab: Some(WidgetList::SAMPLES),
        //     },
        //     Control::Instruments,
        //     control_rc.clone(),
        //     |_| println!("Instruments activated"),
        // );
        // let samples = ToggleButton::new(
        //     "Samples",
        //     CharRect::new(29, 31, 31, 45),
        //     NextWidget {
        //         left: Some(WidgetList::INSTRUMENTS),
        //         right: Some(WidgetList::INSTRUMENTS),
        //         up: Some(WidgetList::COMPATIBLE_GXX),
        //         down: Some(WidgetList::MONO),
        //         tab: Some(WidgetList::INSTRUMENTS),
        //         shift_tab: Some(WidgetList::INSTRUMENTS),
        //     },
        //     Control::Samples,
        //     control_rc,
        //     |_| println!("Samples activated"),
        // );

        // let stereo_mono_rs = Rc::new(Cell::new(Playback::Stereo));
        // let stereo = ToggleButton::new(
        //     "Stereo",
        //     CharRect::new(32, 34, 16, 30),
        //     NextWidget {
        //         left: Some(WidgetList::MONO),
        //         right: Some(WidgetList::MONO),
        //         up: Some(WidgetList::INSTRUMENTS),
        //         down: Some(WidgetList::LINEAR_SLIDES),
        //         tab: Some(WidgetList::MONO),
        //         shift_tab: Some(WidgetList::MONO),
        //     },
        //     Playback::Stereo,
        //     stereo_mono_rs.clone(),
        //     |_| println!("stereo activated"),
        // );

        // let mono = ToggleButton::new(
        //     "Mono",
        //     CharRect::new(32, 34, 31, 45),
        //     NextWidget {
        //         left: Some(WidgetList::STEREO),
        //         right: Some(WidgetList::STEREO),
        //         up: Some(WidgetList::SAMPLES),
        //         down: Some(WidgetList::AMIGA_SLIDES),
        //         tab: Some(WidgetList::STEREO),
        //         shift_tab: Some(WidgetList::STEREO),
        //     },
        //     Playback::Mono,
        //     stereo_mono_rs,
        //     |_| println!("stereo activated"),
        // );

        // let pitch_slides_rc = Rc::new(Cell::new(PitchSlides::Linear));
        // let linear_slides = ToggleButton::new(
        //     "Linear",
        //     CharRect::new(35, 37, 16, 30),
        //     NextWidget {
        //         left: Some(WidgetList::AMIGA_SLIDES),
        //         right: Some(WidgetList::AMIGA_SLIDES),
        //         up: Some(WidgetList::STEREO),
        //         down: Some(WidgetList::MODULE_PATH),
        //         tab: Some(WidgetList::AMIGA_SLIDES),
        //         shift_tab: Some(WidgetList::AMIGA_SLIDES),
        //     },
        //     PitchSlides::Linear,
        //     pitch_slides_rc.clone(),
        //     |_| println!("pitch slides set to linear"),
        // );
        // let amiga_slides = ToggleButton::new(
        //     "Amiga",
        //     CharRect::new(35, 37, 31, 45),
        //     NextWidget {
        //         left: Some(WidgetList::LINEAR_SLIDES),
        //         right: Some(WidgetList::LINEAR_SLIDES),
        //         up: Some(WidgetList::MONO),
        //         down: Some(WidgetList::MODULE_PATH),
        //         tab: Some(WidgetList::LINEAR_SLIDES),
        //         shift_tab: Some(WidgetList::LINEAR_SLIDES),
        //     },
        //     PitchSlides::Amiga,
        //     pitch_slides_rc,
        //     |_| println!("set to amiga pitch slide"),
        // );

        // let module_path = TextInScroll::new(
        //     CharPosition::new(13, 42),
        //     64,
        //     NextWidget {
        //         up: Some(WidgetList::LINEAR_SLIDES),
        //         down: Some(WidgetList::SAMPLE_PATH),
        //         tab: Some(WidgetList::SAMPLE_PATH),
        //         shift_tab: Some(WidgetList::AMIGA_SLIDES), // whyy???
        //         ..Default::default()
        //     },
        //     |text| println!("Module path set to {text}"),
        // );
        // let sample_path = TextInScroll::new(
        //     CharPosition::new(13, 43),
        //     64,
        //     NextWidget {
        //         up: Some(WidgetList::MODULE_PATH),
        //         shift_tab: Some(WidgetList::MODULE_PATH),
        //         down: Some(WidgetList::INSTRUMENT_PATH),
        //         tab: Some(WidgetList::INSTRUMENT_PATH),
        //         ..Default::default()
        //     },
        //     |text| println!("Sample path set to {text}"),
        // );
        // let instrument_path = TextInScroll::new(
        //     CharPosition::new(13, 44),
        //     64,
        //     NextWidget {
        //         up: Some(WidgetList::MODULE_PATH),
        //         shift_tab: Some(WidgetList::MODULE_PATH),
        //         down: Some(WidgetList::SAVE),
        //         tab: Some(WidgetList::SAVE),
        //         ..Default::default()
        //     },
        //     |text| println!("Instrument path set to {text}"),
        // );

        // let save = Button::new(
        //     "Save all Preferences",
        //     CharRect::new(46, 48, 28, 51),
        //     NextWidget {
        //         up: Some(WidgetList::INSTRUMENT_PATH),
        //         ..Default::default()
        //     },
        //     || {
        //         println!("save preferences");
        //     },
        // );
        Self {
            song_name,
            initial_tempo,
            initial_speed,
            global_volume,
            // mixing_volume,
            // seperation,
            // old_effects,
            // compatible_gxx,
            // instruments,
            // samples,
            // stereo,
            // mono,
            // linear_slides,
            // amiga_slides,
            // module_path,
            // sample_path,
            // instrument_path,
            // save,
            selected: Self::SONG_NAME,
        }
    }
}
