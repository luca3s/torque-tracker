use symphonia::{
    core::{io::MediaSourceStream, sample, codecs::DecoderOptions, errors::Error, audio::SampleBuffer},
    default,
};

pub enum FileError {
    CantOpen,
    UnknownFormat,
}

pub fn read_file<S: sample::Sample>(
    path: &std::path::Path,
    collection_handle: basedrop::Handle,
) -> Result<basedrop::Shared<symphonia::core::audio::SampleBuffer<S>>, FileError> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Err(FileError::CantOpen),
    };

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = {
        let probed_result = default::get_probe().format(
            &symphonia::core::probe::Hint::new(),
            mss,
            &Default::default(),
            &Default::default(),
        );
        match probed_result {
            Ok(p) => p,
            Err(_) => return Err(FileError::UnknownFormat),
        }
    };

    let mut format = probed.format;

    let track = match format.tracks().iter().find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL) {
        Some(t) => t,
        None => return Err(FileError::UnknownFormat),
    };

    let mut decoder = match default::get_codecs().make(&track.codec_params, &Default::default()) {
        Ok(d) => d,
        Err(_) => return Err(FileError::UnknownFormat),
    };

    let sample_buffer: SampleBuffer<S> = SampleBuffer::new(duration, spec);
    let track_id = track.id;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => todo!(),
        };

        while !format.metadata().is_latest() {
            format.metadata().pop();
        }

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Consume the decoded audio samples (see below).

            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                continue;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                continue;
            }
            Err(err) => {
                // An unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        }
    }
}
