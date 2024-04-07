use bit_struct::u10;

const PALETTE_SIZE: usize = 16;

pub type RGB8 = [u8; 3];
// pub type RGBA8 = [u8; 4];

// for RGB10A2 structure see: https://developer.apple.com/documentation/metal/mtlpixelformat/rgb10a2unorm
// bit_field macro takes fields from high to low, thats why its ordered like that and not like the name would suggest
bit_struct::bit_struct! {
    pub struct RGB10A2(u32) {
        alpha: bit_struct::u2,
        blue: bit_struct::u10,
        green: bit_struct::u10,
        red: bit_struct::u10,
    }
}

impl From<RGB8> for RGB10A2 {
    /// sets Alpha to 0
    fn from(value: RGB8) -> Self {
        // unwraps are okay because its u8 * 4 which results in a u10
        // in between its a u16 but it is from(u8), so it never panics
        // maybe i can later use unsafe instead
        // multiply by 4 because: 4 * 2^8 = 2^10
        RGB10A2::new(
            bit_struct::u2!(0),
            u10::new(u16::from(value[2]) * 4).unwrap(),
            u10::new(u16::from(value[1]) * 4).unwrap(),
            u10::new(u16::from(value[0]) * 4).unwrap(),
        )
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
        self.0[index].raw()
    }
}

impl From<Palette<RGB8>> for Palette<RGB10A2> {
    fn from(value: Palette<RGB8>) -> Self {
        Self(value.0.map(RGB10A2::from))
    }
}
