use std::path::Path;

use file_formats::file_handling::load_file;

mod file_formats;
mod visual;
mod playback;

fn main() {
    load_file(Path::new("/Users/lucasbaumann/Music/Tracker/house.it"));
    visual::event_loop::run();
}
