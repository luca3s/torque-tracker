use std::sync::mpsc::{Receiver, TryRecvError};

use basedrop::Shared;

use super::{
    channel::Pan,
    constants::{MAX_CHANNELS, MAX_SAMPLES},
    pattern::{Event, Pattern},
    sample::SampleData,
};

pub struct AudioWorker {
    samples: [Option<Shared<SampleData>>; MAX_SAMPLES],
    pattern: left_right::ReadHandle<Pattern>,
    volume: [u8; MAX_CHANNELS],
    pan: [Pan; MAX_CHANNELS],
    manager: Receiver<WorkerMsg>,
}

impl AudioWorker {
    pub fn update_self(&mut self) {
        match self.manager.try_recv() {
            Ok(WorkerMsg::Sample { id, sample }) => self.samples[usize::from(id)] = sample,
            Ok(WorkerMsg::StopPlayback) => (),
            Ok(WorkerMsg::PlaybackFrom) => (),
            Ok(WorkerMsg::PlayEvent(_)) => (),
            Ok(WorkerMsg::SetVolume { channel, volume }) => {
                self.volume[usize::from(channel)] = volume
            }
            Ok(WorkerMsg::SetPanning { channel, pan }) => self.pan[usize::from(channel)] = pan,
            Ok(WorkerMsg::LoadSong {
                samples,
                volume,
                pan,
            }) => {
                self.samples = *samples;
                self.volume = *volume;
                self.pan = *pan;
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!(), // panic or pause playback
        }
    }

    pub fn work(&mut self, buf: &mut [u8]) {}

    pub fn silence(&mut self, buf: &mut [u8]) {}
}

pub enum WorkerMsg {
    Sample {
        id: u8,
        sample: Option<Shared<SampleData>>,
    },
    StopPlayback,
    // need some way to encode information about pattern / position
    PlaybackFrom,
    PlayEvent(Event),
    SetVolume {
        channel: u8,
        volume: u8,
    },
    SetPanning {
        channel: u8,
        pan: Pan,
    },
    LoadSong {
        samples: Box<[Option<Shared<SampleData>>; MAX_SAMPLES]>,
        volume: Box<[u8; MAX_CHANNELS]>,
        pan: Box<[Pan; MAX_CHANNELS]>,
    },
}
