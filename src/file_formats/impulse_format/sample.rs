#[derive(Debug)]
pub enum VibratoWave {
    Sine = 0,
    RampDown = 1,
    Square = 2,
    Random = 3,
}

impl TryFrom<u8> for VibratoWave {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sine),
            1 => Ok(Self::RampDown),
            2 => Ok(Self::Square),
            3 => Ok(Self::Random),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct ImpulseSampleHeader {
    dos_filename: Box<[u8]>,
    sample_name: String,
    global_volume: u8,
    flags: u8,
    default_volume: u8,
    convert: u8,
    default_pan: u8,
    length: u32,
    loop_start: u32,
    loop_end: u32,
    c5_speed: u32,
    sustain_start: u32,
    sustain_end: u32,
    data_file_offset: u32,
    vibrato_speed: u8,
    vibrato_depth: u8,
    vibrato_type: VibratoWave,
    vibrato_rate: u8,
}

impl ImpulseSampleHeader {
    pub fn load(buf: &[u8]) -> Self {
        assert!(buf[0] == b'I');
        assert!(buf[1] == b'M');
        assert!(buf[2] == b'P');
        assert!(buf[3] == b'S');

        let dos_filename = buf[4..=15].to_vec().into_boxed_slice();
        assert!(buf[16] == 0);

        let global_volume = buf[17];
        assert!(global_volume <= 64);

        let flags = buf[18];
        let default_volume = buf[19];
        let str = buf[20..=45].split(|b| *b == 0).next().unwrap().to_vec();
        let sample_name = String::from_utf8(str).unwrap();

        let convert = buf[46];

        let default_pan = buf[47];

        // in samples, not bytes
        let length = u32::from_le_bytes(buf[48..=51].try_into().unwrap());
        let loop_start = u32::from_le_bytes(buf[52..=55].try_into().unwrap());
        let loop_end = u32::from_le_bytes(buf[56..=59].try_into().unwrap());

        // bytes per second at c5
        let c5_speed = u32::from_le_bytes(buf[60..=63].try_into().unwrap());
        assert!(c5_speed <= 9999999);

        // in samples, not bytes
        let sustain_start = u32::from_le_bytes(buf[64..=67].try_into().unwrap());
        let sustain_end = u32::from_le_bytes(buf[68..=71].try_into().unwrap());
        let data_file_offset = u32::from_le_bytes(buf[72..=75].try_into().unwrap());

        let vibrato_speed = buf[76];
        assert!(vibrato_speed <= 64);
        let vibrato_depth = buf[77];
        assert!(vibrato_depth <= 64);
        let vibrato_type = VibratoWave::try_from(buf[78]).unwrap();
        let vibrato_rate = buf[79];
        assert!(vibrato_rate <= 64);

        Self {
            dos_filename,
            sample_name,
            global_volume,
            flags,
            default_volume,
            convert,
            default_pan,
            length,
            loop_start,
            loop_end,
            c5_speed,
            sustain_start,
            sustain_end,
            data_file_offset,
            vibrato_speed,
            vibrato_depth,
            vibrato_type,
            vibrato_rate,
        }
    }
}
