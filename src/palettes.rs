pub const PALETTE_SIZE: usize = 16;

pub type RGB8 = [u8; 3];

/// See https://docs.rs/softbuffer/latest/softbuffer/struct.Buffer.html#data-representation
pub struct ZRGB(u32);

impl ZRGB {
    const fn from_rgb8(value: RGB8) -> Self {
        // needs be_bytes. otherwise everything is green
        Self(u32::from_be_bytes([0, value[0], value[1], value[2]]))
    }
}

impl From<RGB8> for ZRGB {
    fn from(value: RGB8) -> Self {
        Self::from_rgb8(value)
    }
}

/// see: https://developer.apple.com/documentation/metal/mtlpixelformat/rgb10a2unorm
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RGB10A2(pub u32);

impl RGB10A2 {
    const fn from_rgb8(value: RGB8) -> Self {
        let mut storage: u32 = 0;

        let blue: u32 = (value[2] as u32 * 4) << 20;
        storage |= blue;

        let green: u32 = (value[1] as u32 * 4) << 10;
        storage |= green;

        let red: u32 = value[0] as u32 * 4;
        storage |= red;

        Self(storage)
    }
}

impl From<RGB8> for RGB10A2 {
    fn from(value: RGB8) -> Self {
        Self::from_rgb8(value)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RGBA8(pub u32);

impl From<RGB8> for RGBA8 {
    fn from(value: RGB8) -> Self {
        Self(u32::from_le_bytes([value[0], value[1], value[2], 0]))
    }
}

#[repr(transparent)]
pub struct Palette<Color>(pub [Color; PALETTE_SIZE]);

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

// i can't do blanket impl because the two colors could be the same :(
impl From<Palette<RGB8>> for Palette<RGB10A2> {
    fn from(value: Palette<RGB8>) -> Self {
        Self(value.0.map(RGB10A2::from))
    }
}

impl From<Palette<RGB8>> for Palette<ZRGB> {
    fn from(value: Palette<RGB8>) -> Self {
        Self(value.0.map(ZRGB::from))
    }
}

impl Palette<ZRGB> {
    pub fn get_raw(&self, index: u8) -> u32 {
        self.0[usize::from(index)].0
    }
}

impl From<Palette<RGB8>> for Palette<RGBA8> {
    fn from(value: Palette<RGB8>) -> Self {
        Self(value.0.map(RGBA8::from))
    }
}
