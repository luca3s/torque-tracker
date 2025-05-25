use std::sync::Arc;

use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::{
    coordinates::WINDOW_SIZE,
    palettes::{Palette, RGB8},
};

#[cfg(feature = "gpu_scaling")]
pub struct RenderBackend {
    backend: crate::gpu::GPUState,
}

#[cfg(feature = "gpu_scaling")]
impl RenderBackend {
    pub fn new(window: Arc<Window>, palette: Palette<RGB8>) -> Self {
        let mut backend = smol::block_on(crate::gpu::GPUState::new(window));
        backend.queue_palette_update(palette.into());

        Self { backend }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.backend.resize(size);
    }

    pub fn render(
        &mut self,
        frame_buffer: &[[u8; WINDOW_SIZE.0]; WINDOW_SIZE.1],
        event_loop: &ActiveEventLoop,
    ) {
        match self.backend.render(frame_buffer) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.backend.reinit_surface(),
            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
            Err(e) => eprint!("{:?}", e),
        }
    }
}

#[cfg(not(feature = "gpu_scaling"))]
pub struct RenderBackend {
    backend: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    width: u32,
    height: u32,
    palette: Palette<crate::palettes::ZRGB>,
}

#[cfg(not(feature = "gpu_scaling"))]
impl RenderBackend {
    pub fn new(window: Arc<Window>, palette: Palette<RGB8>) -> Self {
        let size = window.inner_size();
        let context = softbuffer::Context::new(window.clone()).unwrap();
        Self {
            backend: softbuffer::Surface::new(&context, window).unwrap(),
            width: size.width,
            height: size.height,
            palette: palette.into(),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.width = size.width;
        self.height = size.height;
        self.backend
            .resize(
                std::num::NonZeroU32::new(size.width).unwrap(),
                std::num::NonZeroU32::new(size.height).unwrap(),
            )
            .unwrap()
    }

    pub fn render(
        &mut self,
        frame_buffer: &[[u8; WINDOW_SIZE.0]; WINDOW_SIZE.1],
        _: &ActiveEventLoop,
    ) {
        let mut buffer = self.backend.buffer_mut().unwrap();
        assert!(buffer.len() == usize::try_from(self.width * self.height).unwrap());
        let x_step = WINDOW_SIZE.0 as f32 / self.width as f32;
        let y_step = WINDOW_SIZE.1 as f32 / self.height as f32;
        for (y_idx, row) in buffer
            .chunks_exact_mut(self.width.try_into().unwrap())
            .enumerate()
        {
            for (x_idx, pixel) in row.iter_mut().enumerate() {
                let x_idx = x_idx as f32 * x_step;
                let y_idx = y_idx as f32 * y_step;
                *pixel = self
                    .palette
                    .get_raw(frame_buffer[y_idx.floor() as usize][x_idx.floor() as usize]);
            }
        }
        buffer.present().unwrap();
    }
}
