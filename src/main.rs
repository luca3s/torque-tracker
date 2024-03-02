// use audio::AudioManager;
// use pattern::PatternManager;
use rendering::run;
//
mod rendering;
// mod audio;
// mod pattern;

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
    pollster::block_on(run(event_loop, window));
}
