use std::num::NonZeroU8;

use super::constants::MAX_PATTERNS;

#[derive(Clone, Copy, Debug, Default)]
pub struct Event {
    pub note: u8,
    pub instr: u8,
    pub vol: VolumeEffect,
    pub command: Option<(NonZeroU8, u8)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum VolumeEffect {
    FineVolSlideUp(u8),
    FineVolSlideDown(u8),
    VolSlideUp(u8),
    VolSlideDown(u8),
    PitchSlideUp(u8),
    PitchSlideDown(u8),
    SlideToNoteWithSpeed(u8),
    VibratoWithSpeed(u8),
    Volume(u8),
    Panning(u8),
    /// Uses Instr / Sample Default Volume
    #[default]
    None,
}

impl TryFrom<u8> for VolumeEffect {
    type Error = u8;

    /// IT Tracker Format Conversion
    /// no way to get None, as then it just doesn't get set
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..=64 => Ok(Self::Volume(value)),
            65..=74 => Ok(Self::FineVolSlideUp(value - 65)),
            75..=84 => Ok(Self::FineVolSlideDown(value - 75)),
            85..=94 => Ok(Self::VolSlideUp(value - 85)),
            95..=104 => Ok(Self::VolSlideDown(value - 95)),
            105..=114 => Ok(Self::PitchSlideDown(value - 105)),
            115..=124 => Ok(Self::PitchSlideUp(value - 115)),
            128..=192 => Ok(Self::Panning(value - 128)),
            193..=202 => Ok(Self::SlideToNoteWithSpeed(value - 193)),
            203..=212 => Ok(Self::VibratoWithSpeed(value - 203)),
            _ => Err(value),
        }
    }
}

pub type Row = Vec<(u8, Event)>;

#[derive(Clone, Debug)]
pub struct Pattern {
    // rows: Vec<Row>,
    pub rows: Box<[Row]>,
}

impl Default for Pattern {
    fn default() -> Self {
        Self::new(Self::DEFAULT_ROWS)
    }
}

impl Pattern {
    const DEFAULT_ROWS: usize = 64;

    pub fn new(rows: usize) -> Self {
        Self {
            rows: vec![Row::default(); rows].into_boxed_slice(),
        }
    }

    pub fn set_length(&mut self, new_len: usize) {
        let vec = match new_len.cmp(&self.rows.len()) {
            std::cmp::Ordering::Less => {
                let mut vec: Vec<Row> = Vec::with_capacity(new_len);
                for row in &mut self.rows[0..new_len] {
                    vec.push(std::mem::take(row));
                }
                vec
            }
            std::cmp::Ordering::Equal => return,
            std::cmp::Ordering::Greater => {
                let mut vec: Vec<Row> = Vec::with_capacity(new_len);
                for row in self.rows.iter_mut() {
                    vec.push(std::mem::take(row));
                }
                for _ in self.rows.len()..new_len {
                    vec.push(Vec::new());
                }
                vec
            }
        };

        self.rows = vec.into_boxed_slice();
    }

    pub fn set_event(&mut self, row: usize, channel: u8, event: Event) {
        let new_event = event;
        if let Some((_, event)) = self.rows[row].iter_mut().find(|(c, _)| *c == channel) {
            *event = new_event;
        } else {
            self.rows[row].push((channel, new_event));
        }
    }

    /// if there is no event, does nothing
    pub fn remove_event(&mut self, row: usize, channel: u8) {
        let i = self.rows[row].iter().position(|(c, _)| *c == channel);
        if let Some(i) = i {
            self.rows[row].swap_remove(i);
        }
    }
}

/// assumes the Operations are correct (not out of bounds, ...)
pub enum PatternOperation {
    Load(Box<[Pattern; MAX_PATTERNS]>),
    SetLenght {
        pattern: usize,
        new_len: usize,
    },
    SetEvent {
        pattern: usize,
        row: usize,
        channel: u8,
        event: Event,
    },
    RemoveEvent {
        pattern: usize,
        row: usize,
        channel: u8,
    },
}

impl left_right::Absorb<PatternOperation> for [Pattern; MAX_PATTERNS] {
    fn absorb_first(&mut self, operation: &mut PatternOperation, other: &Self) {
        // don't need it mutable
        let operation: &PatternOperation = operation;
        match operation {
            PatternOperation::Load(patterns) => *self = *patterns.clone(),
            PatternOperation::SetLenght { pattern, new_len } => self[*pattern].set_length(*new_len),
            PatternOperation::SetEvent {
                pattern,
                row,
                channel,
                event,
            } => self[*pattern].set_event(*row, *channel, *event),
            PatternOperation::RemoveEvent {
                pattern,
                row,
                channel,
            } => self[*pattern].remove_event(*row, *channel),
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone()
    }
}
