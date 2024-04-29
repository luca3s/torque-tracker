use std::path::Path;

use crate::{
    file_formats::impulse_format::{
        header::ImpulseHeader, pattern::load_pattern, sample::ImpulseSampleHeader,
    },
    playback::{constants::MAX_PATTERNS, pattern::Pattern},
};

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

    let mut patterns: [Pattern; MAX_PATTERNS] = [(); 240].map(|_| Pattern::default());
    if let Some(pattern_offsets) = &header.pattern_offsets {
        for (i, offset) in pattern_offsets.iter().enumerate() {
            if *offset != 0 {
                patterns[i] = load_pattern(&file_buf[usize::try_from(*offset).unwrap()..]);
            }
        }
    }
}
