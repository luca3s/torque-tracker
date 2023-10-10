use cpal::{StreamConfig, Stream, traits::{HostTrait, DeviceTrait, StreamTrait}, Sample, StreamError};
use font8x8::{FontUnicode, BASIC_FONTS, UnicodeFonts};
use rendering::{VideoManager, draw_single_char};
use winit::{window, event::{Event, WindowEvent}, event_loop::ControlFlow};

struct AudioManager {
    config: StreamConfig,
    stream: Stream,

}

mod rendering;

impl AudioManager {
    // can fail if there are no audio devices, the default audio device disconnects during the funtion or the device doesnt give a config
    // i just now use default config. if it doesnt have 2 channels and they accept f32 i cry and it crashes
    pub fn init() -> Result<Self, ()> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(())?;

        let default_config = device.default_output_config().ok().ok_or(())?;
        if default_config.channels() != 2 {
            return Err(());
        }

        let config = StreamConfig::from(default_config);
        let stream = device.build_output_stream(&config, write_silence::<f32>, audio_err, None).ok().ok_or(())?;
        stream.play().ok().ok_or(())?;
        Ok(AudioManager {
            config,
            stream,
        })
    }
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

fn audio_err(_: StreamError) {
    println!("AUIDO ERROR");
}

const WINDOW_TITLE: &str = "RustRacker";
const FONT_SIZE: usize = 8;
const WINDOW_SIZE: (usize, usize) = (FONT_SIZE * 80, FONT_SIZE * 50);

fn main() {
    // audio setup
    let audio_manager = AudioManager::init();
    let mut video_manager = VideoManager::init();
    video_manager.event_loop.run(move |event, window, control_flow| {
        // print!("loop");
        match event {
            Event::RedrawRequested(_) => {
                let window_size = video_manager.window.inner_size();
                // video_manager.pixels.resize_surface(window_size.width, window_size.height).unwrap();
                // print!("redraw");
                for p in video_manager.pixels.frame_mut().chunks_exact_mut(4) {
                    p.copy_from_slice(&[0, 255, 255, 255]);
                }
                // draw_single_char(BASIC_FONTS.get('a').unwrap(), (0, 0), [0, 0, 0, 0], [0, 255, 255, 255], video_manager.pixels.frame_mut());
                // video_manager.pixels.render().unwrap();
                video_manager.pixels.render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);
                    Ok(())
                }).unwrap();
            },
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),
                WindowEvent::Resized(new_size) => video_manager.pixels.resize_surface(new_size.width, new_size.height).unwrap(),
                _ => (),
            }
            _ => (),
        }
        video_manager.window.request_redraw();
    });

}
