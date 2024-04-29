use std::path::Path;

use file_formats::file_handling::load_file;

mod file_formats;
mod playback;
mod visual;

fn main() {
    load_file(Path::new("/Users/lucasbaumann/Music/Tracker/test-1.it"));
    visual::event_loop::run();
}
