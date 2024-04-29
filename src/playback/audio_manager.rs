use std::sync::mpsc::SyncSender;

use basedrop::{Collector, Shared};

use super::{
    audio_worker::WorkerMsg,
    constants::MAX_PATTERNS,
    pattern::{Pattern, PatternOperation},
    sample::SampleData,
};

pub struct AudioManager {
    pattern: left_right::WriteHandle<[Pattern; MAX_PATTERNS], PatternOperation>,
    gc: Collector,
    worker_send: Option<SyncSender<WorkerMsg>>,
    // keep the samples available.
    sample_list: Box<[Option<Shared<SampleData>>; 100]>,
}

impl AudioManager {
    pub fn new() -> Self {
        todo!()
    }

    pub fn init_audio(&mut self) {}

    pub fn set_pattern(&mut self) {}
}
