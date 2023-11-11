use std::{sync::{mpsc::{sync_channel, SyncSender, Receiver, Sender, channel}, Arc}, thread, time::Instant};

use arrayvec::ArrayVec;
use cpal::{StreamConfig, Stream, Sample, StreamError, traits::{HostTrait, DeviceTrait, StreamTrait}, StreamInstant};

use crate::audio::constant::get_pan_values;


mod constant;
const TRACK_COUNT: usize = 64;  // 1..64 visible to the user. track 0 is reserved for direct input playback
const INV_TRACK_COUNT: f32 = 0.015625;  // 1 / 64
const MAX_BUFFER_SIZE: usize = 1024;

pub struct AudioManager {
    config: StreamConfig,
    stream: Stream,
    
    mixer_data: MixerData,
    mixer_data_send: tokio::sync::watch::Sender<MixerData>, // needs to send each time the MixerData gets changed

    // time_recv: tokio::sync::watch::Receiver<Option<StreamInstant>>,      // is None until the stream callback function gets called for the first time
    track_outs: ArrayVec<SyncSender<AudioFrame>, TRACK_COUNT>,
    worker_thread_work_send: Vec<Sender<TrackMessage>>,
    // track number, track state after finishing playing
    worker_thread_feedback: Vec<Receiver<(usize, TrackState)>>,
}

impl AudioManager {
    // can fail if there are no audio devices, the default audio device disconnects during the funtion or the device doesnt give a config
    // i just now use default config. audio device needs to have a mode to accept 2 channels with f32
    pub fn init() -> Result<(Self, tokio::sync::watch::Receiver<Option<StreamInstant>>), ()> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(())?;

        let default_config = device.default_output_config().ok().ok_or(())?;
        if default_config.channels() != 2 {
            return Err(());
        }
        let mut config = StreamConfig::from(default_config);
        config.buffer_size = cpal::BufferSize::Fixed(256);
        println!("{:?}", config.buffer_size);
        let buffer_size = match config.buffer_size {
            cpal::BufferSize::Default => panic!("should always be a fixed value, never rely on system default"),
            cpal::BufferSize::Fixed(val) => val,
        };
        
        let mut track_outs: ArrayVec<SyncSender<AudioFrame>, TRACK_COUNT> = ArrayVec::new();
        let mut to_mixer: ArrayVec<Receiver<AudioFrame>, TRACK_COUNT> = ArrayVec::new();
        for i in 0..TRACK_COUNT {
            let (send, recv) = sync_channel(buffer_size as usize);
            track_outs.push(send);
            to_mixer.push(recv);
        }

        // let current_time = Arc::new(AtomicPtr::new(&mut None));
        
        // maybe i should add another channel in which the audio callback can communicate to the audio manager if some error has occured

        let (time_send, time_recv) = tokio::sync::watch::channel(None);
        let (mixer_data_send, mut mixer_data_recv) = tokio::sync::watch::channel(MixerData::default());
        // dont know if i really need to declare it outside the closure but i fear it outherwise gets build everytime
        let mut mixer_data = MixerData::default();
        let stream = device
            .build_output_stream(&config, move |data: &mut [f32], cbinfo: &cpal::OutputCallbackInfo| {
                
                println!("{:?}", cbinfo.timestamp());
                // if this send fails the recv has disconnected which means the AudioManager was dropped. To avoid noise for the user the output gets filled with silence
                if time_send.send(Some(cbinfo.timestamp().callback)).is_err() {
                    write_silence(data);
                    // println!("Time send Error");
                    return;
                }
                // check if the mixer_data has changed and update the internal accordingly
                // can unwrap because if the 
                if let Ok(changed) = mixer_data_recv.has_changed() {
                    if changed {
                        mixer_data = mixer_data_recv.borrow_and_update().clone();
                    }
                } else {
                    write_silence(data);
                    println!("Mixer Data recv Error");
                    return;
                }

                // normal audio processing
                for (frame_num, frame_data) in data.chunks_exact_mut(2).enumerate() {
                    // clean up the audio buffer
                    frame_data[0] = Sample::EQUILIBRIUM;
                    frame_data[1] = Sample::EQUILIBRIUM;
                    for (track_num, track) in to_mixer.iter().enumerate().filter(|(track_num, _)| !mixer_data.muted[*track_num]).rev() {
                        // 'find_time: loop {
                        //     // println!("inside loop");
                        //     if let Ok(track_frame) = track.try_recv() {
                        //         if track_frame.num == frame_num {
                        //             if let Some(data) = track_frame.data {
                        //                 // compute the panned and attenueated values
                        //                 let (l_out, r_out) = match data {
                        //                     AudioFrameData::Mono(s) => {
                        //                         let s = s * mixer_data.volume[track_num];
                        //                         // unwrap for now as the pan values dont even get set so they cant be out of bounds
                        //                         let pan = get_pan_values(mixer_data.panning[track_num]).unwrap();
                        //                         (s*pan.0, s*pan.1) // left: cosine, right: sine
                        //                     },
                        //                     AudioFrameData::Stereo(l, r) => {
                        //                         let (l, r) = (l*mixer_data.volume[track_num], r*mixer_data.volume[track_num]);
                        //                         // unwrap for now as the pan values dont even get set so they cant be out of bounds
                        //                         let pan = get_pan_values(mixer_data.panning[track_num]).unwrap();
                        //                         (l*pan.0, r*pan.1)
                        //                     },
                        //                 };
                        //                 // i assume 0 is left, but im not sure.
                        //                 frame_data[0] = l_out;
                        //                 frame_data[1] = r_out;
                        //             }
                        //             break 'find_time;
                        //         }
                        //     } else {
                        //         println!("not fast enough");
                        //         break 'find_time;
                        //     }
                        // }
                        if let Ok(track_frame) = track.try_recv() {
                            if let Some(data) = track_frame.data {
                                let (l_out, r_out) = match data {
                                    AudioFrameData::Mono(s) => {
                                        let s = s * mixer_data.volume[track_num];
                                        // unwrap for now as the pan values dont even get set so they cant be out of bounds
                                        let pan = get_pan_values(mixer_data.panning[track_num]).unwrap();
                                        (s*pan.0, s*pan.1) // left: cosine, right: sine
                                    },
                                    AudioFrameData::Stereo(l, r) => {
                                        let (l, r) = (l*mixer_data.volume[track_num], r*mixer_data.volume[track_num]);
                                        // unwrap for now as the pan values dont even get set so they cant be out of bounds
                                        let pan = get_pan_values(mixer_data.panning[track_num]).unwrap();
                                        (l*pan.0, r*pan.1)
                                    },
                                };
                                // i assume 0 is left, but im not sure.
                                frame_data[0] = l_out;
                                frame_data[1] = r_out;
                            }
                        }
                    }
                    // make the output quieter so it doesnt clip, may be way too silent
                    frame_data[0] *= INV_TRACK_COUNT;
                    frame_data[1] *= INV_TRACK_COUNT;
                }
            }, audio_err, None)
            .ok()
            .ok_or(())?;
        stream.play().ok().ok_or(())?;


        let worker_thread_count = 1;
        // for now i just do 8 audio work threads. no idea how many i need / are ideal
        let mut worker_thread_work_send = Vec::with_capacity(worker_thread_count);
        let mut worker_thread_feedback = Vec::with_capacity(worker_thread_count);
        // let mut work_input_queues = Vec::with_capacity(8);
        for _ in 0..worker_thread_count {
            let (work_send, work_recv) = channel();
            // in the beginning the workers dont have any work to do so the value is 0
            let (fb_send, fb_recv) = channel();
            worker_thread_work_send.push(work_send);
            worker_thread_feedback.push(fb_recv);
            // work_input_queues.push(recv);
            thread::spawn(move || {
                let mut current_work: Vec<PlayingSample> = Vec::new();
                let mut c = 0;
                let mut d: f32 = -0.5;
                loop {
                    let msg = match work_recv.try_recv() {
                        Ok(msg) => Some(msg),
                        Err(err) => match err {
                            std::sync::mpsc::TryRecvError::Empty => None,
                            // if the sender has disconnected that means that *a* main thread has died (dont know yet how i will build that up), so the workers should die too
                            std::sync::mpsc::TryRecvError::Disconnected => break,
                        },
                    };

                    // TODO: really process that message
                    if let Some(msg) = msg {
                        match msg {
                            TrackMessage::StopPlayback => todo!(),
                            TrackMessage::PlaySample(playing_sample) => current_work.push(playing_sample),
                        }
                    }

                    // TODO: output actual audio
                    for track in current_work.iter_mut() {
                        track.output.send(AudioFrame {
                            data: Some(AudioFrameData::Mono(d)),
                            num: 0,
                        }).unwrap();
                    }

                    c += 1;
                    if c == 200 {
                        d *= -1.;
                    }
                    c = c % 200; 

                    // TODO: when a sample/track is finished send the trackState on the Feedback channel
                    // for track in current_work.iter_mut() {

                    // }
                }
            });
        }


        Ok((AudioManager { config, stream, track_outs, worker_thread_work_send , worker_thread_feedback, mixer_data: MixerData::default(), mixer_data_send }, time_recv))
    }

    /// shit function just to test the mixer while nothing else exists
    pub fn send_work(&mut self) {
        self.mixer_data.muted[0] = false;
        self.update_mixer_data();
        self.worker_thread_work_send[0].send(TrackMessage::PlaySample(PlayingSample {
            track_num: 0,
            output: self.track_outs[0].clone(),
        })).unwrap();
    }

    fn update_mixer_data(&mut self) {
        self.mixer_data_send.send(self.mixer_data.clone()).unwrap();
    }
}

fn write_silence<T: Sample>(data: &mut [T]) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

fn audio_err(_: StreamError) {
    println!("AUIDO ERROR");
}

// struct TrackMessage {
//     track_num: u8,
//     time: StreamInstant,
//     output: SyncSender<AudioFrame>,
// }

enum TrackMessage {
    StopPlayback,
    PlaySample(PlayingSample),

}

struct SampleManager {
    samples: [Arc<[u8]>; 100],
}

#[derive(Clone)]
struct MixerData {
    volume: [f32; TRACK_COUNT],
    panning: [i8; TRACK_COUNT],
    muted: [bool; TRACK_COUNT],
}

struct MixerDataChange {
    num: usize,
    data: MixerDataTypes,
}

impl MixerDataChange {
    fn checked_new(mut num: usize, mut data: MixerDataTypes) -> Self {
        num = num.clamp(0, TRACK_COUNT-1);
        data = match data {
            MixerDataTypes::Volume(v) => MixerDataTypes::Volume(v.clamp(0., 1.)),
            MixerDataTypes::Panning(p) => MixerDataTypes::Panning(p.clamp(-32, 32)),
            MixerDataTypes::Muted(m) => MixerDataTypes::Muted(m),
        };
        Self { num, data }
    }
}

enum MixerDataTypes {
    Volume(f32),
    Panning(i8),
    Muted(bool),
}

impl Default for MixerData {
    fn default() -> Self {
        Self { volume: [1.; TRACK_COUNT], panning: [0; TRACK_COUNT] , muted: [true; TRACK_COUNT] }
    }
}

struct AudioFrame {
    data: Option<AudioFrameData>,
    /// Number of Frame in the current output buffer
    num: usize,
}

enum AudioFrameData {
    Mono(f32),
    /// left, right
    Stereo(f32, f32),
}

struct PlayingSample {
    track_num: i8,
    // track_state: TrackState,
    // sample: Arc<[u8]>,
    // cursor: usize,
    output: SyncSender<AudioFrame>,
}

struct TrackState {
    volume: f32,
    panning: i8,
}

impl Default for TrackState {
    fn default() -> Self {
        Self { volume: 1., panning: 0 }
    }
}