use crate::playback::channel::Pan;

#[derive(Debug)]
enum PatternOrder {
    Number(u8),
    EndOfSong,
    NextOrder,
}

impl TryFrom<u8> for PatternOrder {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            255 => Ok(Self::EndOfSong),
            254 => Ok(Self::NextOrder),
            0..=199 => Ok(Self::Number(value)),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct ImpulseHeader {
    song_name: String,
    philight: u16,

    created_with: u16,
    compatible_with: u16,
    flags: u16,
    special: u16,

    global_volume: u8,
    mix_volume: u8,
    initial_speed: u8,
    initial_tempo: u8,
    pan_seperation: u8,
    pitch_wheel_depth: u8,
    message_lenght: u16,
    message_offset: u32,

    channel_pan: [Pan; 64],
    channel_volume: [u8; 64],

    orders: Option<Vec<PatternOrder>>, // lenght is oder_num

    pub instr_offsets: Option<Box<[u32]>>,
    pub sample_offsets: Option<Box<[u32]>>,
    pub pattern_offsets: Option<Box<[u32]>>,
}

// https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT
impl ImpulseHeader {
    /// load from the beginning of the file, lenght isn't constant, but at least 192 bytes
    pub fn load_from_buf(buf: &[u8]) -> Self {
        assert!(buf.len() >= 192);
        assert_eq!(buf[0], b'I');
        assert_eq!(buf[1], b'M');
        assert_eq!(buf[2], b'P');
        assert_eq!(buf[3], b'M');

        let song_name = String::from_utf8(
            buf[0x04..=0x1D]
                .split(|b| *b == 0)
                .next()
                .unwrap()
                .to_owned(),
        )
        .unwrap();

        let philight = u16::from_le_bytes([buf[0x1E], buf[0x1F]]);

        let order_num = u16::from_le_bytes([buf[0x20], buf[0x21]]);
        let instr_num = u16::from_le_bytes([buf[0x22], buf[0x23]]);
        let sample_num = u16::from_le_bytes([buf[0x24], buf[0x25]]);
        let pattern_num = u16::from_le_bytes([buf[0x26], buf[0x27]]);
        let created_with = u16::from_le_bytes([buf[0x28], buf[0x29]]);
        let compatible_with = u16::from_le_bytes([buf[0x2A], buf[0x2B]]);
        let flags = u16::from_le_bytes([buf[0x2C], buf[0x2D]]);
        let special = u16::from_le_bytes([buf[0x2E], buf[0x2F]]);

        let global_volume = buf[0x30];
        assert!(global_volume <= 128);
        let mix_volume = buf[0x31];
        assert!(mix_volume <= 128);
        let initial_speed = buf[0x32];
        let initial_tempo = buf[0x33];
        let pan_seperation = buf[0x34];
        let pitch_wheel_depth = buf[0x35];
        let message_lenght = u16::from_le_bytes([buf[0x36], buf[0x37]]);
        let message_offset = u32::from_le_bytes([buf[0x38], buf[0x39], buf[0x3A], buf[0x3B]]);
        let _reserved = u32::from_le_bytes([buf[0x3C], buf[0x3D], buf[0x3E], buf[0x3F]]);

        let pan_vals: [u8; 64] = buf[0x40..0x80].try_into().unwrap();
        let channel_pan: [Pan; 64] = pan_vals.map(|pan| Pan::try_from(pan).unwrap());

        let channel_volume: [u8; 64] = buf[0x80..0xC0].try_into().unwrap();
        channel_volume.iter().for_each(|vol| assert!(*vol <= 64));

        let orders_end = usize::from(order_num) + 0xC0; // not inclusive
        let orders: Option<Vec<PatternOrder>> = if order_num == 0 {
            None
        } else {
            let slice = &buf[0xC0..orders_end];
            Some(
                slice
                    .iter()
                    .map(|order| PatternOrder::try_from(*order).unwrap())
                    .collect(),
            )
        };

        let instr_offset_end = orders_end + (usize::from(instr_num) * 4);
        let instr_offsets: Option<Box<[u32]>> = if instr_num == 0 {
            None
        } else {
            Some(
                buf[orders_end..instr_offset_end]
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect(),
            )
        };

        let sample_offset_end = instr_offset_end + (usize::from(sample_num) * 4);
        let sample_offsets: Option<Box<[u32]>> = if sample_num == 0 {
            None
        } else {
            Some(
                buf[instr_offset_end..sample_offset_end]
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect(),
            )
        };

        let pattern_offset_end = sample_offset_end + (usize::from(pattern_num) * 4);
        let pattern_offsets: Option<Box<[u32]>> = if pattern_num == 0 {
            None
        } else {
            Some(
                buf[sample_offset_end..pattern_offset_end]
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect(),
            )
        };

        Self {
            song_name,
            philight,
            created_with,
            compatible_with,
            flags,
            special,
            global_volume,
            mix_volume,
            initial_speed,
            initial_tempo,
            pan_seperation,
            pitch_wheel_depth,
            message_lenght,
            message_offset,
            channel_pan,
            channel_volume,
            orders,
            instr_offsets,
            sample_offsets,
            pattern_offsets,
        }
    }
}
