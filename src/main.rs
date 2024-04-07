#![feature(slice_flatten)] // needed to flatten the framebuffer before sending it to the GPU

use visual::event_loop::run;

mod visual;

fn main() {
    // env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    // audio setup
    // let (mut audio_manager, time_recv) = AudioManager::init().expect("Audio initialisation failed");
    // audio_manager.send_work();
    // let mut pattern_manager = PatternManager::init(time_recv);
    run(event_loop, window);
}
