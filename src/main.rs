#![feature(slice_flatten)] // needed to flatten the framebuffer before sending it to the GPU

mod visual;

fn main() {
    // env_logger::init();

    // audio setup
    // let (mut audio_manager, time_recv) = AudioManager::init().expect("Audio initialisation failed");
    // audio_manager.send_work();
    // let mut pattern_manager = PatternManager::init(time_recv);
    visual::event_loop::run();
}
