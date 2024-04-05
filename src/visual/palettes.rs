use super::coordinates::PIXEL_SIZE;

type Color = [u8; 3];
// pub type Palette = [Color; 16];
pub struct Palette([Color; 16]);

impl Palette {
    pub const CAMOUFLAGE: Palette = Palette([
        [0, 0, 0],
        [124, 88, 68],
        [180, 148, 120],
        [232, 232, 200],
        [176, 0, 84],
        [252, 252, 84],
        [68, 152, 84],
        [76, 12, 24],
        [32, 84, 0],
        [24, 116, 44],
        [56, 156, 116],
        [220, 232, 224],
        [160, 160, 160],
        [140, 20, 84],
        [88, 64, 60],
        [52, 48, 44],
    ]);

    /// returns Color as used by the GPU in the Framebuffer, opacity set to 0
    pub fn get_extended_color(&self, color: usize) -> [u8; PIXEL_SIZE] {
        [self.0[color][0], self.0[color][1], self.0[color][2], 0]
    }

    pub fn get_packed_color(&self, color: usize) -> u32 {
        bytemuck::cast([self.0[color][0], self.0[color][1], self.0[color][2], 0])
    }
}
