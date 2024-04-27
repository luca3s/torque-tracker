use std::path::Path;

use crate::file_formats::impulse_format::{header::ImpulseHeader, sample::ImpulseSampleHeader, pattern::ImpulsePattern};

pub fn load_file(path: &Path) {
    let file_buf = std::fs::read(path).unwrap();
    let header = ImpulseHeader::load_from_buf(&file_buf);
    println!("{header:?}");

    if let Some(samples) = &header.sample_offsets {
        println!("sample offsets {samples:?}");

        for offset in samples.iter() {
            let sample = ImpulseSampleHeader::load(&file_buf[usize::try_from(*offset).unwrap()..]);
            println!("sample: {sample:?}");
        }
    }

    if let Some(patterns) = &header.pattern_offsets {
        for offset in patterns.iter() {
            ImpulsePattern::load(&file_buf[usize::try_from(*offset).unwrap()..]);
        }
    }

}
