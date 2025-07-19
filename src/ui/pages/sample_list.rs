use std::{
    collections::VecDeque,
    io::{Cursor, Write},
    iter::zip,
    num::NonZero,
    str::from_utf8,
};

use torque_tracker_engine::{
    project::{
        note_event::Note,
        song::{Song, SongOperation},
    },
    sample::{Sample, SampleMetaData},
};
use winit::keyboard::{Key, NamedKey};

use crate::{
    app::{EXECUTOR, GlobalEvent, SONG_OP_SEND},
    coordinates::{CharPosition, CharRect},
    draw_buffer::DrawBuffer,
    ui::{
        header::HeaderEvent,
        pages::{Page, PageEvent, PageResponse, pattern::PatternPageEvent},
    },
};

#[derive(Debug)]
pub enum SampleListEvent {
    SetSample(u8, String, SampleMetaData),
    SelectSample(u8),
}

pub struct SampleList {
    selected: u8,
    sample_view: u8,
    samples: [Option<(String, SampleMetaData)>; Song::MAX_SAMPLES_INSTR],
    event_proxy: winit::event_loop::EventLoopProxy<GlobalEvent>,
}

impl SampleList {
    const SAMPLE_VIEW_COUNT: u8 = 34;
    pub fn new(event_proxy: winit::event_loop::EventLoopProxy<GlobalEvent>) -> Self {
        Self {
            selected: 0,
            samples: [const { None }; Song::MAX_SAMPLES_INSTR],
            sample_view: 0,
            event_proxy,
        }
    }

    pub fn process_event(
        &mut self,
        event: SampleListEvent,
        events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        match event {
            // this event is from the pattern page, so i don't have to send it there
            SampleListEvent::SelectSample(s) => {
                self.select_sample(s);
                self.send_to_header(events);
                PageResponse::RequestRedraw
            }
            SampleListEvent::SetSample(idx, name, meta) => {
                self.samples[usize::from(idx)] = Some((name, meta));
                if self.selected == idx {
                    self.send_to_header(events);
                }
                PageResponse::RequestRedraw
            }
        }
    }

    fn select_sample(&mut self, selected: u8) {
        self.selected = selected;
        self.sample_view = if self.selected < self.sample_view {
            self.selected
        } else if self.selected > self.sample_view + Self::SAMPLE_VIEW_COUNT {
            self.selected - Self::SAMPLE_VIEW_COUNT
        } else {
            self.sample_view
        };
    }

    fn send_to_header(&self, events: &mut VecDeque<GlobalEvent>) {
        let name: Box<str> = self.samples[usize::from(self.selected)]
            .as_ref()
            .map(|(n, _)| Box::from(n.as_str()))
            .unwrap_or(Box::from(""));
        events.push_back(GlobalEvent::Header(HeaderEvent::SetSample(
            self.selected,
            name,
        )));
    }

    fn send_to_pattern(&self, events: &mut VecDeque<GlobalEvent>) {
        events.push_back(GlobalEvent::PageEvent(PageEvent::Pattern(
            PatternPageEvent::SetSampleInstr(self.selected),
        )));
    }
}

impl Page for SampleList {
    fn draw(&mut self, draw_buffer: &mut DrawBuffer) {
        // samples
        {
            const BASE_POS: CharPosition = CharPosition::new(2, 13);
            let mut buf = [0; 2];
            for (i, n) in
                (self.sample_view..=self.sample_view + Self::SAMPLE_VIEW_COUNT).enumerate()
            {
                // number
                let mut curse: Cursor<&mut [u8]> = Cursor::new(&mut buf);
                write!(curse, "{:02}", n).unwrap();
                let str = from_utf8(&buf).unwrap();
                draw_buffer.draw_string(str, BASE_POS + CharPosition::new(0, i), 0, 2);

                // name
                let name = self.samples[usize::from(n)]
                    .as_ref()
                    .map(|(n, _)| n.as_str())
                    .unwrap_or("");
                let background_color = if self.selected == n { 14 } else { 0 };
                draw_buffer.draw_string_length(
                    name,
                    BASE_POS + CharPosition::new(3, i),
                    24,
                    6,
                    background_color,
                );
            }
        }
    }

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(2, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        events: &mut VecDeque<crate::app::GlobalEvent>,
    ) -> PageResponse {
        if !key_event.state.is_pressed() {
            return PageResponse::None;
        }

        if key_event.logical_key == Key::Named(NamedKey::ArrowUp) && modifiers.state().is_empty() {
            if let Some(s) = self.selected.checked_sub(1) {
                self.select_sample(s);
                self.send_to_header(events);
                self.send_to_pattern(events);
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
            && modifiers.state().is_empty()
        {
            if self.selected + 1 < 100 {
                self.select_sample(self.selected + 1);
                self.send_to_header(events);
                self.send_to_pattern(events);
                return PageResponse::RequestRedraw;
            }
        } else if key_event.logical_key == Key::Named(NamedKey::Enter)
            && modifiers.state().is_empty()
        {
            let dialog = rfd::AsyncFileDialog::new()
                // TODO: figure out which formats i support and sync it with the symphonia features
                // .add_filter("supported audio formats", &["wav"])
                .pick_file();
            let proxy = self.event_proxy.clone();
            let idx = self.selected;
            EXECUTOR
                .spawn(async move {
                    let file = dialog.await;
                    let Some(file) = file else {
                        return;
                    };
                    let file_name = file.file_name();
                    // HOW TO SYMPHONIA: https://github.com/pdeljanov/Symphonia/blob/master/symphonia/examples/basic-interleaved.rs
                    // IO is not async as symphonia doesn't support async IO.
                    // This is fine as i have two background threads and don't
                    // do IO that often.
                    let Ok(file) = std::fs::File::open(file.path()) else {
                        eprintln!("error opening file");
                        return;
                    };
                    let mss = symphonia::core::io::MediaSourceStream::new(
                        Box::new(file),
                        Default::default(),
                    );
                    let probe = symphonia::default::get_probe();
                    let Ok(probed) = probe.format(
                        // TODO: add file extension to the hint
                        &symphonia::core::probe::Hint::new(),
                        mss,
                        &Default::default(),
                        &Default::default(),
                    ) else {
                        eprintln!("format error");
                        return;
                    };
                    let mut format = probed.format;
                    let Some(track) = format
                        .tracks()
                        .iter()
                        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
                    else {
                        eprintln!("no decodable track found");
                        return;
                    };
                    let Ok(mut decoder) = symphonia::default::get_codecs()
                        .make(&track.codec_params, &Default::default())
                    else {
                        eprintln!("no decoder found");
                        return;
                    };
                    let track_id = track.id;
                    let Some(sample_rate) = track.codec_params.sample_rate else {
                        eprintln!("no sample rate");
                        return;
                    };
                    let Some(sample_rate) = NonZero::new(sample_rate) else {
                        eprintln!("sample rate = 0");
                        return;
                    };
                    let mut buf = Vec::new();
                    // i don't know yet. after the first iteration of the loop this is set
                    let mut stereo: Option<bool> = None;
                    loop {
                        let packet = format.next_packet();
                        let packet = match packet {
                            Ok(p) => p,
                            // this is used as a end of stream signal. don't ask me why
                            Err(symphonia::core::errors::Error::IoError(e))
                                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                            {
                                break;
                            }
                            Err(e) => {
                                eprintln!("decoding error: {e:?}");
                                return;
                            }
                        };

                        if packet.track_id() != track_id {
                            continue;
                        }
                        match decoder.decode(&packet) {
                            Ok(audio_buf) => {
                                fn append_to_buf<T>(
                                    buf: &mut Vec<f32>,
                                    in_buf: &symphonia::core::audio::AudioBuffer<T>,
                                    stereo: &mut Option<bool>,
                                ) where
                                    T: symphonia::core::sample::Sample,
                                    f32: symphonia::core::conv::FromSample<T>,
                                {
                                    use symphonia::core::{
                                        audio::{Channels, Signal},
                                        conv::FromSample,
                                    };
                                    if in_buf
                                        .spec()
                                        .channels
                                        .contains(Channels::FRONT_LEFT | Channels::FRONT_RIGHT)
                                    {
                                        // stereo + plus maybe other channels that i ignore
                                        assert!(stereo.is_none() || *stereo == Some(true));
                                        *stereo = Some(true);
                                        let left = in_buf.chan(0);
                                        let right = in_buf.chan(1);
                                        assert!(left.len() == right.len());
                                        let iter = zip(left, right).flat_map(|(l, r)| {
                                            [f32::from_sample(*l), f32::from_sample(*r)]
                                        });
                                        buf.extend(iter);
                                    } else if in_buf.spec().channels.contains(Channels::FRONT_LEFT)
                                    {
                                        // assert not
                                        assert!(stereo.is_none() || *stereo == Some(false));
                                        *stereo = Some(false);
                                        buf.extend(
                                            in_buf
                                                .chan(0)
                                                .iter()
                                                .map(|sample| f32::from_sample(*sample)),
                                        );
                                    } else {
                                        eprintln!("no usable channel in sample data")
                                    }
                                }
                                use symphonia::core::audio::AudioBufferRef;
                                match audio_buf {
                                    AudioBufferRef::U8(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::U16(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::U24(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::U32(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::S8(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::S16(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::S24(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::S32(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::F32(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                    AudioBufferRef::F64(d) => {
                                        append_to_buf(&mut buf, &d, &mut stereo)
                                    }
                                }
                            }
                            Err(symphonia::core::errors::Error::DecodeError(_)) => (),
                            Err(_) => break,
                        }
                    }
                    // hopefully both of these compile to a memcopy...
                    let sample = if stereo.unwrap() {
                        Sample::new_stereo_interpolated(buf)
                    } else {
                        Sample::new_mono(buf)
                    };
                    // TODO: get the real metadata / sane defaults / configurable
                    let meta = SampleMetaData {
                        default_volume: 32,
                        global_volume: 32,
                        default_pan: None,
                        vibrato_speed: 0,
                        vibrato_depth: 0,
                        vibrato_rate: 0,
                        vibrato_waveform: Default::default(),
                        sample_rate,
                        base_note: Note::new(64).unwrap(),
                    };
                    // send to UI
                    proxy
                        .send_event(GlobalEvent::PageEvent(PageEvent::SampleList(
                            SampleListEvent::SetSample(idx, file_name, meta),
                        )))
                        .unwrap();
                    drop(proxy);
                    // send to playback
                    let operation = SongOperation::SetSample(usize::from(idx), meta, sample);
                    SONG_OP_SEND.get().unwrap().send(operation).await.unwrap();
                })
                .detach();
        }
        // TODO: add PageUp and PageDown

        PageResponse::None
    }
}
