use pixels::{Pixels, SurfaceTexture};
use winit::{window::{Window, WindowBuilder}, dpi::LogicalSize, event_loop::EventLoop};

use crate::{WINDOW_SIZE, WINDOW_TITLE};

pub struct VideoManager {
    pub window: Window,
    pub pixels: Pixels,
    pub event_loop: EventLoop<()>,
}

impl VideoManager {
    pub fn init() -> Self {
        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(WINDOW_SIZE.0 as f64, WINDOW_SIZE.1 as f64);
            WindowBuilder::new().with_title(WINDOW_TITLE).with_inner_size(size).build(&event_loop).unwrap()
        };
        let pixels = {
            let size = window.inner_size();
            let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
            Pixels::new(WINDOW_SIZE.0 as u32, WINDOW_SIZE.1 as u32, surface_texture).unwrap()
        };
        Self { window, pixels, event_loop }
    }
}

type Color = [u8; 4];

pub fn draw_single_char(character: [u8; 8], position: (usize, usize), foreground: Color, background: Color, mut screen: &mut [u8]) {
    for (y, line) in character.iter().enumerate() {
        for x in 0..8 {
            let color = match (line >> x) & 1 == 1 {
                true => foreground,
                false => background,
            };
            screen[4*((y * WINDOW_SIZE.0) + x) + 0] = color[0];
            screen[4*((y * WINDOW_SIZE.0) + x) + 1] = color[1];
            screen[4*((y * WINDOW_SIZE.0) + x) + 2] = color[2];
            screen[4*((y * WINDOW_SIZE.0) + x) + 3] = color[3];
        }
    }
}
