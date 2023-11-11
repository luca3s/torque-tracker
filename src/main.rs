use audio::AudioManager;
use pattern::PatternManager;
use rendering::run_event_loop;

mod rendering;
mod audio;
mod pattern;

fn main() {
    env_logger::init();
    // audio setup
    let (mut audio_manager, time_recv) = AudioManager::init().expect("Audio initialisation failed");
    audio_manager.send_work();
    let mut pattern_manager = PatternManager::init(time_recv);
    pollster::block_on(run_event_loop());
}
