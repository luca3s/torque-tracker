const PALETTE_SIZE: usize = 16;

pub type RGB8 = [u8; 3];

// for RGB10A2 structure see: https://developer.apple.com/documentation/metal/mtlpixelformat/rgb10a2unorm
pub struct RGB10A2(u32);

impl From<RGB8> for RGB10A2 {
    fn from(value: RGB8) -> Self {
        let mut storage: u32 = 0;
        
        let blue: u32 = (u32::from(value[2]) * 4) << 20;
        storage += blue;
        
        let green: u32 = (u32::from(value[1]) * 4) << 10;
        storage += green;

        let red: u32 = u32::from(value[0]) * 4;
        storage += red;

        Self(storage)
    }
}

pub struct Palette<Color>([Color; PALETTE_SIZE]);

// only needed, because its easier to set u8 color values than u10 by hand, so i store them in u8 and convert to RGB10A2
impl Palette<RGB8> {
    pub const CAMOUFLAGE: Palette<RGB8> = Palette([
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
}

impl Palette<RGB10A2> {
    pub fn get_raw(&self, index: usize) -> u32 {
        self.0[index].0
    }
}

impl From<Palette<RGB8>> for Palette<RGB10A2> {
    fn from(value: Palette<RGB8>) -> Self {
        Self(value.0.map(RGB10A2::from))
    }
}
