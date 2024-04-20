mod visual;
mod audio;

fn main() {
    let audio_manager = audio::manager::AudioManager::new();
    visual::event_loop::run();
}
