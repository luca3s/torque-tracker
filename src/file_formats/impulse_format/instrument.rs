#[derive(Debug)]
enum NewNoteAction {
    Cut = 0,
    Continue = 1,
    NoteOff = 2,
    NoteFade = 3,
}

impl TryFrom<u8> for NewNoteAction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Cut),
            1 => Ok(Self::Continue),
            2 => Ok(Self::NoteOff),
            3 => Ok(Self::NoteFade),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
enum DuplicateCheckType {
    Off = 0,
    Note = 1,
    Sample = 2,
    Instrument = 3,
}

impl TryFrom<u8> for DuplicateCheckType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Note),
            2 => Ok(Self::Sample),
            3 => Ok(Self::Instrument),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
enum DuplicateCheckAction {
    Cut = 0,
    NoteOff = 1,
    NoteFade = 2,
}

impl TryFrom<u8> for DuplicateCheckAction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Cut),
            1 => Ok(Self::NoteOff),
            2 => Ok(Self::NoteFade),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct ImpulseInstrument {
    dos_file_name: [u8; 12],
    new_note_action: NewNoteAction,
    duplicate_check_type: DuplicateCheckType,
    duplicate_check_action: DuplicateCheckAction,
    fade_out: u16,
    pitch_pan_seperation: i8,
    pitch_pan_center: u8,
    global_volume: u8,
    default_pan: Option<u8>,
    random_volume: u8,
    random_panning: u8,
    created_with: u16,
    number_of_samples: u8,
    name: String,
    initial_filter_cutoff: u8,
    initial_filter_resonance: u8,
    midi_channel: u8,
    midi_prgram: u8,
    midi_bank: u16,
    note_sample_table: [(u8, u8); 120],
    envelopes: [ImpulseEnvelope; 3],
}

impl ImpulseInstrument {
    /// size in file is 554 bytes
    pub fn load(buf: &[u8]) -> Self {
        assert_eq!(buf.len(), 554);
        assert!(buf[0] == b'I');
        assert!(buf[1] == b'M');
        assert!(buf[2] == b'P');
        assert!(buf[3] == b'I');

        let dos_file_name: [u8; 12] = buf[0x04..=0x0F].try_into().unwrap();

        assert_eq!(buf[0x10], 0);
        let new_note_action = NewNoteAction::try_from(buf[0x11]).unwrap();
        let duplicate_check_type = DuplicateCheckType::try_from(buf[0x12]).unwrap();
        let duplicate_check_action = DuplicateCheckAction::try_from(buf[0x13]).unwrap();
        let fade_out = u16::from_le_bytes([buf[0x14], buf[0x15]]);
        let pitch_pan_seperation = i8::from_le_bytes([buf[0x16]]);
        assert!(pitch_pan_seperation >= -32);
        assert!(pitch_pan_seperation <= 32);
        let pitch_pan_center = buf[0x17];
        assert!(pitch_pan_center <= 119);
        let global_volume = buf[0x18];
        assert!(global_volume <= 128);
        let default_pan = if buf[0x19] == 128 {
            None
        } else {
            assert!(buf[0x19] <= 64);
            Some(buf[0x19])
        };
        let random_volume = buf[0x1A];
        assert!(random_volume <= 100);
        let random_pan = buf[0x1B];
        assert!(random_pan <= 100);
        let created_with = u16::from_le_bytes([buf[0x1C], buf[0x1D]]);
        let num_of_samples = buf[0x1E];

        let name = String::from_utf8(
            buf[0x20..=0x39]
                .split(|b| *b == 0)
                .next()
                .unwrap()
                .to_owned(),
        )
        .unwrap();

        let inital_filter_cutoff = buf[0x3A];
        let initial_filter_resonance = buf[0x3B];
        let midi_channel = buf[0x3C];
        let midi_program = buf[0x3D];
        let midi_bank = u16::from_le_bytes([buf[0x3E], buf[0x3F]]);
        let note_sample_table: [(u8, u8); 120] = buf[0x030..0x130]
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect::<Vec<(u8, u8)>>()
            .try_into()
            .unwrap();

        let volume_envelope = ImpulseEnvelope::load(&buf[0x130..0x182]);
        let pan_envelope = ImpulseEnvelope::load(&buf[0x182..0x1D4]);
        let pitch_envelope = ImpulseEnvelope::load(&buf[0x1D4..]);

        todo!()
    }
}

/// flags and node values are interpreted differently depending on the type of evelope.
/// doesnt affect loading
#[derive(Debug)]
pub struct ImpulseEnvelope {
    flags: u8,
    num_node_points: u8,
    loop_start: u8,
    loop_end: u8,
    sustain_loop_start: u8,
    sustain_loop_end: u8,
    nodes: [(u8, u16); 25],
}

impl ImpulseEnvelope {
    pub fn load(buf: &[u8]) -> Self {
        assert_eq!(buf.len(), 82);

        let flags = buf[0];
        let num_node_points = buf[2];
        let loop_start = buf[3];
        let loop_end = buf[4];
        let sustain_loop_start = buf[5];
        let sustain_loop_end = buf[6];

        // chunks_exact leaves remainder of one in this case but the value of that bit isnt being used
        let nodes: [(u8, u16); 25] = buf[7..]
            .chunks_exact(3)
            .map(|chunk| (chunk[0], u16::from_le_bytes([chunk[1], chunk[2]])))
            .collect::<Vec<(u8, u16)>>()
            .try_into()
            .unwrap();

        Self {
            flags,
            num_node_points,
            loop_start,
            loop_end,
            sustain_loop_start,
            sustain_loop_end,
            nodes,
        }
    }
}
