pub mod app;
pub mod coordinates;
pub mod draw_buffer;
#[cfg(feature = "gpu_scaling")]
pub mod gpu;
pub mod palettes;
pub mod render;
pub mod ui;

fn main() {
    app::run();
}
