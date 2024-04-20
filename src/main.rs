mod audio;
mod visual;

fn main() {
    let audio_manager = audio::manager::AudioManager::new();
    visual::event_loop::run();
}
