use std::sync::mpsc::{Receiver, TryRecvError};

use basedrop::Shared;

use super::{
    pattern::{Event, Pattern},
    sample::SampleData,
};

pub struct AudioWorker {
    samples: Box<[Option<Shared<SampleData>>; 100]>,
    pattern: left_right::ReadHandle<Pattern>,
    manager: Receiver<WorkerMsg>,
}

impl AudioWorker {
    pub fn update_self(&mut self) {
        match self.manager.try_recv() {
            Ok(WorkerMsg::Sample { id, sample }) => self.samples[id] = sample,
            Ok(WorkerMsg::StopPlayback) => (),
            Ok(WorkerMsg::PlaybackFrom) => (),
            Ok(WorkerMsg::PlayEvent(_)) => (),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!(), // panic or pause playback
        }
    }

    pub fn work(&mut self, buf: &mut [u8]) {}

    pub fn silence(&mut self, buf: &mut [u8]) {}
}

pub enum WorkerMsg {
    Sample {
        id: usize,
        sample: Option<Shared<SampleData>>,
    },
    StopPlayback,
    // need some way to encode information about pattern / position
    PlaybackFrom,
    PlayEvent(Event),
}
