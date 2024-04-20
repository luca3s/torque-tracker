use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo, Sample, Stream,
};

enum StreamBuildError {
    Device,
    Format,
}

pub struct AudioManager {
    stream: Stream,
    collector: basedrop::Collector,
}

impl AudioManager {
    pub fn new() -> Self {
        let stream = match Self::build_stream() {
            Ok(stream) => stream,
            Err(_) => panic!(), // need handling
        };

        stream.play().unwrap();

        AudioManager {
            stream,
            collector: basedrop::Collector::new(),
        }
    }

    fn build_stream() -> Result<Stream, StreamBuildError> {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err(StreamBuildError::Device),
        };

        let mut configs = match device.supported_output_configs() {
            Ok(c) => c,
            Err(_) => return Err(StreamBuildError::Device),
        };

        let config = match configs.find(|c| c.channels() == 2) {
            Some(c) => {
                match c.try_with_sample_rate(cpal::SampleRate(44_100)) {
                    Some(config) => config,
                    // cant panic ever. min sample rate probably above 44_100. (i hope)
                    None => c.with_sample_rate(c.min_sample_rate()),
                }
            }
            None => return Err(StreamBuildError::Format),
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.config(),
                fill_silence::<f32>,
                error_callback,
                None,
            ),
            _ => return Err(StreamBuildError::Format),
        };

        match stream {
            Ok(stream) => Ok(stream),
            Err(err) => match err {
                cpal::BuildStreamError::DeviceNotAvailable => Err(StreamBuildError::Device),
                cpal::BuildStreamError::StreamConfigNotSupported => {
                    panic!("config is built incorrectly")
                }
                _ => panic!("Cpal Error"),
            },
        }
    }
}

fn fill_silence<S: Sample>(data: &mut [S], info: &OutputCallbackInfo) {
    data.fill(S::EQUILIBRIUM);
    println!("{:?}", info.timestamp());
}

fn error_callback(err: cpal::StreamError) {
    println!("{err:?}")
}
